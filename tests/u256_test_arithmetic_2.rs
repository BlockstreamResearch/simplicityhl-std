mod common;

use primitive_types::U256;

use crate::common::helper::{DEFAULT_BOOL, generate_u256};
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_arithmetic_2::U256TestArithmetic2Program;
use simplicityhl_std::artifacts::u256_test_arithmetic_2::derived_u256_test_arithmetic_2::{
    U256TestArithmetic2Arguments, U256TestArithmetic2Witness,
};

enum FunctionToTest {
    Split256Into64,
    Add256,
    Add256_128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> U256TestArithmetic2Program {
    U256TestArithmetic2Program::new(U256TestArithmetic2Arguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    expected: Option<[u8; 32]>,
    expected_bool: bool,
) -> U256TestArithmetic2Witness {
    U256TestArithmetic2Witness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected,
        expected_bool,
    }
}

mod u256_tests_arithmetic {
    use super::*;

    #[simplex::test]
    fn u256_test_split_256_into_64(context: simplex::TestContext) -> anyhow::Result<()> {
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
            ),
            Expect::Ok,
        )
    }
}
