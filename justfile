check *args:
    cargo fmt -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo xtask check {{args}}

gauntlet: check test

test:
    cargo test --workspace

fix:
    cargo fmt
    cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged

# Install the pebble binary to the Cargo bin directory
install:
    cargo install --path crates/pebble --force
