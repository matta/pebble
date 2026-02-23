# Rust API Documentation Requirements

This document codifies the Rust API documentation expectations for this repository. It is based on the Rust API Guidelines: documentation chapter.

## Required Doc Comments
- **Public items** (`pub`): all public modules, structs, enums, traits, functions, methods, constants, and type aliases must have `///` doc comments.
- **Public re-exports**: document the re-export or add a module-level doc comment that explains the re-exported surface.
- **Crate root**: include a crate-level `//!` doc comment describing the crate, its purpose, and primary usage.

## What Each Doc Comment Should Include
- **One-line summary**: start with a short, imperative or descriptive sentence.
- **Clarity over completeness**: document the purpose and behavior, not the implementation.
- **Parameters & returns**: mention any non-obvious constraints or invariants.
- **Error conditions**: describe when a function can fail and why.
- **Examples**: provide `rust` code blocks for non-trivial APIs or when a call-site is not obvious.

## Structure & Style
- Use Markdown formatting inside doc comments.
- Prefer `# Examples` with runnable snippets.
- Keep examples minimal and focused.
- Avoid restating type information already in the signature unless it adds semantic clarity.

## Exceptions
- Private items (`pub(crate)` or narrower) do not require doc comments unless the module owner deems it necessary for maintenance.
- Internal test helpers do not require doc comments.

## References
- Rust API Guidelines – Documentation:
  - https://rust-lang.github.io/api-guidelines/documentation.html
