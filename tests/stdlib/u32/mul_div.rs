use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u32::mul_div::MulDivProgram as U32MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u32::mul_div::derived_mul_div::{
    MulDivArguments as U32MulDivTestArguments, MulDivWitness as U32MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U32MulDivTestProgram {
    U32MulDivTestProgram::new(&U32MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
///
/// Every field the chosen arm ignores keeps its default, so a test names only
/// the values it actually depends on.
struct Case {
    witness: U32MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U32MulDivTestWitness {
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
    fn args(mut self, a: u32, b: u32, c: u32) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u32) -> Self {
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
fn mul_div_32_product_is_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX) as u32;
    let b = rand::thread_rng().gen_range(0..=u16::MAX) as u32;
    let c = rand::thread_rng().gen_range(1..=u32::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_32_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u32::MAX);
    let b = u32::MAX;
    let c = rand::thread_rng().gen_range(a..=u32::MAX);

    let res = (a as u64) * (b as u64) / (c as u64);

    case(MulDiv).args(a, b, c).expect(res as u32).run(&context)
}

#[simplex::test]
fn mul_div_32_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u32::MAX);
    let b = u32::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_32_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u32::MAX);
    let b = rand::thread_rng().gen_range(1..=u32::MAX);
    let c = 0;

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}
