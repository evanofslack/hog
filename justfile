default:
    @just --list

# Build an optimized binary at target/release/hog
build:
    cargo build --release

# Run hog with arguments, e.g. `just run files ~ -n 20`
run *args:
    cargo run --release -- {{args}}

test:
    cargo test

# Check formatting and lints
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

fmt:
    cargo fmt

# Install to ~/.cargo/bin
install:
    cargo install --path .
