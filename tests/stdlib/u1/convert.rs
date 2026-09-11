use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u1::convert::ConvertProgram as U1ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u1::convert::derived_convert::{
    ConvertArguments as U1ConvertTestArguments, ConvertWitness as U1ConvertTestWitness,
};

enum FunctionToTest {
    U1ToU8,
    U1ToU16,
    U1ToU32,
    U1ToU64,
    U1ToU128,
    U1ToU256,
    U1ToBool,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U1ConvertTestProgram {
    U1ConvertTestProgram::new(&U1ConvertTestArguments {})
}

fn build_witness(function: u8, a: u8, expected: [u8; 32]) -> U1ConvertTestWitness {
    U1ConvertTestWitness {
        function_index: function,
        first_arg: a,
        expected,
    }
}

#[simplex::test]
fn u1_convert_test_u1_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::U1ToU8), a, U256::from(a).to_big_endian()),
        Expect::Ok,
    )
}

#[simplex::test]
fn u1_convert_test_u1_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    run(
        &context,
        program(),
        build_witness(
            op(FunctionToTest::U1ToU16),
            a,
            U256::from(a).to_big_endian(),
        ),
        Expect::Ok,
    )
}

#[simplex::test]
fn u1_convert_test_u1_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    run(
        &context,
        program(),
        build_witness(
            op(FunctionToTest::U1ToU32),
            a,
            U256::from(a).to_big_endian(),
        ),
        Expect::Ok,
    )
}

#[simplex::test]
fn u1_convert_test_u1_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    run(
        &context,
        program(),
        build_witness(
            op(FunctionToTest::U1ToU64),
            a,
            U256::from(a).to_big_endian(),
        ),
        Expect::Ok,
    )
}

#[simplex::test]
fn u1_convert_test_u1_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    run(
        &context,
        program(),
        build_witness(
            op(FunctionToTest::U1ToU128),
            a,
            U256::from(a).to_big_endian(),
        ),
        Expect::Ok,
    )
}

#[simplex::test]
fn u1_convert_test_u1_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    run(
        &context,
        program(),
        build_witness(
            op(FunctionToTest::U1ToU256),
            a,
            U256::from(a).to_big_endian(),
        ),
        Expect::Ok,
    )
}

#[simplex::test]
fn u1_convert_test_split_u1_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    run(
        &context,
        program(),
        build_witness(
            op(FunctionToTest::U1ToBool),
            a,
            U256::from(a).to_big_endian(),
        ),
        Expect::Ok,
    )
}
