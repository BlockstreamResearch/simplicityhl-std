mod common;

use primitive_types::U256;

use common::u256_wrapper::U256Wrapper;
use common::uint::TestUint;

use simplicityhl_std::artifacts::u256_test::U256TestProgram;
use simplicityhl_std::artifacts::u256_test::derived_u256_test::{
    U256TestArguments, U256TestWitness,
};

// The only per-width code for the common operations.
impl TestUint for U256Wrapper {
    type Program = U256TestProgram;
    type Witness = U256TestWitness;

    const ZERO: U256Wrapper = U256Wrapper(U256::zero());
    const ONE: U256Wrapper = U256Wrapper(U256::one());
    const MAX: U256Wrapper = U256Wrapper(U256::MAX);
    const HALF_MAX: U256Wrapper = U256Wrapper(U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX >> 1]));
    const MUL_BOUND: U256Wrapper = U256Wrapper(U256([0, 0, 1, 0])); // 2^(256/2)

    fn program() -> U256TestProgram {
        U256TestProgram::new(U256TestArguments {})
    }

    fn witness(
        op: u8,
        a: U256Wrapper,
        b: U256Wrapper,
        expected: Option<U256Wrapper>,
    ) -> U256TestWitness {
        U256TestWitness {
            function_index: op,
            first_arg: a.to_big_endian(),
            second_arg: b.to_big_endian(),
            expected: expected.map(|w| w.to_be_bytes()),
        }
    }
}

mod u256_tests {
    use super::*;

    // Stamps the 16 `#[simplex::test]` entry points for U256Wrapper. Logic lives in common::uint.
    uint_tests!(U256Wrapper);
}
