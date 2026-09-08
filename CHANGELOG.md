# Changelog

# Unreleased

- Add `load` and `store` functions to assert state commitments for
  covenants via Taproot leaves. This allows an instance of a smart
  contract to remember state information across multiple transactions.

## [0.0.1]

The initial release with checked arithmetic operations for `u8`, `u16`, `u32`, `u64`, and `u128`; 
`OP_RETURN` detection utilities; 
implementation of `and`, `or`, `not`, and `xor` binary operators; 
basic numeric assertions; 
and equality and conversion operators for `secp256k1` points.
