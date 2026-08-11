mod common;

use primitive_types::U256;
use rand::Rng;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u128_convert_test::U128ConvertTestProgram;
use simplicityhl_std::artifacts::u128_convert_test::derived_u128_convert_test::{
    U128ConvertTestArguments, U128ConvertTestWitness,
};

enum FunctionToTest {
    U128ToU256,
    SplitU128ToU32,
    SplitU128ToU64,
    SafeU128ToU1,
    SafeU128ToU8,
    SafeU128ToU16,
    SafeU128ToU32,
    SafeU128ToU64,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U128ConvertTestProgram {
    U128ConvertTestProgram::new(U128ConvertTestArguments {})
}

fn build_witness(function: u8, a: u128, expected: [u8; 32]) -> U128ConvertTestWitness {
    U128ConvertTestWitness {
        function_index: function,
        first_arg: a,
        expected,
    }
}

mod u128_convert_test {
    use super::*;

    #[simplex::test]
    fn u128_convert_test_u128_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::U128ToU256),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_split_u128_into_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU128ToU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_split_u128_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU128ToU64),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=1);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u1_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(2..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU1),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u8::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u8_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u8::MAX as u128 + 1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU8),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u16::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u16_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u16::MAX as u128 + 1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU16),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u32::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u32_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u32::MAX as u128 + 1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU32),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u64::MAX as u128);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU64),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u128_convert_test_safe_u128_to_u64_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(u64::MAX as u128 + 1..=u128::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU128ToU64),
                a,
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }
}
