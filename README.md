# SimplicityHL Standard Library

This repository contains the standard library for [SimplicityHL](https://github.com/BlockstreamResearch/SimplicityHL).

## Dev Info
### Compilation

To compile the project, execute the following command:

```bash
cargo build
```

To compile the contracts, execute the following command:

```bash
simplex build
```

### Test

To run the tests, execute the following command:

```bash
simplex test --nocapture
```

### Linting

To format the rust files, execute the following command:

```bash
cargo fmt
```

To check the project for common mistakes, execute the following command:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
