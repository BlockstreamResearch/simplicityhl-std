mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u64_test::U64TestProgram;
use simplicityhl_std::artifacts::u64_test::derived_u64_test::{U64TestArguments, U64TestWitness};

// The only per-width code for the common operations.
impl TestUint for u64 {
    type Program = U64TestProgram;
    type Witness = U64TestWitness;

    const ZERO: u64 = 0;
    const ONE: u64 = 1;
    const MAX: u64 = u64::MAX;
    const HALF_MAX: u64 = u64::MAX / 2;
    const MUL_BOUND: u64 = 1 << 32; // 2^(64/2)

    fn program() -> U64TestProgram {
        U64TestProgram::new(U64TestArguments {})
    }

    fn witness(op: u8, a: u64, b: u64, expected: Option<u64>) -> U64TestWitness {
        U64TestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

mod u64_tests {
    use super::*;

    // Stamps the 16 `#[simplex::test]` entry points for u64. Logic lives in common::uint.
    uint_tests!(u64);
}
