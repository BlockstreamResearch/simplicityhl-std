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
    U256TestCompareProgram::new(&U256TestCompareArguments {})
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
