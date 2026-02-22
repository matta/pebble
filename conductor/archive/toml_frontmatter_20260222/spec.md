# Specification: TOML Frontmatter Migration

## Overview
This track implements the transition from YAML to TOML as the exclusive frontmatter format for Pebble task files, as proposed in RFC 004. This change eliminates the deprecated `serde_yaml` dependency and aligns frontmatter with Pebble's existing configuration format. **A key requirement is the complete removal of any mention of "YAML" from the active codebase and documentation.**

## Functional Requirements
1. **Frontmatter Delimiter Change**:
    - The task file frontmatter delimiter must change from `---` to `+++`.
    - Files starting with `+++` are parsed as TOML frontmatter.
    - Files starting with `---` are treated as having no frontmatter.

2. **TOML Serialization**:
    - All frontmatter metadata must be serialized as TOML.
    - String values must be quoted.
    - Datetime fields (`created_at`, `modified_at`, `resolved_at`) must use native, unquoted TOML datetime format (RFC 3339).
    - Arrays (`deps`, `tags`) must be serialized as TOML arrays.

3. **CLI & Parser Updates**:
    - `pebble add` and `pebble update` must emit `+++` delimiters and TOML-formatted metadata.
    - Deserialization must use the `toml` crate.
    - Error messages must refer to "TOML" instead of "YAML".

4. **Scrubbing "YAML" References**:
    - **Rust Source**: All doc comments, error strings, and variable names containing "YAML" (case-insensitive) must be updated to "TOML" or removed.
    - **Active Documentation**: Update `AGENTS.md`, `docs/schema.md`, `docs/cli-contract.md`, and any other non-frozen documentation to replace "YAML" with "TOML".
    - **Exception**: Frozen RFCs (RFC 001, 002, 003) and historical records remain unchanged.

5. **Dependency Management**:
    - Remove `serde_yaml` from `Cargo.toml`.

## Acceptance Criteria
- [ ] All existing test fixtures are updated to use `+++`/TOML.
- [ ] `just check` passes all linting and type checks.
- [ ] `just test` passes all tests.
- [ ] `pebble add` creates files with `+++` delimiters and TOML content.
- [ ] No mentions of "YAML" (case-insensitive) exist in `crates/pebble/src/` or `docs/` (excluding `docs/rfcs/`).
- [ ] `serde_yaml` is removed from the project.

## Out of Scope
- Backward compatibility for YAML frontmatter.
- Support for other frontmatter formats.
