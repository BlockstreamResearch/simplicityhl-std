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

fn program() -> U128TestBitsProgram {
    U128TestBitsProgram::new(&U128TestBitsArguments {})
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

    case(Eq128).args(a, a).expect(0).flag(true).run(&context)
}

#[simplex::test]
fn eq_128_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = a - 1;

    case(Eq128).args(a, b).expect(0).run(&context)
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
