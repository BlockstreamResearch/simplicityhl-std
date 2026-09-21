use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u1::convert::ConvertProgram as U1ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u1::convert::derived_convert::{
    ConvertArguments as U1ConvertTestArguments, ConvertWitness as U1ConvertTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    U1ToU8,
    U1ToU16,
    U1ToU32,
    U1ToU64,
    U1ToU128,
    U1ToU256,
    U1ToBool,
}

fn program() -> U1ConvertTestProgram {
    U1ConvertTestProgram::new(U1ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U1ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U1ConvertTestWitness {
            function_index: function as u8,
            first_arg: 0,
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: u8) -> Self {
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
fn u1_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(U1ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u1_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(U1ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u1_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(U1ToU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u1_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(U1ToU64)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u1_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(U1ToU128)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u1_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(U1ToU256)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u1_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(U1ToBool)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
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
    type U1ConvertFuzzEngineBuilder =
        FuzzEngineBuilder<U1ConvertTestProgram, U1ConvertTestArguments, U1ConvertTestWitness>;

    struct FuzzCaseBuilder {
        case: Case,
        builder: U1ConvertFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<u8>>,
        test_name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: U1ConvertFuzzEngineBuilder,
        test_name: &'static str,
    ) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(function),
            builder,
            inputs: None,
            test_name,
        }
    }

    fn arb_bool() -> impl Strategy<Value = bool> {
        any::<bool>()
    }

    fn arb_bool_u8() -> impl Strategy<Value = u8> {
        arb_bool().prop_map(u8::from)
    }

    impl FuzzCaseBuilder {
        fn strategy(mut self, inputs: impl Strategy<Value = u8> + 'static) -> Self {
            self.inputs = Some(inputs.boxed());
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
            let strategy: BoxedStrategy<(Arguments, WitnessValues)> = inputs
                .prop_map(move |a| {
                    let arguments: Arguments = U1ConvertTestArguments {}.into();
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
                FuzzStrategyBuilder::<U1ConvertTestArguments, U1ConvertTestWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();
            let transaction_builder =
                FinalTransactionBuilder::new(Self::build_initial_tx(), [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, transaction_builder)
                .run_with_check(FuzzExecutionCheck::new(self.test_name, Expect::Ok));

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn u1_to_u8(builder: U1ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U1ToU8, builder, "u1 to u8")
            .strategy(arb_bool_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u1_to_u16(builder: U1ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U1ToU16, builder, "u1 to u16")
            .strategy(arb_bool_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u1_to_u32(builder: U1ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U1ToU32, builder, "u1 to u32")
            .strategy(arb_bool_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u1_to_u64(builder: U1ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U1ToU64, builder, "u1 to u64")
            .strategy(arb_bool_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u1_to_u128(builder: U1ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U1ToU128, builder, "u1 to u128")
            .strategy(arb_bool_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u1_to_u256(builder: U1ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U1ToU256, builder, "u1 to u256")
            .strategy(arb_bool_u8())
            .run()
    }

    #[simplex::fuzz]
    fn split_u1_to_u1(builder: U1ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(U1ToBool, builder, "split u1 to u1")
            .strategy(arb_bool_u8())
            .run()
    }
}
