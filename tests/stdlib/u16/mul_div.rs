use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u16::mul_div::MulDivProgram as U16MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u16::mul_div::derived_mul_div::{
    MulDivArguments as U16MulDivTestArguments, MulDivWitness as U16MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U16MulDivTestProgram {
    U16MulDivTestProgram::new(U16MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U16MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U16MulDivTestWitness {
            function_index: function as u8,
            first_arg: 0,
            second_arg: 0,
            third_arg: 0,
            expected: None,
        },
    }
}

impl Case {
    /// The three operands, `first_arg`, `second_arg` and `third_arg`.
    fn args(mut self, a: u16, b: u16, c: u16) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u16) -> Self {
        self.witness.expected = Some(expected);
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
fn mul_div_16_product_is_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX) as u16;
    let b = rand::thread_rng().gen_range(0..=u8::MAX) as u16;
    let c = rand::thread_rng().gen_range(1..=u16::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_16_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u16::MAX);
    let b = u16::MAX;
    let c = rand::thread_rng().gen_range(a..=u16::MAX);

    let res = (a as u32) * (b as u32) / (c as u32);

    case(MulDiv).args(a, b, c).expect(res as u16).run(&context)
}

#[simplex::test]
fn mul_div_16_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u16::MAX);
    let b = u16::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_16_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u16::MAX);
    let b = rand::thread_rng().gen_range(1..=u16::MAX);
    let c = 0;

    case(MulDiv).args(a, b, c).expect(0).run(&context)
}

mod mul_div_tests_fuzz {
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
    type MulDivFuzzEngineBuilder =
        FuzzEngineBuilder<U16MulDivTestProgram, U16MulDivTestArguments, U16MulDivTestWitness>;

    fn arb_u8() -> impl Strategy<Value = u8> {
        any::<u8>()
    }

    fn arb_non_zero_u16() -> impl Strategy<Value = u16> {
        any::<u16>().prop_filter("u16 should not be zero", |index| *index != 0)
    }

    #[derive(Debug, Default)]
    struct FuzzCase {
        first_arg: Option<u16>,
        second_arg: Option<u16>,
        third_arg: Option<u16>,
        expected: Option<u16>,
    }

    impl FuzzCase {
        fn first_arg(first_arg: u16) -> Self {
            let mut x = Self::default();
            let _ = x.first_arg.insert(first_arg);
            x
        }

        fn second_arg(mut self, second_arg: u16) -> Self {
            let _ = self.second_arg.insert(second_arg);
            self
        }

        fn third_arg(mut self, denominator: u16) -> Self {
            let _ = self.third_arg.insert(denominator);
            self
        }

        fn expect(mut self, expected: u16) -> Self {
            let _ = self.expected.insert(expected);
            self
        }

        fn into_witness(self, witness: U16MulDivTestWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_arg.expect("no first arg in witness"),
                    self.second_arg.expect("no second arg in witness"),
                    self.third_arg.expect("no third arg in witness"),
                )
                .expect(self.expected.expect("no expected arg in witness"))
                .witness
                .into()
        }
    }

    struct FuzzCaseBuilder {
        case: Case,
        builder: MulDivFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(builder: MulDivFuzzEngineBuilder, test_name: &'static str) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(MulDiv),
            builder,
            inputs: None,
            test_name,
            expect: Expect::Ok,
        }
    }

    impl FuzzCaseBuilder {
        fn strategy(mut self, inputs: impl Strategy<Value = FuzzCase> + 'static) -> Self {
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
                .prop_map(move |(fuzz_case)| {
                    let arguments: Arguments = U16MulDivTestArguments {}.into();
                    let witness: WitnessValues = fuzz_case.into_witness(witness.clone());

                    (arguments, witness)
                })
                .boxed();
            let strategy =
                FuzzStrategyBuilder::<U16MulDivTestArguments, U16MulDivTestWitness, _>::new()
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
    fn mul_div_16_product_is_u16(builder: MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (arb_u8(), arb_u8(), arb_non_zero_u16()).prop_map(|(a, b, c)| {
            let a = u16::from(a);
            let b = u16::from(b);

            FuzzCase::first_arg(a)
                .second_arg(b)
                .third_arg(c)
                .expect(a * b / c)
        });

        case_fuzz(builder, "mul_div_16_product_is_u16")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_16_intermediate_overflow(builder: MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (2u16..=u16::MAX).prop_flat_map(|a| {
            (a..=u16::MAX).prop_map(move |c| {
                let b = u16::MAX;
                let expected = ((u32::from(a) * u32::from(b)) / u32::from(c)) as u16;
                FuzzCase::first_arg(a)
                    .second_arg(b)
                    .third_arg(c)
                    .expect(expected)
            })
        });

        case_fuzz(builder, "mul_div_16_intermediate_overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_16_result_overflow(builder: MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (2u16..=u16::MAX).prop_flat_map(|a| {
            (1u16..a).prop_map(move |c| {
                FuzzCase::first_arg(a)
                    .second_arg(u16::MAX)
                    .third_arg(c)
                    .expect(0)
            })
        });

        case_fuzz(builder, "mul_div_16_result_overflow")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_16_div_by_zero(builder: MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (arb_non_zero_u16(), arb_non_zero_u16())
            .prop_map(|(a, b)| FuzzCase::first_arg(a).second_arg(b).third_arg(0).expect(0));

        case_fuzz(builder, "mul_div_16_div_by_zero")
            .strategy(strategy)
            .run()
    }
}
