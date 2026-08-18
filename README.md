[![Tests](https://github.com/BlockstreamResearch/simplicityhl-std/actions/workflows/tests.yml/badge.svg)](https://github.com/BlockstreamResearch/simplicityhl-std/actions/workflows/tests.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# SimplicityHL Standard Library

The standard library implementation for [SimplicityHL](https://github.com/BlockstreamResearch/SimplicityHL).

## Scripts

```md
simf/lib
├── secp256k1
│   └── operations.simf    
│       ├── Conversions between `Ge`, `Gej`, and compressed `Point`.
│       ├── Subtraction for `Fe`, `Scalar`, `Gej`.
│       ├── Equality predicates and their `assert_*` counterparts.
│       └── Safe Jacobian-to-affine normalization.
├── u1
│   └── convert.simf
├── u8
│   ├── convert.simf
│   └── math.simf
├── u16
│   ├── convert.simf
│   └── math.simf
├── u32
│   ├── convert.simf
│   └── math.simf
├── u64
│   ├── convert.simf
│   └── math.simf
├── u128
│   ├── bit.simf
│   │   └── Basic bit operations that are available as jets for `u8`–`u64` but are missing for `u128`.
│   ├── comparison.simf
│   │   └── Basic comparison operations that are available as jets for `u8`–`u64` but are missing for `u128`.
│   ├── convert.simf
│   │   └── Conversions between `u128` and other uint types.
│   └── math.simf
│       └── Overflow-checked arithmetic operations.
├── u256
│   ├── bit.simf
│   │   └── Basic bit operations that are available as jets for `u8`–`u64` but are missing for `u256`.
│   ├── comparison.simf
│   │   └── Basic comparison operations that are available as jets for `u8`–`u64` but are missing for `u256`.
│   ├── convert.simf
│   │   └── Conversions between `u256` and other uint types.
│   └── math.simf
│       └── Overflow-checked arithmetic operations.
├── asserts.simf
│   └── Generic assertion helpers with equality checks between uint values.
├── binary.simf
│   └── Basic binary logic operations: `and`, `or`, `not`, `xor`.
├── op_return.simf
    └── Utilities for detecting and enforcing `OP_RETURN` (null data) outputs.
```

## Installation

First, install [`simplex`](https://github.com/BlockstreamResearch/smplx) development framework:

```bash
curl -L https://smplx.simplicity-lang.org | bash
simplexup
```

Then install `simplicityhl-std` dependency via simplex.

```bash
simplex install https://github.com/BlockstreamResearch/simplicityhl-std
```

> [!NOTE]
> The library works with Simplex version 0.0.7 or higher.

## Contributing

We are open to any mind-blowing ideas! Please take a look at our [contributing guidelines](CONTRIBUTING.md) to get involved.

## License

The library is released under the MIT License.
