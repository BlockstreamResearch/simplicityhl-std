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
