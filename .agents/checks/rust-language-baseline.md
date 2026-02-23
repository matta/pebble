---
name: Rust Language Baseline
description: Validate Rust syntax compatibility claims against Cargo metadata
---
Before flagging Rust syntax compatibility concerns, check `edition` and `rust-version` in `Cargo.toml`.

This repository treats Rust 2024 syntax as canonical.

Do not flag `if let` chain syntax (`if let ... && ...`) as unstable in this codebase.
