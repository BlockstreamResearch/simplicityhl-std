mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u8_math_test::U8MathTestProgram;
use simplicityhl_std::artifacts::u8_math_test::derived_u8_math_test::{
    U8MathTestArguments, U8MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u8 {
    type Program = U8MathTestProgram;
    type Witness = U8MathTestWitness;

    const ZERO: u8 = 0;
    const ONE: u8 = 1;
    const MAX: u8 = u8::MAX;
    const HALF_MAX: u8 = u8::MAX / 2;
    const MUL_BOUND: u8 = 1 << 4; // 2^(8/2)

    fn program() -> U8MathTestProgram {
        U8MathTestProgram::new(U8MathTestArguments {})
    }

    fn witness(op: u8, a: u8, b: u8, expected: Option<u8>) -> U8MathTestWitness {
        U8MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u8_math_tests {
    use super::*;

    // Stamps the 22 `#[simplex::test]` entry points for u8. Logic lives in common::uint.
    uint_tests!(u8);
}
