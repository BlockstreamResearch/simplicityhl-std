use primitive_types::U256;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::math::add::AddProgram as U256TestAddProgram;
use simplicityhl_std::artifacts::tests::u256::math::add::derived_add::{
    AddArguments as U256TestAddArguments, AddWitness as U256TestAddWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Add256,
    Add256_128,
    FullAdd256,
}

fn program() -> U256TestAddProgram {
    U256TestAddProgram::new(&U256TestAddArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256TestAddWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256TestAddWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            third_arg: false,
            expected: None,
            expected_bool: false,
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

    /// `third_arg`: the extra operand only some arms take.
    fn third_arg(mut self, c: bool) -> Self {
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: [u8; 32]) -> Self {
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
fn add_256_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::zero(), U256::MAX / 2);
    let result = (a + b).to_big_endian();

    case(Add256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn add_256_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::MAX);
    let result = (b - 1).to_big_endian();

    case(Add256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn add_256_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let result = (a + b).to_big_endian();

    case(Add256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn add_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let result = (b - 1).to_big_endian();

    case(Add256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn full_add_256_not_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::zero(), U256::MAX / 2);

    let result = (a + b).to_big_endian();
    let result_carry = false;
    let carry_low = false;

    case(FullAdd256)
        .args(a.to_big_endian(), b.to_big_endian())
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}

#[simplex::test]
fn full_add_256_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::MAX);

    let result = (b - 1).to_big_endian();
    let result_carry = true;
    let carry_low = false;

    case(FullAdd256)
        .args(a.to_big_endian(), b.to_big_endian())
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}

#[simplex::test]
fn full_add_256_not_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::zero(), U256::MAX / 2);

    let result = (a + b + 1).to_big_endian();
    let result_carry = false;
    let carry_low = true;

    case(FullAdd256)
        .args(a.to_big_endian(), b.to_big_endian())
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}

#[simplex::test]
fn full_add_256_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::MAX).to_big_endian();

    let result = b;
    let result_carry = true;
    let carry_low = true;

    case(FullAdd256)
        .args(a.to_big_endian(), b)
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}
