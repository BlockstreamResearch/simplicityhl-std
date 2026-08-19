mod common;

use rand::Rng;

use crate::common::core::{run, run_with_op_return};
use crate::common::helper::DEFAULT_BOOL;
use common::core::Expect;

use simplicityhl_std::artifacts::op_return_test::OpReturnTestProgram;
use simplicityhl_std::artifacts::op_return_test::derived_op_return_test::{
    OpReturnTestArguments, OpReturnTestWitness,
};

enum FunctionToTest {
    IsOpReturn,
    AssertOutputIsOpReturn,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_DATA: &[u8; 1] = &[1];

fn program() -> OpReturnTestProgram {
    OpReturnTestProgram::new(&OpReturnTestArguments {})
}

fn build_witness(function: u8, index: u32, expected: bool) -> OpReturnTestWitness {
    OpReturnTestWitness {
        function_index: function,
        index,
        expected,
    }
}

mod op_return_tests {
    use super::*;

    #[simplex::test]
    fn is_output_op_return_true(context: simplex::TestContext) -> anyhow::Result<()> {
        let index = 0;

        run_with_op_return(
            &context,
            program(),
            build_witness(op(FunctionToTest::IsOpReturn), index, true),
            Expect::Ok,
            DEFAULT_DATA,
        )
    }

    #[simplex::test]
    fn is_output_op_return_empty_index_false(context: simplex::TestContext) -> anyhow::Result<()> {
        let index = rand::thread_rng().gen_range(1..=u32::MAX);

        run_with_op_return(
            &context,
            program(),
            build_witness(op(FunctionToTest::IsOpReturn), index, false),
            Expect::Ok,
            DEFAULT_DATA,
        )
    }

    #[simplex::test]
    fn is_output_op_return_false(context: simplex::TestContext) -> anyhow::Result<()> {
        let index = 0;

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::IsOpReturn), index, false),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn assert_output_is_op_return_pass(context: simplex::TestContext) -> anyhow::Result<()> {
        let index = 0;

        run_with_op_return(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::AssertOutputIsOpReturn),
                index,
                DEFAULT_BOOL,
            ),
            Expect::Ok,
            DEFAULT_DATA,
        )
    }

    #[simplex::test]
    fn assert_output_is_op_return_empty_index_fail(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let index = rand::thread_rng().gen_range(1..=u32::MAX);

        run_with_op_return(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::AssertOutputIsOpReturn),
                index,
                DEFAULT_BOOL,
            ),
            Expect::AssertFailed,
            DEFAULT_DATA,
        )
    }

    #[simplex::test]
    fn assert_output_is_op_return_fail(context: simplex::TestContext) -> anyhow::Result<()> {
        let index = 0;

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::AssertOutputIsOpReturn),
                index,
                DEFAULT_BOOL,
            ),
            Expect::AssertFailed,
        )
    }
}
