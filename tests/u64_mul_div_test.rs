mod common;

use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::u64_mul_div_test::U64MulDivTestProgram;
use simplicityhl_std::artifacts::u64_mul_div_test::derived_u64_mul_div_test::{
    U64MulDivTestArguments, U64MulDivTestWitness,
};

const DEFAULT_EXPECTED: u64 = 0;

enum FunctionToTest {
    MulDiv,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U64MulDivTestProgram {
    U64MulDivTestProgram::new(&U64MulDivTestArguments {})
}

fn build_witness(op: u8, a: u64, b: u64, c: u64, expected: Option<u64>) -> U64MulDivTestWitness {
    U64MulDivTestWitness {
        function_index: op,
        first_arg: a,
        second_arg: b,
        third_arg: c,
        expected,
    }
}

mod u64_mul_div_test {
    use super::*;

    #[simplex::test]
    fn u64_test_mul_div_3_product_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX) as u64;
        let b = rand::thread_rng().gen_range(0..=u16::MAX) as u64;
        let c = rand::thread_rng().gen_range(1..=u64::MAX);

        let res = a * b / c;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res)),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_test_mul_div_64_intermediate_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u64::MAX);
        let b = u64::MAX;
        let c = rand::thread_rng().gen_range(a..=u64::MAX);

        let res = (a as u128) * (b as u128) / (c as u128);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res as u64)),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_test_mul_div_64_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u64::MAX);
        let b = u64::MAX;
        let c = rand::thread_rng().gen_range(1..a);

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(DEFAULT_EXPECTED)),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u64_test_mul_div_64_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(1..=u64::MAX);
        let b = rand::thread_rng().gen_range(1..=u64::MAX);
        let c = 0;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(0)),
            Expect::Ok,
        )
    }
}
