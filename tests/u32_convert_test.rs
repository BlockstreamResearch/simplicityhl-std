mod common;

use primitive_types::U256;
use rand::Rng;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u32_convert_test::U32ConvertTestProgram;
use simplicityhl_std::artifacts::u32_convert_test::derived_u32_convert_test::{
    U32ConvertTestArguments, U32ConvertTestWitness,
};

enum FunctionToTest {
    U32ToU64,
    U32ToU128,
    U32ToU256,
    SplitU32ToU8,
    SplitU32ToU16,
    SafeU32ToU1,
    SafeU32ToU8,
    SafeU32ToU16,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U32ConvertTestProgram {
    U32ConvertTestProgram::new(U32ConvertTestArguments {})
}

fn build_witness(function: u8, a: u32, expected: [u8; 32]) -> U32ConvertTestWitness {
    U32ConvertTestWitness {
        function_index: function,
        first_arg: a,
        expected,
    }
}

mod u32_convert_test {
    use super::*;

    #[simplex::test]
    fn u32_convert_test_u32_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U32ToU64),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_u32_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U32ToU128),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_u32_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U32ToU256),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_split_u32_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU32ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_split_u32_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU32ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_safe_u32_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=1);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU32ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_safe_u32_to_u1_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU32ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u32_convert_test_safe_u32_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX as u32);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU32ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_safe_u32_to_u8_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u8::MAX as u32 + 1..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU32ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u32_convert_test_safe_u32_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX as u32);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU32ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u32_convert_test_safe_u32_to_u16_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u16::MAX as u32 + 1..=u32::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU32ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }
}
