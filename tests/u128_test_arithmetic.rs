mod common;

use primitive_types::U256;
use rand::Rng;

use crate::common::helper::DEFAULT_BOOL;
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u128_test_arithmetic::U128TestArithmeticProgram;
use simplicityhl_std::artifacts::u128_test_arithmetic::derived_u128_test_arithmetic::{
    U128TestArithmeticArguments, U128TestArithmeticWitness,
};

enum FunctionToTest {
    IsZero128,
    Lt128,
    Le128,
    Add128,
    Add128_64,
    FullAdd128,
    Sub128,
    FullSub128,
    Mul128,
    CalculateNormalizerBase64,
    EstimateQuotientDigitBase64,
    DivMod128_64,
    DivMod128,
    Div128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: u128 = 0;

fn program() -> U128TestArithmeticProgram {
    U128TestArithmeticProgram::new(U128TestArithmeticArguments {})
}

fn build_witness(
    function: u8,
    a: u128,
    b: u128,
    expected: Option<u128>,
    expected_bool: bool,
    second_expected: u128,
    third_expected: u128,
) -> U128TestArithmeticWitness {
    U128TestArithmeticWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected,
        expected_bool,
        second_expected,
        third_expected,
    }
}

fn split_helper(a: U256) -> (u128, u128) {
    let a_high = (a >> 128).as_u128();
    let a_low = a.low_u128();

    (a_high, a_low)
}

mod u128_tests_arithmetic {
    use super::*;

    #[simplex::test]
    fn u128_test_is_zero_128_true(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = 0;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::IsZero128),
                a,
                DEFAULT_EXPECTED,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_is_zero_128_false(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::IsZero128),
                a,
                DEFAULT_EXPECTED,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_lt_128_less(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = a + 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Lt128),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_lt_128_eq(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = a;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Lt128),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_lt_128_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u128::MAX);
        let b = a - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Lt128),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_le_128_less(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = a + 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Le128),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_le_128_eq(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = a;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Le128),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_le_128_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u128::MAX);
        let b = a - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Le128),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_add_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
        let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
        let result = a + b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add128),
                a,
                b,
                Some(result),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_add_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = u128::MAX;
        let b = rand::thread_rng().gen_range(1..=u128::MAX);
        let result = b - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add128),
                a,
                b,
                Some(result),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_add_128_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
        let b = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
        let result = a + b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add128_64),
                a,
                b,
                Some(result),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_add_128_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = u128::MAX;
        let b = rand::thread_rng().gen_range(1..=u64::MAX) as u128;
        let result = b - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add128_64),
                a,
                b,
                Some(result),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_add_128_not_overflow_carry_low_false(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
        let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
        let result = a + b;
        let result_carry = false;
        let carry_low = 0_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd128),
                a,
                b,
                Some(result),
                result_carry,
                carry_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_add_128_overflow_carry_low_false(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = u128::MAX;
        let b = rand::thread_rng().gen_range(1..=u128::MAX);
        let result = b - 1;
        let result_carry = true;
        let carry_low = 0_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd128),
                a,
                b,
                Some(result),
                result_carry,
                carry_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_add_128_not_overflow_carry_low_true(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
        let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
        let result = a + b + 1;
        let result_carry = false;
        let carry_low = 1_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd128),
                a,
                b,
                Some(result),
                result_carry,
                carry_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_add_128_overflow_carry_low_true(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = u128::MAX;
        let b = rand::thread_rng().gen_range(1..=u128::MAX);
        let result = b;
        let result_carry = true;
        let carry_low = 1_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd128),
                a,
                b,
                Some(result),
                result_carry,
                carry_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_sub_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(0..=a);
        let result = a - b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub128),
                a,
                b,
                Some(result),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_sub_128_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub128),
                a,
                a,
                Some(0),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_sub_128_a_low_eq_b_low(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

        let low: u64 = a as u64;
        let b = ((b_high as u128) << 64) | (low as u128);

        let carry = a < b;
        let result = a.wrapping_sub(b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub128),
                a,
                b,
                Some(result),
                carry,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_sub_128_diff_is_u64_max(context: simplex::TestContext) -> anyhow::Result<()> {
        let a_low: u64 = u64::MAX;

        let a_high = rand::thread_rng().gen_range(0..=u64::MAX);
        let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

        let a = ((a_high as u128) << 64) | (a_low as u128);
        let b = (b_high as u128) << 64; // b_low is 0

        let carry = a < b;
        let result = a.wrapping_sub(b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub128),
                a,
                b,
                Some(result),
                carry,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_sub_128_diff_is_u128_max(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = u128::MAX;
        let b = 0;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub128),
                a,
                b,
                Some(a),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_sub_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = u128::MAX;
        let result = a + 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub128),
                a,
                b,
                Some(result),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_sub_128_borrow_low_false(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(0..=a);
        let result = a - b;
        let result_borrow = false;
        let borrow_low = 0_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullSub128),
                a,
                b,
                Some(result),
                result_borrow,
                borrow_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_sub_128_overflow_borrow_low_false(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = u128::MAX;
        let result = a + 1;
        let result_borrow = true;
        let borrow_low = 0_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullSub128),
                a,
                b,
                Some(result),
                result_borrow,
                borrow_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_sub_128_borrow_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(0..a);
        let result = a - b - 1;
        let result_borrow = false;
        let borrow_low = 1_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullSub128),
                a,
                b,
                Some(result),
                result_borrow,
                borrow_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_full_sub_128_overflow_borrow_low_true(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = u128::MAX;
        let (result, result_borrow) = a.overflowing_sub(b);

        let borrow_low = 1_u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullSub128),
                a,
                b,
                Some(result - 1),
                result_borrow,
                borrow_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_mul_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = rand::thread_rng().gen_range(0..u128::MAX);
        let result = U256::from(a) * U256::from(b);

        let (result_high, result_low) = split_helper(result);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Mul128),
                a,
                b,
                Some(result_high),
                DEFAULT_BOOL,
                result_low,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_calculate_normalizer_base_64_b_is_u64(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let threshold = 1u128 << 63;

        let b = rand::thread_rng().gen_range(1..threshold);

        let norm: u128 = threshold.div_ceil(b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::CalculateNormalizerBase64),
                DEFAULT_EXPECTED,
                b,
                Some(norm),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_calculate_normalizer_base_64_b_is_big_enough_not_normalize(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let threshold = 1u128 << 63;

        let b = rand::thread_rng().gen_range(threshold..=u64::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::CalculateNormalizerBase64),
                DEFAULT_EXPECTED,
                b,
                Some(1),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_calculate_normalizer_base_64_b_is_u128(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let threshold = 1u128 << 63;

        let b = rand::thread_rng().gen_range((u64::MAX as u128) + 1..=u128::MAX);
        let b_high = b >> 64;

        let norm: u128 = threshold.div_ceil(b_high);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::CalculateNormalizerBase64),
                DEFAULT_EXPECTED,
                b,
                Some(norm),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_calculate_normalizer_base_64_b_is_u64_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let b = rand::thread_rng().gen_range((u64::MAX as u128) + 1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::CalculateNormalizerBase64),
                DEFAULT_EXPECTED,
                b,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_test_calculate_normalizer_base_64_b_is_u128_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::CalculateNormalizerBase64),
                DEFAULT_EXPECTED,
                b,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_test_calculate_normalizer_base_64_b_is_zero_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let b = 0;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::CalculateNormalizerBase64),
                DEFAULT_EXPECTED,
                b,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_test_estimate_quotient_digit_base_64(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let threshold = 1u64 << 63;

        let b_high = rand::thread_rng().gen_range(threshold..=u64::MAX);
        let b_low = rand::thread_rng().gen_range(0..=u64::MAX);

        let a_high = rand::thread_rng().gen_range(0..b_high);
        let a_low = rand::thread_rng().gen_range(0..=u128::MAX);

        let a = ((U256::from(a_high)) << 128) | (U256::from(a_low));
        let b = ((b_high as u128) << 64) | (b_low as u128);

        let q = (a / b).as_u128();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::EstimateQuotientDigitBase64),
                a_high as u128,
                a_low,
                Some(q),
                DEFAULT_BOOL,
                b,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_estimate_quotient_digit_base_64_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        // expected to fail because a is to big for q to fit unto u64
        let threshold = 1u64 << 63;

        let b_high = rand::thread_rng().gen_range(threshold..u64::MAX);
        let b_low = rand::thread_rng().gen_range(0..=u64::MAX);

        let a_high = rand::thread_rng().gen_range(b_high..=u64::MAX);
        let a_low = rand::thread_rng().gen_range(0..=u128::MAX);

        let a = ((U256::from(a_high)) << 128) | (U256::from(a_low));
        let b = ((b_high as u128) << 64) | (b_low as u128);

        let q = (a / b).as_u128();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::EstimateQuotientDigitBase64),
                a_high as u128,
                a_low,
                Some(q),
                DEFAULT_BOOL,
                b,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn test_div_mod_128_64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);

        let q = a / b;
        let r = a % b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128_64),
                a,
                b,
                Some(q),
                DEFAULT_BOOL,
                r,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_128_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = 0;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128_64),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = rand::thread_rng().gen_range(a + 1..=u128::MAX);

        let q = a / b;
        let r = a % b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                b,
                Some(q),
                DEFAULT_BOOL,
                r,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_div_64(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = rand::thread_rng().gen_range(1..=u64::MAX);
        let a = rand::thread_rng().gen_range(b..=u64::MAX) as u128;

        let q = a / b as u128;
        let r = a % b as u128;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                b as u128,
                Some(q),
                DEFAULT_BOOL,
                r,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_q_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
        // case where a >= b and a_high = b_high != 0
        let b_low = rand::thread_rng().gen_range(0..=u64::MAX);
        let a_low = rand::thread_rng().gen_range(b_low..=u64::MAX);
        let high = rand::thread_rng().gen_range(1..u64::MAX);

        let a = ((high as u128) << 64) | (a_low as u128);
        let b = ((high as u128) << 64) | (b_low as u128);

        let q = a / b;
        let r = a % b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                b,
                Some(q),
                DEFAULT_BOOL,
                r,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_b_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);
        let a = rand::thread_rng().gen_range(u64::MAX as u128 + 1..=u128::MAX);

        let q = a / b;
        let r = a % b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                b,
                Some(q),
                DEFAULT_BOOL,
                r,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_b_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let b_high = rand::thread_rng().gen_range(1..u64::MAX);
        let a_high = rand::thread_rng().gen_range(b_high + 1..=u64::MAX);

        let a = ((a_high as u128) << 64) | (rand::thread_rng().gen_range(0..u64::MAX) as u128);
        let b = ((b_high as u128) << 64) | (rand::thread_rng().gen_range(0..u64::MAX) as u128);

        let q = a / b;
        let r = a % b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                b,
                Some(q),
                DEFAULT_BOOL,
                r,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_a_equal_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                a,
                Some(1u128),
                DEFAULT_BOOL,
                0u128,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_equal_high_words_max_low_diff(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let high = rand::thread_rng().gen_range(1..=u64::MAX);

        let a = ((high as u128) << 64) | (u64::MAX as u128);
        let b = (high as u128) << 64;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                b,
                Some(1u128),
                DEFAULT_BOOL,
                u64::MAX as u128,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_mod_128_eq_high_words_a_less_than_b(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let high = rand::thread_rng().gen_range(1..=u64::MAX);

        let a = (high as u128) << 64;
        let b = ((high as u128) << 64) | (u64::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod128),
                a,
                b,
                Some(0u128),
                DEFAULT_BOOL,
                a,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(1..=u128::MAX);
        let result = a / b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Div128),
                a,
                b,
                Some(result),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_div_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = 0;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Div128),
                a,
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }
}
