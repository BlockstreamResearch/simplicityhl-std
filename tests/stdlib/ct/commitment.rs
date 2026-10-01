use num_bigint::BigUint;
use simplex::either::Either::{self, Left, Right};

use simplicityhl_std::artifacts::tests::ct::commitment::CommitmentProgram as CtCommitmentTestProgram;
use simplicityhl_std::artifacts::tests::ct::commitment::derived_commitment::{
    CommitmentArguments as CtCommitmentTestArguments, CommitmentWitness as CtCommitmentTestWitness,
};

use crate::common::core::{Expect, run};

use super::helpers::{
    ConfPoint, Ge, blinded_generator, commitment, conf_point, decode, random_amount,
    random_asset_id, random_scalar, scalar, unblinded_generator,
};

use FunctionToTest::*;

enum FunctionToTest {
    ValueCommitment,
    XEq,
    AssertAssetGenerator,
    OpensTo,
}

fn program() -> CtCommitmentTestProgram {
    CtCommitmentTestProgram::new(&CtCommitmentTestArguments {})
}

struct Case {
    witness: CtCommitmentTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: CtCommitmentTestWitness {
            function_index: function as u8,
            point: (0, [0; 32]),
            asset_id: [0; 32],
            v: 0,
            abf: [0; 32],
            vbf: [0; 32],
            asset: Right([0; 32]),
            amount: Right(0),
            expected_ge: ([0; 32], [0; 32]),
        },
    }
}

impl Case {
    /// `point`: the on-chain asset generator or value commitment.
    fn point(mut self, point: ConfPoint) -> Self {
        self.witness.point = point;
        self
    }

    /// The claimed opening: asset id, value and both blinding factors.
    fn opening(mut self, asset_id: [u8; 32], v: u64, abf: &BigUint, vbf: &BigUint) -> Self {
        self.witness.asset_id = asset_id;
        self.witness.v = v;
        self.witness.abf = scalar(abf);
        self.witness.vbf = scalar(vbf);
        self
    }

    /// The on-chain `(asset, amount)` pair.
    fn pair(mut self, asset: Either<ConfPoint, [u8; 32]>, amount: Either<ConfPoint, u64>) -> Self {
        self.witness.asset = asset;
        self.witness.amount = amount;
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

/// A random confidential output and the opening that produced it.
struct Output {
    id: [u8; 32],
    v: u64,
    abf: BigUint,
    vbf: BigUint,
    asset: [u8; 33],
    amount: [u8; 33],
}

fn random_output() -> Output {
    let (id, v, abf, vbf) = (
        random_asset_id(),
        random_amount(),
        random_scalar(),
        random_scalar(),
    );
    let generator = blinded_generator(id, &abf);

    Output {
        id,
        v,
        asset: generator.serialize(),
        amount: commitment(v, generator, &vbf).serialize(),
        abf,
        vbf,
    }
}

// value_commitment
#[simplex::test]
fn value_commitment_matches_elements(context: simplex::TestContext) -> anyhow::Result<()> {
    let out = random_output();

    case(ValueCommitment)
        .opening(out.id, out.v, &out.abf, &out.vbf)
        .expect_ge(decode(out.amount))
        .run(&context)
}

// assert_x_eq
#[simplex::test]
fn x_eq_accepts_negation(context: simplex::TestContext) -> anyhow::Result<()> {
    // Only x is compared, so the bit is ignored by design.
    let amount = random_output().amount;
    let (bit, x) = conf_point(amount);

    case(XEq)
        .point((bit ^ 1, x))
        .expect_ge(decode(amount))
        .run(&context)
}

// assert_asset_generator
#[simplex::test]
fn asset_generator_holds(context: simplex::TestContext) -> anyhow::Result<()> {
    let out = random_output();

    case(AssertAssetGenerator)
        .point(conf_point(out.asset))
        .opening(out.id, 0, &out.abf, &out.vbf)
        .run(&context)
}

// assert_opens_to
#[simplex::test]
fn opens_to_confidential(context: simplex::TestContext) -> anyhow::Result<()> {
    let out = random_output();

    case(OpensTo)
        .pair(Left(conf_point(out.asset)), Left(conf_point(out.amount)))
        .opening(out.id, out.v, &out.abf, &out.vbf)
        .run(&context)
}

#[simplex::test]
fn opens_to_confidential_asset_explicit_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let out = random_output();

    case(OpensTo)
        .pair(Left(conf_point(out.asset)), Right(out.v))
        .opening(out.id, out.v, &out.abf, &out.vbf)
        .run(&context)
}

#[simplex::test]
fn opens_to_explicit_asset_confidential_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let (id, v, vbf) = (random_asset_id(), random_amount(), random_scalar());
    let amount = commitment(v, unblinded_generator(id), &vbf).serialize();

    case(OpensTo)
        .pair(Right(id), Left(conf_point(amount)))
        .opening(id, v, &random_scalar(), &vbf)
        .run(&context)
}

#[simplex::test]
fn opens_to_explicit(context: simplex::TestContext) -> anyhow::Result<()> {
    let (id, v) = (random_asset_id(), random_amount());

    case(OpensTo)
        .pair(Right(id), Right(v))
        .opening(id, v, &random_scalar(), &random_scalar())
        .run(&context)
}

#[simplex::test]
fn opens_to_rejects_other_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let out = random_output();

    case(OpensTo)
        .pair(Left(conf_point(out.asset)), Left(conf_point(out.amount)))
        .opening(out.id, out.v + 1, &out.abf, &out.vbf)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn opens_to_rejects_other_asset(context: simplex::TestContext) -> anyhow::Result<()> {
    let out = random_output();

    case(OpensTo)
        .pair(Left(conf_point(out.asset)), Left(conf_point(out.amount)))
        .opening(random_asset_id(), out.v, &out.abf, &out.vbf)
        .expecting(&context, Expect::AssertFailed)
}
