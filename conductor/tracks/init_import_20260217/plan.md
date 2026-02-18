# Implementation Plan: Pebble Initialization and JSONL Import

This plan outlines the steps to implement the `pebble init` and `pebble import` commands, along with repository-wide initialization checks, following a TDD approach.

## Phase 1: Initialization Awareness and Infrastructure
Focus on making Pebble aware of its own setup state and providing helpful error messages.

- [ ] Task: Implement `Config::is_initialized()` utility
    - [ ] Write failing test to check initialization status in a non-git or non-pebble repo.
    - [ ] Implement logic to detect if a `.pebble` worktree/directory exists.
- [ ] Task: Update CLI to enforce initialization for existing commands
    - [ ] Write failing tests for `list`, `show`, `add` in an uninitialized repo.
    - [ ] Implement a check in the command execution path that suggests `pebble init` on failure.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Initialization Awareness' (Protocol in workflow.md)

## Phase 2: `pebble init` Implementation
Implement the core setup logic for creating orphaned sync branches and worktrees.

- [ ] Task: Implement `pebble init` CLI command structure
    - [ ] Write failing test for the `init` command availability and basic help.
    - [ ] Implement the command entry point in `command.rs`.
- [ ] Task: Implement orphaned branch creation
    - [ ] Write failing test that verifies a new branch created by `init` has no ancestors.
    - [ ] Implement `git checkout --orphan` logic via `run_shell_command` or a git library equivalent.
- [ ] Task: Implement worktree initialization
    - [ ] Write failing test to verify the `.pebble` directory is a valid Git worktree after `init`.
    - [ ] Implement `git worktree add` logic pointing to the orphaned branch.
- [ ] Task: Implement configuration persistence
    - [ ] Write failing test to verify `sync-branch` is correctly stored in local config.
    - [ ] Implement writing defaults to the Pebble configuration file.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: pebble init' (Protocol in workflow.md)

## Phase 3: Shared Merging Logic & `pebble import`
Create a reusable merge module and implement the import functionality.

- [ ] Task: Refactor and extract shared merging logic
    - [ ] Write failing tests for merging two issue objects (field-level updates, ID collisions).
    - [ ] Extract existing merge logic (if any) or implement new logic in `store.rs` or a new `merge.rs`.
- [ ] Task: Implement `pebble import` CLI command
    - [ ] Write failing test for `pebble import <file>` reading a sample JSONL file.
    - [ ] Implement command entry point and file path validation.
- [ ] Task: Implement Git safety and idempotency
    - [ ] Write failing tests for importing into a "dirty" worktree and for re-importing the same file.
    - [ ] Implement checks for uncommitted changes in the `.pebble` worktree before proceeding.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: pebble import' (Protocol in workflow.md)

## Phase 4: Quality Gate and Final Verification
Ensure high code quality and consistency across all new features.

- [ ] Task: Verify 80% Code Coverage
    - [ ] Run coverage tools and address gaps in the new commands and modules.
- [ ] Task: Final Documentation and Style Audit
    - [ ] Ensure all public functions are documented and the orphaned branch nature is clearly explained in code comments.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Integration' (Protocol in workflow.md)
