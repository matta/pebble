check:
    cargo fmt -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo xtask check

test:
    cargo test --workspace

fix:
    cargo fmt
    cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged
