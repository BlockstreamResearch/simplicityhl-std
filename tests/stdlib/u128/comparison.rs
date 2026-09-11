use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::comparison::ComparisonProgram as U128TestCompareProgram;
use simplicityhl_std::artifacts::tests::u128::comparison::derived_comparison::{
    ComparisonArguments as U128TestCompareArguments, ComparisonWitness as U128TestCompareWitness,
};

enum FunctionToTest {
    IsZero128,
    Lt128,
    Le128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: u128 = 0;

fn program() -> U128TestCompareProgram {
    U128TestCompareProgram::new(&U128TestCompareArguments {})
}

fn build_witness(function: u8, a: u128, b: u128, expected_bool: bool) -> U128TestCompareWitness {
    U128TestCompareWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected_bool,
    }
}

#[simplex::test]
fn is_zero_128_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = 0;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::IsZero128), a, DEFAULT_EXPECTED, true),
        Expect::Ok,
    )
}

#[simplex::test]
fn is_zero_128_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::IsZero128), a, DEFAULT_EXPECTED, false),
        Expect::Ok,
    )
}

#[simplex::test]
fn lt_128_less(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a + 1;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::Lt128), a, b, true),
        Expect::Ok,
    )
}

#[simplex::test]
fn lt_128_eq(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::Lt128), a, b, false),
        Expect::Ok,
    )
}

#[simplex::test]
fn lt_128_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = a - 1;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::Lt128), a, b, false),
        Expect::Ok,
    )
}

#[simplex::test]
fn le_128_less(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a + 1;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::Le128), a, b, true),
        Expect::Ok,
    )
}

#[simplex::test]
fn le_128_eq(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::Le128), a, b, true),
        Expect::Ok,
    )
}

#[simplex::test]
fn le_128_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = a - 1;

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::Le128), a, b, false),
        Expect::Ok,
    )
}
