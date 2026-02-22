# Project: Pebble

This project is a CLI task tracker written in Rust, built upon a Markdown-native graph design.

## Project Goals & Immutable Invariants

1. **Markdown-Native Storage**: 
    - Task data is stored entirely in Markdown files with YAML frontmatter.
    - There is no JSONL database, no SQLite database, and no hidden Git worktrees.
    - The files themselves act as the directed graph.

2. **Strict Project Invariants**:
    - **The `id` field is immutable**: The CLI must never change an ID after generation.
    - **No Preemptive Cycle Prevention**: Do **NOT** write cycle-prevention or deadlock-handling logic in the storage/write layer. The read layer naturally handles cycles by refusing to consider a cyclical task as "ready".
    - **One True Edge (`deps`)**: Hierarchy and temporal ordering are entirely collapsed into a single edge: `deps`. Do not re-introduce `parent`/`child` relationships.
    - **Absolute Readiness**: A task is ready if and only if all of its `deps` exist and have a terminal status. Dangling pointers structurally block readiness forever without panicking.

3. **Test-Driven Development (TDD) Mandated**:
    - **Test-Driven Development (TDD)** is strictly mandatory.
    - Agents must write failing unit tests *before* implementing any graph traversal or business logic defined in `docs/graph-semantics.md`.
    - Extensive test coverage is required for all new logic.

4. **Configuration**:
    - TOML configuration in `.pebble/config.toml` dictates `tasks-dir`. 
    - `tasks-dir` must always be a path relative to the project root.

## Architecture Documentation Requirements

When implementing, strictly rely on the specifications extracted from the RFC:
- [Data Layer (`schema.md`)](docs/schema.md) - Contains the strict Rust struct mappings and details dropped audit fields.
- [Interface Layer (`cli-contract.md`)](docs/cli-contract.md) - Exact command, flag, and JSON output specifications.
- [Logic Layer (`graph-semantics.md`)](docs/graph-semantics.md) - Definition of traversal rules, absolute readiness, and dynamic starvation prevention scoring.
- [Historical RFC & Decisions](docs/architecture/) - Only consult frozen architecture documents for pure context, *never* for active implementation details.

## Constraints

- Strictly adhere to TDD.
- **Style Guide**: Adhere to the [.gemini/styleguide.md](.gemini/styleguide.md).
- **Docs Discoverability**: Any new documentation added to the repository must be linked from this file.
- **Clippy Changes Require Approval**: Any modifications to clippy configuration or suppression of clippy warnings must be explicitly discussed and approved by the operator before applying.

## Workflows

### Push the Pebbles
This workflow requires the following steps to be done in order:
1. `just check` must pass.
2. `just test` must pass.
3. `git commit`.
4. `git push`.

## Gates

### Push Gate
- **Requirement**: `just check` must pass cleanly.
- **Enforcement**: Agents must run `just check` and ensure it exits with code 0 before pushing any code to the repository.
