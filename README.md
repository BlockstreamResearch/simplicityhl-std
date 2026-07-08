# SimplicityHL Standard Library

This repository contains the standard library for [SimplicityHL](https://github.com/BlockstreamResearch/SimplicityHL).

> [!NOTE]
> The VS Code syntax-highlighting extension does not yet support modules and imports, so you may see spurious errors when editing files in this repo.

## Installation

Install `simplexup`, then use it to install the pinned Simplex toolchain:

```bash
curl -L https://smplx.simplicity-lang.org | bash
simplexup
```

## Usage

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
simplex test u8_test
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
