use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u16::mul_div::MulDivProgram as U16MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u16::mul_div::derived_mul_div::{
    MulDivArguments as U16MulDivTestArguments, MulDivWitness as U16MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U16MulDivTestProgram {
    U16MulDivTestProgram::new(&U16MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U16MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U16MulDivTestWitness {
            function_index: function as u8,
            first_arg: 0,
            second_arg: 0,
            third_arg: 0,
            expected: None,
        },
    }
}

impl Case {
    /// The three operands, `first_arg`, `second_arg` and `third_arg`.
    fn args(mut self, a: u16, b: u16, c: u16) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u16) -> Self {
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
fn mul_div_16_product_is_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX) as u16;
    let b = rand::thread_rng().gen_range(0..=u8::MAX) as u16;
    let c = rand::thread_rng().gen_range(1..=u16::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_16_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u16::MAX);
    let b = u16::MAX;
    let c = rand::thread_rng().gen_range(a..=u16::MAX);

    let res = (a as u32) * (b as u32) / (c as u32);

    case(MulDiv).args(a, b, c).expect(res as u16).run(&context)
}

#[simplex::test]
fn mul_div_16_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u16::MAX);
    let b = u16::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_16_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u16::MAX);
    let b = rand::thread_rng().gen_range(1..=u16::MAX);
    let c = 0;

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}
