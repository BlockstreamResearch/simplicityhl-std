use crate::common::core::Expect;

use super::core::*;

// 0. ge_to_point
#[simplex::test]
fn ge_to_point_matches_parity(context: simplex::TestContext) -> anyhow::Result<()> {
    // Sample one on-curve point; whatever parity it has, that's what we expect
    // ge_to_point to produce. Covers both parities across repeated runs.

    cases::ge_to_point(random_ge()).run(&context)
}

// 1. point_to_gej
#[simplex::test]
fn point_to_gej_roundtrip(context: simplex::TestContext) -> anyhow::Result<()> {
    cases::point_to_gej(random_ge()).run(&context)
}

// 2. fe_sub
#[simplex::test]
fn fe_sub_self_is_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = random_fe();

    cases::fe_sub(a, a).expect_uint(ZERO).run(&context)
}

#[simplex::test]
fn fe_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
    cases::fe_sub(random_fe(), random_fe()).run(&context)
}

// 3. scalar_sub
#[simplex::test]
fn scalar_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
    cases::scalar_sub(random_scalar(), random_scalar()).run(&context)
}

// 4. gej_sub
#[simplex::test]
fn gej_sub_matches_reference(context: simplex::TestContext) -> anyhow::Result<()> {
    let p = random_public_key();
    let mut q = random_public_key();

    while q == p {
        q = random_public_key();
    }

    cases::gej_sub(p, q).run(&context)
}

// 5. fe_eq
#[simplex::test]
fn fe_eq_reflexive(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = random_fe();

    cases::fe_eq(a, a).run(&context)
}

#[simplex::test]
fn fe_eq_zero_and_p_are_equal(context: simplex::TestContext) -> anyhow::Result<()> {
    cases::fe_eq(ZERO, SECP_P).run(&context)
}

// 6. scalar_eq
#[simplex::test]
fn scalar_eq_s_and_s_plus_n_are_equal(context: simplex::TestContext) -> anyhow::Result<()> {
    // s = 5. s + n has no carry past the low byte.
    let s = u256(5);

    cases::scalar_eq(s, add_u256(s, SECP_N)).run(&context)
}

// 7. ge_eq
#[simplex::test]
fn ge_eq_rejects_negation(context: simplex::TestContext) -> anyhow::Result<()> {
    cases::ge_eq_negation(random_ge()).expecting(&context, Expect::AssertFailed)
}

// 8. gej_point_eq
#[simplex::test]
fn gej_point_eq_rescaled(context: simplex::TestContext) -> anyhow::Result<()> {
    // (λ²X, λ³Y, λZ) is the same affine point as (X, Y, 1).
    let mut lambda = random_fe();

    while lambda == ZERO {
        lambda = random_fe();
    }

    cases::gej_point_eq_rescaled(random_ge(), lambda).run(&context)
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

    cases::gej_point_eq_negation(random_ge()).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn gej_point_eq_off_curve_is_false(context: simplex::TestContext) -> anyhow::Result<()> {
    // x = 0 is off-curve.
    cases::gej_point_eq_off_curve_point(random_ge(), ZERO).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn gej_point_eq_rejects_off_curve_gej(context: simplex::TestContext) -> anyhow::Result<()> {
    // (x, 1, 1) shares x with P but is off-curve.
    // The compressed point shares x and odd parity, isolating the on-curve check.
    let neg_one = sub_mod(ZERO, ONE, SECP_P);
    let mut ge = random_ge();
    while ge.1 == ONE || ge.1 == neg_one {
        ge = random_ge();
    }

    cases::gej_point_eq_off_curve_gej(ge.0, ONE).expecting(&context, Expect::AssertFailed)
}

// 9. safe_gej_normalize
#[simplex::test]
fn safe_gej_normalize_roundtrip(context: simplex::TestContext) -> anyhow::Result<()> {
    cases::safe_gej_normalize(random_ge(), ONE).run(&context)
}
