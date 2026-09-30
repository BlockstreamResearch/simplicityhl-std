# Changelog

## [Unreleased]

- Add `sighash_elements` module (**unaudited; use with caution**) with the sighash modes `NONE`, `SINGLE`, and the `ANYONECANPAY` variants of `ALL`, `NONE` and `SINGLE` (for plain `ALL`, use `jet::sig_all_hash`), plus `sighash(mode)` and `bip_0340_verify_with_mode` for choosing the mode at spend time from a BIP-341 `hash_type` byte (`0x01`, `0x02`, `0x03`, `0x81`, `0x82` or `0x83`; `0x00` is rejected). The `ANYONECANPAY` modes do not commit to the input's index, so the input can move when others are added.

## [0.0.1]

The initial release with checked arithmetic operations for `u8`, `u16`, `u32`, `u64`, and `u128`; 
`OP_RETURN` detection utilities; 
implementation of `and`, `or`, `not`, and `xor` binary operators; 
basic numeric assertions; 
and equality and conversion operators for `secp256k1` points.
