// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use std::fmt::Debug;

use super::uint::{CommonOp, TestUint};

use simplex::simplicityhl::{Arguments, WitnessValues};

use simplex::fuzz::proptest::prelude::Just;
use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
use simplex::fuzz::{Expect, FuzzProgramTrait};
use simplex::{FuzzMode, TestContext};

/// Fuzz-specific program data and number generators for a common unsigned
/// integer width.
pub trait TestUintFuzz: TestUint + Debug + 'static {
    /// Program arguments used for each generated test case.
    type Arguments: Debug + Clone + Into<Arguments> + 'static;

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

fn uint_strategy<T: TestUintFuzz>(
    operation: CommonOp,
    inputs: BoxedStrategy<(T, T, Option<T>)>,
) -> BoxedStrategy<(Arguments, WitnessValues)> {
    let function_index = operation as u8;

    inputs
        .prop_map(move |(a, b, expected)| {
            let arguments: Arguments = T::arguments().into();
            let witness: WitnessValues = T::witness(function_index, a, b, expected).into();
            (arguments, witness)
        })
        .boxed()
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
        CommonOp::Add,
        (T::arb_fitting_addend(), T::arb_fitting_addend())
            .prop_map(|(a, b)| (a, b, Some(a + b)))
            .boxed(),
    )
}

pub fn safe_add_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Add,
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
        CommonOp::Sub,
        T::arb_fitting(T::ZERO, T::MAX)
            .prop_flat_map(|a| T::arb_fitting(T::ZERO, a).prop_map(move |b| (a, b, Some(a - b))))
            .boxed(),
    )
}

pub fn safe_sub_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Sub,
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
        CommonOp::Mul,
        (T::arb_fitting_multiplier(), T::arb_fitting_multiplier())
            .prop_map(|(a, b)| (a, b, Some(a * b)))
            .boxed(),
    )
}

pub fn safe_mul_overflow_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Mul,
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
        CommonOp::Div,
        (T::arb_any(), T::arb_non_zero())
            .prop_map(|(a, b)| (a, b, Some(a / b)))
            .boxed(),
    )
}

pub fn safe_div_by_zero_strategy<T: TestUintFuzz>() -> BoxedStrategy<(Arguments, WitnessValues)> {
    uint_strategy(
        CommonOp::Div,
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
    context: TestContext<FuzzMode>,
    strategy: BoxedStrategy<(Arguments, WitnessValues)>,
    expect: Expect,
) -> anyhow::Result<()>
where
    T::Program: FuzzProgramTrait,
    T::Witness: Debug + Clone + 'static,
{
    super::fuzz::run::<T::Program>(&context, strategy, "uint operation", expect);

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
/// uint_fuzz_tests!(u8);
/// ```
///
/// The unsigned type must implement [`TestUintFuzz`], including its bounded
/// number-generation strategy.
#[macro_export]
macro_rules! uint_fuzz_tests {
    ($t:ty) => {

        $crate::uint_fuzz_tests!(@stub $t;
            checked_add_fitting checked_add_fitting_strategy simplex::fuzz::Expect::Accept;
            checked_add_overflow checked_add_overflow_strategy simplex::fuzz::Expect::Accept;
            safe_add_fitting safe_add_fitting_strategy simplex::fuzz::Expect::Accept;
            safe_add_overflow safe_add_overflow_strategy simplex::fuzz::Expect::Reject;
            checked_sub_fitting checked_sub_fitting_strategy simplex::fuzz::Expect::Accept;
            checked_sub_overflow checked_sub_overflow_strategy simplex::fuzz::Expect::Accept;
            safe_sub_fitting safe_sub_fitting_strategy simplex::fuzz::Expect::Accept;
            safe_sub_overflow safe_sub_overflow_strategy simplex::fuzz::Expect::Reject;
            checked_mul_fitting checked_mul_fitting_strategy simplex::fuzz::Expect::Accept;
            checked_mul_overflow checked_mul_overflow_strategy simplex::fuzz::Expect::Accept;
            safe_mul_fitting safe_mul_fitting_strategy simplex::fuzz::Expect::Accept;
            safe_mul_overflow safe_mul_overflow_strategy simplex::fuzz::Expect::Reject;
            checked_div_fitting checked_div_fitting_strategy simplex::fuzz::Expect::Accept;
            checked_div_by_zero checked_div_by_zero_strategy simplex::fuzz::Expect::Accept;
            safe_div_fitting safe_div_fitting_strategy simplex::fuzz::Expect::Accept;
            safe_div_by_zero safe_div_by_zero_strategy simplex::fuzz::Expect::Reject;
            gt_greater gt_greater_strategy simplex::fuzz::Expect::Accept;
            gt_equal gt_equal_strategy simplex::fuzz::Expect::Accept;
            gt_less gt_less_strategy simplex::fuzz::Expect::Accept;
            ge_greater ge_greater_strategy simplex::fuzz::Expect::Accept;
            ge_equal ge_equal_strategy simplex::fuzz::Expect::Accept;
            ge_less ge_less_strategy simplex::fuzz::Expect::Accept;
        );
    };
    (@stub $t:ty; $(
        $name:ident $strategy:ident $expected_execution:expr;
    )+) => {
        $(
            #[simplex::fuzz]
            fn $name(context: simplex::TestContext<simplex::FuzzMode>) -> anyhow::Result<()> {
                $crate::common::uint_fuzz::run_uint_fuzz::<$t>(
                    context,
                    $crate::common::uint_fuzz::$strategy::<$t>(),
                    $expected_execution,
                )
            }
        )+
    };
}
