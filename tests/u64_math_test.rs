mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u64_math_test::U64MathTestProgram;
use simplicityhl_std::artifacts::u64_math_test::derived_u64_math_test::{
    U64MathTestArguments, U64MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u64 {
    type Program = U64MathTestProgram;
    type Witness = U64MathTestWitness;

    const ZERO: u64 = 0;
    const ONE: u64 = 1;
    const MAX: u64 = u64::MAX;
    const HALF_MAX: u64 = u64::MAX / 2;
    const MUL_BOUND: u64 = 1 << 32; // 2^(64/2)

    fn program() -> U64MathTestProgram {
        U64MathTestProgram::new(U64MathTestArguments {})
    }

    fn witness(op: u8, a: u64, b: u64, expected: Option<u64>) -> U64MathTestWitness {
        U64MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u64_math_tests {
    use super::*;

    // Stamps the 22 `#[simplex::test]` entry points for u64. Logic lives in common::uint.
    uint_tests!(u64);
}
