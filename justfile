# List the available recipes.
default:
    @just --list

# Note: run `simplex build` first. Clippy compiles the test targets, and those
# import `simplicityhl_std::artifacts`, which is generated rather than tracked.

# Run the same lint gates CI runs, in check mode.
fmt:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format every .simf file under simf/ in place.
simfmt:
    find simf -name '*.simf' -print0 | xargs -0 -r simfmt
