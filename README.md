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
│       └── Conversions from `u1` to other uint types and `bool`.
├── u8
├── u16
├── u32
├── u64
│   ├── convert.simf
│   │   └── Conversions between `u8`, `u16`, `u32`, `u64` and other uint types.
│   └── math.simf
│       └── Overflow-checked arithmetic and `gt`/`ge` functions.
├── u128
├── u256
│   ├── bit.simf
│   │   └── Basic bit operations available as jets for `u8`-`u64` but missing for `u128` and `u256`.
│   ├── comparison.simf
│   │   └── Basic comparison operations available as jets for `u8`-`u64` but missing for `u128` and `u256`.
│   ├── convert.simf
│   │   └── Conversions between `u128`, `u256` and other uint types.
│   └── math.simf
│       └── Carry/borrow arithmetic, multiplication, division, and overflow-checked wrappers.
├── asserts.simf
│   └── Assertion helpers: `assert_eq_*` for uints and `bool`, plus `assert_none_*` for `Option`.
├── binary.simf
│   └── Basic binary logic operations: `and`, `or`, `not`, `xor`.
└── op_return.simf
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

## Documentation

Documentation for the standard library functions is maintained in this repository at `docs/stdlib.json`.

You can find [a rendered standard library reference](https://docs.simplicity-lang.org/documentation/stdlib/) on our developer documentation site.

## Usage

After installing the library, import modules from `std::lib`:

```rust
use std::lib::u32::math::checked_add_32;
use std::lib::asserts::assert_eq_32;

fn main() {
    assert_eq_32(unwrap(checked_add_32(1, 2)), 3);
}
```

## Contributing

We are open to any mind-blowing ideas! Please take a look at our [contributing guidelines](CONTRIBUTING.md) to get involved.

## License

The library is released under the MIT License.
