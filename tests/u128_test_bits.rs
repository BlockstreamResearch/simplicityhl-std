mod common;

use rand::Rng;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u128_test_bits::U128TestBitsProgram;
use simplicityhl_std::artifacts::u128_test_bits::derived_u128_test_bits::{
    U128TestBitsArguments, U128TestBitsWitness,
};

enum FunctionToTest {
    And128,
    Or128,
    Eq128,
    LeftShift128,
    RightShift128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_BOOL: bool = false;
const DEFAULT_EXPECTED: u128 = 0;

fn program() -> U128TestBitsProgram {
    U128TestBitsProgram::new(U128TestBitsArguments {})
}

fn build_witness(
    function: u8,
    a: u128,
    b: u128,
    expected: Option<u128>,
    expected_bool: bool,
) -> U128TestBitsWitness {
    U128TestBitsWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected,
        expected_bool,
    }
}

mod u128_tests_bits {
    use super::*;

    #[simplex::test]
    fn u128_test_and_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);
        let b = rand::thread_rng().gen_range(0..=u128::MAX);
        let result = a & b;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::And128), a, b, Some(result), DEFAULT_BOOL),
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
            build_witness(op(FunctionToTest::Or128), a, b, Some(result), DEFAULT_BOOL),
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
            ),
            Expect::Ok,
        )
    }
}
