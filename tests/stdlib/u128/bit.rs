use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::bit::BitProgram as U128TestBitsProgram;
use simplicityhl_std::artifacts::tests::u128::bit::derived_bit::{
    BitArguments as U128TestBitsArguments, BitWitness as U128TestBitsWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    And128,
    Or128,
    Eq128,
    LeftShift128,
    RightShift128,
}

const EXPECT_EQUAL: bool = true;
const EXPECT_NOT_EQUAL: bool = false;

fn program() -> U128TestBitsProgram {
    U128TestBitsProgram::new(U128TestBitsArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U128TestBitsWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U128TestBitsWitness {
            function_index: function as u8,
            first_arg: 0,
            second_arg: 0,
            expected: None,
            expected_bool: false,
        },
    }
}

impl Case {
    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: u128, b: u128) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u128) -> Self {
        self.witness.expected = Some(expected);
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
fn and_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..=u128::MAX);
    let result = a & b;

    case(And128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn or_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..=u128::MAX);
    let result = a | b;

    case(Or128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn eq_128_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(Eq128)
        .args(a, a)
        .expect(0)
        .flag(EXPECT_EQUAL)
        .run(&context)
}

#[simplex::test]
fn eq_128_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = a - 1;

    case(Eq128)
        .args(a, b)
        .expect(0)
        .flag(EXPECT_NOT_EQUAL)
        .run(&context)
}

#[simplex::test]
fn left_shift_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = rand::thread_rng().gen_range(1..=127_u128);
    let val = rand::thread_rng().gen_range(0..=u128::MAX);
    let result = val << shift;

    case(LeftShift128)
        .args(shift, val)
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn left_shift_128_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = 0;
    let val = rand::thread_rng().gen_range(0..=u128::MAX);
    let result = val;

    case(LeftShift128)
        .args(shift, val)
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn left_shift_128_out_of_range(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = rand::thread_rng().gen_range(128..=u8::MAX as u128);
    let val = rand::thread_rng().gen_range(0..=u128::MAX);

    case(LeftShift128).args(shift, val).expect(0).run(&context)
}

#[simplex::test]
fn right_shift_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = rand::thread_rng().gen_range(1..=127_u128);
    let val = rand::thread_rng().gen_range(0..=u128::MAX);
    let result = val >> shift;

    case(RightShift128)
        .args(shift, val)
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn right_shift_128_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = 0;
    let val = rand::thread_rng().gen_range(0..=u128::MAX);
    let result = val;

    case(RightShift128)
        .args(shift, val)
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn right_shift_128_out_of_range(context: simplex::TestContext) -> anyhow::Result<()> {
    let shift = rand::thread_rng().gen_range(128..=u8::MAX as u128);
    let val = rand::thread_rng().gen_range(0..=u128::MAX);

    case(RightShift128).args(shift, val).expect(0).run(&context)
}

mod bit_tests_fuzz {
    use super::*;
    use crate::common::core::FuzzExecutionCheck;
    use primitive_types::U256;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    type Builder =
        FuzzEngineBuilder<U128TestBitsProgram, U128TestBitsArguments, U128TestBitsWitness>;

    fn arb_u8() -> impl Strategy<Value = u8> {
        any::<u8>()
    }

    fn arb_u128() -> impl Strategy<Value = u128> {
        any::<u128>()
    }

    /// Values for one bit operation before building the contract witness.
    #[derive(Debug, Default)]
    struct FuzzCase {
        first_arg: Option<u128>,
        second_arg: Option<u128>,
        expected: Option<u128>,
        expected_bool: Option<bool>,
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

        fn expect(mut self, expected: u128) -> Self {
            let _ = self.expected.insert(expected);
            self
        }

        fn flag(mut self, expected_bool: bool) -> Self {
            let _ = self.expected_bool.insert(expected_bool);
            self
        }

        fn into_witness(self, witness: U128TestBitsWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_arg.expect("no first_arg in witness"),
                    self.second_arg.expect("no second_arg in witness"),
                )
                .expect(self.expected.expect("no expected in witness"))
                .flag(self.expected_bool.expect("no expected_bool in witness"))
                .witness
                .into()
        }
    }

    struct FuzzCaseBuilder {
        case: Case,
        builder: Builder,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: Builder,
        name: &'static str,
    ) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(function),
            builder,
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
                .prop_map(move |case| {
                    let arguments: Arguments = U128TestBitsArguments {}.into();
                    let witness = case.into_witness(witness.clone());
                    (arguments, witness)
                });
            let strategy =
                FuzzStrategyBuilder::<U128TestBitsArguments, U128TestBitsWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();
            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
            let tx_builder = FinalTransactionBuilder::new(tx, [ProgramTarget::Input(0)])?;
            self.builder
                .build(strategy, tx_builder)
                .run_with_check(FuzzExecutionCheck::new(self.name, Expect::Ok));
            Ok(())
        }
    }

    #[simplex::fuzz]
    fn and(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128())
                .prop_map(|(a, b)| FuzzCase::first_arg(a).second_arg(b).expect(a & b))
        };

        case_fuzz(And128, builder, "u128 bitwise and")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn or(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128())
                .prop_map(|(a, b)| FuzzCase::first_arg(a).second_arg(b).expect(a | b))
        };

        case_fuzz(Or128, builder, "u128 bitwise or")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn eq(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128())
                .prop_map(|(a, b)| FuzzCase::first_arg(a).second_arg(b).flag(a == b))
        };

        case_fuzz(Eq128, builder, "u128 equality")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn eq_same(builder: Builder) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|a| FuzzCase::first_arg(a).second_arg(a).flag(EXPECT_EQUAL)) };

        case_fuzz(Eq128, builder, "u128 equality with equal operands")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn eq_different(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_u128().prop_map(|a| {
                FuzzCase::first_arg(a)
                    .second_arg(a.wrapping_add(1))
                    .flag(EXPECT_NOT_EQUAL)
            })
        };

        case_fuzz(Eq128, builder, "u128 inequality")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn left_shift(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u8(), arb_u128()).prop_map(|(shift, value)| {
                let expected = if shift >= 128 { 0 } else { value << shift };
                FuzzCase::first_arg(shift as u128)
                    .second_arg(value)
                    .expect(expected)
            })
        };

        case_fuzz(LeftShift128, builder, "u128 left shift")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn right_shift(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u8(), arb_u128()).prop_map(|(shift, value)| {
                let expected = if shift >= 128 { 0 } else { value >> shift };
                FuzzCase::first_arg(shift as u128)
                    .second_arg(value)
                    .expect(expected)
            })
        };

        case_fuzz(RightShift128, builder, "u128 right shift")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn left_shift_by_zero(builder: Builder) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|value| FuzzCase::first_arg(0).second_arg(value).expect(value)) };

        case_fuzz(LeftShift128, builder, "u128 left shift by zero")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn right_shift_by_zero(builder: Builder) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|value| FuzzCase::first_arg(0).second_arg(value).expect(value)) };

        case_fuzz(RightShift128, builder, "u128 right shift by zero")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn left_shift_out_of_range(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (128u8..=u8::MAX, arb_u128()).prop_map(|(shift, value)| {
                FuzzCase::first_arg(shift as u128)
                    .second_arg(value)
                    .expect(0)
            })
        };

        case_fuzz(LeftShift128, builder, "u128 left shift out of range")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn right_shift_out_of_range(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (128u8..=u8::MAX, arb_u128()).prop_map(|(shift, value)| {
                FuzzCase::first_arg(shift as u128)
                    .second_arg(value)
                    .expect(0)
            })
        };

        case_fuzz(RightShift128, builder, "u128 right shift out of range")
            .strategy(strategy)
            .run()
    }
}
