use secp256k1_zkp::{PublicKey, SECP256K1, SecretKey};

use simplex::fuzz::proptest::prelude::{Just, any};
use simplex::fuzz::proptest::prop_oneof;
use simplex::fuzz::proptest::sample::select;
use simplex::fuzz::proptest::strategy::Strategy;
use simplex::{FuzzMode, TestContext};

use crate::common::fuzz::{accepts, rejects};
use crate::secp256k1::math::{
    Fe, Ge, ONE, SECP_N, SECP_P, U256, ZERO, add_u256, decompress, pk_to_ge, sub_mod, sub_u256,
    u256,
};

use super::case;

/// Values at and above `modulus` are non-canonical and exercise reduction in the contract.
fn arb_u256_mod(modulus: U256) -> impl Strategy<Value = U256> {
    let edges = vec![
        ZERO,
        ONE,
        sub_u256(modulus, ONE),
        modulus,
        add_u256(modulus, ONE),
        [u8::MAX; 32],
    ];

    prop_oneof![select(edges), any::<U256>()]
}

fn arb_fe() -> impl Strategy<Value = Fe> {
    arb_u256_mod(SECP_P).prop_filter("canonical field element", |value| *value < SECP_P)
}

fn arb_non_zero_fe() -> impl Strategy<Value = Fe> {
    arb_fe().prop_filter("non-zero field element", |value| *value != ZERO)
}

fn arb_ge() -> impl Strategy<Value = Ge> {
    any::<[u8; 32]>()
        .prop_filter_map("valid secret key", |bytes| {
            SecretKey::from_slice(&bytes).ok()
        })
        .prop_map(|secret_key| pk_to_ge(&PublicKey::from_secret_key(SECP256K1, &secret_key)))
}

/// A canonical x with no curve point: `x³ + 7` is not a square mod p.
fn arb_off_curve_x() -> impl Strategy<Value = Fe> {
    prop_oneof![Just(ZERO), arb_fe()].prop_filter("off-curve x", |x| decompress((0, *x)).is_none())
}

/// `x` and `x + modulus`: equal modulo `modulus`, the second one non-canonical.
fn arb_shifted_pair(
    x: impl Strategy<Value = U256>,
    modulus: U256,
) -> impl Strategy<Value = (U256, U256)> {
    x.prop_map(move |x| (x, add_u256(x, modulus)))
}

fn arb_distinct_mod(modulus: U256) -> impl Strategy<Value = (U256, U256)> {
    (arb_u256_mod(modulus), arb_u256_mod(modulus)).prop_filter("distinct modulo", move |(a, b)| {
        sub_mod(*a, *b, modulus) != ZERO
    })
}

// 0. ge_to_point
#[simplex::fuzz]
fn ge_to_point_matches_parity(context: TestContext<FuzzMode>) {
    accepts(&context, arb_ge(), case::ge_to_point_compresses);
}

// 1. point_to_gej
#[simplex::fuzz]
fn point_to_gej_roundtrip(context: TestContext<FuzzMode>) {
    accepts(&context, arb_ge(), case::point_to_gej_roundtrip);
}

// 2. fe_sub
#[simplex::fuzz]
fn fe_sub_self_is_zero(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u256_mod(SECP_P), |a| case::fe_sub(a, a, ZERO));
}

#[simplex::fuzz]
fn fe_sub_matches_reference(context: TestContext<FuzzMode>) {
    accepts(
        &context,
        (arb_u256_mod(SECP_P), arb_u256_mod(SECP_P)),
        |(a, b)| case::fe_sub_reference(a, b),
    );
}

// 3. scalar_sub
#[simplex::fuzz]
fn scalar_sub_matches_reference(context: TestContext<FuzzMode>) {
    accepts(
        &context,
        (arb_u256_mod(SECP_N), arb_u256_mod(SECP_N)),
        |(a, b)| case::scalar_sub_reference(a, b),
    );
}

// 4. gej_sub
#[simplex::fuzz]
fn gej_sub_matches_reference(context: TestContext<FuzzMode>) {
    let distinct_points = (arb_ge(), arb_ge()).prop_filter("distinct points", |(p, q)| p != q);

    accepts(&context, distinct_points, |(p, q)| {
        case::gej_sub_reference(p, q)
    });
}

#[simplex::fuzz]
fn gej_sub_self_is_infinity(context: TestContext<FuzzMode>) {
    accepts(&context, arb_ge(), |p| case::gej_sub_reference(p, p));
}

// 5. fe_eq
#[simplex::fuzz]
fn fe_eq_reflexive(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u256_mod(SECP_P), |a| case::fe_eq(a, a));
}

// Generalized case of fe_eq_zero_and_p_are_equal.
#[simplex::fuzz]
fn fe_eq_x_and_x_plus_p_are_equal(context: TestContext<FuzzMode>) {
    // x + p fits in 256 bits only for x < 2^256 - p = 0x1000003D1.
    let x = (0_u128..0x1_0000_03D1).prop_map(u256);

    accepts(&context, arb_shifted_pair(x, SECP_P), |(a, b)| {
        case::fe_eq(a, b)
    });
}

#[simplex::fuzz]
fn fe_eq_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct_mod(SECP_P), |(a, b)| {
        case::fe_eq(a, b)
    });
}

// 6. scalar_eq
#[simplex::fuzz]
fn scalar_eq_s_and_s_plus_n_are_equal(context: TestContext<FuzzMode>) {
    // s + n fits in 256 bits for any s < 2^128 (2^256 - n > 2^128).
    let s = any::<u128>().prop_map(u256);

    accepts(&context, arb_shifted_pair(s, SECP_N), |(a, b)| {
        case::scalar_eq(a, b)
    });
}

#[simplex::fuzz]
fn scalar_eq_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct_mod(SECP_N), |(a, b)| {
        case::scalar_eq(a, b)
    });
}

// 7. ge_eq
#[simplex::fuzz]
fn ge_eq_rejects_negation(context: TestContext<FuzzMode>) {
    rejects(&context, arb_ge(), case::ge_eq_negation);
}

// 8. gej_point_eq
#[simplex::fuzz]
fn gej_point_eq_rescaled(context: TestContext<FuzzMode>) {
    accepts(&context, (arb_ge(), arb_non_zero_fe()), |(ge, lambda)| {
        case::gej_point_eq_rescaled(ge, lambda)
    });
}

#[simplex::fuzz]
fn gej_point_eq_rejects_negation(context: TestContext<FuzzMode>) {
    rejects(&context, arb_ge(), case::gej_point_eq_negation);
}

#[simplex::fuzz]
fn gej_point_eq_off_curve_is_false(context: TestContext<FuzzMode>) {
    rejects(&context, (arb_ge(), arb_off_curve_x()), |(ge, x)| {
        case::gej_point_eq_off_curve_point(ge, x)
    });
}

#[simplex::fuzz]
fn gej_point_eq_rejects_off_curve_gej(context: TestContext<FuzzMode>) {
    // A point with y = 0 would have order two. The group has odd order.
    rejects(&context, arb_ge(), |(x, _)| {
        case::gej_point_eq_off_curve_gej(x, ZERO)
    });
}

// 9. safe_gej_normalize
#[simplex::fuzz]
fn safe_gej_normalize_roundtrip(context: TestContext<FuzzMode>) {
    accepts(&context, (arb_ge(), arb_non_zero_fe()), |(ge, lambda)| {
        case::safe_gej_normalize_rescaled(ge, lambda)
    });
}
