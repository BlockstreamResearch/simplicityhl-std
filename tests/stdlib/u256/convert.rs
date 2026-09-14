use primitive_types::U256;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u256::convert::ConvertProgram as U256ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u256::convert::derived_convert::{
    ConvertArguments as U256ConvertTestArguments, ConvertWitness as U256ConvertTestWitness,
};

use FunctionToTest::*;

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

fn program() -> U256ConvertTestProgram {
    U256ConvertTestProgram::new(&U256ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256ConvertTestWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self
    }

    /// `expected`: the value the arm should produce.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = expected;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        run(context, program(), self.witness, expect)
    }
}

use crate::common::helper::generate_u256;

#[simplex::test]
fn u256_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU8)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU16)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU32)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU64)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU128)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::one());

    case(SafeU256ToU1)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(2), U256::MAX);

    case(SafeU256ToU1)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u8::MAX));

    case(SafeU256ToU8)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u8::MAX) + 1, U256::MAX);

    case(SafeU256ToU8)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u16::MAX));

    case(SafeU256ToU16)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u16::MAX) + 1, U256::MAX);

    case(SafeU256ToU16)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u32::MAX));

    case(SafeU256ToU32)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u32_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u32::MAX) + 1, U256::MAX);

    case(SafeU256ToU32)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u64::MAX));

    case(SafeU256ToU64)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u64::MAX) + 1, U256::MAX);

    case(SafeU256ToU64)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u128::MAX));

    case(SafeU256ToU128)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);

    case(SafeU256ToU128)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}
