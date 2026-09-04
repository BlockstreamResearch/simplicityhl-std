mod common;

use primitive_types::U256;

use crate::common::helper::{DEFAULT_BOOL, generate_u256};
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_add::U256TestAddProgram;
use simplicityhl_std::artifacts::u256_test_add::derived_u256_test_add::{
    U256TestAddArguments, U256TestAddWitness,
};

enum FunctionToTest {
    Add256,
    Add256_128,
    FullAdd256,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U256TestAddProgram {
    U256TestAddProgram::new(&U256TestAddArguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    c: bool,
    expected: Option<[u8; 32]>,
    expected_bool: bool,
) -> U256TestAddWitness {
    U256TestAddWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        third_arg: c,
        expected,
        expected_bool,
    }
}

mod u256_tests_arithmetic {
    use super::*;

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
                DEFAULT_BOOL,
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
                DEFAULT_BOOL,
                Some(result),
                true,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_add_256_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX / 2);
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let result = (a + b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                DEFAULT_BOOL,
                Some(result),
                false,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_add_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = U256::MAX;
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let result = (b - 1).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Add256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                DEFAULT_BOOL,
                Some(result),
                true,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_full_add_256_not_overflow_carry_low_false(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX / 2);
        let b = generate_u256(U256::zero(), U256::MAX / 2);

        let result = (a + b).to_big_endian();
        let result_carry = false;
        let carry_low = false;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd256),
                a.to_big_endian(),
                b.to_big_endian(),
                carry_low,
                Some(result),
                result_carry,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_full_add_256_overflow_carry_low_false(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = U256::MAX;
        let b = generate_u256(U256::one(), U256::MAX);
        
        let result = (b - 1).to_big_endian();
        let result_carry = true;
        let carry_low = false;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd256),
                a.to_big_endian(),
                b.to_big_endian(),
                carry_low,
                Some(result),
                result_carry,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_full_add_256_not_overflow_carry_low_true(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX / 2);
        let b = generate_u256(U256::zero(), U256::MAX / 2);

        let result = (a + b + 1).to_big_endian();
        let result_carry = false;
        let carry_low = true;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd256),
                a.to_big_endian(),
                b.to_big_endian(),
                carry_low,
                Some(result),
                result_carry,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_full_add_256_overflow_carry_low_true(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = U256::MAX;
        let b = generate_u256(U256::one(), U256::MAX).to_big_endian();

        let result = b;
        let result_carry = true;
        let carry_low = true;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::FullAdd256),
                a.to_big_endian(),
                b,
                carry_low,
                Some(result),
                result_carry,
            ),
            Expect::Ok,
        )
    }
}
