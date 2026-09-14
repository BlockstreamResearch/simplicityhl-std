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
