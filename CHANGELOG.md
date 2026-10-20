# Changelog

## [0.2.0]

### Breaking changes
- Removed `safe_` prefixes from math functions, so the default versions are overflow-checked.
- Functions that return a carry, borrow, or widened result were renamed.
- Bumped Simplex to 0.0.12.

### Added
- `mul_div` functions.
- Functions for retrieving and asserting `AssetId` and `Amount`.
- Functions for retrieving script hashes.

### Changed
- Organized code into modules.
- Refactored test structure.

## [0.1.0]

Initial release!
- Checked arithmetic operations for `u8`, `u16`, `u32`, `u64`, `u128` and `u256`.
- `OP_RETURN` detection utilities.
- Implementation of `and`, `or`, `not`, and `xor` binary operators.
- Basic numeric assertions.
- Equality and conversion operators for `secp256k1` points.
