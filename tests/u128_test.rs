mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u128_test::U128TestProgram;
use simplicityhl_std::artifacts::u128_test::derived_u128_test::{
    U128TestArguments, U128TestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u128 {
    type Program = U128TestProgram;
    type Witness = U128TestWitness;

    const ZERO: u128 = 0;
    const ONE: u128 = 1;
    const MAX: u128 = u128::MAX;
    const HALF_MAX: u128 = u128::MAX / 2;
    const MUL_BOUND: u128 = 1 << 64; // 2^(128/2)

    fn program() -> U128TestProgram {
        U128TestProgram::new(U128TestArguments {})
    }

    fn witness(op: u8, a: u128, b: u128, expected: Option<u128>) -> U128TestWitness {
        U128TestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u128_tests {
    use super::*;

    // Stamps the 22 `#[simplex::test]` entry points for u128. Logic lives in common::uint.
    uint_tests!(u128);
}
