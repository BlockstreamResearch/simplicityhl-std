mod common;

use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::u32_mul_div_test::U32MulDivTestProgram;
use simplicityhl_std::artifacts::u32_mul_div_test::derived_u32_mul_div_test::{
    U32MulDivTestArguments, U32MulDivTestWitness,
};

const DEFAULT_EXPECTED: u32 = 0;

enum FunctionToTest {
    MulDiv,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U32MulDivTestProgram {
    U32MulDivTestProgram::new(&U32MulDivTestArguments {})
}

fn build_witness(op: u8, a: u32, b: u32, c: u32, expected: Option<u32>) -> U32MulDivTestWitness {
    U32MulDivTestWitness {
        function_index: op,
        first_arg: a,
        second_arg: b,
        third_arg: c,
        expected,
    }
}

mod u32_mul_div_test {
    use super::*;

    #[simplex::test]
    fn u32_test_mul_div_3_product_is_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX) as u32;
        let b = rand::thread_rng().gen_range(0..=u16::MAX) as u32;
        let c = rand::thread_rng().gen_range(1..=u32::MAX);

        let res = a * b / c;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res)),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_test_mul_div_32_intermediate_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u32::MAX);
        let b = u32::MAX;
        let c = rand::thread_rng().gen_range(a..=u32::MAX);

        let res = (a as u64) * (b as u64) / (c as u64);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res as u32)),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_test_mul_div_32_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u32::MAX);
        let b = u32::MAX;
        let c = rand::thread_rng().gen_range(1..a);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(DEFAULT_EXPECTED)),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u32_test_mul_div_32_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u32::MAX);
        let b = rand::thread_rng().gen_range(1..=u32::MAX);
        let c = 0;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(0)),
            Expect::Ok,
        )
    }
}
