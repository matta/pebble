# Track Specification: Pebble Initialization and JSONL Import

## Overview
This track introduces the `pebble init` command to simplify project setup and a `pebble import` command to allow users to ingest or migrate data from JSONL files (such as those produced by `beads`). It also adds proactive initialization checks to other CLI commands and formalizes the structure of the data synchronization branch.

## Functional Requirements

### 1. `pebble init` Command
- **Purpose**: Initializes a repository for use with Pebble.
- **Behavior**: 
    - **Orphaned Sync Branch**: Automatically creates a dedicated synchronization branch (default: `pebble-data`).
    - **Diverged Implementation**: This branch MUST be created as an **orphan** (`git checkout --orphan`). It must not share any history or common ancestor with `main`, `master`, or any other branch in the repository.
    - **Worktree Setup**: Sets up a Git worktree for Pebble data storage using this orphaned branch.
    - **Configuration**: Configures necessary local Pebble settings (e.g., `sync-branch`).
- **Constraint**: Must fail if the repository already has an initialized Pebble worktree or if there are uncommitted changes that might conflict with setup.

### 2. Initialization Awareness
- **Behavior**: All Pebble commands (e.g., `list`, `show`, `add`) must verify if `pebble init` has been run.
- **Error Handling**: If uninitialized, the command must exit with a status code 1 and print: `Error: Pebble is not initialized in this repository. Run 'pebble init' to get started.`

### 3. `pebble import <file>` Command
- **Purpose**: Imports issues from an external JSONL file into the active Pebble workspace.
- **Behavior**:
    - Reads JSONL objects from the specified file.
    - **Shared Merging Logic**: Delegates to a shared merging module to handle field-level updates and collision detection (e.g., identical IDs with different content).
    - **No 3-Way Merge**: Unlike `pebble merge` (which may eventually support 3-way branch merging), `import` performs a direct update/overwrite based on shared logic because there is no common Git ancestor for external files.
- **Constraints**:
    - **Idempotency**: Importing the same file twice results in no changes on the second run.
    - **Git Safety**: Refuses to import if there are uncommitted changes in the Pebble data directory.

## Non-Functional Requirements
- **Atomicity**: All operations (init/import) must be atomic. If a failure occurs mid-process, the system should not be left in a corrupted state.
- **Documentation**: Explicitly document in the implementation and any user-facing docs that the sync branch is an independent, orphaned branch.
- **TDD**: Implementation must follow the project's TDD workflow with >80% code coverage.

## Acceptance Criteria
- [ ] `pebble init` successfully creates an **orphaned** `pebble-data` branch with no shared history.
- [ ] `pebble init` sets up a worktree tied to this orphaned branch.
- [ ] Running any Pebble command in a fresh repo suggests `pebble init`.
- [ ] `pebble import` correctly merges issues from a `beads`-style JSONL file using shared merging logic for field updates.
- [ ] `pebble import` is idempotent and refuses to run on a "dirty" data worktree.
- [ ] Automated tests cover success and failure modes for both commands.
