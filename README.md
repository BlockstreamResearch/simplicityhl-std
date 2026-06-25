# SimplicityHL Standard Library

This repository contains the standard library for [SimplicityHL](https://github.com/BlockstreamResearch/SimplicityHL).

> [!NOTE]
> If you are using the VS Code syntax highlighting extension, you may see errors because the extension does not support modules and imports yet.

## Dev Info
### Installation

> Project currently uses Simplex version 0.0.6 from the dev branch.

To compile the project, execute the following command:

```bash
cargo build
```

To download simplexup, run:

```bash
curl -L https://smplx.simplicity-lang.org | bash
```

To install a specific Simplex version (in this case from the specific commit):

```bash
simplexup --commit 3afcc04aae8a0a92722b28bc1d16ef27983d4aa1
```

### Compilation

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

To run a specific test:

```bash
simplex test test_name
```

### Linting

To format the rust files, execute the following command:

```bash
cargo fmt
```

To check the project for common mistakes:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
