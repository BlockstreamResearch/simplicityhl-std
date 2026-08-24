mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u16_math_test::U16MathTestProgram;
use simplicityhl_std::artifacts::u16_math_test::derived_u16_math_test::{
    U16MathTestArguments, U16MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u16 {
    type Program = U16MathTestProgram;
    type Witness = U16MathTestWitness;

    const ZERO: u16 = 0;
    const ONE: u16 = 1;
    const MAX: u16 = u16::MAX;
    const HALF_MAX: u16 = u16::MAX / 2;
    const MUL_BOUND: u16 = 1 << 8; // 2^(16/2)

    fn program() -> U16MathTestProgram {
        U16MathTestProgram::new(&U16MathTestArguments {})
    }

    fn witness(op: u8, a: u16, b: u16, expected: Option<u16>) -> U16MathTestWitness {
        U16MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u16_math_tests {
    use super::*;

    // Stamps the 22 `#[simplex::test]` entry points for u16. Logic lives in common::uint.
    uint_tests!(u16);
}
