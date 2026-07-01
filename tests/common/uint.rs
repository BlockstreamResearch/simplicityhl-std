// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use std::ops::{Add, Div, Mul, Sub};

use rand::Rng;
use rand::distributions::uniform::SampleUniform;

use simplex::program::{Program, WitnessTrait};

use super::core::{Expect, run};

/// Dispatch indices for the operations that exist for every unsigned width.
/// These map 1:1 onto the `if_test_this_function(N, ..)` arms in each width's
/// `*_test.simf`.
pub enum CommonOp {
    CheckedAdd,
    SafeAdd,
    CheckedSub,
    SafeSub,
    CheckedMul,
    SafeMul,
    CheckedDiv,
    SafeDiv,
}

/// Width-specific operations number their dispatch indices from here.
/// Slots 0..192 are reserved for common ops; 192..256 for per-width extras.
pub const CUSTOM_BASE: u8 = 192;

/// Everything the generic scenarios need to know about a width: its program /
/// witness types, a few numeric bounds, and how to fill the witness.
/// Implementing this is the only per-width code for the common operations.
pub trait TestUint:
    Copy
    + Ord
    + SampleUniform
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    type Program: AsRef<Program>;
    type Witness: WitnessTrait + 'static;

    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    /// `MAX / 2`; two values up to this can be added without overflowing.
    const HALF_MAX: Self;
    /// `2^(bits/2)`; two values below this can be multiplied without overflowing.
    const MUL_BOUND: Self;

    fn program() -> Self::Program;
    fn witness(op: u8, a: Self, b: Self, expected: Option<Self>) -> Self::Witness;
}

#[inline]
fn op(o: CommonOp) -> u8 {
    o as u8
}

// add
pub fn checked_add_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::HALF_MAX);
    let b = rand::thread_rng().gen_range(T::ZERO..=T::HALF_MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedAdd), a, b, Some(a + b)),
        Expect::Ok,
    )
}

pub fn checked_add_overflow<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(T::ONE..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedAdd), T::MAX, b, None),
        Expect::Ok,
    )
}

pub fn safe_add_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::HALF_MAX);
    let b = rand::thread_rng().gen_range(T::ZERO..=T::HALF_MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeAdd), a, b, Some(a + b)),
        Expect::Ok,
    )
}

pub fn safe_add_overflow<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(T::ONE..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeAdd), T::MAX, b, None),
        Expect::PrunedBranch,
    )
}

// sub
pub fn checked_sub_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX);
    let b = rand::thread_rng().gen_range(T::ZERO..=a);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedSub), a, b, Some(a - b)),
        Expect::Ok,
    )
}

pub fn checked_sub_overflow<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX - T::ONE);
    let b = rand::thread_rng().gen_range(a + T::ONE..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedSub), a, b, None),
        Expect::Ok,
    )
}

pub fn safe_sub_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX);
    let b = rand::thread_rng().gen_range(T::ZERO..=a);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeSub), a, b, Some(a - b)),
        Expect::Ok,
    )
}

pub fn safe_sub_overflow<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX - T::ONE);
    let b = rand::thread_rng().gen_range(a + T::ONE..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeSub), a, b, None),
        Expect::PrunedBranch,
    )
}

// mul
pub fn checked_mul_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..T::MUL_BOUND);
    let b = rand::thread_rng().gen_range(T::ZERO..T::MUL_BOUND);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedMul), a, b, Some(a * b)),
        Expect::Ok,
    )
}

pub fn checked_mul_overflow<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let two = T::ONE + T::ONE;
    let b = rand::thread_rng().gen_range(two..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedMul), T::MAX, b, None),
        Expect::Ok,
    )
}

pub fn safe_mul_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..T::MUL_BOUND);
    let b = rand::thread_rng().gen_range(T::ZERO..T::MUL_BOUND);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeMul), a, b, Some(a * b)),
        Expect::Ok,
    )
}

pub fn safe_mul_overflow<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let two = T::ONE + T::ONE;
    let b = rand::thread_rng().gen_range(two..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeMul), T::MAX, b, None),
        Expect::PrunedBranch,
    )
}

// div
pub fn checked_div_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX);
    let b = rand::thread_rng().gen_range(T::ONE..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedDiv), a, b, Some(a / b)),
        Expect::Ok,
    )
}

pub fn checked_div_by_zero<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::CheckedDiv), a, T::ZERO, None),
        Expect::Ok,
    )
}

pub fn safe_div_fitting<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX);
    let b = rand::thread_rng().gen_range(T::ONE..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeDiv), a, b, Some(a / b)),
        Expect::Ok,
    )
}

pub fn safe_div_by_zero<T: TestUint>(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(T::ZERO..=T::MAX);
    run(
        &context,
        T::program(),
        T::witness(op(CommonOp::SafeDiv), a, T::ZERO, None),
        Expect::PrunedBranch,
    )
}

/// Stamps the `#[simplex::test]` entry points for one width.
///
/// It contains NO test logic. It only maps each scenario name to a concrete
/// `fn` that calls the generic scenario of the same name in this module.
///
/// # Usage
///
/// In a width's test file, after `impl TestUint for <width>`:
///
/// ```ignore
/// uint_tests!(u8);
/// ```
///
/// # How to expand it
///
/// **Add a new scenario** (e.g. `checked_add_max_plus_one`):
/// 1. Write the logic as a generic function above:
///    `pub fn checked_add_max_plus_one<T: TestUint>(context: simplex::TestContext)
///    -> anyhow::Result<()> { ... }`.
/// 2. Add its name to the `@stub` name list below (any position). That alone
///    generates a `#[simplex::test]` for it in every width that calls `uint_tests!`.
///
/// **Remove a scenario:** delete its generic function and its name from the list.
///
/// The name in the `@stub` list must match the generic function name exactly;
/// the stub calls `$crate::common::uint::<name>::<$t>`.
///
/// The first rule fans a `uint_tests!($t)` call out to the internal `@stub` rule,
/// which emits one `#[simplex::test]` stub per name in the list.
#[macro_export]
macro_rules! uint_tests {
    ($t:ty) => {
        $crate::uint_tests!(@stub $t;
            checked_add_fitting checked_add_overflow
            safe_add_fitting    safe_add_overflow
            checked_sub_fitting checked_sub_overflow
            safe_sub_fitting    safe_sub_overflow
            checked_mul_fitting checked_mul_overflow
            safe_mul_fitting    safe_mul_overflow
            checked_div_fitting checked_div_by_zero
            safe_div_fitting    safe_div_by_zero
        );
    };
    (@stub $t:ty; $($name:ident)+) => {
        $(
            #[simplex::test]
            fn $name(context: simplex::TestContext) -> anyhow::Result<()> {
                $crate::common::uint::$name::<$t>(context)
            }
        )+
    };
}
