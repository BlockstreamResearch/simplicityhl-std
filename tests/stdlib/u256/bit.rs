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
    U256TestBitsProgram::new(&U256TestBitsArguments {})
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
