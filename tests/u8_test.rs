mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u8_test::U8TestProgram;
use simplicityhl_std::artifacts::u8_test::derived_u8_test::{U8TestArguments, U8TestWitness};

// The only per-width code for the common operations.
impl TestUint for u8 {
    type Program = U8TestProgram;
    type Witness = U8TestWitness;

    const ZERO: u8 = 0;
    const ONE: u8 = 1;
    const MAX: u8 = u8::MAX;
    const HALF_MAX: u8 = u8::MAX / 2;
    const MUL_BOUND: u8 = 1 << 4; // 2^(8/2)

    fn program() -> U8TestProgram {
        U8TestProgram::new(U8TestArguments {})
    }

    fn witness(op: u8, a: u8, b: u8, expected: Option<u8>) -> U8TestWitness {
        U8TestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u8_tests {
    use super::*;

    // Stamps the 16 `#[simplex::test]` entry points for u8. Logic lives in common::uint.
    uint_tests!(u8);
}
