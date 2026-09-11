// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use primitive_types::U256;
use rand::Rng;

use crate::common::u256_wrapper::U256Wrapper;

// Shared constants and helper functions used across integration tests

pub const DEFAULT_BOOL: bool = false;

pub fn generate_u256(lower_bound: U256, upper_bound: U256) -> U256 {
    assert!(
        lower_bound <= upper_bound,
        "Error: lower bound is greater than upper bound"
    );
    rand::thread_rng()
        .gen_range(U256Wrapper(lower_bound)..=U256Wrapper(upper_bound))
        .0
}

#[test]
fn generate_u256_respects_bounds() {
    let cases = [
        (U256::zero(), U256::MAX),
        (U256::one(), U256::from(u128::MAX)),
        (U256::from(u128::MAX) + 1, U256::MAX),
        (U256::from(u64::MAX) + 1, U256::from(u128::MAX)),
        (U256::MAX, U256::MAX),
    ];

    for (lo, hi) in cases {
        for _ in 0..10_000 {
            let v = generate_u256(lo, hi);
            assert!(lo <= v && v <= hi, "{v:#x} outside [{lo:#x}, {hi:#x}]");
        }
    }
}
