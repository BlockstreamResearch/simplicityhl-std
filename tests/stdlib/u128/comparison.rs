use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::comparison::ComparisonProgram as U128TestCompareProgram;
use simplicityhl_std::artifacts::tests::u128::comparison::derived_comparison::{
    ComparisonArguments as U128TestCompareArguments, ComparisonWitness as U128TestCompareWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    IsZero128,
    Lt128,
    Le128,
}

fn program() -> U128TestCompareProgram {
    U128TestCompareProgram::new(&U128TestCompareArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U128TestCompareWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U128TestCompareWitness {
            function_index: function as u8,
            first_arg: 0,
            second_arg: 0,
            expected_bool: false,
        },
    }
}

impl Case {
    /// Only `first_arg`, for the arms that ignore the second operand.
    fn arg(mut self, a: u128) -> Self {
        self.witness.first_arg = a;
        self
    }

    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: u128, b: u128) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// `expected_bool`: the boolean the arm should report.
    fn flag(mut self, expected_bool: bool) -> Self {
        self.witness.expected_bool = expected_bool;
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
fn is_zero_128_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = 0;

    case(IsZero128).arg(a).flag(true).run(&context)
}

#[simplex::test]
fn is_zero_128_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);

    case(IsZero128).arg(a).run(&context)
}

#[simplex::test]
fn lt_128_less(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a + 1;

    case(Lt128).args(a, b).flag(true).run(&context)
}

#[simplex::test]
fn lt_128_eq(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a;

    case(Lt128).args(a, b).run(&context)
}

#[simplex::test]
fn lt_128_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = a - 1;

    case(Lt128).args(a, b).run(&context)
}

#[simplex::test]
fn le_128_less(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a + 1;

    case(Le128).args(a, b).flag(true).run(&context)
}

#[simplex::test]
fn le_128_eq(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = a;

    case(Le128).args(a, b).flag(true).run(&context)
}

#[simplex::test]
fn le_128_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = a - 1;

    case(Le128).args(a, b).run(&context)
}

mod comparison_tests_fuzz {
    use super::*;

    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::Expect;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::{FuzzMode, TestContext};

    const EXPECTED_TRUE: bool = true;
    const EXPECTED_FALSE: bool = false;

    fn arb_u128() -> impl Strategy<Value = u128> {
        any::<u128>()
    }

    fn arb_non_zero_u128() -> impl Strategy<Value = u128> {
        any::<u128>().prop_filter("u128 should not be zero", |index| *index != 0)
    }

    #[derive(Debug, Default)]
    struct FuzzCase {
        first_arg: Option<u128>,
        second_arg: Option<u128>,
        expected: Option<bool>,
    }

    impl FuzzCase {
        fn first_arg(first_arg: u128) -> Self {
            let mut x = Self::default();
            let _ = x.first_arg.insert(first_arg);
            x
        }

        fn second_arg(mut self, second_arg: u128) -> Self {
            let _ = self.second_arg.insert(second_arg);
            self
        }

        fn expect(mut self, expected: bool) -> Self {
            let _ = self.expected.insert(expected);
            self
        }

        fn into_witness(self, witness: U128TestCompareWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_arg.expect("no first arg in witness"),
                    self.second_arg.expect("no second arg in witness"),
                )
                .flag(self.expected.expect("no expected arg in witness"))
                .witness
                .into()
        }
    }

    struct FuzzCaseBuilder {
        case: Case,
        context: TestContext<FuzzMode>,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        context: TestContext<FuzzMode>,
        name: &'static str,
    ) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(function),
            context,
            inputs: None,
            name,
        }
    }

    impl FuzzCaseBuilder {
        fn strategy(mut self, inputs: impl Strategy<Value = FuzzCase> + 'static) -> Self {
            self.inputs = Some(inputs.boxed());
            self
        }

        fn run(self) -> anyhow::Result<()> {
            let witness = self.case.witness;

            let strategy = self
                .inputs
                .expect("a fuzz strategy must be specified")
                .prop_map(move |fuzz_case| {
                    let arguments: Arguments = U128TestCompareArguments {}.into();
                    let witness: WitnessValues = fuzz_case.into_witness(witness.clone());
                    (arguments, witness)
                });

            crate::common::fuzz::run::<U128TestCompareProgram>(
                &self.context,
                strategy,
                self.name,
                Expect::Accept,
            );
            Ok(())
        }
    }

    // is_zero_128_true - accepts custom inputs for the same operation.
    // a is full-width u128; checks a==0, but does not force the original fixed-zero case.
    #[simplex::fuzz]
    fn is_zero(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(IsZero128, context, "u128 is zero")
            .strategy(arb_u128().prop_map(|a| FuzzCase::first_arg(a).second_arg(0).expect(a == 0)))
            .run()
    }

    // lt_128_less - accepts custom inputs for the same operation.
    // lt_128_eq - accepts custom inputs for the same operation.
    // lt_128_bigger - accepts custom inputs for the same operation.
    // Both operands are full-width u128; compares the host boolean across all orderings.
    #[simplex::fuzz]
    fn lt(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(Lt128, context, "u128 less than")
            .strategy(
                (arb_u128(), arb_u128())
                    .prop_map(|(a, b)| FuzzCase::first_arg(a).second_arg(b).expect(a < b)),
            )
            .run()
    }

    // le_128_less - accepts custom inputs for the same operation.
    // le_128_eq - accepts custom inputs for the same operation.
    // le_128_bigger - accepts custom inputs for the same operation.
    // Both operands are full-width u128; compares the host boolean across all orderings.
    #[simplex::fuzz]
    fn le(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(Le128, context, "u128 less than or equal")
            .strategy(
                (arb_u128(), arb_u128())
                    .prop_map(|(a, b)| FuzzCase::first_arg(a).second_arg(b).expect(a <= b)),
            )
            .run()
    }

    // le_128_eq - accepts custom inputs for the same operation.
    // a=b across the full width; expected le=true.
    #[simplex::fuzz]
    fn equal_operands(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(
            Le128,
            context,
            "u128 less than or equal with equal operands",
        )
        .strategy(
            arb_u128().prop_map(|a| FuzzCase::first_arg(a).second_arg(a).expect(EXPECTED_TRUE)),
        )
        .run()
    }

    // is_zero_128_false - accepts custom inputs for the same operation.
    // 1<=a<=u128::MAX; expected is_zero=false.
    #[simplex::fuzz]
    fn non_zero_operand(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(IsZero128, context, "u128 nonzero operand")
            .strategy(arb_non_zero_u128().prop_map(|value| {
                FuzzCase::first_arg(value)
                    .second_arg(0)
                    .expect(EXPECTED_FALSE)
            }))
            .run()
    }

    // lt_128_less - accepts custom inputs for the same operation.
    // 0<=a<MAX and a<b<=MAX; checks the strict-less ordering.
    #[simplex::fuzz]
    fn lt_strict(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(Lt128, context, "u128 strict less than")
            .strategy((0u128..u128::MAX).prop_flat_map(|a| {
                (a + 1..=u128::MAX)
                    .prop_map(move |b| FuzzCase::first_arg(a).second_arg(b).expect(EXPECTED_TRUE))
            }))
            .run()
    }

    // lt_128_eq - accepts custom inputs for the same operation.
    // a=b across the full width; expected lt=false.
    #[simplex::fuzz]
    fn lt_equal(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(Lt128, context, "u128 less than equal operands")
            .strategy(
                arb_u128()
                    .prop_map(|a| FuzzCase::first_arg(a).second_arg(a).expect(EXPECTED_FALSE)),
            )
            .run()
    }

    // lt_128_bigger - accepts custom inputs for the same operation.
    // 1<=a<=MAX and 0<=b<a; checks the strict-greater ordering.
    #[simplex::fuzz]
    fn lt_greater(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(Lt128, context, "u128 less than with greater first operand")
            .strategy(arb_non_zero_u128().prop_flat_map(|a| {
                (0u128..a)
                    .prop_map(move |b| FuzzCase::first_arg(a).second_arg(b).expect(EXPECTED_FALSE))
            }))
            .run()
    }

    // le_128_less - accepts custom inputs for the same operation.
    // 0<=a<MAX and a<b<=MAX; checks the strict-less ordering.
    #[simplex::fuzz]
    fn le_strict_less(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(Le128, context, "u128 strict less than or equal")
            .strategy((0u128..u128::MAX).prop_flat_map(|a| {
                (a + 1..=u128::MAX)
                    .prop_map(move |b| FuzzCase::first_arg(a).second_arg(b).expect(EXPECTED_TRUE))
            }))
            .run()
    }

    // le_128_bigger - accepts custom inputs for the same operation.
    // 1<=a<=MAX and 0<=b<a; checks the strict-greater ordering.
    #[simplex::fuzz]
    fn le_strict_greater(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(Le128, context, "u128 strict greater than")
            .strategy((1u128..=u128::MAX).prop_flat_map(|a| {
                (0u128..a)
                    .prop_map(move |b| FuzzCase::first_arg(a).second_arg(b).expect(EXPECTED_FALSE))
            }))
            .run()
    }
}
