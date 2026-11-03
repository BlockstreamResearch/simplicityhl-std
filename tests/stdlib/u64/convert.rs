use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u64::convert::ConvertProgram as U64ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u64::convert::derived_convert::{
    ConvertArguments as U64ConvertTestArguments, ConvertWitness as U64ConvertTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    U64ToU128,
    U64ToU256,
    SplitU64IntoU8,
    SplitU64IntoU16,
    SplitU64IntoU32,
    SafeU64ToU1,
    SafeU64ToU8,
    SafeU64ToU16,
    SafeU64ToU32,
}

fn program() -> U64ConvertTestProgram {
    U64ConvertTestProgram::new(&U64ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U64ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U64ConvertTestWitness {
            function_index: function as u8,
            first_arg: 0,
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: u64) -> Self {
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

#[simplex::test]
fn u64_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX);

    case(U64ToU128)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u64_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX);

    case(U64ToU256)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u64_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX);

    case(SplitU64IntoU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u64_into_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX);

    case(SplitU64IntoU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u64_into_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX);

    case(SplitU64IntoU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u64_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(SafeU64ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u64_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u64::MAX);

    case(SafeU64ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u64_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX as u64);

    case(SafeU64ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u64_to_u8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u8::MAX as u64 + 1..=u64::MAX);

    case(SafeU64ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u64_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX as u64);

    case(SafeU64ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u64_to_u16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u16::MAX as u64 + 1..=u64::MAX);

    case(SafeU64ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u64_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX as u64);

    case(SafeU64ToU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u64_to_u32_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u32::MAX as u64 + 1..=u64::MAX);

    case(SafeU64ToU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

mod convert_tests_fuzz {
    use super::*;

    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::Expect;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::{FuzzMode, TestContext};

    struct FuzzCaseBuilder {
        case: Case,
        context: TestContext<FuzzMode>,
        inputs: Option<BoxedStrategy<u64>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        context: TestContext<FuzzMode>,
        test_name: &'static str,
    ) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(function),
            context,
            inputs: None,
            test_name,
            expect: Expect::Accept,
        }
    }

    impl FuzzCaseBuilder {
        fn strategy(mut self, inputs: impl Strategy<Value = u64> + 'static) -> Self {
            self.inputs = Some(inputs.boxed());
            self
        }

        fn expect(mut self, expect: Expect) -> Self {
            self.expect = expect;
            self
        }

        fn run(self) -> anyhow::Result<()> {
            let Case { witness } = self.case;
            let inputs = self.inputs.expect("a fuzz strategy must be specified");

            let strategy = inputs
                .prop_map(move |a| {
                    let arguments: Arguments = U64ConvertTestArguments {}.into();

                    let witness: WitnessValues = Case {
                        witness: witness.clone(),
                    }
                    .arg(a)
                    .expect(U256::from(a).to_big_endian())
                    .witness
                    .into();

                    (arguments, witness)
                })
                .boxed();

            crate::common::fuzz::run::<U64ConvertTestProgram>(
                &self.context,
                strategy,
                self.test_name,
                self.expect,
            );

            Ok(())
        }
    }

    fn arb_bool() -> impl Strategy<Value = bool> {
        any::<bool>()
    }

    fn arb_u8() -> impl Strategy<Value = u8> {
        any::<u8>()
    }

    fn arb_u16() -> impl Strategy<Value = u16> {
        any::<u16>()
    }

    fn arb_u32() -> impl Strategy<Value = u32> {
        any::<u32>()
    }

    fn arb_u64() -> impl Strategy<Value = u64> {
        any::<u64>()
    }

    fn arb_bool_u64() -> impl Strategy<Value = u64> {
        arb_bool().prop_map(u64::from)
    }

    fn arb_u8_u64() -> impl Strategy<Value = u64> {
        arb_u8().prop_map(u64::from)
    }

    fn arb_u16_u64() -> impl Strategy<Value = u64> {
        arb_u16().prop_map(u64::from)
    }

    fn arb_u32_u64() -> impl Strategy<Value = u64> {
        arb_u32().prop_map(u64::from)
    }

    #[simplex::fuzz]
    fn u64_to_u128(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(U64ToU128, context, "u64 to u128")
            .strategy(arb_u64())
            .run()
    }

    #[simplex::fuzz]
    fn u64_to_u256(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(U64ToU256, context, "u64 to u256")
            .strategy(arb_u64())
            .run()
    }

    #[simplex::fuzz]
    fn split_u64_into_u8(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SplitU64IntoU8, context, "split u64 into u8")
            .strategy(arb_u64())
            .run()
    }

    #[simplex::fuzz]
    fn split_u64_into_u16(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SplitU64IntoU16, context, "split u64 into u16")
            .strategy(arb_u64())
            .run()
    }

    #[simplex::fuzz]
    fn split_u64_into_u32(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SplitU64IntoU32, context, "split u64 into u32")
            .strategy(arb_u64())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u1(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU1, context, "safe u64 to u1")
            .strategy(arb_bool_u64())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u1_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU1, context, "safe u64 to u1 overflow")
            .strategy(2u64..=u64::MAX)
            .expect(Expect::Reject)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u8(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU8, context, "safe u64 to u8")
            .strategy(arb_u8_u64())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u8_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU8, context, "safe u64 to u8 overflow")
            .strategy((u64::from(u8::MAX) + 1)..=u64::MAX)
            .expect(Expect::Reject)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u16(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU16, context, "safe u64 to u16")
            .strategy(arb_u16_u64())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u16_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU16, context, "safe u64 to u16 overflow")
            .strategy((u64::from(u16::MAX) + 1)..=u64::MAX)
            .expect(Expect::Reject)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u32(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU32, context, "safe u64 to u32")
            .strategy(arb_u32_u64())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u64_to_u32_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU64ToU32, context, "safe u64 to u32 overflow")
            .strategy((u64::from(u32::MAX) + 1)..=u64::MAX)
            .expect(Expect::Reject)
            .run()
    }
}
