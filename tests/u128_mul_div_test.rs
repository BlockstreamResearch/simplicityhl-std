mod common;

use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::u128_mul_div_test::U128MulDivTestProgram;
use simplicityhl_std::artifacts::u128_mul_div_test::derived_u128_mul_div_test::{
    U128MulDivTestArguments, U128MulDivTestWitness,
};

const DEFAULT_EXPECTED: u128 = 0;

enum FunctionToTest {
    MulDiv,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U128MulDivTestProgram {
    U128MulDivTestProgram::new(&U128MulDivTestArguments {})
}

fn build_witness(
    op: u8,
    a: u128,
    b: u128,
    c: u128,
    expected: Option<u128>,
) -> U128MulDivTestWitness {
    U128MulDivTestWitness {
        function_index: op,
        first_arg: a,
        second_arg: b,
        third_arg: c,
        expected,
    }
}

mod u128_mul_div_test {
    use super::*;

    #[simplex::test]
    fn u128_test_mul_div_3_product_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX) as u128;
        let b = rand::thread_rng().gen_range(0..=u16::MAX) as u128;
        let c = rand::thread_rng().gen_range(1..=u128::MAX);

        let res = a * b / c;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res)),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_mul_div_128_intermediate_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u128::MAX);
        let b = u128::MAX;
        let c = rand::thread_rng().gen_range(a..=u128::MAX);

        let res = U256::from(a) * U256::from(b) / U256::from(c);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res.low_u128())),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_test_mul_div_128_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u128::MAX);
        let b = u128::MAX;
        let c = rand::thread_rng().gen_range(1..a);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(DEFAULT_EXPECTED)),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_test_mul_div_128_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u128::MAX);
        let b = rand::thread_rng().gen_range(1..=u128::MAX);
        let c = 0;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(0)),
            Expect::Ok,
        )
    }
}
