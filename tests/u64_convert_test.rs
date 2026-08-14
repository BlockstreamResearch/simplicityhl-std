mod common;

use primitive_types::U256;
use rand::Rng;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u64_convert_test::U64ConvertTestProgram;
use simplicityhl_std::artifacts::u64_convert_test::derived_u64_convert_test::{
    U64ConvertTestArguments, U64ConvertTestWitness,
};

enum FunctionToTest {
    U64ToU128,
    U64ToU256,
    SplitU64IntoU16,
    SplitU64IntoU32,
    SafeU64ToU1,
    SafeU64ToU8,
    SafeU64ToU16,
    SafeU64ToU32,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U64ConvertTestProgram {
    U64ConvertTestProgram::new(U64ConvertTestArguments {})
}

fn build_witness(function: u8, a: u64, expected: [u8; 32]) -> U64ConvertTestWitness {
    U64ConvertTestWitness {
        function_index: function,
        first_arg: a,
        expected,
    }
}

mod u64_convert_test {
    use super::*;

    #[simplex::test]
    fn u64_convert_test_u64_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U64ToU128),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_u64_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U64ToU256),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_split_u64_into_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU64IntoU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_split_u64_into_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU64IntoU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=1);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u1_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX as u64);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u8_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u8::MAX as u64 + 1..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX as u64);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u16_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u16::MAX as u64 + 1..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u32::MAX as u64);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u64_convert_test_safe_u64_to_u32_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u32::MAX as u64 + 1..=u64::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU64ToU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }
}
