use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::bit::BitProgram as U256TestBitsProgram;
use simplicityhl_std::artifacts::tests::u256::bit::derived_bit::{
    BitArguments as U256TestBitsArguments, BitWitness as U256TestBitsWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    And256,
    Or256,
    LeftShift256,
    RightShift256,
}

fn program() -> U256TestBitsProgram {
    U256TestBitsProgram::new(U256TestBitsArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256TestBitsWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256TestBitsWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            expected: None,
        },
    }
}

impl Case {
    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: [u8; 32], b: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: [u8; 32]) -> Self {
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
fn and_256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::zero(), U256::MAX);
    let result = (a & b).to_big_endian();

    case(And256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn or_256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::zero(), U256::MAX);
    let result = (a | b).to_big_endian();

    case(Or256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn left_shift_256(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = rand::thread_rng().gen_range(1..u8::MAX);
    let val = generate_u256(U256::zero(), U256::MAX);
    let result = (val << shift).to_big_endian();

    case(LeftShift256)
        .args(U256::from(shift).to_big_endian(), val.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn left_shift_256_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = 0;
    let val = generate_u256(U256::zero(), U256::MAX).to_big_endian();
    let result = val;

    case(LeftShift256)
        .args(U256::from(shift).to_big_endian(), val)
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn left_shift_256_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = u8::MAX;
    let val = generate_u256(U256::zero(), U256::MAX);
    let result = (val << shift).to_big_endian();

    case(LeftShift256)
        .args(U256::from(shift).to_big_endian(), val.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn right_shift_256(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = rand::thread_rng().gen_range(1..u8::MAX);
    let val = generate_u256(U256::zero(), U256::MAX);
    let result = (val >> shift).to_big_endian();

    case(RightShift256)
        .args(U256::from(shift).to_big_endian(), val.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn right_shift_256_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = 0;
    let val = generate_u256(U256::zero(), U256::MAX).to_big_endian();
    let result = val;

    case(RightShift256)
        .args(U256::from(shift).to_big_endian(), val)
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn right_shift_256_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = u8::MAX;
    let val = generate_u256(U256::zero(), U256::MAX);
    let result = (val >> shift).to_big_endian();

    case(RightShift256)
        .args(U256::from(shift).to_big_endian(), val.to_big_endian())
        .expect(result)
        .run(&context)
}

mod bit_tests_fuzz {
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
    type U256BitFuzzEngineBuilder =
        FuzzEngineBuilder<U256TestBitsProgram, U256TestBitsArguments, U256TestBitsWitness>;

    // (A, B, Result)
    type BitInputs = (U256, U256, U256);

    fn arb_u256() -> impl Strategy<Value = U256> {
        any::<[u8; 32]>().prop_map(|bytes| U256::from_big_endian(&bytes))
    }

    fn non_zero_u8_shift() -> impl Strategy<Value = u8> {
        any::<u8>().prop_filter("shift is unequal to zero", |shift| *shift != 0)
    }

    struct CaseFuzz {
        case: Case,
        builder: U256BitFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<BitInputs>>,
        test_name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: U256BitFuzzEngineBuilder,
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
        fn strategy(mut self, inputs: impl Strategy<Value = BitInputs> + 'static) -> Self {
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
                .prop_map(move |(a, b, expected)| {
                    (
                        a.to_big_endian(),
                        b.to_big_endian(),
                        expected.to_big_endian(),
                    )
                })
                .prop_map(move |(a, b, expected)| {
                    let arguments: Arguments = U256TestBitsArguments {}.into();
                    let witness: WitnessValues = Case {
                        witness: witness.clone(),
                    }
                    .args(a, b)
                    .expect(expected)
                    .witness
                    .into();

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U256TestBitsArguments, U256TestBitsWitness, _>::new()
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
    fn and_256(fuzz_engine_builder: U256BitFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_u256();

            (a, b).prop_map(|(a, b)| (a, b, a & b))
        };

        case_fuzz(And256, fuzz_engine_builder, "u256 bitwise and")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn or_256(fuzz_engine_builder: U256BitFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_u256();

            (a, b).prop_map(|(a, b)| (a, b, a | b))
        };

        case_fuzz(Or256, fuzz_engine_builder, "u256 bitwise or")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn left_shift_256(fuzz_engine_builder: U256BitFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let shift = non_zero_u8_shift();
            let value = arb_u256();

            (shift, value).prop_map(|(shift, value)| {
                let expected = value << shift;

                (U256::from(shift), value, expected)
            })
        };

        case_fuzz(LeftShift256, fuzz_engine_builder, "u256 left shift")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn left_shift_256_by_zero(fuzz_engine_builder: U256BitFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(|value| (U256::zero(), value, value));

        case_fuzz(LeftShift256, fuzz_engine_builder, "u256 left shift by zero")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn left_shift_256_max(fuzz_engine_builder: U256BitFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(|value| {
            let expected = value << u8::MAX;

            (U256::from(u8::MAX), value, expected)
        });

        case_fuzz(LeftShift256, fuzz_engine_builder, "u256 left shift by max")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn right_shift_256(fuzz_engine_builder: U256BitFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let shift = non_zero_u8_shift();
            let value = arb_u256();

            (shift, value).prop_map(|(shift, value)| {
                let expected = value >> shift;

                (U256::from(shift), value, expected)
            })
        };

        case_fuzz(RightShift256, fuzz_engine_builder, "u256 right shift")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn right_shift_256_by_zero(
        fuzz_engine_builder: U256BitFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(|value| (U256::zero(), value, value));

        case_fuzz(
            RightShift256,
            fuzz_engine_builder,
            "u256 right shift by zero",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn right_shift_256_max(fuzz_engine_builder: U256BitFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(|value| {
            let expected = value >> u8::MAX;

            (U256::from(u8::MAX), value, expected)
        });

        case_fuzz(
            RightShift256,
            fuzz_engine_builder,
            "u256 right shift by max",
        )
        .strategy(strategy)
        .run()
    }
}
