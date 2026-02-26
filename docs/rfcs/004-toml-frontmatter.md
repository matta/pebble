[STATUS: SUPERSEDED by RFC 005]

# RFC 004: TOML Frontmatter (Exclusive)

## 1. Summary

Replace YAML frontmatter with TOML frontmatter as the exclusive metadata format for Pebble task files. Delimiters change from `---` to `+++`. The `serde_yaml` dependency is removed and replaced by the `toml` crate (already present for config parsing).

## 2. Motivation

### 2.1 The `serde_yaml` Ecosystem Crisis

In March 2024 the canonical Rust YAML library `serde_yaml` was archived and deprecated by its maintainer (dtolnay). With tens of millions of downloads and >4,000 downstream dependents, its abandonment triggered ecosystem-wide instability. The core reason: YAML's sprawling specification is fundamentally incompatible with Rust's strict safety guarantees.

The underlying parser, `unsafe-libyaml`, is auto-translated from C and relies extensively on `unsafe` Rust, compromising the memory-safety promises of the language. YAML's anchors, aliases, and implicit type coercion make constructing a safe AST nearly impossible and expose parsers to denial-of-service attacks (e.g. "Billion Laughs" via recursive alias expansion).

The successor landscape is fragmented. The most prominent fork (`serde_yml`) was widely identified by the Rust community as containing AI-generated code, injecting unnecessary cryptographic dependencies, and disabling its issue tracker — it is broadly considered unsafe. Legitimate forks (`serde-yaml-ng`, `serde-yaml-bw`) still depend on `unsafe-libyaml`. The pure-Rust alternative `serde-saphyr` is promising but young, and represents a specialized workaround for problems created entirely by YAML's poor foundational design.

**Bottom line**: Pebble currently depends on a deprecated, unmaintained library with known safety issues. Switching frontmatter format eliminates this dependency entirely rather than gambling on an uncertain successor.

### 2.2 YAML's Implicit Type Coercion

YAML silently coerces unquoted values into unexpected types:

| Input | YAML 1.1 interpretation | YAML 1.2 interpretation |
|---|---|---|
| `NO` | boolean `false` | string `"NO"` |
| `on` | boolean `true` | string `"on"` |
| `3.0` | float `3.0` | float `3.0` |
| `2026-02-22` | datetime (sometimes) | depends on parser |

The infamous "Norway Problem" — where the country code `NO` silently becomes `false` — is a symptom of a deeper design flaw. YAML 1.2 attempted to fix this, but tooling support remains fragmented, and many parsers still operate under 1.1 rules. Authors must defensively quote strings, negating YAML's supposed ergonomic advantage.

TOML refuses to guess. Strings require quotes. Booleans are strictly `true`/`false`. Datetimes are a first-class type with unambiguous RFC 3339 syntax. There is exactly one way to express each type.

### 2.3 Architectural Alignment with Pebble

Pebble already uses TOML for its configuration file (`.pebble/config.toml`). Adopting TOML for frontmatter means the entire project uses **one serialization format** for all structured data, reducing cognitive overhead for contributors and eliminating a dependency (`serde_yaml`).

Pebble's frontmatter schema is flat — a handful of scalar fields plus two small arrays (`needs`, `tags`). This is exactly the structure TOML excels at. TOML's verbosity penalty for deeply nested data is irrelevant here.

### 2.4 Native Datetime Support

TOML treats datetimes as a first-class primitive. A field like `created_at = 2026-02-22T10:30:00Z` is guaranteed to parse as a datetime object — no quoting, no parser-dependent coercion. Given that Pebble's schema contains three datetime fields (`created_at`, `modified_at`, `resolved_at`), this is a direct ergonomic and safety win.

### 2.5 Rust Ecosystem Preference

The Rust community has broadly converged on TOML as the preferred configuration and metadata format. `Cargo.toml` is the canonical example. Zola, the Rust-native static site generator, treats TOML as its primary frontmatter language for the same safety reasons. The `toml` crate is mature, well-maintained, and written in safe Rust.

## 3. Specification

### 3.1 Delimiter

Task files use `+++` as frontmatter delimiters (replacing `---`):

```markdown
+++
id = "proj-0kq"
title = "Deploy staging environment"
status = "todo"
created_at = 2026-01-15T10:30:00Z
needs = ["proj-abc", "proj-def"]
tags = ["infra", "deploy"]
+++

Run the canary deploy pipeline against the `staging` cluster.
```

### 3.2 Format Rules

- All string values must be quoted.
- `status` is a quoted string matching the enum: `"todo"`, `"in_progress"`, `"done"`, `"canceled"`.
- `created_at`, `modified_at`, `resolved_at` are bare RFC 3339 datetimes (TOML native type — no quotes).
- `needs` and `tags` are TOML arrays of quoted strings.
- `priority` is a bare integer (no quotes).
- Comments (`#`) are permitted in frontmatter (useful for human notes about metadata).
- Field meanings, optionality, and validation constraints are defined by the data-layer schema in [`docs/schema.md`](../schema.md). This RFC only specifies TOML encoding and delimiters.

### 3.3 Parser Changes

- The parser detects `+++` on the first line (replacing `---`).
- Files that begin with `---` are treated as having no frontmatter; the `---` line is part of the Markdown body and no YAML parsing occurs.
- The extracted block is deserialized via the `toml` crate (replacing `serde_yaml`).
- `serde_yaml` is removed from `Cargo.toml` entirely.
- Error messages update from "YAML frontmatter" to "TOML frontmatter".

### 3.4 Serializer Changes

- `pebble add` and `pebble update` emit TOML frontmatter between `+++` delimiters.

### 3.5 Unknown Fields

The current policy is unchanged: unknown keys are ignored on reads, warned by `doctor`/`fix`, and rejected by `check`. TOML's strict syntax means malformed input fails loudly at parse time rather than silently coercing.

## 4. Impact

### 4.1 Source Code

| File | Change |
|---|---|
| `crates/pebble/Cargo.toml` | Remove `serde_yaml` dependency |
| `crates/pebble/src/parser.rs` | `---` → `+++`, `serde_yaml::from_str` → `toml::from_str` |
| `crates/pebble/src/commands_write.rs` | Serialize frontmatter as TOML between `+++` |
| `crates/pebble/src/models.rs` | Update doc comments from "YAML" to "TOML" |
| All test files | Update fixtures from `---`/YAML to `+++`/TOML |

### 4.2 Live Specification Documents

| Document | What changes |
|---|---|
| `AGENTS.md` | "YAML frontmatter" → "TOML frontmatter", example delimiters |
| `docs/schema.md` | Delimiter description, Rust struct doc comments, example |
| `docs/cli-contract.md` | "YAML" → "TOML" in scanning/error sections |
| `docs/graph-semantics.md` | No changes needed (format-agnostic) |

### 4.3 Frozen RFCs

RFC 001 and RFC 002 are frozen historical documents. They remain as-is. This RFC supersedes their frontmatter format decisions.

### 4.4 Existing Task Files

There are currently zero production Pebble repositories. Migration is a non-issue. If any test fixture `.md` files exist under `tasks-dir`, they are updated as part of this RFC's implementation.

## 5. Alternatives Considered

### 5.1 Adopt a `serde_yaml` Fork

Forks like `serde-yaml-ng` or `serde-yaml-bw` exist, but all depend on `unsafe-libyaml`. `serde-saphyr` bypasses the AST but is young. None eliminate the fundamental problem: YAML's specification is too complex for a safe, performant Rust parser. Adopting a fork trades one unmaintained dependency for an uncertain one.

### 5.2 Support Both YAML and TOML

Dual-format parsing adds complexity to the parser, serializer, tests, documentation, and user mental model. Pebble's "worse is better" philosophy favors one format with zero ambiguity.

### 5.3 JSON Frontmatter

JSON lacks comments, multiline strings, and native datetime support. It is excessively strict for human-authored metadata. Not appropriate for a tool designed to be "equally useful for humans and AI agents."

## 6. Verification Plan

### 6.1 Automated Tests

- All existing parser and CLI tests are updated to use `+++`/TOML fixtures. Passing `just test` confirms correctness.
- `just check` confirms lint/clippy cleanliness.

### 6.2 Manual Verification

- Run `pebble add "Test task"` and inspect the generated `.md` file to confirm `+++`/TOML output.
- Run `pebble show <id> --json` and confirm all fields parse correctly.
- Edit a task file by hand with TOML frontmatter and confirm `pebble list` reads it.
