mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u128_math_test::U128MathTestProgram;
use simplicityhl_std::artifacts::u128_math_test::derived_u128_math_test::{
    U128MathTestArguments, U128MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u128 {
    type Program = U128MathTestProgram;
    type Witness = U128MathTestWitness;

    const ZERO: u128 = 0;
    const ONE: u128 = 1;
    const MAX: u128 = u128::MAX;
    const HALF_MAX: u128 = u128::MAX / 2;
    const MUL_BOUND: u128 = 1 << 64; // 2^(128/2)

    fn program() -> U128MathTestProgram {
        U128MathTestProgram::new(&U128MathTestArguments {})
    }

    fn witness(op: u8, a: u128, b: u128, expected: Option<u128>) -> U128MathTestWitness {
        U128MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u128_math_tests {
    use super::*;

    // Stamps the 22 `#[simplex::test]` entry points for u128. Logic lives in common::uint.
    uint_tests!(u128);
}
