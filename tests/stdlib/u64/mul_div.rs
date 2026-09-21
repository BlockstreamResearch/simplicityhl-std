use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u64::mul_div::MulDivProgram as U64MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u64::mul_div::derived_mul_div::{
    MulDivArguments as U64MulDivTestArguments, MulDivWitness as U64MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U64MulDivTestProgram {
    U64MulDivTestProgram::new(U64MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U64MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U64MulDivTestWitness {
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
    fn args(mut self, a: u64, b: u64, c: u64) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u64) -> Self {
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
fn mul_div_64_product_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX) as u64;
    let b = rand::thread_rng().gen_range(0..=u32::MAX) as u64;
    let c = rand::thread_rng().gen_range(1..=u64::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_64_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u64::MAX);
    let b = u64::MAX;
    let c = rand::thread_rng().gen_range(a..=u64::MAX);

    let res = (a as u128) * (b as u128) / (c as u128);

    case(MulDiv).args(a, b, c).expect(res as u64).run(&context)
}

#[simplex::test]
fn mul_div_64_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u64::MAX);
    let b = u64::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_64_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u64::MAX);
    let b = rand::thread_rng().gen_range(1..=u64::MAX);
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
    use simplex::fuzz::proptest::prelude::{Just, any};
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type U64MulDivFuzzEngineBuilder =
        FuzzEngineBuilder<U64MulDivTestProgram, U64MulDivTestArguments, U64MulDivTestWitness>;

    fn arb_u32() -> impl Strategy<Value = u32> {
        any::<u32>()
    }

    fn arb_non_zero_u64() -> impl Strategy<Value = u64> {
        any::<u64>().prop_filter("u64 should not be zero", |index| *index != 0)
    }

    #[derive(Debug, Default)]
    struct FuzzCase {
        first_numerator: Option<u64>,
        second_numerator: Option<u64>,
        divisor: Option<u64>,
        expected: Option<u64>,
    }

    impl FuzzCase {
        fn first_numerator(first_arg: u64) -> Self {
            let mut x = Self::default();
            let _ = x.first_numerator.insert(first_arg);
            x
        }

        fn second_numerator(mut self, second_arg: u64) -> Self {
            let _ = self.second_numerator.insert(second_arg);
            self
        }

        fn divisor(mut self, denominator: u64) -> Self {
            let _ = self.divisor.insert(denominator);
            self
        }

        fn expect(mut self, expected: u64) -> Self {
            let _ = self.expected.insert(expected);
            self
        }

        fn into_witness(self, witness: U64MulDivTestWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_numerator.expect("no first arg in witness"),
                    self.second_numerator.expect("no second arg in witness"),
                    self.divisor.expect("no third arg in witness"),
                )
                .expect(self.expected.expect("no expected arg in witness"))
                .witness
                .into()
        }
    }

    struct CaseFuzz {
        case: Case,
        builder: U64MulDivFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(builder: U64MulDivFuzzEngineBuilder, test_name: &'static str) -> CaseFuzz {
        CaseFuzz {
            case: case(MulDiv),
            builder,
            inputs: None,
            test_name,
            expect: Expect::Ok,
        }
    }

    impl CaseFuzz {
        fn strategy(mut self, inputs: impl Strategy<Value = FuzzCase> + 'static) -> Self {
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
                .prop_map(move |(fuzz_case)| {
                    let arguments: Arguments = U64MulDivTestArguments {}.into();
                    let witness: WitnessValues = fuzz_case.into_witness(witness.clone());

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U64MulDivTestArguments, U64MulDivTestWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();

            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
            let transaction_builder = FinalTransactionBuilder::new(tx, [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, transaction_builder)
                .run_with_check(FuzzExecutionCheck::new(self.test_name, self.expect));

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn mul_div_64_product_is_u64(builder: U64MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (arb_u32(), arb_u32(), arb_non_zero_u64()).prop_map(|(a, b, c)| {
            let (a, b) = (u64::from(a), u64::from(b));

            FuzzCase::first_numerator(a)
                .second_numerator(b)
                .divisor(c)
                .expect(a * b / c)
        });

        case_fuzz(builder, "mul_div_64_product_is_u64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_64_intermediate_overflow(builder: U64MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (2..=u64::MAX).prop_flat_map(|a| {
            (a..=u64::MAX).prop_map(move |c| {
                let expected = (u128::from(a) * u128::from(u64::MAX) / u128::from(c)) as u64;
                FuzzCase::first_numerator(a)
                    .second_numerator(u64::MAX)
                    .divisor(c)
                    .expect(expected)
            })
        });

        case_fuzz(builder, "mul_div_64_intermediate_overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_64_result_overflow(builder: U64MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = (2..=u64::MAX).prop_flat_map(|a| {
            (1..a).prop_map(move |c| {
                FuzzCase::first_numerator(a)
                    .second_numerator(u64::MAX)
                    .divisor(c)
                    .expect(0)
            })
        });

        case_fuzz(builder, "mul_div_64_result_overflow")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_64_div_by_zero(builder: U64MulDivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy =
            (arb_non_zero_u64(), arb_non_zero_u64(), Just(0u64)).prop_map(|(a, b, c)| {
                FuzzCase::first_numerator(a)
                    .second_numerator(b)
                    .divisor(c)
                    .expect(0)
            });

        case_fuzz(builder, "mul_div_64_div_by_zero")
            .strategy(strategy)
            .run()
    }
}
