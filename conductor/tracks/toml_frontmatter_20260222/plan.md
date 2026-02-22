# Implementation Plan: TOML Frontmatter Migration

## Phase 1: Research & Setup [checkpoint: b2d6a57]
- [x] Task: Audit codebase for "YAML" references and identify all test fixtures.
    - [x] Grep for `YAML`, `yaml`, `Yaml` in `crates/pebble/src/`.
    - [x] Grep for `YAML`, `yaml`, `Yaml` in `docs/*.md` (excluding `docs/rfcs/`).
    - [x] List all `.md` files in `crates/pebble/tests/` and `.pebble-test/`.
- [x] Task: Verify current `Cargo.toml` and existing `toml` crate usage.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Research & Setup' (Protocol in workflow.md)

## Phase 2: Core Logic Migration
- [ ] Task: Remove `serde_yaml` dependency and update `Cargo.toml`.
- [ ] Task: Update `models.rs` to reflect TOML frontmatter.
    - [ ] Update doc comments and any internal "YAML" mentions.
- [ ] Task: Implement TOML parsing in `parser.rs`.
    - [ ] Update `split_frontmatter` to detect `+++` instead of `---`.
    - [ ] Update `parse_frontmatter` to use `toml::from_str`.
    - [ ] Add failing unit tests for TOML parsing (TDD).
    - [ ] Implement the change and pass tests.
- [ ] Task: Implement TOML serialization in `commands_write.rs`.
    - [ ] Update `serialize_frontmatter` to use `toml` and `+++` delimiters.
    - [ ] Ensure datetimes are serialized as native TOML types.
    - [ ] Add failing unit tests for TOML serialization (TDD).
    - [ ] Implement the change and pass tests.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Core Logic Migration' (Protocol in workflow.md)

## Phase 3: Documentation & Source Scrubbing
- [ ] Task: Remove all mentions of "YAML" from `crates/pebble/src/`.
    - [ ] Update error messages (e.g., "Failed to parse YAML frontmatter" -> "Failed to parse TOML frontmatter").
    - [ ] Update variable names if any (e.g., `yaml_block` -> `toml_block`).
- [ ] Task: Update active documentation.
    - [ ] Update `AGENTS.md`.
    - [ ] Update `docs/schema.md`.
    - [ ] Update `docs/cli-contract.md`.
    - [ ] Update `GEMINI.md` and `AGENTS.md` if they mention YAML.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Documentation & Source Scrubbing' (Protocol in workflow.md)

## Phase 4: Test Migration & Final Verification
- [ ] Task: Migrate all test fixtures and task files.
    - [ ] Update all `.md` files in `crates/pebble/tests/` to use `+++` and TOML syntax.
    - [ ] Update any tasks in `.pebble-test/tasks/`.
- [ ] Task: Final Verification.
    - [ ] Run `just check` to ensure no linting/type errors.
    - [ ] Run `just test` to ensure all tests pass with TOML.
    - [ ] Manually verify `pebble add` and `pebble show` with the new format.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Test Migration & Final Verification' (Protocol in workflow.md)
