mod common;

use primitive_types::U256;
use std::cmp::max;
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
    fn u256_test_mul_div_256_product_fits_into_u256(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
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
    fn u256_test_mul_div_256_remainder_is_zero(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
        let b = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
        let c = a;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::MulDiv),
                a.to_big_endian(),
                b.to_big_endian(),
                c.to_big_endian(),
                Some(b.to_big_endian()),
            ),
            Expect::Ok,
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
    fn u256_test_mul_div_256_min_denom_high(context: simplex::TestContext) -> anyhow::Result<()> {
        let pow129: U256 = U256::from(2).pow(U256::from(129));

        let a = generate_u256(U256::from(u128::MAX) + 1, pow129);
        let b = generate_u256(U256::from(u128::MAX) + 1, pow129);
        let c = generate_u256(U256::from(u128::MAX) + 1, pow129 - 1);

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

    #[simplex::test]
    fn u256_test_mul_div_256_algorithm_d_512_256_check(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let pow2_128: U256 = U256::from(u128::MAX) + 1;

        let a = generate_u256(pow2_128, U256::MAX);
        let b = generate_u256(pow2_128, U256::MAX - pow2_128);

        let product = a.full_mul(b);
        let result_high =
            U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

        let c = generate_u256(max(U256::from(u128::MAX) + 1, result_high + 1), U256::MAX);

        let res = safe_u512_to_u256(product.div(c).to_big_endian());

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
    fn u256_test_mul_div_256_algorithm_d_512_256_c_is_res_high(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let pow2_128: U256 = U256::from(u128::MAX) + 1;

        let a: U256 = generate_u256(pow2_128, U256::MAX);
        let b = generate_u256(pow2_128, U256::MAX - pow2_128);

        let product = a.full_mul(b);
        let result_high =
            U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

        let c = result_high + 1;

        let res = safe_u512_to_u256(product.div(c).to_big_endian());

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
    fn u256_test_mul_div_256_normalize_to_threshold_512_127_norm_is_1(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let pow2_128: U256 = U256::from(u128::MAX) + 1;

        let a: U256 = generate_u256(pow2_128, U256::MAX);
        let b = generate_u256(pow2_128, U256::MAX - pow2_128);

        let product = a.full_mul(b);
        let result_high =
            U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

        let c = generate_u256(
            max(U256::from(2).pow(U256::from(255)), result_high + 1),
            U256::MAX,
        );

        let res = safe_u512_to_u256(product.div(c).to_big_endian());

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
    fn u256_test_mul_div_256_normalize_to_threshold_512_127_norm_greater_than_1(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
        let b = generate_u256(
            U256::from(u128::MAX) + 1,
            U256::from(2).pow(U256::from(192)),
        );

        let product = a.full_mul(b);
        let result_high =
            U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian()))) + 1;

        let c = generate_u256(
            max(U256::from(u128::MAX) + 1, result_high),
            U256::from(2).pow(U256::from(255)),
        );

        let res = safe_u512_to_u256(product.div(c).to_big_endian());

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
}
