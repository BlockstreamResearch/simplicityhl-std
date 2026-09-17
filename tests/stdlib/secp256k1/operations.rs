use num_bigint::BigUint;
use num_traits::Zero;
use rand::{RngCore, rngs::OsRng};
use secp256k1_zkp::{PublicKey, Secp256k1, SecretKey, rand::rngs::OsRng as SecpOsRng};

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::secp256k1::operations::OperationsProgram as Secp256k1OperationsTestProgram;
use simplicityhl_std::artifacts::tests::secp256k1::operations::derived_operations::{
    OperationsArguments as Secp256k1OperationsTestArguments,
    OperationsWitness as Secp256k1OperationsTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    GeToPoint,
    PointToGej,
    FeSub,
    ScalarSub,
    GejSub,
    FeEq,
    ScalarEq,
    GeEq,
    GejPointEq,
    SafeGejNormalize,
}

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
const SECP_P: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFC, 0x2F,
];

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
const SECP_N: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B, 0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
];

fn program() -> Secp256k1OperationsTestProgram {
    Secp256k1OperationsTestProgram::new(Secp256k1OperationsTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
#[derive(Clone, Debug)]
struct Case {
    witness: Secp256k1OperationsTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: Secp256k1OperationsTestWitness {
            function_index: function as u8,
            first_uint: [0; 32],
            second_uint: [0; 32],
            first_ge: ([0; 32], [0; 32]),
            second_ge: ([0; 32], [0; 32]),
            first_gej: (([0; 32], [0; 32]), [0; 32]),
            second_gej: (([0; 32], [0; 32]), [0; 32]),
            first_point: (0, [0; 32]),
            expected_uint: [0; 32],
            expected_ge: ([0; 32], [0; 32]),
            expected_gej: (([0; 32], [0; 32]), [0; 32]),
            expected_point: (0, [0; 32]),
        },
    }
}

impl Case {
    /// The two field or scalar operands, `first_uint` and `second_uint`.
    fn uints(mut self, first_uint: [u8; 32], second_uint: [u8; 32]) -> Self {
        self.witness.first_uint = first_uint;
        self.witness.second_uint = second_uint;
        self
    }

    /// Only `first_ge`.
    fn ge(mut self, first_ge: ([u8; 32], [u8; 32])) -> Self {
        self.witness.first_ge = first_ge;
        self
    }

    /// The two affine points, `first_ge` and `second_ge`.
    fn ges(mut self, first_ge: ([u8; 32], [u8; 32]), second_ge: ([u8; 32], [u8; 32])) -> Self {
        self.witness.first_ge = first_ge;
        self.witness.second_ge = second_ge;
        self
    }

    /// Only `first_gej`.
    fn gej(mut self, first_gej: (([u8; 32], [u8; 32]), [u8; 32])) -> Self {
        self.witness.first_gej = first_gej;
        self
    }

    /// The two Jacobian points, `first_gej` and `second_gej`.
    fn gejs(
        mut self,
        first_gej: (([u8; 32], [u8; 32]), [u8; 32]),
        second_gej: (([u8; 32], [u8; 32]), [u8; 32]),
    ) -> Self {
        self.witness.first_gej = first_gej;
        self.witness.second_gej = second_gej;
        self
    }

    /// `first_point`: a compressed point, parity byte and x coordinate.
    fn point(mut self, first_point: (u8, [u8; 32])) -> Self {
        self.witness.first_point = first_point;
        self
    }

    /// `expected_uint`: the field or scalar the arm should produce.
    fn expect_uint(mut self, expected_uint: [u8; 32]) -> Self {
        self.witness.expected_uint = expected_uint;
        self
    }

    /// `expected_ge`: the affine point the arm should produce.
    fn expect_ge(mut self, expected_ge: ([u8; 32], [u8; 32])) -> Self {
        self.witness.expected_ge = expected_ge;
        self
    }

    /// `expected_gej`: the Jacobian point the arm should produce.
    fn expect_gej(mut self, expected_gej: (([u8; 32], [u8; 32]), [u8; 32])) -> Self {
        self.witness.expected_gej = expected_gej;
        self
    }

    /// `expected_point`: the compressed point the arm should produce.
    fn expect_point(mut self, expected_point: (u8, [u8; 32])) -> Self {
        self.witness.expected_point = expected_point;
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

fn secp_p() -> BigUint {
    BigUint::from_bytes_be(&SECP_P)
}
fn secp_n() -> BigUint {
    BigUint::from_bytes_be(&SECP_N)
}

fn to_32_be(x: &BigUint) -> [u8; 32] {
    let b = x.to_bytes_be();

    let mut out = [0u8; 32];
    out[32 - b.len()..].copy_from_slice(&b);

    out
}

fn mod_p(x: &BigUint) -> BigUint {
    x % secp_p()
}
fn mod_n(x: &BigUint) -> BigUint {
    x % secp_n()
}

fn fe_sub_ref(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let p = secp_p();
    let a = BigUint::from_bytes_be(&a) % &p;
    let b = BigUint::from_bytes_be(&b) % &p;

    to_32_be(&((a + &p - b) % &p))
}

fn scalar_sub_ref(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let n = secp_n();
    let a = BigUint::from_bytes_be(&a) % &n;
    let b = BigUint::from_bytes_be(&b) % &n;

    to_32_be(&((a + &n - b) % &n))
}

fn fe_negate_ref(a: [u8; 32]) -> [u8; 32] {
    let p = secp_p();
    let a = BigUint::from_bytes_be(&a) % &p;

    if a.is_zero() {
        [0u8; 32]
    } else {
        to_32_be(&(&p - &a))
    }
}

fn fe_mul_ref(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let p = secp_p();
    let a = BigUint::from_bytes_be(&a);
    let b = BigUint::from_bytes_be(&b);

    to_32_be(&((a * b) % &p))
}

// Randomness helpers
fn random_uint_bytes() -> [u8; 32] {
    let mut b = [0u8; 32];
    OsRng.fill_bytes(&mut b);

    b
}

fn random_fe_bytes() -> [u8; 32] {
    to_32_be(&mod_p(&BigUint::from_bytes_be(&random_uint_bytes())))
}

fn random_scalar_bytes() -> [u8; 32] {
    to_32_be(&mod_n(&BigUint::from_bytes_be(&random_uint_bytes())))
}

fn random_ge_bytes() -> ([u8; 32], [u8; 32]) {
    let secp = Secp256k1::new();
    let sk = SecretKey::new(&mut SecpOsRng);
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let ser = pk.serialize_uncompressed(); // 0x04 || x(32) || y(32)

    let mut x = [0u8; 32];
    x.copy_from_slice(&ser[1..33]);

    let mut y = [0u8; 32];
    y.copy_from_slice(&ser[33..65]);

    (x, y)
}

fn ge_to_gej(ge: ([u8; 32], [u8; 32])) -> (([u8; 32], [u8; 32]), [u8; 32]) {
    let mut one = [0u8; 32];
    one[31] = 1;

    (ge, one)
}

fn compress(ge: ([u8; 32], [u8; 32])) -> (u8, [u8; 32]) {
    (ge.1[31] & 1, ge.0)
}

fn pk_to_gej(pk: &PublicKey) -> (([u8; 32], [u8; 32]), [u8; 32]) {
    let ser = pk.serialize_uncompressed();

    let mut x = [0u8; 32];
    x.copy_from_slice(&ser[1..33]);

    let mut y = [0u8; 32];
    y.copy_from_slice(&ser[33..65]);

    ge_to_gej((x, y))
}

// 0. ge_to_point
#[simplex::test]
fn ge_to_point_matches_parity(context: simplex::TestContext) -> anyhow::Result<()> {
    // Sample one on-curve point; whatever parity it has, that's what we expect
    // ge_to_point to produce. Covers both parities across repeated runs.
    let ge = random_ge_bytes();
    let expected_parity = ge.1[31] & 1; // 0 (even y) or 1 (odd y)

    case(GeToPoint)
        .ge(ge)
        .expect_point((expected_parity, ge.0))
        .run(&context)
}

// 1. point_to_gej
#[simplex::test]
fn point_to_gej_roundtrip(context: simplex::TestContext) -> anyhow::Result<()> {
    let ge = random_ge_bytes();
    let point = compress(ge);

    case(PointToGej)
        .point(point)
        .expect_point(point)
        .run(&context)
}

// 2. fe_sub
#[simplex::test]
fn fe_sub_self_is_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = random_fe_bytes();

    case(FeSub).uints(a, a).expect_uint([0u8; 32]).run(&context)
}

#[simplex::test]
fn fe_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = random_fe_bytes();
    let b = random_fe_bytes();
    let exp = fe_sub_ref(a, b);

    case(FeSub).uints(a, b).expect_uint(exp).run(&context)
}

// 3. scalar_sub
#[simplex::test]
fn scalar_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = random_scalar_bytes();
    let b = random_scalar_bytes();
    let exp = scalar_sub_ref(a, b);

    case(ScalarSub).uints(a, b).expect_uint(exp).run(&context)
}

// 4. gej_sub
#[simplex::test]
fn gej_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
    let secp = Secp256k1::new();
    let sk_p = SecretKey::new(&mut SecpOsRng);
    let sk_q = SecretKey::new(&mut SecpOsRng);

    let p = PublicKey::from_secret_key(&secp, &sk_p);
    let q = PublicKey::from_secret_key(&secp, &sk_q);
    let diff = p.combine(&q.negate(&secp)).expect("p - q non-infinity");

    case(GejSub)
        .gejs(pk_to_gej(&p), pk_to_gej(&q))
        .expect_gej(pk_to_gej(&diff))
        .run(&context)
}

// 5. fe_eq
#[simplex::test]
fn fe_eq_reflexive(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = random_fe_bytes();

    case(FeEq).uints(a, a).run(&context)
}

#[simplex::test]
fn fe_eq_zero_and_p_are_equal(context: simplex::TestContext) -> anyhow::Result<()> {
    case(FeEq).uints([0u8; 32], SECP_P).run(&context)
}

// 6. scalar_eq
#[simplex::test]
fn scalar_eq_s_and_s_plus_n_are_equal(context: simplex::TestContext) -> anyhow::Result<()> {
    // s = 5. s + n has no carry past the low byte.
    let mut s = [0u8; 32];
    s[31] = 5;

    let mut s_plus_n = SECP_N;
    s_plus_n[31] = s_plus_n[31].wrapping_add(5);

    case(ScalarEq).uints(s, s_plus_n).run(&context)
}

// 7. ge_eq
#[simplex::test]
fn ge_eq_rejects_negation(context: simplex::TestContext) -> anyhow::Result<()> {
    let ge = random_ge_bytes();
    let neg_y = fe_negate_ref(ge.1);

    case(GeEq)
        .ges(ge, (ge.0, neg_y))
        .expecting(&context, Expect::AssertFailed)
}

// 8. gej_point_eq
#[simplex::test]
fn gej_point_eq_rescaled(context: simplex::TestContext) -> anyhow::Result<()> {
    // (λ²X, λ³Y, λZ) is the same affine point as (X, Y, 1).
    let ge = random_ge_bytes();
    let lambda = random_fe_bytes();
    let l2 = fe_mul_ref(lambda, lambda);
    let l3 = fe_mul_ref(l2, lambda);
    let g = ((fe_mul_ref(ge.0, l2), fe_mul_ref(ge.1, l3)), lambda);

    case(GejPointEq).gej(g).point(compress(ge)).run(&context)
}

#[simplex::test]
fn gej_point_eq_rejects_negation(context: simplex::TestContext) -> anyhow::Result<()> {
    // Property under test: gej_point_eq(P, -P) == false.
    //
    // Construction:
    //   ge          = random on-curve point P = (x, y)
    //   (parity, x) = compressed form of +P
    //   ge_to_gej(ge)      = Gej encoding of +P
    //   (parity ^ 1, x)    = compressed form of -P
    //
    // decompress((parity ^ 1, x)) inside gej_point_eq recovers (x, -y) = -P.
    // Since P ≠ -P on secp256k1, the equivalence must return false.
    // expected_bool = false locks that in.
    let ge = random_ge_bytes();
    let (parity, x) = compress(ge);

    case(GejPointEq)
        .gej(ge_to_gej(ge))
        .point((parity ^ 1, x))
        .expecting(&context, Expect::AssertFailed)
}

// 9. safe_gej_normalize
#[simplex::test]
fn safe_gej_normalize_roundtrip(context: simplex::TestContext) -> anyhow::Result<()> {
    let ge = random_ge_bytes();

    case(SafeGejNormalize)
        .gej(ge_to_gej(ge))
        .expect_ge(ge)
        .run(&context)
}

mod fuzz {
    use std::fmt::Debug;

    use super::*;

    use crate::common::core::FuzzExecutionCheck;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::{Just, any};
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);

    type Ge = ([u8; 32], [u8; 32]);
    type Gej = (Ge, [u8; 32]);

    type Secp256k1FuzzEngineBuilder = FuzzEngineBuilder<
        Secp256k1OperationsTestProgram,
        Secp256k1OperationsTestArguments,
        Secp256k1OperationsTestWitness,
    >;

    fn arb_fe() -> impl Strategy<Value = [u8; 32]> {
        any::<[u8; 32]>().prop_filter("valid secp256k1 field element", |value| *value < SECP_P)
    }

    fn arb_non_zero_fe() -> impl Strategy<Value = [u8; 32]> {
        arb_fe().prop_filter("non-zero secp256k1 field element", |value| {
            *value != [0; 32]
        })
    }

    fn arb_scalar() -> impl Strategy<Value = [u8; 32]> {
        any::<[u8; 32]>().prop_filter("valid secp256k1 scalar", |value| *value < SECP_N)
    }

    fn arb_secret_key() -> impl Strategy<Value = SecretKey> {
        any::<[u8; 32]>().prop_filter_map("valid secp256k1 secret key", |bytes| {
            SecretKey::from_slice(&bytes).ok()
        })
    }

    fn ge_from_secret_key(secret_key: SecretKey) -> Ge {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let serialized = public_key.serialize_uncompressed();

        let mut x = [0; 32];
        x.copy_from_slice(&serialized[1..33]);

        let mut y = [0; 32];
        y.copy_from_slice(&serialized[33..65]);

        (x, y)
    }

    fn arb_ge() -> impl Strategy<Value = Ge> {
        arb_secret_key().prop_map(ge_from_secret_key)
    }

    /// Fuzz counterparts for every `#[simplex::test]` case with randomized inputs.
    /// Deterministic cases stay as regular tests.
    struct CaseFuzz {
        cases: BoxedStrategy<Case>,
        builder: Secp256k1FuzzEngineBuilder,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: Secp256k1FuzzEngineBuilder,
        test_name: &'static str,
    ) -> CaseFuzz {
        CaseFuzz {
            cases: Just(case(function)).boxed(),
            builder,
            test_name,
            expect: Expect::Ok,
        }
    }

    impl CaseFuzz {
        fn inject<T>(
            mut self,
            values: impl Strategy<Value = T> + 'static,
            inject: impl Fn(Case, T) -> Case + 'static,
        ) -> Self
        where
            T: Debug + 'static,
        {
            self.cases = (self.cases, values)
                .prop_map(move |(case, values)| inject(case, values))
                .boxed();
            self
        }

        fn expect(mut self, expect: Expect) -> Self {
            self.expect = expect;
            self
        }

        fn build_initial_tx() -> FinalTransaction {
            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
            tx
        }

        fn run(self) -> anyhow::Result<()> {
            let strategy = self
                .cases
                .prop_map(|case| {
                    let arguments: Arguments = Secp256k1OperationsTestArguments {}.into();
                    let witness: WitnessValues = case.witness.into();

                    (arguments, witness)
                })
                .boxed();

            let strategy = FuzzStrategyBuilder::<
                Secp256k1OperationsTestArguments,
                Secp256k1OperationsTestWitness,
                _,
            >::new()
            .with_custom_strategy(strategy)
            .build();

            let transaction_builder =
                FinalTransactionBuilder::new(Self::build_initial_tx(), [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, transaction_builder)
                .run_with_check(FuzzExecutionCheck::new(self.test_name, self.expect));

            Ok(())
        }
    }

    // 0. ge_to_point
    #[simplex::fuzz]
    fn ge_to_point_matches_parity(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        // Sample one on-curve point; whatever parity it has, that's what we expect
        // ge_to_point to produce.
        case_fuzz(GeToPoint, fuzz_engine_builder, "ge_to_point")
            .inject(arb_ge(), |case, ge| {
                let expected_parity = ge.1[31] & 1; // 0 (even y) or 1 (odd y)

                case.ge(ge).expect_point((expected_parity, ge.0))
            })
            .run()
    }

    // 1. point_to_gej
    #[simplex::fuzz]
    fn point_to_gej_roundtrip(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(PointToGej, fuzz_engine_builder, "point_to_gej")
            .inject(arb_ge(), |case, ge| {
                let point = compress(ge);
                case.point(point).expect_point(point)
            })
            .run()
    }

    // 2. fe_sub
    #[simplex::fuzz]
    fn fe_sub_self_is_zero(fuzz_engine_builder: Secp256k1FuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(FeSub, fuzz_engine_builder, "fe_sub self is zero")
            .inject(arb_fe(), |case, a| case.uints(a, a).expect_uint([0u8; 32]))
            .run()
    }

    #[simplex::fuzz]
    fn fe_sub_matches_reference(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(FeSub, fuzz_engine_builder, "fe_sub")
            .inject((arb_fe(), arb_fe()), |case, (a, b)| {
                let exp = fe_sub_ref(a, b);
                case.uints(a, b).expect_uint(exp)
            })
            .run()
    }

    // 3. scalar_sub
    #[simplex::fuzz]
    fn scalar_sub_matches_reference(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(ScalarSub, fuzz_engine_builder, "scalar_sub")
            .inject((arb_scalar(), arb_scalar()), |case, (a, b)| {
                let exp = scalar_sub_ref(a, b);
                case.uints(a, b).expect_uint(exp)
            })
            .run()
    }

    // 4. gej_sub
    #[simplex::fuzz]
    fn gej_sub_matches_reference(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(GejSub, fuzz_engine_builder, "gej_sub")
            .inject(
                (arb_secret_key(), arb_secret_key())
                    .prop_filter("distinct secp256k1 points", |(p, q)| p != q),
                |case, (p_secret, q_secret)| {
                    let secp = Secp256k1::new();
                    let p = PublicKey::from_secret_key(&secp, &p_secret);
                    let q = PublicKey::from_secret_key(&secp, &q_secret);
                    let diff = p.combine(&q.negate(&secp)).expect("p - q non-infinity");

                    case.gejs(pk_to_gej(&p), pk_to_gej(&q))
                        .expect_gej(pk_to_gej(&diff))
                },
            )
            .run()
    }

    // 5. fe_eq
    #[simplex::fuzz]
    fn fe_eq_reflexive(fuzz_engine_builder: Secp256k1FuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(FeEq, fuzz_engine_builder, "fe_eq_reflexive")
            .inject(arb_fe(), |case, value| case.uints(value, value))
            .run()
    }

    // 6. scalar_eq has only a deterministic case and remains a regular test.

    // 7. ge_eq
    #[simplex::fuzz]
    fn ge_eq_rejects_negation(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(GeEq, fuzz_engine_builder, "ge_eq negation")
            .inject(arb_ge(), |case, ge| {
                case.ges(ge, (ge.0, fe_negate_ref(ge.1)))
            })
            .expect(Expect::AssertFailed)
            .run()
    }

    // 8. gej_point_eq
    #[simplex::fuzz]
    fn gej_point_eq_rescaled(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(GejPointEq, fuzz_engine_builder, "gej_point_eq_rescaled")
            .inject((arb_ge(), arb_non_zero_fe()), |case, (ge, lambda)| {
                let lambda_squared = fe_mul_ref(lambda, lambda);
                let lambda_cubed = fe_mul_ref(lambda_squared, lambda);
                let gej: Gej = (
                    (
                        fe_mul_ref(ge.0, lambda_squared),
                        fe_mul_ref(ge.1, lambda_cubed),
                    ),
                    lambda,
                );

                case.gej(gej).point(compress(ge))
            })
            .run()
    }

    #[simplex::fuzz]
    fn gej_point_eq_rejects_negation(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(GejPointEq, fuzz_engine_builder, "gej_point_eq negation")
            .inject(arb_ge(), |case, ge| {
                let (parity, x) = compress(ge);
                case.gej(ge_to_gej(ge)).point((parity ^ 1, x))
            })
            .expect(Expect::AssertFailed)
            .run()
    }

    // 9. safe_gej_normalize
    #[simplex::fuzz]
    fn safe_gej_normalize_roundtrip(
        fuzz_engine_builder: Secp256k1FuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            SafeGejNormalize,
            fuzz_engine_builder,
            "safe_gej_normalize_roundtrip",
        )
        .inject((arb_ge(), arb_non_zero_fe()), |case, (ge, lambda)| {
            let lambda_squared = fe_mul_ref(lambda, lambda);
            let lambda_cubed = fe_mul_ref(lambda_squared, lambda);
            let gej: Gej = (
                (
                    fe_mul_ref(ge.0, lambda_squared),
                    fe_mul_ref(ge.1, lambda_cubed),
                ),
                lambda,
            );

            case.gej(gej).expect_ge(ge)
        })
        .run()
    }
}
