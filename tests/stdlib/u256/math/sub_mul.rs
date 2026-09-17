use primitive_types::U256;
use primitive_types::U512;
use rand::Rng;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::math::sub_mul::SubMulProgram as U256TestSubMulProgram;
use simplicityhl_std::artifacts::tests::u256::math::sub_mul::derived_sub_mul::{
    SubMulArguments as U256TestSubMulArguments, SubMulWitness as U256TestSubMulWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Sub256,
    Mul256,
    Mul256_64,
    Mul256_128,
    Mul512_128,
    SafeMul256_128,
}

fn program() -> U256TestSubMulProgram {
    U256TestSubMulProgram::new(U256TestSubMulArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256TestSubMulWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256TestSubMulWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            third_arg: 0,
            expected: None,
            expected_bool: false,
            second_expected: [0; 32],
            third_expected: [0; 32],
        },
    }
}

impl Case {
    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: [u8; 32], b: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// `third_arg`: the extra operand only some arms take.
    fn third_arg(mut self, c: u128) -> Self {
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = Some(expected);
        self
    }

    /// `expected_bool`: the boolean the arm should report.
    fn flag(mut self, expected_bool: bool) -> Self {
        self.witness.expected_bool = expected_bool;
        self
    }

    /// `second_expected`: the arm's second result.
    fn second(mut self, second_expected: [u8; 32]) -> Self {
        self.witness.second_expected = second_expected;
        self
    }

    /// `third_expected`: the arm's third result.
    fn third(mut self, third_expected: [u8; 32]) -> Self {
        self.witness.third_expected = third_expected;
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

fn split_u512(a: [u8; 64]) -> ([u8; 32], [u8; 32]) {
    let high = U256::from_big_endian(&a[0..32]);
    let low = U256::from_big_endian(&a[32..64]);

    (high.to_big_endian(), low.to_big_endian())
}

#[simplex::test]
fn sub_256_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::zero(), a);
    let result = (a - b).to_big_endian();

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn sub_256_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX).to_big_endian();

    case(Sub256).args(a, a).expect([0; 32]).run(&context)
}

#[simplex::test]
fn sub_256_a_low_eq_b_low(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u128::MAX);

    let low: u128 = a.low_u128();
    let b = (U256::from(b_high) << 128) | U256::from(low);

    let (result, carry) = a.overflowing_sub(b);

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result.to_big_endian())
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_256_diff_is_u128_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a_low: u128 = u128::MAX;

    let a_high = rand::thread_rng().gen_range(0..=u128::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u128::MAX);

    let a = (U256::from(a_high) << 128) | U256::from(a_low);
    let b = (U256::from(b_high)) << 128; // b_low is 0

    let (result, carry) = a.overflowing_sub(b);

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result.to_big_endian())
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_256_diff_is_u256_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX.to_big_endian();
    let b = U256::zero();

    case(Sub256)
        .args(a, b.to_big_endian())
        .expect(a)
        .run(&context)
}

#[simplex::test]
fn sub_256_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX - 1);
    let b = U256::MAX;
    let result = a + 1;

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result.to_big_endian())
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn mul_256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::MAX);
    let result = a.full_mul(b).to_big_endian();

    let (result_high, result_low) = split_u512(result);

    case(Mul256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_256_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u64::MAX));
    let result = a.full_mul(b).to_big_endian();

    let (result_high, result_low) = split_u512(result);

    case(Mul256_64)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_256_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let result = a.full_mul(b).to_big_endian();

    let (result_high, result_low) = split_u512(result);

    case(Mul256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_512_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a_1 = generate_u256(U256::one(), U256::MAX);
    let a_0 = generate_u256(U256::one(), U256::MAX);
    let b = rand::thread_rng().gen_range(1..=u128::MAX);

    let result_low = U512::from(a_0) * U512::from(b);
    let result_high = U512::from(a_1) * U512::from(b);

    let (res_1, res_0) = split_u512(result_low.to_big_endian());
    let (res_3, res_2) = split_u512(result_high.to_big_endian());

    let res_2_1 = U512::from_big_endian(&res_1) + U512::from_big_endian(&res_2);
    let (res_3_1, res_2_1) = split_u512(res_2_1.to_big_endian());

    let res_3_final =
        (U256::from_big_endian(&res_3_1) + U256::from_big_endian(&res_3)).to_big_endian();

    case(Mul512_128)
        .args(a_1.to_big_endian(), a_0.to_big_endian())
        .third_arg(b)
        .expect(res_3_final)
        .second(res_2_1)
        .third(res_0)
        .run(&context)
}

#[simplex::test]
fn safe_mul_256_128_fitting(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u128::MAX));
    let b = generate_u256(U256::zero(), U256::from(u128::MAX));

    case(SafeMul256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect((a * b).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_mul_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let b = U256::from(u128::MAX);

    case(SafeMul256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}
