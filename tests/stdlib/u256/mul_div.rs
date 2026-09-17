use primitive_types::U256;
use std::cmp::max;
use std::ops::Div;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::mul_div::MulDivProgram as U256MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u256::mul_div::derived_mul_div::{
    MulDivArguments as U256MulDivTestArguments, MulDivWitness as U256MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U256MulDivTestProgram {
    U256MulDivTestProgram::new(U256MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256MulDivTestWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            third_arg: [0; 32],
            expected: None,
        },
    }
}

impl Case {
    /// The three operands, `first_arg`, `second_arg` and `third_arg`.
    fn args(mut self, a: [u8; 32], b: [u8; 32], c: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = Some(expected);
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

fn safe_u512_to_u256(a: [u8; 64]) -> [u8; 32] {
    let high = U256::from_big_endian(&a[0..32]);
    let low = U256::from_big_endian(&a[32..64]);

    assert!(high == U256::zero());

    low.to_big_endian()
}

#[simplex::test]
fn mul_div_256_product_fits_into_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u128::MAX));
    let b = generate_u256(U256::zero(), U256::from(u128::MAX));
    let c = generate_u256(U256::one(), U256::MAX);

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(2), U256::MAX);
    let b = U256::MAX;
    let c = generate_u256(a, U256::MAX);

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(2), U256::MAX);
    let b = U256::MAX;
    let c = generate_u256(U256::one(), a);

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_256_remainder_is_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let b = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let c = a;

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(b.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn mul_div_256_denominator_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let c = generate_u256(b, U256::from(u128::MAX));

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_min_denom_high(context: simplex::TestContext) -> anyhow::Result<()> {
    let pow129: U256 = U256::from(2).pow(U256::from(129));

    let a = generate_u256(U256::from(u128::MAX) + 1, pow129);
    let b = generate_u256(U256::from(u128::MAX) + 1, pow129);
    let c = generate_u256(U256::from(u128::MAX) + 1, pow129 - 1);

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::MAX);
    let c = U256::zero();

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect([0; 32])
        .run(&context)
}

#[simplex::test]
fn mul_div_256_algorithm_d_512_256_check(context: simplex::TestContext) -> anyhow::Result<()> {
    let pow2_128: U256 = U256::from(u128::MAX) + 1;

    let a = generate_u256(pow2_128, U256::MAX);
    let b = generate_u256(pow2_128, U256::MAX - pow2_128);

    let product = a.full_mul(b);
    let result_high = U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

    let c = generate_u256(max(U256::from(u128::MAX) + 1, result_high + 1), U256::MAX);

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_algorithm_d_512_256_c_is_res_high(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let pow2_128: U256 = U256::from(u128::MAX) + 1;

    let a: U256 = generate_u256(pow2_128, U256::MAX);
    let b = generate_u256(pow2_128, U256::MAX - pow2_128);

    let product = a.full_mul(b);
    let result_high = U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

    let c = result_high + 1;

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_normalize_to_threshold_512_127_norm_is_1(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let pow2_128: U256 = U256::from(u128::MAX) + 1;

    let a: U256 = generate_u256(pow2_128, U256::MAX);
    let b = generate_u256(pow2_128, U256::MAX - pow2_128);

    let product = a.full_mul(b);
    let result_high = U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

    let c = generate_u256(
        max(U256::from(2).pow(U256::from(255)), result_high + 1),
        U256::MAX,
    );

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_normalize_to_threshold_512_127_norm_greater_than_1(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let b = generate_u256(
        U256::from(u128::MAX) + 1,
        U256::from(2).pow(U256::from(192)),
    );

    let product = a.full_mul(b);
    let result_high =
        U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian()))) + 1;

    let c = generate_u256(
        max(U256::from(u128::MAX) + 1, result_high),
        U256::from(2).pow(U256::from(255)) - 1,
    );

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}
