mod common;

use common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::binary::BinaryProgram as BinaryTestProgram;
use simplicityhl_std::artifacts::tests::binary::derived_binary::{
    BinaryArguments as BinaryTestArguments, BinaryWitness as BinaryTestWitness,
};

mod binary_tests {
    use super::*;

    #[simplex::test]
    fn binary_test(context: simplex::TestContext) -> anyhow::Result<()> {
        let program = BinaryTestProgram::new(&BinaryTestArguments {});
        run(&context, program, BinaryTestWitness {}, Expect::Ok)
    }
}
