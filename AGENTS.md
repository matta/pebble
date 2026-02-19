# Project: Pebble

This project is a Rust re-implementation of the existing `beads` tool, with a drastically reduced feature set.

## Project Goals

1.  **Drastically Simpler**: Focus on core features, avoiding complexity.
2.  **Reduced Feature Set**:
    - Backend: **JSONL ONLY**. No SQLite support.
    - Configuration: **Strict**. The only supported format is TOML configuration in `.pebble/config.toml`. Other configurations are unsupported. (Note: Original `beads` used `.beads/config.yaml`).
    - Daemon Mode: **NOT SUPPORTED**. The program will not run as a daemon. Configuration suggesting daemon mode results in run time errors.
3.  **TDD approach**:
    - **Test-Driven Development (TDD)** is mandatory.
    - Extensive test coverage is required.
    - Tests should exercise the functionality thoroughly.
4.  **Target Feature Set**:
    - `bd` version `0.49.6 (c064f2aa)` functionality.
    - The repository `../beads` is synced to this release commit.
    - Use `../mydoo` as a reference testbed where beads is set up correctly.

## Implementation Details

- **Testbed**: Use `../mydoo` for understanding the intended workflow and verifying behavior against the original `bd` tool.
- **Reference**: The `../beads` directory contains the original implementation (Go). Consult it for behavior clarification when needed, but do not copy-paste code directly. Focus on re-implementation in idiomatic Rust.
- **Configuration Parsing**: Must parse `.pebble/config.toml` format. Specifically handle `sync-branch`.
- **Database**: **JSONL**. The `issues.jsonl` file is the single source of truth.
- **Architecture: Worktree-Only Data Storage**: 
    - **No Local Copy**: The `issues.jsonl` file does **NOT** exist in the user's working directory (e.g., `.beads/issues.jsonl`).
    - **Sync-Branch Location**: The data resides **exclusively** in a Git worktree checked out to the configured `sync-branch` (default `pebble-sync`).
    - **Operations**: All `pebble` commands (read/write) must locate this worktree (default `.git/pebble-worktrees/<branch>`) and operate directly on the file within it.
    - **Sync Command**: `pebble sync` is strictly a wrapper for Git operations (`fetch`, `merge`, `push`) **within the worktree**. It does **not** copy files between the worktree and the main working directory.

## Data Model

The `Issue` struct must strictly adhere to the following schema derived from `issues.jsonl`:

```rust
struct Issue {
    id: String,           // e.g., "mydoo-0kq"
    title: String,
    description: String,
    status: String,       // e.g., "closed", "open"
    priority: i32,        // e.g., 0
    issue_type: String,   // e.g., "epic", "task"
    owner: String,        // e.g., "matt@rfc20.org"
    created_at: String,   // RFC3339 timestamp
    created_by: String,   // Display name
    updated_at: String,   // RFC3339 timestamp
    closed_at: Option<String>, // RFC3339 timestamp, nullable/optional
    close_reason: Option<String>, // e.g., "Closed", nullable/optional
}
```

## Constraints

- Strictly adhere to TDD. Write failing tests first, then implement.
- Maintain compatibility with the existing `bd` command interface where applicable for the subset of features supported.
- Do not implement daemon mode.
- **Style Guide**: Adhere to the [Style Guide](.gemini/styleguide.md).

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
