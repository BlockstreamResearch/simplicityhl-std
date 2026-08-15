# Changelog

## Unreleased

- Add relative timelock enforcement functions (`enforce_relative_distance`
  and `enforce_relative_duration`). These functions are replacements for
  the deprecated jets `jet::check_lock_distance` and
  `jet::check_lock_duration`.

## [0.0.1]

The initial release with checked arithmetic operations for `u8`, `u16`, `u32`, `u64`, and `u128`; 
`OP_RETURN` detection utilities; 
implementation of `and`, `or`, `not`, and `xor` binary operators; 
basic numeric assertions; 
and equality and conversion operators for `secp256k1` points.
