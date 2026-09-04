mod common;

use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::u8_mul_div_test::U8MulDivTestProgram;
use simplicityhl_std::artifacts::u8_mul_div_test::derived_u8_mul_div_test::{
    U8MulDivTestArguments, U8MulDivTestWitness,
};

const DEFAULT_EXPECTED: u8 = 0;

enum FunctionToTest {
    MulDiv,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U8MulDivTestProgram {
    U8MulDivTestProgram::new(&U8MulDivTestArguments {})
}

fn build_witness(op: u8, a: u8, b: u8, c: u8, expected: Option<u8>) -> U8MulDivTestWitness {
    U8MulDivTestWitness {
        function_index: op,
        first_arg: a,
        second_arg: b,
        third_arg: c,
        expected,
    }
}

mod u8_mul_div_test {
    use super::*;

    #[simplex::test]
    fn u8_test_mul_div_8_product_is_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u8::MAX);
        let b = u8::MAX / a;
        let c = rand::thread_rng().gen_range(1..=u8::MAX);

        let res = a * b / c;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res)),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_test_mul_div_8_intermediate_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u8::MAX);
        let b = u8::MAX;
        let c = rand::thread_rng().gen_range(a..=u8::MAX);

        let res = (a as u16) * (b as u16) / (c as u16);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res as u8)),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_test_mul_div_8_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u8::MAX);
        let b = u8::MAX;
        let c = rand::thread_rng().gen_range(1..a);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(DEFAULT_EXPECTED)),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u8_test_mul_div_8_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u8::MAX);
        let b = rand::thread_rng().gen_range(1..=u8::MAX);
        let c = 0;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(0)),
            Expect::Ok,
        )
    }
}
