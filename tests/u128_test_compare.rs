mod common;

use rand::Rng;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u128_test_compare::U128TestCompareProgram;
use simplicityhl_std::artifacts::u128_test_compare::derived_u128_test_compare::{
    U128TestCompareArguments, U128TestCompareWitness,
};

enum FunctionToTest {
    IsZero128,
    Lt128,
    Le128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: u128 = 0;

fn program() -> U128TestCompareProgram {
    U128TestCompareProgram::new(U128TestCompareArguments {})
}

fn build_witness(function: u8, a: u128, b: u128, expected_bool: bool) -> U128TestCompareWitness {
    U128TestCompareWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected_bool,
    }
}

mod u128_tests_arithmetic {
    use super::*;

    #[simplex::test]
    fn u128_test_is_zero_128_true(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = 0;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::IsZero128), a, DEFAULT_EXPECTED, true),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_is_zero_128_false(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::IsZero128), a, DEFAULT_EXPECTED, false),
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
            build_witness(op(FunctionToTest::Lt128), a, b, true),
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
            build_witness(op(FunctionToTest::Lt128), a, b, false),
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
            build_witness(op(FunctionToTest::Lt128), a, b, false),
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
            build_witness(op(FunctionToTest::Le128), a, b, true),
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
            build_witness(op(FunctionToTest::Le128), a, b, true),
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
            build_witness(op(FunctionToTest::Le128), a, b, false),
            Expect::Ok,
        )
    }
}
