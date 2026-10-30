# Changelog

## [0.2.0]

### Breaking changes
- Removed `safe_` prefixes from math functions, so the default versions are overflow-checked.
- `u128`/`u256` functions that return a carry, borrow, or widened result got `_carry`, `_borrow`, and `_wide` suffixes (e.g. `add_128_carry`, `sub_256_borrow`, `mul_128_wide`).

### Added
- `mul_div` functions for all uint types.
- Functions for retrieving and asserting `AssetId` and `Amount`.
- Functions for retrieving script hashes.
- `full_add_256`, `full_mul_256_64`, `mul_512_128_wide`, `calculate_normalizer_base_128` and `algorithm_d_256_128` helpers.

### Changed
- Bumped Simplex to 0.0.12.
- Organized code into modules.
- Refactored test structure.
- Added a CI check that keeps `docs/stdlib.json` in sync with the library.

## [0.1.0]

Initial release!
- Checked arithmetic operations for `u8`, `u16`, `u32`, `u64`, `u128` and `u256`.
- `OP_RETURN` detection utilities.
- Implementation of `and`, `or`, `not`, and `xor` binary operators.
- Basic numeric assertions.
- Equality and conversion operators for `secp256k1` points.
