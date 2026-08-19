mod common;

use primitive_types::U256;
use rand::Rng;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u16_convert_test::U16ConvertTestProgram;
use simplicityhl_std::artifacts::u16_convert_test::derived_u16_convert_test::{
    U16ConvertTestArguments, U16ConvertTestWitness,
};

enum FunctionToTest {
    U16ToU32,
    U16ToU64,
    U16ToU128,
    U16ToU256,
    SplitU16IntoU8,
    SafeU16ToU1,
    SafeU16ToU8,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U16ConvertTestProgram {
    U16ConvertTestProgram::new(&U16ConvertTestArguments {})
}

fn build_witness(function: u8, a: u16, expected: [u8; 32]) -> U16ConvertTestWitness {
    U16ConvertTestWitness {
        function_index: function,
        first_arg: a,
        expected,
    }
}

mod u16_convert_test {
    use super::*;

    #[simplex::test]
    fn u16_convert_test_u16_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U16ToU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u16_convert_test_u16_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U16ToU64),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u16_convert_test_u16_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U16ToU128),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u16_convert_test_u16_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U16ToU256),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u16_convert_test_split_u16_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU16IntoU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u16_convert_test_safe_u16_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=1);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU16ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u16_convert_test_safe_u16_to_u1_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u16::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU16ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u16_convert_test_safe_u16_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX as u16);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU16ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u16_convert_test_safe_u16_to_u8_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u8::MAX as u16 + 1..=u16::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU16ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }
}
