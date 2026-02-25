---
name: Modern Rust Module Structure
description: Require modern module file layout; prohibit mod.rs
---
### Modern Rust Module Structure

Use the modern Rust module layout: a parent module lives in `parent.rs`, and its children live in `parent/`.

#### Rules
- Avoid `mod.rs` files.
- Avoid `#[path]` module path overrides.

#### `#[path]` Exception
Use `#[path]` only for platform-conditional modules that implement a shared portability abstraction.

#### Example
Desired structure:
- `src/lib.rs` (contains `mod util;`)
- `src/util.rs` (contains `mod config;`)
- `src/util/config.rs`

Legacy structure to avoid:
- `src/lib.rs` (contains `mod util;`)
- `src/util/mod.rs` (contains `mod config;`)
- `src/util/config.rs`
