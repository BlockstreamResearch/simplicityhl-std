use crate::common::case::DispatchCase;
use crate::common::core::Expect::{AssertFailed, PrunedBranch};
use crate::secp256k1::math::{
    Fe, Ge, Gej, INFINITY, ONE, SECP_N, SECP_P, Scalar, ZERO, add_u256, compress, g_times,
    ge_negate, ge_to_gej, rescale, u256, x_one,
};

use super::case;

// 0. ge_to_point
#[test]
fn ge_to_point_matches_parity() {
    // G and -G have opposite y parity (p is odd), so both branches are covered.
    let g: Ge = g_times(1);

    case::ge_to_point_compresses(g).accepts();
    case::ge_to_point_compresses(ge_negate(g)).accepts();
}

// 1. point_to_gej
#[test]
fn point_to_gej_roundtrip() {
    let g: Ge = g_times(1);

    case::point_to_gej_roundtrip(g).accepts();
    case::point_to_gej_roundtrip(ge_negate(g)).accepts();
}

#[test]
fn point_to_gej_rejects_invalid_x() {
    // Off-curve x = 0: `decompress` returns `None`.
    case::point_to_gej((0, ZERO), (0, ZERO)).rejects(PrunedBranch);

    // Unreduced x = p ≡ 0: the same off-curve x.
    case::point_to_gej((0, SECP_P), (0, ZERO)).rejects(PrunedBranch);
}

#[test]
fn point_to_gej_reduces_x() {
    let (parity, x) = compress(x_one());

    // `decompress` reads x mod p: (parity, 1 + p) decompresses to the point with x = 1.
    case::point_to_gej((parity, add_u256(x, SECP_P)), (parity, x)).accepts();
}

// 2. fe_sub
#[test]
fn fe_sub_self_is_zero() {
    let a: Fe = [0xEF; 32];

    case::fe_sub(a, a, ZERO).accepts();
}

#[test]
fn fe_sub_underflow_and_unreduced() {
    case::fe_sub_reference([0xFE; 32], ZERO).accepts();
    case::fe_sub_reference([0xFF; 32], ZERO).accepts();

    case::fe_sub_reference(ZERO, ONE).accepts();
    case::fe_sub_reference(SECP_P, ONE).accepts();
    case::fe_sub_reference(ONE, SECP_P).accepts();
}

#[test]
fn fe_sub_matches_reference() {
    let a: Fe = [0xFE; 32];
    let b: Fe = [0xEF; 32];

    case::fe_sub_reference(a, b).accepts();
    case::fe_sub_reference(b, a).accepts();
}

// 3. scalar_sub
#[test]
fn scalar_sub_matches_reference() {
    let a: Scalar = [0xA5; 32];
    let b: Scalar = [0x5A; 32];

    case::scalar_sub_reference(a, b).accepts();
    case::scalar_sub_reference(b, a).accepts();
}

#[test]
fn scalar_sub_self_is_zero() {
    let a: Scalar = [0xA5; 32];

    case::scalar_sub(a, a, ZERO).accepts();
}

#[test]
fn scalar_sub_underflow_and_unreduced() {
    case::scalar_sub_reference([0xFF; 32], ZERO).accepts();
    case::scalar_sub_reference(ZERO, ONE).accepts();
    case::scalar_sub_reference(SECP_N, ONE).accepts();
    case::scalar_sub_reference(ONE, SECP_N).accepts();
}

// 4. gej_sub
#[test]
fn gej_sub_matches_reference() {
    let p: Ge = g_times(0x01);
    let q: Ge = g_times(0x10);

    case::gej_sub_reference(p, q).accepts();
    case::gej_sub_reference(q, p).accepts();
}

#[test]
fn gej_sub_edge_cases() {
    let g: Ge = g_times(1);
    let two_g: Gej = ge_to_gej(g_times(2));

    // P - P = inf
    case::gej_sub_reference(g, g).accepts();

    // G - (-G) = G + G = 2G
    case::gej_sub(ge_to_gej(g), ge_to_gej(ge_negate(g)), two_g).accepts();

    // 3G - G = 2G, both operands with non-unit Z
    case::gej_sub(
        rescale(g_times(3), [0x4E; 32]),
        rescale(g, [0x2A; 32]),
        two_g,
    )
    .accepts();
}

#[test]
fn gej_sub_infinity_operands() {
    let g: Gej = ge_to_gej(g_times(1));
    let minus_g: Gej = ge_to_gej(ge_negate(g_times(1)));

    // G - inf = G
    case::gej_sub(g, INFINITY, g).accepts();

    // inf - G = -G
    case::gej_sub(INFINITY, g, minus_g).accepts();

    // inf - inf = inf
    case::gej_sub(INFINITY, INFINITY, INFINITY).accepts();
}

// 5. fe_eq
#[test]
fn fe_eq_reflexive() {
    let a: Fe = [0xFE; 32];

    case::fe_eq(a, a).accepts();
}

#[test]
fn fe_eq_zero_and_p_are_equal() {
    case::fe_eq(ZERO, SECP_P).accepts();
}

#[test]
fn fe_eq_rejects_distinct() {
    case::fe_eq(ZERO, ONE).rejects(AssertFailed);
    // n ≢ 0 (mod p): catches reducing by the wrong modulus.
    case::fe_eq(ZERO, SECP_N).rejects(AssertFailed);
}

// 6. scalar_eq
#[test]
fn scalar_eq_s_and_s_plus_n_are_equal() {
    let s: Scalar = u256(5);

    case::scalar_eq(s, add_u256(s, SECP_N)).accepts();
}

#[test]
fn scalar_eq_rejects_distinct() {
    case::scalar_eq(ZERO, ONE).rejects(AssertFailed);
    // p ≢ 0 (mod n): catches reducing by the wrong modulus.
    case::scalar_eq(ZERO, SECP_P).rejects(AssertFailed);
}

// 7. ge_eq
#[test]
fn ge_eq_reflexive() {
    let g: Ge = g_times(1);

    case::ge_eq(g, g).accepts();
}

#[test]
fn ge_eq_reduces_coordinates() {
    let (x, y) = x_one();

    case::ge_eq((x, y), (add_u256(x, SECP_P), y)).accepts();
}

#[test]
fn ge_eq_rejects_negation() {
    case::ge_eq_negation(g_times(1)).rejects(AssertFailed);
}

#[test]
fn ge_eq_rejects_different_x() {
    case::ge_eq(g_times(1), g_times(2)).rejects(AssertFailed);
}

// 8. gej_point_eq
#[test]
fn gej_point_eq_rescaled() {
    case::gej_point_eq_rescaled(g_times(1), [0x4E; 32]).accepts();
}

#[test]
fn gej_point_eq_rejects_negation() {
    case::gej_point_eq_negation(g_times(1)).rejects(AssertFailed);
}

#[test]
fn gej_point_eq_off_curve_is_false() {
    // x = 0 is off-curve: 7 is not a square mod p.
    case::gej_point_eq_off_curve_point(g_times(1), ZERO).rejects(AssertFailed);
}

#[test]
fn gej_point_eq_rejects_off_curve_gej() {
    // (x, 1) shares x with G but is off-curve.
    case::gej_point_eq_off_curve_gej(g_times(1).0, ONE).rejects(AssertFailed);
}

#[test]
fn gej_point_eq_rejects_infinity() {
    case::gej_point_eq(INFINITY, compress(g_times(1))).rejects(AssertFailed);
}

#[test]
fn gej_point_eq_rejects_other_point() {
    case::gej_point_eq(ge_to_gej(g_times(1)), compress(g_times(2))).rejects(AssertFailed);
}

#[test]
fn gej_point_eq_reduces_x() {
    let (x, y) = x_one();
    let x_unreduced = add_u256(x, SECP_P);
    let (parity, _) = compress((x, y));

    // Reduced Gej, unreduced point.
    case::gej_point_eq(ge_to_gej((x, y)), (parity, x_unreduced)).accepts();

    // Unreduced Gej, reduced point.
    case::gej_point_eq(ge_to_gej((x_unreduced, y)), (parity, x)).accepts();
}

// 9. safe_gej_normalize
#[test]
fn safe_gej_normalize_roundtrip() {
    let g: Ge = g_times(1);

    case::safe_gej_normalize_rescaled(g, ONE).accepts();
    case::safe_gej_normalize_rescaled(g, [0x4E; 32]).accepts();
}

#[test]
fn safe_gej_normalize_rejects_infinity() {
    // `gej_normalize` returns `None` for infinity, so `unwrap` fails before the comparison.
    case::safe_gej_normalize(INFINITY, Ge::default()).rejects(PrunedBranch);
}
