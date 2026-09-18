// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use std::fmt::Debug;

use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
use simplex::fuzz::engine::FuzzStrategyBuilder;
use simplex::fuzz::proptest::prelude::Just;
use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
use simplex::fuzz::{FuzzEngineBuilder, FuzzError, FuzzableProgram};
use simplex::program::{
    ArgumentsTrait, ProgramFactory, RandomArguments, RandomWitness, WitnessTrait,
};
use simplex::simplicityhl::{Arguments, WitnessValues};
use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

use super::core::{Expect, FuzzExecutionCheck};
use super::uint::{CommonOp, TestUint};

const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);

/// Fuzz-specific program data and number generators for a common unsigned
/// integer width.
pub trait TestUintFuzz: TestUint + Debug + 'static {
    /// Program arguments used for each generated test case.
    type Arguments: ArgumentsTrait + RandomArguments + Debug + Clone + Into<Arguments> + 'static;

    /// Returns the fixed program arguments for this uint test program.
    fn arguments() -> Self::Arguments;

    /// Generates any representable value of this uint type.
    fn arb_any() -> BoxedStrategy<Self>;

    /// Generates any representable value except zero.
    fn arb_non_zero() -> BoxedStrategy<Self>;

    /// Generates a value in the inclusive interval `low..=high`.
    ///
    /// Implementations should preserve both bounds exactly. This lets shared
    /// scenarios construct ranges such as `0..=a` and `(a + 1)..=MAX` without
    /// changing their input shape.
    fn arb_fitting(low: Self, high: Self) -> BoxedStrategy<Self>;

    /// Generates an addend in the fitting range used by `uint.rs`.
    fn arb_fitting_addend() -> BoxedStrategy<Self> {
        Self::arb_fitting(Self::ZERO, Self::HALF_MAX)
    }

    /// Generates a multiplier in the fitting range used by `uint.rs`.
    fn arb_fitting_multiplier() -> BoxedStrategy<Self> {
        Self::arb_fitting(Self::ZERO, Self::MUL_BOUND - Self::ONE)
    }
}

fn initial_transaction() -> FinalTransaction {
    let mut tx = FinalTransaction::new();
    tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
    tx
}

fn transaction_builder() -> Result<FinalTransactionBuilder, FuzzError> {
    FinalTransactionBuilder::new(initial_transaction(), [PROGRAM_TARGET])
}

fn uint_strategy<T: TestUintFuzz>(
    operation: CommonOp,
    inputs: BoxedStrategy<(T, T, Option<T>)>,
) -> BoxedStrategy<(Arguments, WitnessValues)> {
    let function_index = operation as u8;

    FuzzStrategyBuilder::<T::Arguments, T::Witness, _>::new()
        .with_custom_strategy(inputs.prop_map(move |(a, b, expected)| {
            let arguments: Arguments = T::arguments().into();
            let witness: WitnessValues = T::witness(function_index, a, b, expected).into();

            (arguments, witness)
        }))
        .build()
}

pub fn checked_add_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedAdd,
        (T::arb_fitting_addend(), T::arb_fitting_addend())
            .prop_map(|(a, b)| (a, b, Some(a + b)))
            .boxed(),
    )
}

pub fn checked_add_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedAdd,
        (Just(T::MAX), T::arb_non_zero())
            .prop_map(|(a, b)| (a, b, None))
            .boxed(),
    )
}

pub fn safe_add_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeAdd,
        (T::arb_fitting_addend(), T::arb_fitting_addend())
            .prop_map(|(a, b)| (a, b, Some(a + b)))
            .boxed(),
    )
}

pub fn safe_add_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeAdd,
        (Just(T::MAX), T::arb_non_zero())
            .prop_map(|(a, b)| (a, b, None))
            .boxed(),
    )
}

pub fn checked_sub_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedSub,
        T::arb_fitting(T::ZERO, T::MAX)
            .prop_flat_map(|a| T::arb_fitting(T::ZERO, a).prop_map(move |b| (a, b, Some(a - b))))
            .boxed(),
    )
}

pub fn checked_sub_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedSub,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_flat_map(|a| T::arb_fitting(a + T::ONE, T::MAX).prop_map(move |b| (a, b, None)))
            .boxed(),
    )
}

pub fn safe_sub_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeSub,
        T::arb_fitting(T::ZERO, T::MAX)
            .prop_flat_map(|a| T::arb_fitting(T::ZERO, a).prop_map(move |b| (a, b, Some(a - b))))
            .boxed(),
    )
}

pub fn safe_sub_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeSub,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_flat_map(|a| T::arb_fitting(a + T::ONE, T::MAX).prop_map(move |b| (a, b, None)))
            .boxed(),
    )
}

pub fn checked_mul_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedMul,
        (T::arb_fitting_multiplier(), T::arb_fitting_multiplier())
            .prop_map(|(a, b)| (a, b, Some(a * b)))
            .boxed(),
    )
}

pub fn checked_mul_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedMul,
        (Just(T::MAX), T::arb_fitting(T::ONE + T::ONE, T::MAX))
            .prop_map(|(a, b)| (a, b, None))
            .boxed(),
    )
}

pub fn safe_mul_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeMul,
        (T::arb_fitting_multiplier(), T::arb_fitting_multiplier())
            .prop_map(|(a, b)| (a, b, Some(a * b)))
            .boxed(),
    )
}

pub fn safe_mul_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeMul,
        (Just(T::MAX), T::arb_fitting(T::ONE + T::ONE, T::MAX))
            .prop_map(|(a, b)| (a, b, None))
            .boxed(),
    )
}

pub fn checked_div_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedDiv,
        (T::arb_any(), T::arb_non_zero())
            .prop_map(|(a, b)| (a, b, Some(a / b)))
            .boxed(),
    )
}

pub fn checked_div_by_zero_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)>
{
    uint_strategy(
        CommonOp::CheckedDiv,
        T::arb_any().prop_map(|a| (a, T::ZERO, None)).boxed(),
    )
}

pub fn safe_div_fitting_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeDiv,
        (T::arb_any(), T::arb_non_zero())
            .prop_map(|(a, b)| (a, b, Some(a / b)))
            .boxed(),
    )
}

pub fn safe_div_by_zero_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::SafeDiv,
        T::arb_any().prop_map(|a| (a, T::ZERO, None)).boxed(),
    )
}

pub fn gt_greater_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Gt,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_flat_map(|b| {
                T::arb_fitting(b + T::ONE, T::MAX).prop_map(move |a| (a, b, Some(T::ZERO)))
            })
            .boxed(),
    )
}

pub fn gt_equal_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Gt,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_map(|a| (a, a, None))
            .boxed(),
    )
}

pub fn gt_less_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Gt,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_flat_map(|a| T::arb_fitting(a + T::ONE, T::MAX).prop_map(move |b| (a, b, None)))
            .boxed(),
    )
}

pub fn ge_greater_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Ge,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_flat_map(|b| {
                T::arb_fitting(b + T::ONE, T::MAX).prop_map(move |a| (a, b, Some(T::ZERO)))
            })
            .boxed(),
    )
}

pub fn ge_equal_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Ge,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_map(|a| (a, a, Some(T::ZERO)))
            .boxed(),
    )
}

pub fn ge_less_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Ge,
        T::arb_fitting(T::ZERO, T::MAX - T::ONE)
            .prop_flat_map(|a| T::arb_fitting(a + T::ONE, T::MAX).prop_map(move |b| (a, b, None)))
            .boxed(),
    )
}

pub fn run_uint_fuzz<T: TestUintFuzz>(
    fuzz_engine_builder: FuzzEngineBuilder<T::Program, T::Arguments, T::Witness>,
    strategy: BoxedStrategy<(Arguments, WitnessValues)>,
    expect: Expect,
) -> anyhow::Result<()>
where
    T::Program: FuzzableProgram<T::Program> + ProgramFactory<T::Program> + Clone + 'static,
    T::Witness: WitnessTrait + RandomWitness + Debug + Clone + 'static,
{
    let transaction_builder = transaction_builder()?;

    fuzz_engine_builder
        .build(strategy, transaction_builder)
        .run_with_check(FuzzExecutionCheck::new("uint operation", expect));

    Ok(())
}

/// Stamps the common `#[simplex::fuzz]` entry points for one unsigned width.
///
/// Each expansion still emits one independently selectable fuzz test per
/// scenario. The macro only removes the identical runner wiring from the
/// width-specific test modules.
///
/// # Usage
///
/// ```ignore
/// uint_fuzz_tests!(u8, U8MathFuzzEngineBuilder);
/// ```
///
/// The type must implement [`TestUintFuzz`], including its bounded
/// number-generation strategy.
#[macro_export]
macro_rules! uint_fuzz_tests {
    ($t:ty, $fuzz_engine_builder:ty) => {
        use simplex::fuzz;

        $crate::uint_fuzz_tests!(@stub $t, $fuzz_engine_builder;
            checked_add_fitting   checked_add_fitting_strategy   Ok
            checked_add_overflow  checked_add_overflow_strategy  Ok
            safe_add_fitting      safe_add_fitting_strategy      Ok
            safe_add_overflow     safe_add_overflow_strategy     PrunedBranch
            checked_sub_fitting   checked_sub_fitting_strategy   Ok
            checked_sub_overflow  checked_sub_overflow_strategy  Ok
            safe_sub_fitting      safe_sub_fitting_strategy      Ok
            safe_sub_overflow     safe_sub_overflow_strategy     PrunedBranch
            checked_mul_fitting   checked_mul_fitting_strategy   Ok
            checked_mul_overflow  checked_mul_overflow_strategy  Ok
            safe_mul_fitting      safe_mul_fitting_strategy      Ok
            safe_mul_overflow     safe_mul_overflow_strategy     PrunedBranch
            checked_div_fitting   checked_div_fitting_strategy   Ok
            checked_div_by_zero   checked_div_by_zero_strategy   Ok
            safe_div_fitting      safe_div_fitting_strategy      Ok
            safe_div_by_zero      safe_div_by_zero_strategy      PrunedBranch
            gt_greater            gt_greater_strategy            Ok
            gt_equal              gt_equal_strategy              Ok
            gt_less               gt_less_strategy               Ok
            ge_greater            ge_greater_strategy            Ok
            ge_equal              ge_equal_strategy              Ok
            ge_less               ge_less_strategy               Ok
        );
    };
    (@stub $t:ty, $fuzz_engine_builder:ty; $(
        $name:ident $strategy:ident $expected_execution:ident
    )+) => {
        $(
            #[simplex::fuzz]
            fn $name(fuzz_engine_builder: $fuzz_engine_builder) -> anyhow::Result<()> {
                $crate::common::uint_fuzz::run_uint_fuzz::<$t>(
                    fuzz_engine_builder,
                    $crate::common::uint_fuzz::$strategy::<$t>(),
                    $crate::common::core::Expect::$expected_execution,
                )
            }
        )+
    };
}
