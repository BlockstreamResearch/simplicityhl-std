use crate::common::core::{Expect, run};
use rand::Rng;

use simplicityhl_std::artifacts::tests::u64::mul_div::MulDivProgram as U64MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u64::mul_div::derived_mul_div::{
    MulDivArguments as U64MulDivTestArguments, MulDivWitness as U64MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U64MulDivTestProgram {
    U64MulDivTestProgram::new(&U64MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U64MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U64MulDivTestWitness {
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
    fn args(mut self, a: u64, b: u64, c: u64) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u64) -> Self {
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
fn mul_div_64_product_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX) as u64;
    let b = rand::thread_rng().gen_range(0..=u32::MAX) as u64;
    let c = rand::thread_rng().gen_range(1..=u64::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_64_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u64::MAX);
    let b = u64::MAX;
    let c = rand::thread_rng().gen_range(a..=u64::MAX);

    let res = (a as u128) * (b as u128) / (c as u128);

    case(MulDiv).args(a, b, c).expect(res as u64).run(&context)
}

#[simplex::test]
fn mul_div_64_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u64::MAX);
    let b = u64::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_64_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u64::MAX);
    let b = rand::thread_rng().gen_range(1..=u64::MAX);
    let c = 0;

    case(MulDiv).args(a, b, c).expect(0).run(&context)
}
