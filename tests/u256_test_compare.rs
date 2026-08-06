mod common;

use primitive_types::U256;

use crate::common::helper::generate_u256;
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_compare::U256TestCompareProgram;
use simplicityhl_std::artifacts::u256_test_compare::derived_u256_test_compare::{
    U256TestCompareArguments, U256TestCompareWitness,
};

enum FunctionToTest {
    IsZero256,
    Lt256,
    Le256,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> U256TestCompareProgram {
    U256TestCompareProgram::new(U256TestCompareArguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    expected_bool: bool,
) -> U256TestCompareWitness {
    U256TestCompareWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected_bool,
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
            build_witness(op(FunctionToTest::IsZero256), a, DEFAULT_EXPECTED, true),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_is_zero_256_false(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::IsZero256), a, DEFAULT_EXPECTED, false),
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
                true,
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
            build_witness(op(FunctionToTest::Lt256), a, a, false),
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
                false,
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
                true,
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
            build_witness(op(FunctionToTest::Le256), a, a, true),
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
                false,
            ),
            Expect::Ok,
        )
    }
}
