mod common;

use primitive_types::U256;
use rand::Rng;

use crate::common::helper::{DEFAULT_BOOL, generate_u256};
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_bits::U256TestBitsProgram;
use simplicityhl_std::artifacts::u256_test_bits::derived_u256_test_bits::{
    U256TestBitsArguments, U256TestBitsWitness,
};

enum FunctionToTest {
    And256,
    Or256,
    Eq256,
    LeftShift256,
    RightShift256,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> U256TestBitsProgram {
    U256TestBitsProgram::new(U256TestBitsArguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    expected: Option<[u8; 32]>,
    expected_bool: bool,
) -> U256TestBitsWitness {
    U256TestBitsWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected,
        expected_bool,
    }
}

mod u256_tests_bits {
    use super::*;

    #[simplex::test]
    fn u256_test_and_256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::zero(), U256::MAX);
        let result = (a & b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::And256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_or_256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::zero(), U256::MAX);
        let result = (a | b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Or256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_eq_256_true(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Eq256),
                a,
                a,
                Some(DEFAULT_EXPECTED),
                true,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_eq_256_false(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = a - 1;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Eq256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(DEFAULT_EXPECTED),
                false,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_left_shift_256(context: simplex::TestContext) -> anyhow::Result<()> {
        let shift = rand::thread_rng().gen_range(1..=127_u8);
        let val = generate_u256(U256::zero(), U256::MAX);
        let result = (val << shift).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::LeftShift256),
                U256::from(shift).to_big_endian(),
                val.to_big_endian(),
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_left_shift_256_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let shift = 0;
        let val = generate_u256(U256::zero(), U256::MAX).to_big_endian();
        let result = val;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::LeftShift256),
                U256::from(shift).to_big_endian(),
                val,
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_left_shift_256_max(context: simplex::TestContext) -> anyhow::Result<()> {
        let shift = u8::MAX;
        let val = generate_u256(U256::zero(), U256::MAX);
        let result = (val << shift).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::LeftShift256),
                U256::from(shift).to_big_endian(),
                val.to_big_endian(),
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_right_shift_256(context: simplex::TestContext) -> anyhow::Result<()> {
        let shift = rand::thread_rng().gen_range(1..=127_u128);
        let val = generate_u256(U256::zero(), U256::MAX);
        let result = (val >> shift).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::RightShift256),
                U256::from(shift).to_big_endian(),
                val.to_big_endian(),
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_right_shift_256_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let shift = 0;
        let val = generate_u256(U256::zero(), U256::MAX).to_big_endian();
        let result = val;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::RightShift256),
                U256::from(shift).to_big_endian(),
                val,
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_right_shift_256_max(context: simplex::TestContext) -> anyhow::Result<()> {
        let shift = u8::MAX;
        let val = generate_u256(U256::zero(), U256::MAX);
        let result = (val >> shift).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::RightShift256),
                U256::from(shift).to_big_endian(),
                val.to_big_endian(),
                Some(result),
                DEFAULT_BOOL,
            ),
            Expect::Ok,
        )
    }
}
