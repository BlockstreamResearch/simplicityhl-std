use num_bigint::BigUint;
use simplex::either::Either::{self, Left, Right};

use simplicityhl_std::artifacts::tests::ct::relations::RelationsProgram as CtRelationsTestProgram;
use simplicityhl_std::artifacts::tests::ct::relations::derived_relations::{
    RelationsArguments as CtRelationsTestArguments, RelationsWitness as CtRelationsTestWitness,
};

use crate::common::core::{Expect, run};

use super::helpers::{
    ConfPoint, Ge, add_n, blinded_generator, commitment, conf_point, decode, mul_n, random_amount,
    random_asset_id, random_scalar, scalar, sub_n, total_blinding, unblinded_commitment,
    unblinded_generator,
};

use FunctionToTest::*;

enum FunctionToTest {
    AssetToGe,
    AmountToGej,
    Balanced,
    IsBalanced,
    SumEq,
    ValueEq,
    IsValueEq,
    AssetEq,
    IsAssetEq,
    ScaledEq,
    IsScaledEq,
    RatioEq,
}

type Asset1 = Either<ConfPoint, [u8; 32]>;
type Amount1 = Either<ConfPoint, u64>;

fn program() -> CtRelationsTestProgram {
    CtRelationsTestProgram::new(&CtRelationsTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: CtRelationsTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: CtRelationsTestWitness {
            function_index: function as u8,
            p: (0, [0; 32]),
            q: (0, [0; 32]),
            r: (0, [0; 32]),
            k: 0,
            k2: 0,
            s: [0; 32],
            asset_a: Right([0; 32]),
            amount_a: Right(0),
            asset_b: Right([0; 32]),
            amount_b: Right(0),
            asset_c: Right([0; 32]),
            amount_c: Right(0),
            expected_ge: ([0; 32], [0; 32]),
        },
    }
}

impl Case {
    fn points(mut self, p: ConfPoint, q: ConfPoint) -> Self {
        self.witness.p = p;
        self.witness.q = q;
        self
    }

    fn third(mut self, r: ConfPoint) -> Self {
        self.witness.r = r;
        self
    }

    fn multipliers(mut self, k: u64, k2: u64) -> Self {
        self.witness.k = k;
        self.witness.k2 = k2;
        self
    }

    /// `s`: the blinding difference the relation holds up to.
    fn blinding(mut self, s: &BigUint) -> Self {
        self.witness.s = scalar(s);
        self
    }

    fn input_a(mut self, asset: Asset1, amount: Amount1) -> Self {
        self.witness.asset_a = asset;
        self.witness.amount_a = amount;
        self
    }

    fn input_b(mut self, asset: Asset1, amount: Amount1) -> Self {
        self.witness.asset_b = asset;
        self.witness.amount_b = amount;
        self
    }

    fn output_c(mut self, asset: Asset1, amount: Amount1) -> Self {
        self.witness.asset_c = asset;
        self.witness.amount_c = amount;
        self
    }

    /// `expected_ge`: the affine point the arm should produce.
    fn expect_ge(mut self, expected_ge: Ge) -> Self {
        self.witness.expected_ge = expected_ge;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        run(context, program(), self.witness, expect)
    }
}

/// A confidential output: blinded asset `id` holding `v`.
struct Output {
    asset: ConfPoint,
    amount: ConfPoint,
    /// `v*abf + vbf`
    blinding: BigUint,
}

fn output(id: [u8; 32], v: u64) -> Output {
    let (abf, vbf) = (random_scalar(), random_scalar());
    let generator = blinded_generator(id, &abf);

    Output {
        asset: conf_point(generator.serialize()),
        amount: conf_point(commitment(v, generator, &vbf).serialize()),
        blinding: total_blinding(v, &abf, &vbf),
    }
}

// asset_to_ge
#[simplex::test]
fn asset_to_ge_confidential(context: simplex::TestContext) -> anyhow::Result<()> {
    let h = blinded_generator(random_asset_id(), &random_scalar()).serialize();

    case(AssetToGe)
        .input_a(Left(conf_point(h)), Right(0))
        .expect_ge(decode(h))
        .run(&context)
}

#[simplex::test]
fn asset_to_ge_explicit(context: simplex::TestContext) -> anyhow::Result<()> {
    let id = random_asset_id();

    case(AssetToGe)
        .input_a(Right(id), Right(0))
        .expect_ge(decode(unblinded_generator(id).serialize()))
        .run(&context)
}

// amount_to_gej
#[simplex::test]
fn amount_to_gej_confidential(context: simplex::TestContext) -> anyhow::Result<()> {
    let generator = blinded_generator(random_asset_id(), &random_scalar());
    let c = commitment(random_amount(), generator, &random_scalar()).serialize();

    case(AmountToGej)
        .input_a(Left(conf_point(generator.serialize())), Left(conf_point(c)))
        .expect_ge(decode(c))
        .run(&context)
}

#[simplex::test]
fn amount_to_gej_explicit_amount_confidential_asset(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let generator = blinded_generator(random_asset_id(), &random_scalar());
    let v = random_amount();

    case(AmountToGej)
        .input_a(Left(conf_point(generator.serialize())), Right(v))
        .expect_ge(decode(unblinded_commitment(v, generator).serialize()))
        .run(&context)
}

#[simplex::test]
fn amount_to_gej_explicit(context: simplex::TestContext) -> anyhow::Result<()> {
    let id = random_asset_id();
    let v = random_amount();

    case(AmountToGej)
        .input_a(Right(id), Right(v))
        .expect_ge(decode(
            unblinded_commitment(v, unblinded_generator(id)).serialize(),
        ))
        .run(&context)
}

// assert_balanced, is_balanced
//
// Inputs: confidential `v1` and explicit `v2`. Output: confidential `v_out`.
fn balance_case(function: FunctionToTest, v_out_delta: u64) -> Case {
    let id = random_asset_id();
    let (v1, v2) = (random_amount(), random_amount());

    let a = output(id, v1);
    let c = output(id, v1 + v2 + v_out_delta);

    case(function)
        .input_a(Left(a.asset), Left(a.amount))
        .input_b(Right(id), Right(v2))
        .output_c(Left(c.asset), Left(c.amount))
        .blinding(&sub_n(&a.blinding, &c.blinding))
}

#[simplex::test]
fn balanced_mixed_explicit_and_confidential(context: simplex::TestContext) -> anyhow::Result<()> {
    balance_case(Balanced, 0).run(&context)
}

#[simplex::test]
fn balanced_rejects_inflation(context: simplex::TestContext) -> anyhow::Result<()> {
    balance_case(Balanced, 1).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn is_balanced_mixed_explicit_and_confidential(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    balance_case(IsBalanced, 0).run(&context)
}

#[simplex::test]
fn is_balanced_rejects_inflation(context: simplex::TestContext) -> anyhow::Result<()> {
    balance_case(IsBalanced, 1).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn balanced_rejects_other_asset(context: simplex::TestContext) -> anyhow::Result<()> {
    let (v1, v2) = (random_amount(), random_amount());
    let a = output(random_asset_id(), v1);
    let c = output(random_asset_id(), v1 + v2);
    let id_b = random_asset_id();

    case(Balanced)
        .input_a(Left(a.asset), Left(a.amount))
        .input_b(Right(id_b), Right(v2))
        .output_c(Left(c.asset), Left(c.amount))
        .blinding(&sub_n(&a.blinding, &c.blinding))
        .expecting(&context, Expect::AssertFailed)
}

// assert_sum_eq
fn sum_case(sum_delta: u64) -> Case {
    let id = random_asset_id();
    let (v1, v2) = (random_amount(), random_amount());

    let c1 = output(id, v1);
    let c2 = output(id, v2);
    let sum = output(id, v1 + v2 + sum_delta);

    case(SumEq)
        .points(c1.amount, c2.amount)
        .third(sum.amount)
        .blinding(&sub_n(&add_n(&c1.blinding, &c2.blinding), &sum.blinding))
}

#[simplex::test]
fn sum_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    sum_case(0).run(&context)
}

#[simplex::test]
fn sum_eq_rejects_wrong_sum(context: simplex::TestContext) -> anyhow::Result<()> {
    sum_case(1).expecting(&context, Expect::AssertFailed)
}

// assert_value_eq, is_value_eq
//
// `q` and `p` hold `v_q` of `id_q` and `v_p` of `id_p`, each blinded differently.
fn value_eq_case(function: FunctionToTest, same_value: bool, same_asset: bool) -> Case {
    let id_p = random_asset_id();
    let id_q = if same_asset { id_p } else { random_asset_id() };
    let v_p = random_amount();
    let v_q = if same_value { v_p } else { v_p + 1 };

    let p = output(id_p, v_p);
    let q = output(id_q, v_q);

    case(function)
        .points(p.amount, q.amount)
        .blinding(&sub_n(&q.blinding, &p.blinding))
}

#[simplex::test]
fn value_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    value_eq_case(ValueEq, true, true).run(&context)
}

#[simplex::test]
fn value_eq_rejects_other_value(context: simplex::TestContext) -> anyhow::Result<()> {
    value_eq_case(ValueEq, false, true).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn value_eq_rejects_other_asset(context: simplex::TestContext) -> anyhow::Result<()> {
    value_eq_case(ValueEq, true, false).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn is_value_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    value_eq_case(IsValueEq, true, true).run(&context)
}

#[simplex::test]
fn is_value_eq_rejects_other_value(context: simplex::TestContext) -> anyhow::Result<()> {
    value_eq_case(IsValueEq, false, true).expecting(&context, Expect::AssertFailed)
}

// assert_asset_eq, is_asset_eq
fn asset_eq_case(function: FunctionToTest, same_asset: bool) -> Case {
    let id_p = random_asset_id();
    let id_q = if same_asset { id_p } else { random_asset_id() };
    let (abf_p, abf_q) = (random_scalar(), random_scalar());

    case(function)
        .points(
            conf_point(blinded_generator(id_p, &abf_p).serialize()),
            conf_point(blinded_generator(id_q, &abf_q).serialize()),
        )
        .blinding(&sub_n(&abf_q, &abf_p))
}

#[simplex::test]
fn asset_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    asset_eq_case(AssetEq, true).run(&context)
}

#[simplex::test]
fn asset_eq_rejects_other_asset(context: simplex::TestContext) -> anyhow::Result<()> {
    asset_eq_case(AssetEq, false).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn is_asset_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    asset_eq_case(IsAssetEq, true).run(&context)
}

#[simplex::test]
fn is_asset_eq_rejects_other_asset(context: simplex::TestContext) -> anyhow::Result<()> {
    asset_eq_case(IsAssetEq, false).expecting(&context, Expect::AssertFailed)
}

// assert_scaled_eq, is_scaled_eq
//
// `q` holds `k*v + q_delta`, `p` holds `v`.
fn scaled_eq_case(function: FunctionToTest, q_delta: u64) -> Case {
    let id = random_asset_id();
    let v = random_amount();
    let k = random_amount() % 1_000 + 2;

    let p = output(id, v);
    let q = output(id, k * v + q_delta);

    case(function)
        .points(p.amount, q.amount)
        .multipliers(k, 0)
        .blinding(&sub_n(&q.blinding, &mul_n(&BigUint::from(k), &p.blinding)))
}

#[simplex::test]
fn scaled_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    scaled_eq_case(ScaledEq, 0).run(&context)
}

#[simplex::test]
fn scaled_eq_rejects_wrong_multiple(context: simplex::TestContext) -> anyhow::Result<()> {
    scaled_eq_case(ScaledEq, 1).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn is_scaled_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    scaled_eq_case(IsScaledEq, 0).run(&context)
}

#[simplex::test]
fn is_scaled_eq_rejects_wrong_multiple(context: simplex::TestContext) -> anyhow::Result<()> {
    scaled_eq_case(IsScaledEq, 1).expecting(&context, Expect::AssertFailed)
}

// assert_ratio_eq
//
// `x` holds `5*w`, `y` holds `3*w + y_delta`, so `3*x == 5*y` iff `y_delta == 0`.
fn ratio_case(y_delta: u64) -> Case {
    let id = random_asset_id();
    let w = random_amount();
    let (a, b) = (3u64, 5u64);

    let x = output(id, b * w);
    let y = output(id, a * w + y_delta);

    case(RatioEq)
        .points(x.amount, y.amount)
        .multipliers(a, b)
        .blinding(&sub_n(
            &mul_n(&BigUint::from(a), &x.blinding),
            &mul_n(&BigUint::from(b), &y.blinding),
        ))
}

#[simplex::test]
fn ratio_eq_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    ratio_case(0).run(&context)
}

#[simplex::test]
fn ratio_eq_rejects_wrong_ratio(context: simplex::TestContext) -> anyhow::Result<()> {
    ratio_case(1).expecting(&context, Expect::AssertFailed)
}
