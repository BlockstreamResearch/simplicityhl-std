mod common;

use primitive_types::U256;

use crate::common::helper::{DEFAULT_BOOL, generate_u256};
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_arithmetic_1::U256TestArithmetic1Program;
use simplicityhl_std::artifacts::u256_test_arithmetic_1::derived_u256_test_arithmetic_1::{
    U256TestArithmetic1Arguments, U256TestArithmetic1Witness,
};

enum FunctionToTest {
    IsZero256,
    Lt256,
    Le256,
    Split256Into64,
    Add256,
    Add256_128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> U256TestArithmetic1Program {
    U256TestArithmetic1Program::new(U256TestArithmetic1Arguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    expected: Option<[u8; 32]>,
    expected_bool: bool,
    second_expected: [u8; 32],
) -> U256TestArithmetic1Witness {
    U256TestArithmetic1Witness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected,
        expected_bool,
        second_expected,
    }
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
}
