mod common;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::binary_test::BinaryTestProgram;
use simplicityhl_std::artifacts::binary_test::derived_binary_test::{
    BinaryTestArguments, BinaryTestWitness,
};

mod binary_tests {
    use super::*;

    #[simplex::test]
    fn binary_test(context: simplex::TestContext) -> anyhow::Result<()> {
        let program = BinaryTestProgram::new(&BinaryTestArguments {});
        run(&context, program, BinaryTestWitness {}, Expect::Ok)
    }
}
