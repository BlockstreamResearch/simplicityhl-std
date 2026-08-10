mod common;

use primitive_types::U256;
use rand::Rng;

use crate::common::helper::{DEFAULT_BOOL, generate_u256};
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_sub_mul::U256TestSubMulProgram;
use simplicityhl_std::artifacts::u256_test_sub_mul::derived_u256_test_sub_mul::{
    U256TestSubMulArguments, U256TestSubMulWitness,
};

enum FunctionToTest {
    Sub256,
    Mul256,
    Mul256_64,
    Mul256_128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> U256TestSubMulProgram {
    U256TestSubMulProgram::new(U256TestSubMulArguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    expected: Option<[u8; 32]>,
    expected_bool: bool,
    second_expected: [u8; 32],
) -> U256TestSubMulWitness {
    U256TestSubMulWitness {
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
    fn u256_test_mul_256_64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = generate_u256(U256::one(), U256::from(u64::MAX));
        let result = a.full_mul(b).to_big_endian();

        let (result_high, result_low) = split_u512(result);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Mul256_64),
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
    fn u256_test_mul_256_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let result = a.full_mul(b).to_big_endian();

        let (result_high, result_low) = split_u512(result);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Mul256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result_high),
                DEFAULT_BOOL,
                result_low,
            ),
            Expect::Ok,
        )
    }
}
