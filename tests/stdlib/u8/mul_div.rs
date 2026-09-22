use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u8::mul_div::MulDivProgram as U8MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u8::mul_div::derived_mul_div::{
    MulDivArguments as U8MulDivTestArguments, MulDivWitness as U8MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U8MulDivTestProgram {
    U8MulDivTestProgram::new(U8MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U8MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U8MulDivTestWitness {
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
    fn args(mut self, a: u8, b: u8, c: u8) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u8) -> Self {
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
fn mul_div_8_product_is_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u8::MAX);
    let b = u8::MAX / a;
    let c = rand::thread_rng().gen_range(1..=u8::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_8_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u8::MAX);
    let b = u8::MAX;
    let c = rand::thread_rng().gen_range(a..=u8::MAX);

    let res = (a as u16) * (b as u16) / (c as u16);

    case(MulDiv).args(a, b, c).expect(res as u8).run(&context)
}

#[simplex::test]
fn mul_div_8_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u8::MAX);
    let b = u8::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_8_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u8::MAX);
    let b = rand::thread_rng().gen_range(1..=u8::MAX);
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
    type U8MulDivFuzzEngineBuilder =
        FuzzEngineBuilder<U8MulDivTestProgram, U8MulDivTestArguments, U8MulDivTestWitness>;

    fn arb_non_zero_u8() -> impl Strategy<Value = u8> {
        any::<u8>().prop_filter("u8 should not be zero", |index| *index != 0)
    }

    #[derive(Debug, Default)]
    struct FuzzCase {
        first_arg: Option<u8>,
        second_arg: Option<u8>,
        denominator: Option<u8>,
        expected: Option<u8>,
    }

    impl FuzzCase {
        fn first_numerator(first_arg: u8) -> Self {
            let mut x = Self::default();
            let _ = x.first_arg.insert(first_arg);
            x
        }

        fn second_numerator(mut self, second_arg: u8) -> Self {
            let _ = self.second_arg.insert(second_arg);
            self
        }

        fn denominator(mut self, denominator: u8) -> Self {
            let _ = self.denominator.insert(denominator);
            self
        }

        fn expect(mut self, expected: u8) -> Self {
            let _ = self.expected.insert(expected);
            self
        }

        fn into_witness(self, witness: U8MulDivTestWitness, expect_failure: bool) -> WitnessValues {
            let case = Case { witness }.args(
                self.first_arg.expect("no first arg in witness"),
                self.second_arg.expect("no second arg in witness"),
                self.denominator.expect("no denominator in witness"),
            );
            let case = if expect_failure {
                case
            } else {
                case.expect(self.expected.expect("no expected arg in witness"))
            };
            case.witness.into()
        }
    }

    struct FuzzCaseBuilder {
        case: Case,
        builder: U8MulDivFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(builder: U8MulDivFuzzEngineBuilder, test_name: &'static str) -> FuzzCaseBuilder {
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
            let expect_failure = matches!(self.expect, Expect::AssertFailed);
            let strategy = inputs
                .prop_map(move |fuzz_case| {
                    let arguments: Arguments = U8MulDivTestArguments {}.into();
                    let witness: WitnessValues =
                        fuzz_case.into_witness(witness.clone(), expect_failure);

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U8MulDivTestArguments, U8MulDivTestWitness, _>::new()
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
    fn mul_div_8_product_is_u8(builder: U8MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (arb_non_zero_u8(), arb_non_zero_u8()).prop_map(|(a, c)| {
            let b = u8::MAX / a;
            FuzzCase::first_numerator(a)
                .second_numerator(b)
                .denominator(c)
                .expect(a * b / c)
        });

        case_fuzz(builder, "mul_div_8 product is u8")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_8_intermediate_overflow(builder: U8MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (2u8..=u8::MAX).prop_flat_map(|a| {
            (a..=u8::MAX).prop_map(move |c| {
                let b = u8::MAX;
                let expected = (u16::from(a) * u16::from(b) / u16::from(c)) as u8;
                FuzzCase::first_numerator(a)
                    .second_numerator(b)
                    .denominator(c)
                    .expect(expected)
            })
        });

        case_fuzz(builder, "mul_div_8 intermediate overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_8_result_overflow(builder: U8MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (2u8..=u8::MAX).prop_flat_map(|a| {
            (1u8..a).prop_map(move |c| {
                FuzzCase::first_numerator(a)
                    .second_numerator(u8::MAX)
                    .denominator(c)
            })
        });

        case_fuzz(builder, "mul_div_8 result overflow")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_8_div_by_zero(builder: U8MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (arb_non_zero_u8(), arb_non_zero_u8()).prop_map(|(a, b)| {
            FuzzCase::first_numerator(a)
                .second_numerator(b)
                .denominator(0)
                .expect(0)
        });

        case_fuzz(builder, "mul_div_8 div by zero")
            .strategy(strategy)
            .run()
    }
}
