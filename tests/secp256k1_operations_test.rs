mod common;

use num_bigint::BigUint;
use num_traits::Zero;
use rand::{RngCore, rngs::OsRng};
use secp256k1_zkp::{PublicKey, Secp256k1, SecretKey, rand::rngs::OsRng as SecpOsRng};

use common::core::{Expect, run};

use simplicityhl_std::artifacts::secp256k1_operations_test::Secp256k1OperationsTestProgram;
use simplicityhl_std::artifacts::secp256k1_operations_test::derived_secp256k1_operations_test::{
    Secp256k1OperationsTestArguments, Secp256k1OperationsTestWitness,
};

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

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_UINT: [u8; 32] = [0u8; 32];
const DEFAULT_GE: ([u8; 32], [u8; 32]) = ([0u8; 32], [0u8; 32]);
const DEFAULT_GEJ: (([u8; 32], [u8; 32]), [u8; 32]) = (([0u8; 32], [0u8; 32]), [0u8; 32]);
const DEFAULT_POINT: (u8, [u8; 32]) = (0, [0u8; 32]);

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

#[allow(clippy::too_many_arguments)]
fn build_witness(
    function: u8,
    first_uint: [u8; 32],
    second_uint: [u8; 32],
    first_ge: ([u8; 32], [u8; 32]),
    second_ge: ([u8; 32], [u8; 32]),
    first_gej: (([u8; 32], [u8; 32]), [u8; 32]),
    second_gej: (([u8; 32], [u8; 32]), [u8; 32]),
    first_point: (u8, [u8; 32]),
    expected_uint: [u8; 32],
    expected_ge: ([u8; 32], [u8; 32]),
    expected_gej: (([u8; 32], [u8; 32]), [u8; 32]),
    expected_point: (u8, [u8; 32]),
) -> Secp256k1OperationsTestWitness {
    Secp256k1OperationsTestWitness {
        function_index: function,
        first_uint,
        second_uint,
        first_ge,
        second_ge,
        first_gej,
        second_gej,
        first_point,
        expected_uint,
        expected_ge,
        expected_gej,
        expected_point,
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

mod secp256k1_operations_tests {
    use super::*;

    // 0. ge_to_point
    #[simplex::test]
    fn ge_to_point_matches_parity(context: simplex::TestContext) -> anyhow::Result<()> {
        // Sample one on-curve point; whatever parity it has, that's what we expect
        // ge_to_point to produce. Covers both parities across repeated runs.
        let ge = random_ge_bytes();
        let expected_parity = ge.1[31] & 1; // 0 (even y) or 1 (odd y)

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::GeToPoint),
                DEFAULT_UINT,
                DEFAULT_UINT,
                ge,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                (expected_parity, ge.0),
            ),
            Expect::Ok,
        )
    }

    // 1. point_to_gej
    #[simplex::test]
    fn point_to_gej_roundtrip(context: simplex::TestContext) -> anyhow::Result<()> {
        let ge = random_ge_bytes();
        let point = compress(ge);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::PointToGej),
                DEFAULT_UINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                point,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                point,
            ),
            Expect::Ok,
        )
    }

    // 2. fe_sub
    #[simplex::test]
    fn fe_sub_self_is_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = random_fe_bytes();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FeSub),
                a,
                a,
                DEFAULT_GE,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                [0u8; 32],
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn fe_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = random_fe_bytes();
        let b = random_fe_bytes();
        let exp = fe_sub_ref(a, b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FeSub),
                a,
                b,
                DEFAULT_GE,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                exp,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
    }

    // 3. scalar_sub
    #[simplex::test]
    fn scalar_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = random_scalar_bytes();
        let b = random_scalar_bytes();
        let exp = scalar_sub_ref(a, b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::ScalarSub),
                a,
                b,
                DEFAULT_GE,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                exp,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
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

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::GejSub),
                DEFAULT_UINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GE,
                pk_to_gej(&p),
                pk_to_gej(&q),
                DEFAULT_POINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                pk_to_gej(&diff),
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
    }

    // 5. fe_eq
    #[simplex::test]
    fn fe_eq_reflexive(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = random_fe_bytes();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FeEq),
                a,
                a,
                DEFAULT_GE,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn fe_eq_zero_and_p_are_equal(context: simplex::TestContext) -> anyhow::Result<()> {
        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FeEq),
                [0u8; 32],
                SECP_P,
                DEFAULT_GE,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
    }

    // 6. scalar_eq
    #[simplex::test]
    fn scalar_eq_s_and_s_plus_n_are_equal(context: simplex::TestContext) -> anyhow::Result<()> {
        // s = 5. s + n has no carry past the low byte.
        let mut s = [0u8; 32];
        s[31] = 5;

        let mut s_plus_n = SECP_N;
        s_plus_n[31] = s_plus_n[31].wrapping_add(5);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::ScalarEq),
                s,
                s_plus_n,
                DEFAULT_GE,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
    }

    // 7. ge_eq
    #[simplex::test]
    fn ge_eq_rejects_negation(context: simplex::TestContext) -> anyhow::Result<()> {
        let ge = random_ge_bytes();
        let neg_y = fe_negate_ref(ge.1);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::GeEq),
                DEFAULT_UINT,
                DEFAULT_UINT,
                ge,
                (ge.0, neg_y),
                DEFAULT_GEJ,
                DEFAULT_GEJ,
                DEFAULT_POINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::AssertFailed,
        )
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

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::GejPointEq),
                DEFAULT_UINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GE,
                g,
                DEFAULT_GEJ,
                compress(ge),
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
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

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::GejPointEq),
                DEFAULT_UINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GE,
                ge_to_gej(ge),
                DEFAULT_GEJ,
                (parity ^ 1, x),
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::AssertFailed,
        )
    }

    // 9. safe_gej_normalize
    #[simplex::test]
    fn safe_gej_normalize_roundtrip(context: simplex::TestContext) -> anyhow::Result<()> {
        let ge = random_ge_bytes();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeGejNormalize),
                DEFAULT_UINT,
                DEFAULT_UINT,
                DEFAULT_GE,
                DEFAULT_GE,
                ge_to_gej(ge),
                DEFAULT_GEJ,
                DEFAULT_POINT,
                DEFAULT_UINT,
                ge,
                DEFAULT_GEJ,
                DEFAULT_POINT,
            ),
            Expect::Ok,
        )
    }
}
