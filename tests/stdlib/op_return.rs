use rand::Rng;

use crate::common::core::{Expect, run, run_with_op_return};

use simplicityhl_std::artifacts::tests::op_return::OpReturnProgram as OpReturnTestProgram;
use simplicityhl_std::artifacts::tests::op_return::derived_op_return::{
    OpReturnArguments as OpReturnTestArguments, OpReturnWitness as OpReturnTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    IsOpReturn,
    AssertOutputIsOpReturn,
}

const DEFAULT_DATA: &[u8; 1] = &[1];

fn program() -> OpReturnTestProgram {
    OpReturnTestProgram::new(&OpReturnTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads and whether the
/// spend carries an `OP_RETURN` output.
struct Case {
    witness: OpReturnTestWitness,
    data: Option<&'static [u8]>,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: OpReturnTestWitness {
            function_index: function as u8,
            index: 0,
            expected: false,
        },
        data: None,
    }
}

impl Case {
    /// `index`: the output the arm inspects.
    fn index(mut self, index: u32) -> Self {
        self.witness.index = index;
        self
    }

    /// `expected`: the boolean the arm should report.
    fn flag(mut self, flag: bool) -> Self {
        self.witness.expected = flag;
        self
    }

    /// Add an `OP_RETURN` output carrying `data` to the spend.
    fn op_return(mut self, data: &'static [u8]) -> Self {
        self.data = Some(data);
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        match self.data {
            Some(data) => run_with_op_return(context, program(), self.witness, expect, data),
            None => run(context, program(), self.witness, expect),
        }
    }
}

#[simplex::test]
fn is_output_op_return_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(IsOpReturn)
        .index(index)
        .flag(true)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn is_output_op_return_empty_index_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(1..=u32::MAX);

    case(IsOpReturn)
        .index(index)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn is_output_op_return_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(IsOpReturn).index(index).run(&context)
}

#[simplex::test]
fn assert_output_is_op_return_pass(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(AssertOutputIsOpReturn)
        .index(index)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn assert_output_is_op_return_empty_index_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(1..=u32::MAX);

    case(AssertOutputIsOpReturn)
        .index(index)
        .op_return(DEFAULT_DATA)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_output_is_op_return_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;

    case(AssertOutputIsOpReturn)
        .index(index)
        .expecting(&context, Expect::AssertFailed)
}
