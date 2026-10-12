# Changelog

## [0.0.2]

- Added `mul_div` functions.
- Organized code into modules.
- Refactored test structure.
- Removed `safe_` prefixes from math functions, so the default versions are overflow-checked.
- Added functions for retrieving and asserting `AssetId` and `Amount`.

## [0.0.1]

Initial release!
- Сhecked arithmetic operations for `u8`, `u16`, `u32`, `u64`, `u128` and `u256`.
- `OP_RETURN` detection utilities.
- Implementation of `and`, `or`, `not`, and `xor` binary operators.
- Basic numeric assertions.
- Equality and conversion operators for `secp256k1` points.
