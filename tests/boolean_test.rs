mod common;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::boolean_test::BooleanTestProgram;
use simplicityhl_std::artifacts::boolean_test::derived_boolean_test::{
    BooleanTestArguments, BooleanTestWitness,
};

mod boolean_tests {
    use super::*;
    
    #[simplex::test]
    fn boolean_test(context: simplex::TestContext) -> anyhow::Result<()> {
        let program = BooleanTestProgram::new(BooleanTestArguments {});
        run(&context, program, BooleanTestWitness {}, Expect::Ok)
    }
}
