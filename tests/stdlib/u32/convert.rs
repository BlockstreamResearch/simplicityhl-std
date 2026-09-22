use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u32::convert::ConvertProgram as U32ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u32::convert::derived_convert::{
    ConvertArguments as U32ConvertTestArguments, ConvertWitness as U32ConvertTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    U32ToU64,
    U32ToU128,
    U32ToU256,
    SplitU32IntoU8,
    SplitU32IntoU16,
    SafeU32ToU1,
    SafeU32ToU8,
    SafeU32ToU16,
}

fn program() -> U32ConvertTestProgram {
    U32ConvertTestProgram::new(U32ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U32ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U32ConvertTestWitness {
            function_index: function as u8,
            first_arg: 0,
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: u32) -> Self {
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
fn u32_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX);

    case(U32ToU64)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u32_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX);

    case(U32ToU128)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u32_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX);

    case(U32ToU256)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u32_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX);

    case(SplitU32IntoU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u32_into_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX);

    case(SplitU32IntoU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u32_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(SafeU32ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u32_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u32::MAX);

    case(SafeU32ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u32_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX as u32);

    case(SafeU32ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u32_to_u8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u8::MAX as u32 + 1..=u32::MAX);

    case(SafeU32ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u32_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX as u32);

    case(SafeU32ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u32_to_u16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u16::MAX as u32 + 1..=u32::MAX);

    case(SafeU32ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

mod convert_tests_fuzz {
    use super::*;

    use crate::common::core::FuzzExecutionCheck;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type ConvertFuzzEngineBuilder =
        FuzzEngineBuilder<U32ConvertTestProgram, U32ConvertTestArguments, U32ConvertTestWitness>;

    // (first_arg)
    type ConvertInputs = u32;

    struct FuzzCaseBuilder {
        case: Case,
        builder: ConvertFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<ConvertInputs>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: ConvertFuzzEngineBuilder,
        test_name: &'static str,
    ) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(function),
            builder,
            inputs: None,
            test_name,
            expect: Expect::Ok,
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

    fn arb_bool_u32() -> impl Strategy<Value = u32> {
        arb_bool().prop_map(u32::from)
    }

    fn arb_u8_u32() -> impl Strategy<Value = u32> {
        arb_u8().prop_map(u32::from)
    }

    fn arb_u16_u32() -> impl Strategy<Value = u32> {
        arb_u16().prop_map(u32::from)
    }

    impl FuzzCaseBuilder {
        fn strategy(mut self, inputs: impl Strategy<Value = ConvertInputs> + 'static) -> Self {
            self.inputs = Some(inputs.boxed());
            self
        }

        fn expect(mut self, expect: Expect) -> Self {
            self.expect = expect;
            self
        }

        fn build_initial_tx() -> FinalTransaction {
            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
            tx
        }

        fn run(self) -> anyhow::Result<()> {
            let Case { witness } = self.case;
            let inputs = self.inputs.expect("a fuzz strategy must be specified");
            let strategy = inputs
                .prop_map(move |a| {
                    let arguments: Arguments = U32ConvertTestArguments {}.into();
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
            let strategy =
                FuzzStrategyBuilder::<U32ConvertTestArguments, U32ConvertTestWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();
            let transaction_builder =
                FinalTransactionBuilder::new(Self::build_initial_tx(), [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, transaction_builder)
                .run_with_check(FuzzExecutionCheck::new(self.test_name, self.expect));

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn u32_to_u64(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U32ToU64, builder, "u32_to_u64")
            .strategy(arb_u32())
            .run()
    }

    #[simplex::fuzz]
    fn u32_to_u128(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U32ToU128, builder, "u32_to_u128")
            .strategy(arb_u32())
            .run()
    }

    #[simplex::fuzz]
    fn u32_to_u256(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U32ToU256, builder, "u32_to_u256")
            .strategy(arb_u32())
            .run()
    }

    #[simplex::fuzz]
    fn split_u32_into_u8(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SplitU32IntoU8, builder, "split_u32_into_u8")
            .strategy(arb_u32())
            .run()
    }

    #[simplex::fuzz]
    fn split_u32_into_u16(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SplitU32IntoU16, builder, "split_u32_into_u16")
            .strategy(arb_u32())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u32_to_u1(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU32ToU1, builder, "safe_u32_to_u1")
            .strategy(arb_bool_u32())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u32_to_u1_overflow(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU32ToU1, builder, "safe_u32_to_u1_overflow")
            .strategy(2u32..=u32::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u32_to_u8(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU32ToU8, builder, "safe_u32_to_u8")
            .strategy(arb_u8_u32())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u32_to_u8_overflow(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU32ToU8, builder, "safe_u32_to_u8_overflow")
            .strategy(u8::MAX as u32 + 1..=u32::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u32_to_u16(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU32ToU16, builder, "safe_u32_to_u16")
            .strategy(arb_u16_u32())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u32_to_u16_overflow(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU32ToU16, builder, "safe_u32_to_u16_overflow")
            .strategy(u16::MAX as u32 + 1..=u32::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }
}
