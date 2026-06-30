mod common;

use common::{Expect, run};

use simplicityhl_std::artifacts::logical_ops_test::LogicalOpsTestProgram;
use simplicityhl_std::artifacts::logical_ops_test::derived_logical_ops_test::{
    LogicalOpsTestArguments, LogicalOpsTestWitness,
};

mod logical_ops_tests {
    use super::*;

    #[simplex::test]
    fn logical_operations_test(context: simplex::TestContext) -> anyhow::Result<()> {
        let program = LogicalOpsTestProgram::new(LogicalOpsTestArguments {});
        run(&context, program, LogicalOpsTestWitness {}, Expect::Ok)
    }
}
