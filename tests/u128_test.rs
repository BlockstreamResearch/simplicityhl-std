mod common;

use primitive_types::U256;
use rand::Rng;

use common::core::{Expect, run};
use common::uint::TestUint;

use simplicityhl_std::artifacts::u128_test::U128TestProgram;
use simplicityhl_std::artifacts::u128_test::derived_u128_test::{
    U128TestArguments, U128TestWitness,
};

enum FunctionToTest {
    And128,
    Or128,
    Eq128,
    LeftShift128,
    RightShift128,
    IsZero128,
    Lt128,
    Le128,
    Add128,
    Add128_64,
    Sub128,
    Mul128,
    Split256Into64,
    NormalizeToThreshold,
    AlgorithmD,
    DivMod128_64,
    DivMod128,
    Div128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8 + 192
}

const DEFAULT_BOOL: bool = false;
const DEFAULT_EXPECTED: u128 = 0;

// The only per-width code for the common operations.
impl TestUint for u128 {
    type Program = U128TestProgram;
    type Witness = U128TestWitness;

    const ZERO: u128 = 0;
    const ONE: u128 = 1;
    const MAX: u128 = u128::MAX;
    const HALF_MAX: u128 = u128::MAX / 2;
    const MUL_BOUND: u128 = 1 << 64; // 2^(128/2)

    fn program() -> U128TestProgram {
        U128TestProgram::new(U128TestArguments {})
    }

    fn witness(op: u8, a: u128, b: u128, expected: Option<u128>) -> U128TestWitness {
        U128TestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
            ..Default::default()
        }
    }
}

fn program() -> U128TestProgram {
    U128TestProgram::new(U128TestArguments {})
}

fn build_witness(
    function: u8,
    a: u128,
    b: u128,
    expected: Option<u128>,
    expected_bool: bool,
    second_expected: u128,
    third_expected: u128,
) -> U128TestWitness {
    U128TestWitness {
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

mod u128_tests {
    use super::*;

    // Stamps the 16 `#[simplex::test]` entry points for u128. Logic lives in common::uint.
    uint_tests!(u128);

    #[simplex::test]
    fn u128_test_and_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(0..=u128::MAX);
        let result = a & b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::And128),
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
    fn u128_test_or_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(0..=u128::MAX);
        let result = a | b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Or128),
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
    fn u128_test_eq_128_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Eq128),
                a,
                a,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_eq_128_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u128::MAX);
        let b = a - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Eq128),
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
    fn u128_test_left_shift_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=(u8::MAX / 2 + 1) as u128);
        let b = rand::thread_rng().gen_range(0..=u128::MAX);
        let result = b << a;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::LeftShift128),
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
    fn u128_test_right_shift_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=(u8::MAX / 2 + 1) as u128);
        let b = rand::thread_rng().gen_range(0..=u128::MAX);
        let result = b >> a;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::RightShift128),
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
    fn u128_test_split_256_into_64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(0..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Split256Into64),
                a,
                b,
                Some(a),
                DEFAULT_BOOL,
                b,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_normalize_to_threshold_b_is_u64(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let threshold = 1u128 << 63;

        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(1..threshold);

        let norm: u128 = threshold.div_ceil(b);

        let result_a = U256::from(a) * U256::from(norm);
        let result_b = b * norm;
        let (result_a_high, result_a_low) = split_helper(result_a);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::NormalizeToThreshold),
                a,
                b,
                Some(result_a_high),
                false,
                result_a_low,
                result_b,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_normalize_to_threshold_b_is_big_enough_not_normalize(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let threshold = 1 << 63;

        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(threshold..=u64::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::NormalizeToThreshold),
                a,
                b,
                Some(0),
                false,
                a,
                b,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_normalize_to_threshold_b_is_u128(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let threshold = 1u128 << 63;

        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range((u64::MAX as u128) + 1..=u128::MAX);

        let b_high = b >> 64;

        let norm: u128 = threshold.div_ceil(b_high);

        let result_a = U256::from(a) * U256::from(norm);
        let result_b = b * norm;
        let (result_a_high, result_a_low) = split_helper(result_a);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::NormalizeToThreshold),
                a,
                b,
                Some(result_a_high),
                true,
                result_a_low,
                result_b,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_normalize_to_threshold_b_is_u64_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(u64::MAX as u128..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::NormalizeToThreshold),
                a,
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
    fn u128_test_normalize_to_threshold_b_is_u128_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::NormalizeToThreshold),
                a,
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
    fn u128_test_normalize_to_threshold_b_is_zero_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = 0;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::NormalizeToThreshold),
                a,
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
    fn u128_test_algorithm_d(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = rand::thread_rng().gen_range(u64::MAX as u128..u128::MAX);
        let a = rand::thread_rng().gen_range(b..=u128::MAX);

        let q = a / b;
        let r = a - q * b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::AlgorithmD),
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
    fn u128_test_algorithm_d_fail(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);
        let a = rand::thread_rng().gen_range(b..=u128::MAX);

        let q = a / b;
        let r = a - q * b;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::AlgorithmD),
                a,
                b,
                Some(q),
                DEFAULT_BOOL,
                r,
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
        let r = a - q * b;

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
        let b = rand::thread_rng().gen_range(a..=u128::MAX);

        let q = a / b;
        let r = a - q * b;

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
    fn u128_test_div_mod_128_fits_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u64::MAX) as u128;
        let b = rand::thread_rng().gen_range(1..=u64::MAX) as u128;

        let q = a / b;
        let r = a - q * b;

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
    fn u128_test_div_mod_128_q_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..u128::MAX);
        let b = ((a >> 64u128) << 64u128) + rand::thread_rng().gen_range(0..u64::MAX) as u128;

        let q = a / b;
        let r = a - q * b;

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
        let b = rand::thread_rng().gen_range(1..u64::MAX as u128);
        let a = rand::thread_rng().gen_range(b..=u128::MAX);

        let q = a / b;
        let r = a - q * b;

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
        let b = rand::thread_rng().gen_range(u64::MAX as u128..u128::MAX);
        let a = rand::thread_rng().gen_range(b + 1..=u128::MAX);

        let q = a / b;
        let r = a - q * b;

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
