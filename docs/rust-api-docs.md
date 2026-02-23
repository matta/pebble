# Rust Documentation Requirements

This document codifies the Rust documentation expectations for this repository. It is adapted from the [Rust API Guidelines: documentation chapter](https://rust-lang.github.io/api-guidelines/documentation.html), with adjustments for a **binary-crate-only workspace** where `pub` visibility serves intra-crate access, not external API stability.

## Guiding Principle

Visibility keywords are an unreliable proxy for "needs docs" in a binary crate. Instead, documentation requirements are based on **role**—how widely an item is shared, and how hard it is to understand from its signature alone.

## Tier 1 — Full Doc Comments

**Applies to:** items that cross a module boundary (`pub`, `pub(crate)`, `pub(super)`) and the crate root.

| Item kind | Requirement |
|-----------|-------------|
| Crate root | `//!` comment describing purpose and primary usage |
| Modules | One-line `//!` summary when the module name alone is ambiguous |
| Structs, enums, traits | Summary + field/variant docs for anything non-obvious |
| Functions & methods | Summary + parameters/returns + error conditions |
| Constants & type aliases | Summary explaining the value or purpose |

### What Each Doc Comment Should Include
- **One-line summary**: start with a short, imperative or descriptive sentence.
- **Clarity over completeness**: document the purpose and behavior, not the implementation.
- **Parameters & returns**: mention any non-obvious constraints or invariants.
- **Error conditions**: describe when a function can fail and why.
- **Examples**: provide `rust` code blocks for non-trivial APIs or when a call-site is not obvious.

## Tier 2 — Summary Doc Comment

**Applies to:** private items that are non-trivial.

A **one-liner `///` comment** is required for any private function, struct, enum, or constant that is:
- Longer than roughly 10 lines, **or**
- Has non-obvious behavior that the name and signature don't fully convey.

Struct fields that aren't self-explanatory should also have a brief inline doc comment.

## Tier 3 — No Doc Comment Required

- Trivial one-liner helpers or obvious getters/setters.
- `Default`, `Display`, and similar trait impls where behavior is clear from the type.
- Test functions and test helpers.
- Internal serde default functions (e.g., `default_tasks_dir`).

## Structure & Style
- Use Markdown formatting inside doc comments.
- Prefer `# Examples` with runnable snippets.
- Keep examples minimal and focused.
- Avoid restating type information already in the signature unless it adds semantic clarity.

## References
- Rust API Guidelines – Documentation:
  - https://rust-lang.github.io/api-guidelines/documentation.html
