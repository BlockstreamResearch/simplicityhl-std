use rand::Rng;

use crate::common::core::{Expect, run, run_with_op_return};

use simplicityhl_std::artifacts::tests::op_return::OpReturnProgram as OpReturnTestProgram;
use simplicityhl_std::artifacts::tests::op_return::derived_op_return::{
    OpReturnArguments as OpReturnTestArguments, OpReturnWitness as OpReturnTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    IsOpReturn,
    AssertOutputIsOpReturn,
}

const DEFAULT_DATA: &[u8; 1] = &[1];

fn program() -> OpReturnTestProgram {
    OpReturnTestProgram::new(OpReturnTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads and whether the
/// spend carries an `OP_RETURN` output.
struct Case {
    witness: OpReturnTestWitness,
    data: Option<&'static [u8]>,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: OpReturnTestWitness {
            function_index: function as u8,
            index: 0,
            expected: false,
        },
        data: None,
    }
}

impl Case {
    /// `index`: the output the arm inspects.
    fn index(mut self, index: u32) -> Self {
        self.witness.index = index;
        self
    }

    /// `expected`: the boolean the arm should report.
    fn flag(mut self, flag: bool) -> Self {
        self.witness.expected = flag;
        self
    }

    /// Add an `OP_RETURN` output carrying `data` to the spend.
    fn op_return(mut self, data: &'static [u8]) -> Self {
        self.data = Some(data);
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        match self.data {
            Some(data) => run_with_op_return(context, program(), self.witness, expect, data),
            None => run(context, program(), self.witness, expect),
        }
    }
}

#[simplex::test]
fn is_output_op_return_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(IsOpReturn)
        .index(index)
        .flag(true)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn is_output_op_return_empty_index_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(1..=u32::MAX);

    case(IsOpReturn)
        .index(index)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn is_output_op_return_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(IsOpReturn).index(index).run(&context)
}

#[simplex::test]
fn assert_output_is_op_return_pass(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(AssertOutputIsOpReturn)
        .index(index)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn assert_output_is_op_return_empty_index_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(1..=u32::MAX);

    case(AssertOutputIsOpReturn)
        .index(index)
        .op_return(DEFAULT_DATA)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_output_is_op_return_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(AssertOutputIsOpReturn)
        .index(index)
        .expecting(&context, Expect::AssertFailed)
}

mod op_return_tests_fuzz {
    use super::*;

    use crate::common::core::FuzzExecutionCheck;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::{Just, any};
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{
        FinalTransaction, PartialInput, PartialOutput, RequiredSignature, UTXO,
    };

    const EXPECTED_IS_OP_RETURN: bool = true;

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type OpReturnFuzzEngineBuilder =
        FuzzEngineBuilder<OpReturnTestProgram, OpReturnTestArguments, OpReturnTestWitness>;

    fn arb_non_zero_u32() -> impl Strategy<Value = u32> {
        any::<u32>().prop_filter("output index should not be zero", |index| *index != 0)
    }

    struct CaseFuzz {
        case: Case,
        builder: OpReturnFuzzEngineBuilder,
        indices: BoxedStrategy<u32>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: OpReturnFuzzEngineBuilder,
        test_name: &'static str,
    ) -> CaseFuzz {
        CaseFuzz {
            case: case(function),
            builder,
            indices: Just(0_u32).boxed(),
            test_name,
            expect: Expect::Ok,
        }
    }

    impl CaseFuzz {
        fn strategy(mut self, indices: impl Strategy<Value = u32> + 'static) -> Self {
            self.indices = indices.boxed();
            self
        }

        fn flag(mut self, flag: bool) -> Self {
            self.case = self.case.flag(flag);
            self
        }

        fn op_return(mut self, data: &'static [u8]) -> Self {
            self.case = self.case.op_return(data);
            self
        }

        fn expect(mut self, expect: Expect) -> Self {
            self.expect = expect;
            self
        }

        fn build_initial_tx(data: Option<&[u8]>) -> FinalTransaction {
            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);

            if let Some(data) = data {
                tx.add_output(PartialOutput::new_metadata(data));
            }

            tx
        }

        fn run(self) -> anyhow::Result<()> {
            let Case { witness, data } = self.case;

            let strategy = self
                .indices
                .prop_map(move |index| {
                    let arguments: Arguments = OpReturnTestArguments {}.into();
                    let witness: WitnessValues = OpReturnTestWitness {
                        index,
                        ..witness.clone()
                    }
                    .into();

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<OpReturnTestArguments, OpReturnTestWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();

            let transaction_builder =
                FinalTransactionBuilder::new(CaseFuzz::build_initial_tx(data), [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, transaction_builder)
                .run_with_check(FuzzExecutionCheck::new(self.test_name, self.expect));

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn is_output_op_return_true(
        fuzz_engine_builder: OpReturnFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            IsOpReturn,
            fuzz_engine_builder,
            "is_output_op_return with OP_RETURN",
        )
        .flag(EXPECTED_IS_OP_RETURN)
        .op_return(DEFAULT_DATA)
        .run()
    }

    #[simplex::fuzz]
    fn is_output_op_return_empty_index_false(
        fuzz_engine_builder: OpReturnFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            IsOpReturn,
            fuzz_engine_builder,
            "is_output_op_return outside outputs",
        )
        .strategy(arb_non_zero_u32())
        .op_return(DEFAULT_DATA)
        .run()
    }

    #[simplex::fuzz]
    fn is_output_op_return_false(
        fuzz_engine_builder: OpReturnFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            IsOpReturn,
            fuzz_engine_builder,
            "is_output_op_return without outputs",
        )
        .run()
    }

    #[simplex::fuzz]
    fn assert_output_is_op_return_pass(
        fuzz_engine_builder: OpReturnFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertOutputIsOpReturn,
            fuzz_engine_builder,
            "assert_output_is_op_return with OP_RETURN",
        )
        .op_return(DEFAULT_DATA)
        .run()
    }

    #[simplex::fuzz]
    fn assert_output_is_op_return_empty_index_fail(
        fuzz_engine_builder: OpReturnFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertOutputIsOpReturn,
            fuzz_engine_builder,
            "assert_output_is_op_return outside outputs",
        )
        .strategy(arb_non_zero_u32())
        .op_return(DEFAULT_DATA)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_output_is_op_return_fail(
        fuzz_engine_builder: OpReturnFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertOutputIsOpReturn,
            fuzz_engine_builder,
            "assert_output_is_op_return without outputs",
        )
        .expect(Expect::AssertFailed)
        .run()
    }
}
