use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u8::mul_div::MulDivProgram as U8MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u8::mul_div::derived_mul_div::{
    MulDivArguments as U8MulDivTestArguments, MulDivWitness as U8MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U8MulDivTestProgram {
    U8MulDivTestProgram::new(U8MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U8MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U8MulDivTestWitness {
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
    fn args(mut self, a: u8, b: u8, c: u8) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u8) -> Self {
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
fn mul_div_8_product_is_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u8::MAX);
    let b = u8::MAX / a;
    let c = rand::thread_rng().gen_range(1..=u8::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_8_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u8::MAX);
    let b = u8::MAX;
    let c = rand::thread_rng().gen_range(a..=u8::MAX);

    let res = (a as u16) * (b as u16) / (c as u16);

    case(MulDiv).args(a, b, c).expect(res as u8).run(&context)
}

#[simplex::test]
fn mul_div_8_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u8::MAX);
    let b = u8::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_8_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u8::MAX);
    let b = rand::thread_rng().gen_range(1..=u8::MAX);
    let c = 0;

    case(MulDiv).args(a, b, c).expect(0).run(&context)
}
