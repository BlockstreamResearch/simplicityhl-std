use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u32::mul_div::MulDivProgram as U32MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u32::mul_div::derived_mul_div::{
    MulDivArguments as U32MulDivTestArguments, MulDivWitness as U32MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U32MulDivTestProgram {
    U32MulDivTestProgram::new(&U32MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
///
/// Every field the chosen arm ignores keeps its default, so a test names only
/// the values it actually depends on.
struct Case {
    witness: U32MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U32MulDivTestWitness {
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
    fn args(mut self, a: u32, b: u32, c: u32) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u32) -> Self {
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
fn mul_div_32_product_is_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX) as u32;
    let b = rand::thread_rng().gen_range(0..=u16::MAX) as u32;
    let c = rand::thread_rng().gen_range(1..=u32::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_32_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u32::MAX);
    let b = u32::MAX;
    let c = rand::thread_rng().gen_range(a..=u32::MAX);

    let res = (a as u64) * (b as u64) / (c as u64);

    case(MulDiv).args(a, b, c).expect(res as u32).run(&context)
}

#[simplex::test]
fn mul_div_32_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u32::MAX);
    let b = u32::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_32_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u32::MAX);
    let b = rand::thread_rng().gen_range(1..=u32::MAX);
    let c = 0;

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

mod mul_div_tests_fuzz {
    use super::*;

    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::fuzz_check::{Expect, FuzzExecutionCheck};
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::{FuzzMode, TestContext};

    fn arb_u16() -> impl Strategy<Value = u16> {
        any::<u16>()
    }

    fn arb_non_zero_u32() -> impl Strategy<Value = u32> {
        any::<u32>().prop_filter("u32 should not be zero", |index| *index != 0)
    }

    #[derive(Debug, Default)]
    struct FuzzCase {
        first_numerator: Option<u32>,
        second_numerator: Option<u32>,
        denominator: Option<u32>,
        expected: Option<u32>,
    }

    impl FuzzCase {
        fn from_numerator(first_numerator: u32) -> Self {
            let mut x = Self::default();
            let _ = x.first_numerator.insert(first_numerator);
            x
        }

        fn mul(mut self, second_numerator: u32) -> Self {
            let _ = self.second_numerator.insert(second_numerator);
            self
        }

        fn div(mut self, denominator: u32) -> Self {
            let _ = self.denominator.insert(denominator);
            self
        }

        fn expect(mut self, expected: u32) -> Self {
            let _ = self.expected.insert(expected);
            self
        }

        fn into_witness(
            self,
            witness: U32MulDivTestWitness,
            expect_failure: bool,
        ) -> WitnessValues {
            let case = Case { witness }.args(
                self.first_numerator.expect("no first numerator in witness"),
                self.second_numerator
                    .expect("no second numerator in witness"),
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
        context: TestContext<FuzzMode>,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(context: TestContext<FuzzMode>, test_name: &'static str) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(MulDiv),
            context,
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

        fn run(self) -> anyhow::Result<()> {
            let Case { witness } = self.case;
            let inputs = self.inputs.expect("a fuzz strategy must be specified");
            let expect_failure = matches!(self.expect, Expect::AssertFailed);

            let strategy = inputs
                .prop_map(move |fuzz_case| {
                    let arguments: Arguments = U32MulDivTestArguments {}.into();
                    let witness: WitnessValues =
                        fuzz_case.into_witness(witness.clone(), expect_failure);

                    (arguments, witness)
                })
                .boxed();

            self.context
                .engine::<U32MulDivTestProgram, U32MulDivTestArguments, U32MulDivTestWitness>()
                .with_custom_strategy(strategy)
                .with_custom_check(FuzzExecutionCheck::new(self.test_name, self.expect))
                .run();

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn mul_div_32_product_is_u32(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (arb_u16(), arb_u16(), arb_non_zero_u32()).prop_map(|(a, b, c)| {
            let a = u32::from(a);
            let b = u32::from(b);

            FuzzCase::from_numerator(a).mul(b).div(c).expect(a * b / c)
        });

        case_fuzz(context, "mul_div_32_product_is_u32")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_32_intermediate_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (2u32..=u32::MAX).prop_flat_map(|a| {
            (a..=u32::MAX).prop_map(move |c| {
                let b = u32::MAX;
                let expected = ((u64::from(a) * u64::from(b)) / u64::from(c)) as u32;

                FuzzCase::from_numerator(a).mul(b).div(c).expect(expected)
            })
        });

        case_fuzz(context, "mul_div_32_intermediate_overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_32_result_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (2u32..=u32::MAX).prop_flat_map(|a| {
            (1u32..a).prop_map(move |c| FuzzCase::from_numerator(a).mul(u32::MAX).div(c))
        });

        case_fuzz(context, "mul_div_32_result_overflow")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_32_div_by_zero(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (arb_non_zero_u32(), arb_non_zero_u32())
            .prop_map(|(a, b)| FuzzCase::from_numerator(a).mul(b).div(0).expect(0));

        case_fuzz(context, "mul_div_32_div_by_zero")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }
}
