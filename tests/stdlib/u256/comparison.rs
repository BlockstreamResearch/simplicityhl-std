use primitive_types::U256;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::comparison::ComparisonProgram as U256TestCompareProgram;
use simplicityhl_std::artifacts::tests::u256::comparison::derived_comparison::{
    ComparisonArguments as U256TestCompareArguments, ComparisonWitness as U256TestCompareWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    IsZero256,
    Lt256,
    Le256,
}

fn program() -> U256TestCompareProgram {
    U256TestCompareProgram::new(U256TestCompareArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256TestCompareWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256TestCompareWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            expected_bool: false,
        },
    }
}

impl Case {
    /// Only `first_arg`, for the arms that ignore the second operand.
    fn arg(mut self, a: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self
    }

    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: [u8; 32], b: [u8; 32]) -> Self {
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
fn is_zero_256_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = [0; 32];

    case(IsZero256).arg(a).flag(true).run(&context)
}

#[simplex::test]
fn is_zero_256_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

    case(IsZero256).arg(a).run(&context)
}

#[simplex::test]
fn lt_256_less(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX - 1);
    let b = a + 1;

    case(Lt256)
        .args(a.to_big_endian(), b.to_big_endian())
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn lt_256_eq(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX).to_big_endian();

    case(Lt256).args(a, a).run(&context)
}

#[simplex::test]
fn lt_256_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = a - 1;

    case(Lt256)
        .args(a.to_big_endian(), b.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn le_256_less(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX - 1);
    let b = a + 1;

    case(Le256)
        .args(a.to_big_endian(), b.to_big_endian())
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn le_256_eq(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

    case(Le256).args(a, a).flag(true).run(&context)
}

#[simplex::test]
fn le_256_bigger(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = a - 1;

    case(Le256)
        .args(a.to_big_endian(), b.to_big_endian())
        .run(&context)
}

mod comparison_tests_fuzz {
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

    const EXPECTED_TRUE: bool = true;
    const EXPECTED_FALSE: bool = false;

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type U256ComparisonFuzzEngineBuilder =
        FuzzEngineBuilder<U256TestCompareProgram, U256TestCompareArguments, U256TestCompareWitness>;

    // (A, B, Expected)
    type ComparisonInputs = (U256, U256, bool);

    fn arb_u256() -> impl Strategy<Value = U256> {
        any::<[u8; 32]>().prop_map(|bytes| U256::from_big_endian(&bytes))
    }

    fn arb_non_zero_u256() -> impl Strategy<Value = U256> {
        arb_u256().prop_map(|value| value.max(U256::one()))
    }

    fn arb_u256_in_range(low: U256, high: U256) -> impl Strategy<Value = U256> {
        assert!(low <= high);

        let range = high - low;

        arb_u256().prop_map(move |value| {
            if range == U256::MAX {
                value
            } else {
                low + value % (range + U256::one())
            }
        })
    }

    struct CaseFuzz {
        case: Case,
        builder: U256ComparisonFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<ComparisonInputs>>,
        test_name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: U256ComparisonFuzzEngineBuilder,
        test_name: &'static str,
    ) -> CaseFuzz {
        CaseFuzz {
            case: case(function),
            builder,
            inputs: None,
            test_name,
        }
    }

    impl CaseFuzz {
        fn strategy(mut self, inputs: impl Strategy<Value = ComparisonInputs> + 'static) -> Self {
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

            let strategy = inputs
                .prop_map(move |(a, b, expected)| (a.to_big_endian(), b.to_big_endian(), expected))
                .prop_map(move |(a, b, expected)| {
                    let arguments: Arguments = U256TestCompareArguments {}.into();
                    let witness: WitnessValues = Case {
                        witness: witness.clone(),
                    }
                    .args(a, b)
                    .flag(expected)
                    .witness
                    .into();

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U256TestCompareArguments, U256TestCompareWitness, _>::new()
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
    fn is_zero_256_false(
        fuzz_engine_builder: U256ComparisonFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = arb_non_zero_u256().prop_map(|a| (a, U256::zero(), EXPECTED_FALSE));

        case_fuzz(IsZero256, fuzz_engine_builder, "u256 is non-zero")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn lt_256_less(fuzz_engine_builder: U256ComparisonFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::zero(), U256::MAX - 1);

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(a, U256::MAX);

                b.prop_map(move |b| (a, b, EXPECTED_TRUE))
            })
        };

        case_fuzz(Lt256, fuzz_engine_builder, "u256 less than")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn lt_256_eq(fuzz_engine_builder: U256ComparisonFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(|a| (a, a, EXPECTED_FALSE));

        case_fuzz(Lt256, fuzz_engine_builder, "u256 less than equal")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn lt_256_bigger(fuzz_engine_builder: U256ComparisonFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::zero(), a);

                b.prop_map(move |b| (a, b, EXPECTED_FALSE))
            })
        };

        case_fuzz(Lt256, fuzz_engine_builder, "u256 greater than")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn le_256_less(fuzz_engine_builder: U256ComparisonFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::zero(), U256::MAX - 1);

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(a, U256::MAX);

                b.prop_map(move |b| (a, b, EXPECTED_TRUE))
            })
        };

        case_fuzz(Le256, fuzz_engine_builder, "u256 less than or equal")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn le_256_eq(fuzz_engine_builder: U256ComparisonFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(|a| (a, a, EXPECTED_TRUE));

        case_fuzz(Le256, fuzz_engine_builder, "u256 less than or equal equal")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn le_256_bigger(fuzz_engine_builder: U256ComparisonFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::zero(), a);

                b.prop_map(move |b| (a, b, EXPECTED_FALSE))
            })
        };

        case_fuzz(Le256, fuzz_engine_builder, "u256 greater than")
            .strategy(strategy)
            .run()
    }
}
