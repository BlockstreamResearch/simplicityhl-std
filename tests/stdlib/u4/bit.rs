use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u4::bit::BitProgram as U4TestBitProgram;
use simplicityhl_std::artifacts::tests::u4::bit::derived_bit::{
    BitArguments as U4TestBitArguments, BitWitness as U4TestBitWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    And4,
    Or4,
    Eq4,
}

const U4_MAX: u8 = 15;

fn program() -> U4TestBitProgram {
    U4TestBitProgram::new(&U4TestBitArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U4TestBitWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U4TestBitWitness {
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
    fn args(mut self, a: u8, b: u8) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// The u4 value the arm should return
    fn expect(mut self, expected: u8) -> Self {
        self.witness.expected = Some(expected);
        self
    }

    /// `expected_bool`: the boolean the arm should report.
    fn expected_bool(mut self, expected_bool: bool) -> Self {
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
fn and_4(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=U4_MAX);
    let b = rand::thread_rng().gen_range(0..=U4_MAX);
    let result = a & b;

    case(And4).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn or_4(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=U4_MAX);
    let b = rand::thread_rng().gen_range(0..=U4_MAX);
    let result = a | b;

    case(Or4).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn eq_4_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=U4_MAX);

    case(Eq4).args(a, a).expected_bool(true).run(&context)
}

#[simplex::test]
fn eq_4_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=U4_MAX);
    let mut b = rand::thread_rng().gen_range(0..=U4_MAX);

    while a == b {
        b = rand::thread_rng().gen_range(0..=U4_MAX);
    }

    case(Eq4).args(a, b).expected_bool(false).run(&context)
}
