//! Every integration test for the standard library.
//!
//! One target, so the module path is the test name: a test in
//! `u256/math/div.rs` is `u256::math::div::<name>`. That is what
//! `simplex test u256` and `simplex test u256::math` filter on.
//! The layout mirrors `simf/tests`, which mirrors `simf/lib`.

mod common;

mod asserts;
mod binary;
mod op_return;
mod secp256k1;
mod u1;
mod u128;
mod u16;
mod u256;
mod u32;
mod u64;
mod u8;
