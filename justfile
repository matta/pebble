check:
    cargo fmt -- --check
    cargo clippy --workspace --all-targets -- -D warnings

fix:
    cargo fmt
    cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged
