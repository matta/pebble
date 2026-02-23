---
name: Rust Documentation Requirements
description: Ensure Rust items are documented per the tiered policy in docs/rust-api-docs.md
---
- **Tier 1** (full doc comment): all items visible beyond their defining module (`pub`, `pub(crate)`, `pub(super)`) — structs, enums, traits, functions, methods, constants, type aliases, and the crate root `//!`.
- **Tier 2** (one-liner minimum): private items that are non-trivial (>~10 lines or non-obvious behavior).
- **Tier 3** (exempt): trivial helpers, trait impls with obvious behavior, test code, serde defaults.
