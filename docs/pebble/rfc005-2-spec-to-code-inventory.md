+++
id = "pebl-zjpn6fbfmp"
title = "RFC005-2 Spec-to-code inventory"
status = "done"
priority = 0
created_at = 2026-03-01T16:39:17.452093522+00:00
modified_at = 2026-03-01T16:53:15.4062932+00:00
resolved_at = 2026-03-01T16:53:15.406282202+00:00
needs = ["pebl-urd2fpbmfk"]
tags = ["planning", "rfc005"]
+++
Goal:
Create an explicit file-level inventory so implementation work is mechanical.

Do exactly this:
1. Read `docs/rfcs/005-yaml-frontmatter.md`.
2. Create a checklist in this task body with exact files to touch, grouped by category:
   - Parser/read path files
   - Writer/mutation path files
   - Tests/fixtures
   - Docs (`docs/schema.md`, `docs/cli-contract.md`, root `AGENTS.md`)
3. For each file, add one sentence describing the required YAML change.
4. Mark each line `[ ]` (unchecked); do not implement anything yet.

Acceptance Criteria:
- This task body contains a complete, file-by-file action list.
- Another agent can implement from the list without deciding where to edit.

Implementation Checklist:

Parser/read path files
- [ ] `Cargo.toml` - add `serde-saphyr` (and any required YAML/validation companions selected for RFC005) so read-path code can deserialize YAML frontmatter without TOML parsing.
- [ ] `crates/pebble/src/parser.rs` - replace `+++` boundary detection and TOML deserialization with YAML `---` boundary scanning and `serde_saphyr::from_str` into `TaskFrontmatter`.
- [ ] `crates/pebble/src/graph.rs` - update the task-file prefilter and parse-error messaging to treat YAML-frontmatter files (`---` start) as parse candidates and skip non-frontmatter files consistently.
- [ ] `crates/pebble/src/models.rs` - adjust frontmatter model typing/serde attributes needed for YAML deserialization (including unknown-field capture) and update Rustdoc/examples that currently document TOML-only parsing.
- [ ] `crates/pebble/src/main.rs` - update crate-level storage format docs to describe Markdown task files with YAML frontmatter instead of TOML.

Writer/mutation path files
- [ ] `crates/pebble/src/models.rs` - replace TOML serialization in `TaskNode::to_markdown` with YAML frontmatter emission wrapped in `---` delimiters while preserving trailing-newline/body behavior.
- [ ] `crates/pebble/src/commands_add.rs` - ensure `add` writes new task files using the updated YAML `TaskNode` writer and keeps existing ID/needs semantics unchanged.
- [ ] `crates/pebble/src/commands_write.rs` - ensure `update`/mutation flows persist YAML-frontmatter task files via the new serializer while retaining timestamp and mutation semantics.
- [ ] `crates/pebble/src/commands_fix.rs` - ensure fix-mode write-backs (`created_at` repairs) serialize task files using YAML frontmatter.

Tests/fixtures
- [ ] `crates/pebble/src/parser.rs` (unit tests module) - convert parser fixtures/assertions to YAML delimiters and YAML payload syntax, including missing/unclosed/invalid-frontmatter cases.
- [ ] `crates/pebble/src/models.rs` (unit tests module) - replace TOML frontmatter parse/serialize tests with YAML equivalents for `TaskFrontmatter` and `TaskNode` serialization behavior.
- [ ] `crates/pebble/tests/support.rs` - convert helper-generated fixture frontmatter from TOML/`+++` to YAML/`---` so integration tests exercise the new format by default.
- [ ] `crates/pebble/tests/cli_check.rs` - update inline fixture task files and expected diagnostics text to YAML frontmatter and YAML-focused parse-failure wording.
- [ ] `crates/pebble/tests/cli_fix.rs` - convert TOML-frontmatter fixture content to YAML and keep assertions focused on fix semantics rather than format artifacts.
- [ ] `crates/pebble/tests/cli_list_filters.rs` - rewrite generated fixture frontmatter builders to emit YAML blocks instead of TOML key/value text.
- [ ] `crates/pebble/tests/cli_list_sort.rs` - convert any inline task fixtures from TOML delimiters/syntax to YAML frontmatter equivalents.
- [ ] `crates/pebble/tests/cli_mutation_semantics.rs` - replace all TOML fixture snippets and output expectations with YAML-frontmatter forms while preserving mutation/timestamp assertions.
- [ ] `crates/pebble/tests/cli_scan_duplicates.rs` - convert duplicate-ID fixture files to YAML frontmatter and keep duplicate-detection assertions unchanged.
- [ ] `crates/pebble/tests/cli_search.rs` - change helper fixture templates and string-replacement setup from TOML `+++` to YAML `---`.
- [ ] `crates/pebble/tests/cli_show.rs` - convert show-command fixture task files to YAML frontmatter syntax and delimiters.
- [ ] `docs/pebble/*.md` - migrate all existing Pebble task files from TOML `+++` frontmatter to YAML `---` frontmatter using the one-off RFC005 migration script.

Docs (`docs/schema.md`, `docs/cli-contract.md`, root `AGENTS.md`)
- [ ] `docs/schema.md` - update normative data-layer format language and examples from TOML `+++` to YAML `---` while keeping the same task schema/invariants.
- [ ] `docs/cli-contract.md` - update command-contract scanning/error wording to reference YAML frontmatter parsing/skip behavior instead of TOML parsing.
- [ ] `AGENTS.md` - update project invariant text that currently states task metadata is stored in TOML frontmatter so docs match RFC005/YAML behavior.
