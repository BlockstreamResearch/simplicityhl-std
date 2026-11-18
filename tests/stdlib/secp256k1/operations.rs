use secp256k1_zkp::PublicKey;

use super::core::*;

/// The curve point with `x = 1` yields the smallest value that fits in 256 bits after
/// computing `x + p` (preventing overflow so modular reduction can be tested).
fn x_one() -> Ge {
    let mut compressed = [0x02; 33];
    compressed[1..].copy_from_slice(&ONE);

    pk_to_ge(&PublicKey::from_slice(&compressed).expect("x = 1 is on the curve"))
}

// 0. ge_to_point
#[test]
fn ge_to_point_matches_parity() {
    // G and -G have opposite y parity (p is odd), so both branches are covered.
    let g: Ge = g_times(1);

    cases::ge_to_point(g).unit().accepts();
    cases::ge_to_point(ge_negate(g)).unit().accepts();
}

// 1. point_to_gej
#[test]
fn point_to_gej_roundtrip() {
    let g: Ge = g_times(1);

    cases::point_to_gej(g).unit().accepts();
    cases::point_to_gej(ge_negate(g)).unit().accepts()
}

#[test]
fn point_to_gej_rejects_invalid_x() {
    // Off-curve x: the point (0, 0).
    cases::point_to_gej((ZERO, ZERO)).unit().rejects();

    // Unreduced x: the point (0, p). p ≡ 0, so this is the same off-curve x.
    cases::point_to_gej((SECP_P, ZERO)).unit().rejects();
}

#[test]
fn point_to_gej_reduces_x() {
    let (x, y): (Fe, Fe) = x_one();
    let x_scaled: U256 = add_u256(x, SECP_P);

    // decompress reads x mod p: (parity, 1 + p) decompresses to the point with x = 1.
    cases::point_to_gej((x_scaled, y))
        .expect_point(compress((x, y)))
        .unit()
        .accepts();
}

// 2. fe_sub
#[test]
fn fe_sub_self_is_zero() {
    let a: U256 = [0xEF; 32];

    cases::fe_sub(a, a).expect_uint(ZERO).unit().accepts();
}

#[test]
fn fe_sub_underflow_and_unreduced() {
    let a: U256 = [0xFE; 32];
    let b: U256 = [0xFF; 32];

    cases::fe_sub(a, ZERO).unit().accepts();
    cases::fe_sub(b, ZERO).unit().accepts();

    cases::fe_sub(ZERO, ONE).unit().accepts();
    cases::fe_sub(SECP_P, ONE).unit().accepts();
    cases::fe_sub(ONE, SECP_P).unit().accepts();
}

#[test]
fn fe_sub_matches_reference() {
    let a: U256 = [0xFE; 32];
    let b: U256 = [0xEF; 32];

    cases::fe_sub(a, b).unit().accepts();
    cases::fe_sub(b, a).unit().accepts();
}

// 3. scalar_sub
#[test]
fn scalar_sub_matches_reference() {
    let a: Scalar = [0xA5; 32];
    let b: Scalar = [0x5A; 32];

    cases::scalar_sub(a, b).unit().accepts();
    cases::scalar_sub(b, a).unit().accepts();
}

#[test]
fn scalar_sub_is_zero() {
    let a: Scalar = [0xA5; 32];

    cases::scalar_sub(a, a).expect_uint(ZERO).unit().accepts();
}

#[test]
fn scalar_sub_underflow_and_unreduced() {
    let a: Scalar = [0xFF; 32];

    cases::scalar_sub(a, ZERO).unit().accepts();
    cases::scalar_sub(ZERO, ONE).unit().accepts();
    cases::scalar_sub(SECP_N, ONE).unit().accepts();
    cases::scalar_sub(ONE, SECP_N).unit().accepts();
}

// 4. gej_sub
#[test]
fn gej_sub_matches_reference() {
    let p = public_key(0x01);
    let q = public_key(0x10);

    cases::gej_sub(p, q).unit().accepts();
    cases::gej_sub(q, p).unit().accepts();
}

#[test]
fn gej_sub_edge_cases() {
    let p = public_key(0x01);
    let g: Ge = g_times(1);
    let two_g: Gej = ge_to_gej(g_times(2));

    // P - P = inf
    cases::gej_sub(p, p).unit().accepts();

    // G - (-G) = G + G = 2G
    cases::gej_sub_jacobian(ge_to_gej(g), ge_to_gej(ge_negate(g)), two_g)
        .unit()
        .accepts();

    // 3G - G = 2G
    cases::gej_sub_jacobian(
        rescale(g_times(3), [0x4E; 32]),
        rescale(g, [0x2A; 32]),
        two_g,
    )
    .unit()
    .accepts();
}

#[test]
fn gej_sub_infinity_operands() {
    let g: Gej = ge_to_gej(g_times(1));
    let minus_g: Gej = ge_to_gej(ge_negate(g_times(1)));

    // G - inf = G
    cases::gej_sub_jacobian(g, INFINITY, g).unit().accepts();

    // inf - G = -G
    cases::gej_sub_jacobian(INFINITY, g, minus_g)
        .unit()
        .accepts();

    // inf - inf = inf
    cases::gej_sub_jacobian(INFINITY, INFINITY, INFINITY)
        .unit()
        .accepts();
}

// 5. fe_eq
#[test]
fn fe_eq_reflexive() {
    let a: Fe = [0xFE; 32];

    cases::fe_eq(a, a).unit().accepts();
}

#[test]
fn fe_eq_zero_and_p_are_equal() {
    cases::fe_eq(ZERO, SECP_P).unit().accepts();
}

#[test]
fn fe_eq_rejects_distinct() {
    cases::fe_eq(ZERO, ONE).unit().rejects();
    // n ≢ 0 (mod p): catches reducing by the wrong modulus.
    cases::fe_eq(ZERO, SECP_N).unit().rejects();
}

// 6. scalar_eq
#[test]
fn scalar_eq_s_and_s_plus_n_are_equal() {
    let s: Scalar = u256(5);

    cases::scalar_eq(s, add_u256(s, SECP_N)).unit().accepts();
}

#[test]
fn scalar_eq_rejects_distinct() {
    cases::scalar_eq(ZERO, ONE).unit().rejects();
    // p ≢ 0 (mod n): catches reducing by the wrong modulus.
    cases::scalar_eq(ZERO, SECP_P).unit().rejects();
}

// 7. ge_eq
#[test]
fn ge_eq_reflexive() {
    let g: Ge = g_times(1);
    let (x, y): (Fe, Fe) = x_one();

    cases::ge_eq(g, g).unit().accepts();

    // Coordinates are compared mod p.
    cases::ge_eq((x, y), (add_u256(x, SECP_P), y))
        .unit()
        .accepts();
}

#[test]
fn ge_eq_rejects_negation() {
    let ge: Ge = g_times(1);

    cases::ge_eq_negation(ge).unit().rejects();
}

#[test]
fn ge_eq_rejects_different_x() {
    cases::ge_eq(g_times(1), g_times(2)).unit().rejects();
}

// 8. gej_point_eq
#[test]
fn gej_point_eq_rescaled() {
    // (λ²X, λ³Y, λZ) is the same affine point as (X, Y, 1).
    let lambda: Fe = [0x4E; 32];
    let ge: Ge = g_times(1);

    cases::gej_point_eq_rescaled(ge, lambda).unit().accepts();
}

#[test]
fn gej_point_eq_rejects_negation() {
    let ge: Ge = g_times(1);

    // Property under test: gej_point_eq(P, -P) == false.
    //
    // Construction:
    //   ge          = on-curve point P = (x, y), here G
    //   (parity, x) = compressed form of +P
    //   ge_to_gej(ge)      = Gej encoding of +P
    //   (parity ^ 1, x)    = compressed form of -P
    //
    // decompress((parity ^ 1, x)) inside gej_point_eq recovers (x, -y) = -P.
    // Since P ≠ -P on secp256k1, the equivalence must return false.
    // expected_bool = false locks that in.
    cases::gej_point_eq_negation(ge).unit().rejects()
}

#[test]
fn gej_point_eq_off_curve_is_false() {
    let ge: Ge = g_times(1);

    // x = 0 is off-curve.
    cases::gej_point_eq_off_curve_point(ge, ZERO)
        .unit()
        .rejects()
}

#[test]
fn gej_point_eq_rejects_off_curve_gej() {
    let ge: Ge = g_times(1);

    // (x, 1, 1) shares x with P but is off-curve.
    // The compressed point shares x and odd parity, isolating the on-curve check.
    cases::gej_point_eq_off_curve_gej(ge.0, ONE)
        .unit()
        .rejects()
}

#[test]
fn gej_point_eq_rejects_infinity() {
    let g_point: Point = compress(g_times(1));

    cases::gej_point_eq(INFINITY, g_point).unit().rejects();
}

#[test]
fn gej_point_eq_rejects_other_point() {
    let a: Gej = ge_to_gej(g_times(1));
    let b: Point = compress(g_times(2));

    cases::gej_point_eq(a, b).unit().rejects();
}

#[test]
fn gej_point_eq_reduces_x() {
    let (x, y) = x_one();
    let x_scaled = add_u256(x, SECP_P);
    let (parity, _) = compress((x, y));

    // Reduced Gej, unreduced point.
    cases::gej_point_eq(ge_to_gej((x, y)), (parity, x_scaled))
        .unit()
        .accepts();

    // Unreduced Gej, reduced point.
    cases::gej_point_eq(ge_to_gej((x_scaled, y)), (parity, x))
        .unit()
        .accepts();
}

// 9. safe_gej_normalize
#[test]
fn safe_gej_normalize_roundtrip() {
    let ge: Ge = g_times(1);

    cases::safe_gej_normalize_rescaled(ge, ONE).unit().accepts();
    cases::safe_gej_normalize_rescaled(ge, [0x4E; 32])
        .unit()
        .accepts();
}

#[test]
fn safe_gej_normalize_rejects_infinity() {
    cases::safe_gej_normalize(INFINITY, Ge::default())
        .unit()
        .rejects();
}
