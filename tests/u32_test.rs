mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u32_test::U32TestProgram;
use simplicityhl_std::artifacts::u32_test::derived_u32_test::{U32TestArguments, U32TestWitness};

// The only per-width code for the common operations.
impl TestUint for u32 {
    type Program = U32TestProgram;
    type Witness = U32TestWitness;

    const ZERO: u32 = 0;
    const ONE: u32 = 1;
    const MAX: u32 = u32::MAX;
    const HALF_MAX: u32 = u32::MAX / 2;
    const MUL_BOUND: u32 = 1 << 16; // 2^(32/2)

    fn program() -> U32TestProgram {
        U32TestProgram::new(U32TestArguments {})
    }

    fn witness(op: u8, a: u32, b: u32, expected: Option<u32>) -> U32TestWitness {
        U32TestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u32_tests {
    use super::*;

    // Stamps the 16 `#[simplex::test]` entry points for u32. Logic lives in common::uint.
    uint_tests!(u32);
}
