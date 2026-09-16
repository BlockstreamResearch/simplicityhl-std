use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::math::primitives::PrimitivesProgram as U128BasicMathTestProgram;
use simplicityhl_std::artifacts::tests::u128::math::primitives::derived_primitives::{
    PrimitivesArguments as U128BasicMathTestArguments,
    PrimitivesWitness as U128BasicMathTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Add128,
    Add128_64,
    FullAdd128,
    Sub128,
    FullSub128,
    Mul128,
    Mul128_64,
    CalculateNormalizerBase64,
    EstimateQuotientDigitBase64,
    DivMod128_64,
    DivMod128,
    Div128,
}

fn program() -> U128BasicMathTestProgram {
    U128BasicMathTestProgram::new(&U128BasicMathTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U128BasicMathTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U128BasicMathTestWitness {
            function_index: function as u8,
            first_arg: 0,
            second_arg: 0,
            expected: None,
            expected_bool: false,
            second_expected: 0,
        },
    }
}

impl Case {
    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: u128, b: u128) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u128) -> Self {
        self.witness.expected = Some(expected);
        self
    }

    /// `expected_bool`: the carry or borrow an arm reports beside its value.
    fn flag(mut self, flag: bool) -> Self {
        self.witness.expected_bool = flag;
        self
    }

    /// `second_expected`: the spare slot. A remainder, a low word, or a carry
    /// fed back in, depending on the arm.
    fn second(mut self, second: u128) -> Self {
        self.witness.second_expected = second;
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

fn split_helper(a: U256) -> (u128, u128) {
    let a_high = (a >> 128).as_u128();
    let a_low = a.low_u128();

    (a_high, a_low)
}

#[simplex::test]
fn add_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let result = a + b;

    case(Add128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn add_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = b - 1;

    case(Add128)
        .args(a, b)
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn add_128_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
    let result = a + b;

    case(Add128_64).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn add_128_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u64::MAX) as u128;
    let result = b - 1;

    case(Add128_64)
        .args(a, b)
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn full_add_128_not_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let result = a + b;
    let result_carry = false;
    let carry_low = 0_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn full_add_128_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = b - 1;
    let result_carry = true;
    let carry_low = 0_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn full_add_128_not_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let result = a + b + 1;
    let result_carry = false;
    let carry_low = 1_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn full_add_128_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = b;
    let result_carry = true;
    let carry_low = 1_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn sub_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..=a);
    let result = a - b;

    case(Sub128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn sub_128_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(Sub128).args(a, a).expect(0).run(&context)
}

#[simplex::test]
fn sub_128_a_low_eq_b_low(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

    let low: u64 = a as u64;
    let b = ((b_high as u128) << 64) | (low as u128);

    let carry = a < b;
    let result = a.wrapping_sub(b);

    case(Sub128)
        .args(a, b)
        .expect(result)
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_128_diff_is_u64_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a_low: u64 = u64::MAX;

    let a_high = rand::thread_rng().gen_range(0..=u64::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

    let a = ((a_high as u128) << 64) | (a_low as u128);
    let b = (b_high as u128) << 64; // b_low is 0

    let carry = a < b;
    let result = a.wrapping_sub(b);

    case(Sub128)
        .args(a, b)
        .expect(result)
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_128_diff_is_u128_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = 0;

    case(Sub128).args(a, b).expect(a).run(&context)
}

#[simplex::test]
fn sub_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = u128::MAX;
    let result = a + 1;

    case(Sub128)
        .args(a, b)
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_borrow_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..=a);
    let result = a - b;
    let result_borrow = false;
    let borrow_low = 0_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_overflow_borrow_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = u128::MAX;
    let result = a + 1;
    let result_borrow = true;
    let borrow_low = 0_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_borrow_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..a);
    let result = a - b - 1;
    let result_borrow = false;
    let borrow_low = 1_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_overflow_borrow_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = u128::MAX;
    let (result, result_borrow) = a.overflowing_sub(b);

    let borrow_low = 1_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result - 1)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn mul_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(0..u128::MAX);
    let result = U256::from(a) * U256::from(b);

    let (result_high, result_low) = split_helper(result);

    case(Mul128)
        .args(a, b)
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_128_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(0..u64::MAX);
    let result = U256::from(a) * U256::from(b);

    let (result_high, result_low) = split_helper(result);

    case(Mul128_64)
        .args(a, b as u128)
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u128 << 63;

    let b = rand::thread_rng().gen_range(1..threshold);

    let norm: u128 = threshold.div_ceil(b);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(norm)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_big_enough_not_normalize(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let threshold = 1u128 << 63;

    let b = rand::thread_rng().gen_range(threshold..=u64::MAX as u128);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(1)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u128 << 63;

    let b = rand::thread_rng().gen_range((u64::MAX as u128) + 1..=u128::MAX);
    let b_high = b >> 64;

    let norm: u128 = threshold.div_ceil(b_high);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(norm)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u64_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range((u64::MAX as u128) + 1..=u128::MAX);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u128_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(0)
        .flag(true)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_zero_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let b = 0;

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn estimate_quotient_digit_base_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u64 << 63;

    let b_high = rand::thread_rng().gen_range(threshold..=u64::MAX);
    let b_low = rand::thread_rng().gen_range(0..=u64::MAX);

    let a_high = rand::thread_rng().gen_range(0..b_high);
    let a_low = rand::thread_rng().gen_range(0..=u128::MAX);

    let a = ((U256::from(a_high)) << 128) | (U256::from(a_low));
    let b = ((b_high as u128) << 64) | (b_low as u128);

    let q = (a / b).as_u128();

    case(EstimateQuotientDigitBase64)
        .args(a_high as u128, a_low)
        .expect(q)
        .second(b)
        .run(&context)
}

#[simplex::test]
fn estimate_quotient_digit_base_64_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    // expected to fail because a is to big for q to fit unto u64
    let threshold = 1u64 << 63;

    let b_high = rand::thread_rng().gen_range(threshold..u64::MAX);
    let b_low = rand::thread_rng().gen_range(0..=u64::MAX);

    let a_high = rand::thread_rng().gen_range(b_high + 1..=u64::MAX);
    let a_low = rand::thread_rng().gen_range(0..=u128::MAX);

    let a = ((U256::from(a_high)) << 128) | (U256::from(a_low));
    let b = ((b_high as u128) << 64) | (b_low as u128);

    let q = (a / b).as_u128();

    case(EstimateQuotientDigitBase64)
        .args(a_high as u128, a_low)
        .expect(q)
        .second(b)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn div_mod_128_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);

    let q = a / b;
    let r = a % b;

    case(DivMod128_64)
        .args(a, b)
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = 0;

    case(DivMod128_64)
        .args(a, b)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn div_mod_128_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(a + 1..=u128::MAX);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_div_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(1..=u64::MAX);
    let a = rand::thread_rng().gen_range(b..=u64::MAX) as u128;

    let q = a / b as u128;
    let r = a % b as u128;

    case(DivMod128)
        .args(a, b as u128)
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_q_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
    // case where a >= b and a_high = b_high != 0
    let b_low = rand::thread_rng().gen_range(0..=u64::MAX);
    let a_low = rand::thread_rng().gen_range(b_low..=u64::MAX);
    let high = rand::thread_rng().gen_range(1..u64::MAX);

    let a = ((high as u128) << 64) | (a_low as u128);
    let b = ((high as u128) << 64) | (b_low as u128);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_b_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);
    let a = rand::thread_rng().gen_range(u64::MAX as u128 + 1..=u128::MAX);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_b_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let b_high = rand::thread_rng().gen_range(1..u64::MAX);
    let a_high = rand::thread_rng().gen_range(b_high + 1..=u64::MAX);

    let a = ((a_high as u128) << 64) | (rand::thread_rng().gen_range(0..u64::MAX) as u128);
    let b = ((b_high as u128) << 64) | (rand::thread_rng().gen_range(0..u64::MAX) as u128);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_a_equal_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);

    case(DivMod128)
        .args(a, a)
        .expect(1u128)
        .second(0u128)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_equal_high_words_max_low_diff(context: simplex::TestContext) -> anyhow::Result<()> {
    let high = rand::thread_rng().gen_range(1..=u64::MAX);

    let a = ((high as u128) << 64) | (u64::MAX as u128);
    let b = (high as u128) << 64;

    case(DivMod128)
        .args(a, b)
        .expect(1u128)
        .second(u64::MAX as u128)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_eq_high_words_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let high = rand::thread_rng().gen_range(1..=u64::MAX);

    let a = (high as u128) << 64;
    let b = ((high as u128) << 64) | (u64::MAX as u128);

    case(DivMod128)
        .args(a, b)
        .expect(0u128)
        .second(a)
        .run(&context)
}

#[simplex::test]
fn div_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = a / b;

    case(Div128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn div_128_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = 0;

    case(Div128).args(a, b).expect(0).run(&context)
}
