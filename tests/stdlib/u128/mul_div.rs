use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::mul_div::MulDivProgram as U128MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u128::mul_div::derived_mul_div::{
    MulDivArguments as U128MulDivTestArguments, MulDivWitness as U128MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U128MulDivTestProgram {
    U128MulDivTestProgram::new(U128MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U128MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U128MulDivTestWitness {
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
    fn args(mut self, a: u128, b: u128, c: u128) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u128) -> Self {
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
fn mul_div_128_product_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
    let b = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
    let c = rand::thread_rng().gen_range(1..=u128::MAX);

    let res = a * b / c;

    case(MulDiv).args(a, b, c).expect(res).run(&context)
}

#[simplex::test]
fn mul_div_128_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u128::MAX);
    let b = u128::MAX;
    let c = rand::thread_rng().gen_range(a..=u128::MAX);

    let res = U256::from(a) * U256::from(b) / U256::from(c);

    case(MulDiv)
        .args(a, b, c)
        .expect(res.low_u128())
        .run(&context)
}

#[simplex::test]
fn mul_div_128_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u128::MAX);
    let b = u128::MAX;
    let c = rand::thread_rng().gen_range(1..a);

    case(MulDiv)
        .args(a, b, c)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_128_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let c = 0;

    case(MulDiv).args(a, b, c).expect(0).run(&context)
}
