mod common;

use primitive_types::U256;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_convert_test::U256ConvertTestProgram;
use simplicityhl_std::artifacts::u256_convert_test::derived_u256_convert_test::{
    U256ConvertTestArguments, U256ConvertTestWitness,
};

enum FunctionToTest {
    SplitU256IntoU8,
    SplitU256IntoU16,
    SplitU256IntoU32,
    SplitU256IntoU64,
    SplitU256IntoU128,
    SafeU256ToU1,
    SafeU256ToU8,
    SafeU256ToU16,
    SafeU256ToU32,
    SafeU256ToU64,
    SafeU256ToU128,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

fn program() -> U256ConvertTestProgram {
    U256ConvertTestProgram::new(U256ConvertTestArguments {})
}

fn build_witness(function: u8, a: [u8; 32], expected: [u8; 32]) -> U256ConvertTestWitness {
    U256ConvertTestWitness {
        function_index: function,
        first_arg: a,
        expected,
    }
}

mod u256_convert_test {
    use crate::common::helper::generate_u256;

    use super::*;

    #[simplex::test]
    fn u256_convert_test_u256_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU256IntoU8),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_u256_into_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU256IntoU16),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_u256_into_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU256IntoU32),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_u256_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU256IntoU64),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_u256_into_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SplitU256IntoU128),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::one());

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU1),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u1_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(2), U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU1),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::from(u8::MAX));

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU8),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u8_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(u8::MAX) + 1, U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU8),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::from(u16::MAX));

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU16),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u16_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(u16::MAX) + 1, U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU16),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::from(u32::MAX));

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU32),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u32_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(u32::MAX) + 1, U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU32),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::from(u64::MAX));

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU64),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u64_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(u64::MAX) + 1, U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU64),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::from(u128::MAX));

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU128),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_convert_test_safe_u256_to_u128_overflow(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::SafeU256ToU128),
                a.to_big_endian(),
                U256::from(a).to_big_endian(),
            ),
            Expect::AssertFailed,
        )
    }
}
