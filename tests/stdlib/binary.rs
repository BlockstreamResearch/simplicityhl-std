use simplicityhl_std::artifacts::tests::binary::BinaryProgram as BinaryTestProgram;
use simplicityhl_std::artifacts::tests::binary::derived_binary::{
    BinaryArguments as BinaryTestArguments, BinaryWitness as BinaryTestWitness,
};

use crate::common::core::Expect;
use crate::common::unit;

#[test]
fn all_ops() {
    let program = BinaryTestProgram::new(&BinaryTestArguments {});

    unit::check(program.as_ref(), BinaryTestWitness {}, Expect::Ok);
}
