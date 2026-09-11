use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u16::mul_div::MulDivProgram as U16MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u16::mul_div::derived_mul_div::{
    MulDivArguments as U16MulDivTestArguments, MulDivWitness as U16MulDivTestWitness,
};

const DEFAULT_EXPECTED: u16 = 0;

enum FunctionToTest {
    MulDiv,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U16MulDivTestProgram {
    U16MulDivTestProgram::new(&U16MulDivTestArguments {})
}

fn build_witness(op: u8, a: u16, b: u16, c: u16, expected: Option<u16>) -> U16MulDivTestWitness {
    U16MulDivTestWitness {
        function_index: op,
        first_arg: a,
        second_arg: b,
        third_arg: c,
        expected,
    }
}

#[simplex::test]
fn mul_div_16_product_is_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX) as u16;
    let b = rand::thread_rng().gen_range(0..=u8::MAX) as u16;
    let c = rand::thread_rng().gen_range(1..=u16::MAX);

    let res = a * b / c;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res)),
        Expect::Ok,
    )
}

#[simplex::test]
fn mul_div_16_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u16::MAX);
    let b = u16::MAX;
    let c = rand::thread_rng().gen_range(a..=u16::MAX);

    let res = (a as u32) * (b as u32) / (c as u32);

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(res as u16)),
        Expect::Ok,
    )
}

#[simplex::test]
fn mul_div_16_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u16::MAX);
    let b = u16::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(DEFAULT_EXPECTED)),
        Expect::AssertFailed,
    )
}

#[simplex::test]
fn mul_div_16_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u16::MAX);
    let b = rand::thread_rng().gen_range(1..=u16::MAX);
    let c = 0;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::MulDiv), a, b, c, Some(0)),
        Expect::Ok,
    )
}
