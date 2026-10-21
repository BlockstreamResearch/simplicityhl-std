use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::mul_div::MulDivProgram as U128MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u128::mul_div::derived_mul_div::{
    MulDivArguments as U128MulDivTestArguments, MulDivWitness as U128MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U128MulDivTestProgram {
    U128MulDivTestProgram::new(&U128MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U128MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U128MulDivTestWitness {
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
    fn args(mut self, a: u128, b: u128, c: u128) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u128) -> Self {
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
fn mul_div_128_product_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
    let b = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
    let c = rand::thread_rng().gen_range(1..=u128::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_128_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u128::MAX);
    let b = u128::MAX;
    let c = rand::thread_rng().gen_range(a..=u128::MAX);

    let res = U256::from(a) * U256::from(b) / U256::from(c);

    case(MulDiv)
        .args(a, b, c)
        .expect(res.low_u128())
        .run(&context)
}

#[simplex::test]
fn mul_div_128_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u128::MAX);
    let b = u128::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_128_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let c = 0;

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

mod mul_div_tests_fuzz {
    use super::*;

    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::Expect;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::{FuzzMode, TestContext};

    fn arb_u64() -> impl Strategy<Value = u64> {
        any::<u64>()
    }

    fn arb_non_zero_u128() -> impl Strategy<Value = u128> {
        any::<u128>().prop_filter("u128 should not be zero", |value| *value != 0)
    }

    #[derive(Debug, Default)]
    struct FuzzCase {
        first_numerator: Option<u128>,
        second_numerator: Option<u128>,
        denominator: Option<u128>,
        expected: Option<u128>,
    }

    impl FuzzCase {
        fn from_numerator(first_numerator: u128) -> Self {
            Self {
                first_numerator: Some(first_numerator),
                ..Self::default()
            }
        }

        fn mul(mut self, second_numerator: u128) -> Self {
            self.second_numerator = Some(second_numerator);
            self
        }

        fn div(mut self, denominator: u128) -> Self {
            self.denominator = Some(denominator);
            self
        }

        fn expect(mut self, expected: u128) -> Self {
            self.expected = Some(expected);
            self
        }

        fn into_witness(
            self,
            witness: U128MulDivTestWitness,
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
                case.expect(self.expected.expect("no expected result in witness"))
            };

            case.witness.into()
        }
    }

    struct FuzzCaseBuilder {
        case: Case,
        context: TestContext<FuzzMode>,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        expect: Expect,
        test_name: &'static str,
    }

    fn case_fuzz(context: TestContext<FuzzMode>, test_name: &'static str) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(MulDiv),
            context,
            inputs: None,
            expect: Expect::Accept,
            test_name,
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
            let expect_failure = matches!(self.expect, Expect::Reject);

            let strategy = inputs
                .prop_map(move |case| {
                    let arguments: Arguments = U128MulDivTestArguments {}.into();
                    let witness = case.into_witness(witness.clone(), expect_failure);

                    (arguments, witness)
                })
                .boxed();

            crate::common::fuzz::run::<U128MulDivTestProgram>(
                &self.context,
                strategy,
                self.test_name,
                self.expect,
            );

            Ok(())
        }
    }

    // mul_div_128_product_is_u128 - accepts custom inputs for the same operation.
    // 0<=a,b<=u64::MAX and c>0; a*b fits u128; checks floor(a*b/c).
    #[simplex::fuzz]
    fn product_fits_u128(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u64(), arb_u64(), arb_non_zero_u128()).prop_map(|(a, b, c)| {
                let (a, b) = (a as u128, b as u128);

                FuzzCase::from_numerator(a).mul(b).div(c).expect(a * b / c)
            })
        };

        case_fuzz(context, "u128 mul div with fitting product")
            .strategy(strategy)
            .run()
    }

    // mul_div_128_intermediate_overflow - accepts custom inputs for the same operation.
    // 2<=a<=MAX, b=MAX, a<=c<=MAX; the widened product overflows u128 but the quotient fits.
    #[simplex::fuzz]
    fn intermediate_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (2u128..=u128::MAX).prop_flat_map(|a| {
                (a..=u128::MAX).prop_map(move |c| {
                    let expected =
                        (U256::from(a) * U256::from(u128::MAX) / U256::from(c)).low_u128();

                    FuzzCase::from_numerator(a)
                        .mul(u128::MAX)
                        .div(c)
                        .expect(expected)
                })
            })
        };

        case_fuzz(context, "u128 mul div with intermediate overflow")
            .strategy(strategy)
            .run()
    }

    // mul_div_128_result_overflow - accepts custom inputs for the same operation.
    // 2<=a<=MAX, b=MAX, 1<=c<a; the quotient exceeds MAX and must fail.
    #[simplex::fuzz]
    fn result_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (2u128..=u128::MAX).prop_flat_map(|a| {
                (1..a).prop_map(move |c| FuzzCase::from_numerator(a).mul(u128::MAX).div(c))
            })
        };

        case_fuzz(context, "u128 mul div result overflow")
            .strategy(strategy)
            .expect(Expect::Reject)
            .run()
    }

    // mul_div_128_div_by_zero - accepts custom inputs for the same operation.
    // a,b>0 and c=0; expects an assertion failure; the failure witness has no expected result.
    #[simplex::fuzz]
    fn divide_by_zero(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_non_zero_u128(), arb_non_zero_u128())
                .prop_map(|(a, b)| FuzzCase::from_numerator(a).mul(b).div(0).expect(0))
        };

        case_fuzz(context, "u128 mul div by zero")
            .strategy(strategy)
            .expect(Expect::Reject)
            .run()
    }
}
