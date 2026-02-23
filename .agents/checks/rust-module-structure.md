---
name: Modern Rust Module Structure
description: Strongly recommend the new-style module naming convention (avoiding mod.rs)
---
### Modern Rust Module Structure

It is strongly encouraged to use the modern Rust module naming convention (introduced in Rust 1.30) where nested modules are placed in a directory named after their parent, with the parent's source in a file of the same name.

**Avoid `mod.rs` files.**

#### Example
Desired structure:
- `src/lib.rs` (contains `mod util;`)
- `src/util.rs` (contains `mod config;`)
- `src/util/config.rs`

Legacy structure to avoid:
- `src/lib.rs` (contains `mod util;`)
- `src/util/mod.rs` (contains `mod config;`)
- `src/util/config.rs`

This convention is more consistent and avoids having many files named `mod.rs` within a project.
