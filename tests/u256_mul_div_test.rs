mod common;

use primitive_types::U256;
use std::ops::Div;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::u256_mul_div_test::U256MulDivTestProgram;
use simplicityhl_std::artifacts::u256_mul_div_test::derived_u256_mul_div_test::{
    U256MulDivTestArguments, U256MulDivTestWitness,
};

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

enum FunctionToTest {
    MulDiv,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U256MulDivTestProgram {
    U256MulDivTestProgram::new(&U256MulDivTestArguments {})
}

fn build_witness(
    op: u8,
    a: [u8; 32],
    b: [u8; 32],
    c: [u8; 32],
    expected: Option<[u8; 32]>,
) -> U256MulDivTestWitness {
    U256MulDivTestWitness {
        function_index: op,
        first_arg: a,
        second_arg: b,
        third_arg: c,
        expected,
    }
}

fn safe_u512_to_u256(a: [u8; 64]) -> [u8; 32] {
    let high = U256::from_big_endian(&a[0..32]);
    let low = U256::from_big_endian(&a[32..64]);

    assert!(high == U256::zero());

    low.to_big_endian()
}

mod u256_mul_div_test {
    use super::*;

    #[simplex::test]
    fn u256_test_mul_div_256_product_is_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::from(u128::MAX));
        let b = generate_u256(U256::zero(), U256::from(u128::MAX));
        let c = generate_u256(U256::one(), U256::MAX);

        let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::MulDiv),
                a.to_big_endian(),
                b.to_big_endian(),
                c.to_big_endian(),
                Some(res),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_mul_div_256_intermediate_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(2), U256::MAX);
        let b = U256::MAX;
        let c = generate_u256(a, U256::MAX);

        let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::MulDiv),
                a.to_big_endian(),
                b.to_big_endian(),
                c.to_big_endian(),
                Some(res),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_mul_div_256_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(2), U256::MAX);
        let b = U256::MAX;
        let c = generate_u256(U256::one(), a);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::MulDiv),
                a.to_big_endian(),
                b.to_big_endian(),
                c.to_big_endian(),
                Some(DEFAULT_EXPECTED),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u256_test_mul_div_256_denominator_is_u128(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let c = generate_u256(b, U256::from(u128::MAX));

        let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::MulDiv),
                a.to_big_endian(),
                b.to_big_endian(),
                c.to_big_endian(),
                Some(res),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_mul_div_256_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX);
        let b = generate_u256(U256::one(), U256::MAX);
        let c = U256::zero();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::MulDiv),
                a.to_big_endian(),
                b.to_big_endian(),
                c.to_big_endian(),
                Some([0; 32]),
            ),
            Expect::Ok,
        )
    }
}
