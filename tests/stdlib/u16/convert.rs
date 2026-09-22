use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u16::convert::ConvertProgram as U16ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u16::convert::derived_convert::{
    ConvertArguments as U16ConvertTestArguments, ConvertWitness as U16ConvertTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    U16ToU32,
    U16ToU64,
    U16ToU128,
    U16ToU256,
    SplitU16IntoU8,
    SafeU16ToU1,
    SafeU16ToU8,
}

fn program() -> U16ConvertTestProgram {
    U16ConvertTestProgram::new(U16ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U16ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U16ConvertTestWitness {
            function_index: function as u8,
            first_arg: 0,
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: u16) -> Self {
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
fn u16_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX);

    case(U16ToU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u16_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX);

    case(U16ToU64)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u16_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX);

    case(U16ToU128)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u16_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX);

    case(U16ToU256)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u16_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX);

    case(SplitU16IntoU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u16_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(SafeU16ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u16_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u16::MAX);

    case(SafeU16ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u16_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX as u16);

    case(SafeU16ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u16_to_u8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u8::MAX as u16 + 1..=u16::MAX);

    case(SafeU16ToU8)
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
        FuzzEngineBuilder<U16ConvertTestProgram, U16ConvertTestArguments, U16ConvertTestWitness>;

    // (first_arg)
    type ConvertInputs = u16;

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

    fn arb_bool_u16() -> impl Strategy<Value = u16> {
        arb_bool().prop_map(u16::from)
    }

    fn arb_u8_u16() -> impl Strategy<Value = u16> {
        arb_u8().prop_map(u16::from)
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
                    let arguments: Arguments = U16ConvertTestArguments {}.into();
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
                FuzzStrategyBuilder::<U16ConvertTestArguments, U16ConvertTestWitness, _>::new()
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
    fn u16_to_u32(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U16ToU32, builder, "u16_to_u32")
            .strategy(arb_u16())
            .run()
    }

    #[simplex::fuzz]
    fn u16_to_u64(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U16ToU64, builder, "u16_to_u64")
            .strategy(arb_u16())
            .run()
    }

    #[simplex::fuzz]
    fn u16_to_u128(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U16ToU128, builder, "u16_to_u128")
            .strategy(arb_u16())
            .run()
    }

    #[simplex::fuzz]
    fn u16_to_u256(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U16ToU256, builder, "u16_to_u256")
            .strategy(arb_u16())
            .run()
    }

    #[simplex::fuzz]
    fn split_u16_into_u8(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SplitU16IntoU8, builder, "split_u16_into_u8")
            .strategy(arb_u16())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u16_to_u1(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU16ToU1, builder, "safe_u16_to_u1")
            .strategy(arb_bool_u16())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u16_to_u1_overflow(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU16ToU1, builder, "safe_u16_to_u1_overflow")
            .strategy(2u16..=u16::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u16_to_u8(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU16ToU8, builder, "safe_u16_to_u8")
            .strategy(arb_u8_u16())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u16_to_u8_overflow(builder: ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(SafeU16ToU8, builder, "safe_u16_to_u8_overflow")
            .strategy(u8::MAX as u16 + 1..=u16::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }
}
