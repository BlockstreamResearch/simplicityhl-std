mod common;

use primitive_types::U256;
use rand::Rng;

use crate::common::helper::{DEFAULT_BOOL, generate_u256};
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_arithmetic::U256TestArithmeticProgram;
use simplicityhl_std::artifacts::u256_test_arithmetic::derived_u256_test_arithmetic::{
    U256TestArithmeticArguments, U256TestArithmeticWitness,
};

enum FunctionToTest {
    IsZero256,
    Lt256,
    Le256,
    Split256Into64,
    Add256,
    Add256_128,
    Sub256,
    Mul256,
    DivMod256_64,
    DivMod256_128,
    DivMod256,
    Div256,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> U256TestArithmeticProgram {
    U256TestArithmeticProgram::new(U256TestArithmeticArguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    expected: Option<[u8; 32]>,
    expected_bool: bool,
    second_expected: [u8; 32],
) -> U256TestArithmeticWitness {
    U256TestArithmeticWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected,
        expected_bool,
        second_expected,
    }
}

fn split_u512(a: [u8; 64]) -> ([u8; 32], [u8; 32]) {
    let high = U256::from_big_endian(&a[0..32]);
    let low = U256::from_big_endian(&a[32..64]);

    (high.to_big_endian(), low.to_big_endian())
}

mod u256_tests_arithmetic {
    use super::*;

    #[simplex::test]
    fn u256_test_is_zero_256_true(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = [0; 32];

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::IsZero256),
                a,
                DEFAULT_EXPECTED,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_is_zero_256_false(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::IsZero256),
                a,
                DEFAULT_EXPECTED,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_lt_256_less(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX - 1);
        let b = a + 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Lt256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_lt_256_eq(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Lt256),
                a,
                a,
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_lt_256_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = a - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Lt256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_le_256_less(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX - 1);
        let b = a + 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Le256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_le_256_eq(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Le256),
                a,
                a,
                Some(DEFAULT_EXPECTED),
                true,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_le_256_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = a - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Le256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(DEFAULT_EXPECTED),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_split_256_into_64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Split256Into64),
                a,
                DEFAULT_EXPECTED,
                Some(a),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_add_256_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX / 2);
        let b = generate_u256(U256::zero(), U256::MAX / 2);
        let result = (a + b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_add_256_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = U256::MAX;
        let b = generate_u256(U256::one(), U256::MAX);
        let result = (b - 1).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                true,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_add_256_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX / 2);
        let b = generate_u256(U256::one(), U256::from(u128::MAX)) as U256;
        let result = (a + b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_add_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = U256::MAX;
        let b = generate_u256(U256::one(), U256::from(u128::MAX)) as U256;
        let result = (b - 1).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                true,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_sub_256_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::zero(), a);
        let result = (a - b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_sub_256_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub256),
                a,
                a,
                Some([0; 32]),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_sub_256_a_low_eq_b_low(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b_high = rand::thread_rng().gen_range(0..=u128::MAX);

        let low: u128 = a.low_u128();
        let b = (U256::from(b_high) << 128) | U256::from(low);

        let (result, carry) = a.overflowing_sub(b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result.to_big_endian()),
                carry,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_sub_256_diff_is_u128_max(context: simplex::TestContext) -> anyhow::Result<()> {
        let a_low: u128 = u128::MAX;

        let a_high = rand::thread_rng().gen_range(0..=u128::MAX);
        let b_high = rand::thread_rng().gen_range(0..=u128::MAX);

        let a = (U256::from(a_high) << 128) | U256::from(a_low);
        let b = (U256::from(b_high)) << 128; // b_low is 0

        //let carry = a < b;
        let (result, carry) = a.overflowing_sub(b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result.to_big_endian()),
                carry,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_sub_256_diff_is_u256_max(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = U256::MAX.to_big_endian();
        let b = U256::zero();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub256),
                a,
                b.to_big_endian(),
                Some(a),
                false,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_sub_256_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX - 1);
        let b = U256::MAX;
        let result = a + 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Sub256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result.to_big_endian()),
                true,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_mul_256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = generate_u256(U256::one(), U256::MAX);
        let result = a.full_mul(b).to_big_endian();

        let (result_high, result_low) = split_u512(result);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Mul256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result_high),
                DEFAULT_BOOL,
                result_low,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::one(), U256::from(u64::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_64),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = [0; 32];

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_64),
                a.to_big_endian(),
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::from(u64::MAX) + 1, U256::from(u128::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128_b_fits_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::one(), U256::from(u64::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = [0; 32];

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_128),
                a.to_big_endian(),
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = (generate_u256(U256::one(), U256::from(u128::MAX))).to_big_endian();

        let q = U256::one().to_big_endian();
        let r = U256::zero().to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_128),
                a,
                a,
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX - 1);
        let b = generate_u256(a + 1, U256::MAX);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_div_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let a = generate_u256(b, U256::from(u128::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_q_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
        // case where a >= b and a_high = b_high != 0
        let b_low = generate_u256(U256::zero(), U256::from(u128::MAX));
        let a_low = generate_u256(b_low, U256::from(u128::MAX));
        let high = generate_u256(U256::one(), U256::from(u128::MAX));

        let a = ((high as U256) << 128) | (a_low as U256);
        let b = ((high as U256) << 128) | (b_low as U256);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_b_fits_into_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_b_is_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = generate_u256(U256::one(), U256::MAX - 1);
        let a = generate_u256(b + 1, U256::MAX);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                DEFAULT_BOOL,
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_a_equal_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a,
                a,
                Some(U256::one().to_big_endian()),
                DEFAULT_BOOL,
                [0; 32],
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_equal_high_words_max_low_diff(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let high = generate_u256(U256::one(), U256::from(u128::MAX));

        let a = ((high << 128) | (U256::from(u128::MAX))).to_big_endian();
        let b = (high << 128).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a,
                b,
                Some(U256::one().to_big_endian()),
                DEFAULT_BOOL,
                U256::from(u128::MAX).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_eq_high_words_a_less_than_b(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let high = generate_u256(U256::one(), U256::from(u128::MAX));

        let a = ((high as U256) << 128).to_big_endian();
        let b = (((high as U256) << 128) | (U256::from(u128::MAX))).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a,
                b,
                Some([0; 32]),
                DEFAULT_BOOL,
                a,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::one(), U256::MAX);
        let result = (a / b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Div256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_256_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = [0; 32];

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Div256),
                a.to_big_endian(),
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_BOOL,
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }
}
