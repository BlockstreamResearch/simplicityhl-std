# SimplicityHL Standard Library

This repository contains the standard library for [SimplicityHL](https://github.com/BlockstreamResearch/SimplicityHL).

> [!NOTE]
> The VS Code syntax-highlighting extension does not yet support modules and imports, so you may see spurious errors when editing files in this repo.

## Modules

- asserts
- binary
- op_return
- u8
- u16
- u32
- u64
- u128
- secp256k1

---
`Asserts`

Generic assertion helpers with equality checks between uint values and validation of Option uint variants.

---
`Binary`

Basic binary logic operations: and, or, not, xor.

---
`OP_RETURN`

Utilities for detecting and enforcing OP_RETURN (null data) outputs.

---
`u8`

Operations for the u8 type:

- overflow-checked arithmetic operations;
- comparison helpers.

---
`u16`

Operations for the u16 type:

- overflow-checked arithmetic operations;
- comparison helpers.

---
`u32`

Operations for the u32 type:

- overflow-checked arithmetic operations;
- comparison helpers.

---
`u64`

Operations for the u64 type:

- overflow-checked arithmetic operations;
- comparison helpers.

---
`u128`

Operations for the u128 type:

- overflow-checked arithmetic operations;
- comparison helpers;
- basic operations that are available as jets for `u8`-`u64` but are missing for `u128`.

---
`secp256k1`

Operations on the secp256k1 curve:

- subtraction for `Fe`, `Scalar`, `Gej`;
- equality predicates and their `assert_*` counterparts;
- conversions between `Ge`, `Gej`, and compressed `Point`;
- safe Jacobian-to-affine normalization.

`fe_eq` and `scalar_eq` use modular arithmetic, so `fe_eq(0, p) == true`. `ge_eq` and `gej_point_eq` distinguish `P` from `-P`.

## Installation

Install `simplexup`, then use it to install the pinned Simplex toolchain:

```bash
curl -L https://smplx.simplicity-lang.org | bash
simplexup
```

```bash
simplex install simplicityhl-std
```

> [!NOTE]
> Library works with Simplex version 0.0.7 or higher.

## Contributing

### Build

To compile the contracts, execute the following command:

```bash
simplex build
```

To clean up generated artifacts:

```bash
simplex clean
```

### Test

To run the tests with logs (`-v` or `-vv` is available), execute the following command:

```bash
simplex test -v
```

To run the tests using multiple threads:

```bash
simplex test --test-threads 8
```

To run a specific test module:

```bash
simplex test u8_tests
```

To run a specific test:

```bash
simplex test test_name
```

### Lint & format

```bash
cargo fmt
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
