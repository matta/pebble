# Implementation Plan: TOML Frontmatter Migration

## Phase 1: Research & Setup [checkpoint: b2d6a57]
- [x] Task: Audit codebase for "YAML" references and identify all test fixtures.
    - [x] Grep for `YAML`, `yaml`, `Yaml` in `crates/pebble/src/`.
    - [x] Grep for `YAML`, `yaml`, `Yaml` in `docs/*.md` (excluding `docs/rfcs/`).
    - [x] List all `.md` files in `crates/pebble/tests/` and `.pebble-test/`.
- [x] Task: Verify current `Cargo.toml` and existing `toml` crate usage.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Research & Setup' (Protocol in workflow.md)

## Phase 2: Core Logic Migration [checkpoint: 5e611f0]
- [x] Task: Remove `serde_yaml` dependency and update `Cargo.toml`.
- [x] Task: Update `models.rs` to reflect TOML frontmatter.
    - [x] Update doc comments and any internal "YAML" mentions.
- [x] Task: Implement TOML parsing in `parser.rs`.
    - [x] Update `split_frontmatter` to detect `+++` instead of `---`.
    - [x] Update `parse_frontmatter` to use `toml::from_str`.
    - [x] Add failing unit tests for TOML parsing (TDD).
    - [x] Implement the change and pass tests.
- [x] Task: Implement TOML serialization in `commands_write.rs`.
    - [x] Update `serialize_frontmatter` to use `toml` and `+++` delimiters.
    - [x] Ensure datetimes are serialized as native TOML types.
    - [x] Add failing unit tests for TOML serialization (TDD).
    - [x] Implement the change and pass tests.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Core Logic Migration' (Protocol in workflow.md)

## Phase 3: Documentation & Source Scrubbing [checkpoint: 5e611f0]
- [x] Task: Remove all mentions of "YAML" from `crates/pebble/src/`.
    - [x] Update error messages (e.g., "Failed to parse YAML frontmatter" -> "Failed to parse TOML frontmatter").
    - [x] Update variable names if any (e.g., `yaml_block` -> `toml_block`).
- [x] Task: Update active documentation.
    - [x] Update `AGENTS.md`.
    - [x] Update `docs/schema.md`.
    - [x] Update `docs/cli-contract.md`.
    - [x] Update `GEMINI.md` and `AGENTS.md` if they mention YAML.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Documentation & Source Scrubbing' (Protocol in workflow.md)

## Phase 4: Test Migration & Final Verification [checkpoint: 6e1bf45]
- [x] Task: Migrate all test fixtures and task files.
    - [x] Update all `.md` files in `crates/pebble/tests/` to use `+++` and TOML syntax.
    - [x] Update any tasks in `.pebble-test/tasks/`.
- [x] Task: Final Verification.
    - [x] Run `just check` to ensure no linting/type errors.
    - [x] Run `just test` to ensure all tests pass with TOML.
    - [x] Manually verify `pebble add` and `pebble show` with the new format.
- [x] Task: Conductor - User Manual Verification 'Phase 4: Test Migration & Final Verification' (Protocol in workflow.md)

## Phase: Review Fixes
- [x] Task: Apply review suggestions f6d0d2a
