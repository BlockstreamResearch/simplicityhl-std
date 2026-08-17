mod common;

use primitive_types::U256;
use rand::Rng;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u8_convert_test::U8ConvertTestProgram;
use simplicityhl_std::artifacts::u8_convert_test::derived_u8_convert_test::{
    U8ConvertTestArguments, U8ConvertTestWitness,
};

enum FunctionToTest {
    U8ToU16,
    U8ToU32,
    U8ToU64,
    U8ToU128,
    U8ToU256,
    SplitU8IntoU1,
    SafeU8ToU1,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U8ConvertTestProgram {
    U8ConvertTestProgram::new(U8ConvertTestArguments {})
}

fn build_witness(function: u8, a: u8, expected: [u8; 32]) -> U8ConvertTestWitness {
    U8ConvertTestWitness {
        function_index: function,
        first_arg: a,
        expected,
    }
}

mod u8_convert_test {
    use super::*;

    #[simplex::test]
    fn u8_convert_test_u8_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U8ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_convert_test_u8_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U8ToU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_convert_test_u8_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U8ToU64),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_convert_test_u8_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U8ToU128),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_convert_test_u8_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U8ToU256),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_convert_test_split_u8_into_u1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU8IntoU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_convert_test_safe_u8_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=1);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU8ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u8_convert_test_safe_u8_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u8::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU8ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }
}
