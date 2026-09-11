use primitive_types::U256;

use crate::common::u256_wrapper::U256Wrapper;
use crate::common::uint::TestUint;

use simplicityhl_std::artifacts::tests::u256::math::api::ApiProgram as U256MathTestProgram;
use simplicityhl_std::artifacts::tests::u256::math::api::derived_api::{
    ApiArguments as U256MathTestArguments, ApiWitness as U256MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for U256Wrapper {
    type Program = U256MathTestProgram;
    type Witness = U256MathTestWitness;

    const ZERO: U256Wrapper = U256Wrapper(U256::zero());
    const ONE: U256Wrapper = U256Wrapper(U256::one());
    const MAX: U256Wrapper = U256Wrapper(U256::MAX);
    const HALF_MAX: U256Wrapper = U256Wrapper(U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX >> 1]));
    const MUL_BOUND: U256Wrapper = U256Wrapper(U256([0, 0, 1, 0])); // 2^(256/2)

    fn program() -> U256MathTestProgram {
        U256MathTestProgram::new(&U256MathTestArguments {})
    }

    fn witness(
        op: u8,
        a: U256Wrapper,
        b: U256Wrapper,
        expected: Option<U256Wrapper>,
    ) -> U256MathTestWitness {
        U256MathTestWitness {
            function_index: op,
            first_arg: a.to_big_endian(),
            second_arg: b.to_big_endian(),
            expected: expected.map(|w| w.to_be_bytes()),
        }
    }
}

// Stamps the 22 `#[simplex::test]` entry points for U256Wrapper. Logic lives in common::uint.
crate::uint_tests!(U256Wrapper);
