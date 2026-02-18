# Implementation Plan: Renaming and Cleanup

## Phase 1: Environment & Directory Renaming [checkpoint: cd3b841]
This phase focuses on the structural changes required to rename the project's local configuration and tooling references.

- [x] **Task: Rename .beads to .pebble** cd3b841
    - [x] Write a test in `crates/pebble/tests/config_test.rs` to verify that `pebble` looks for a directory named `.pebble` by default.
    - [x] Implement the change in `crates/pebble/src/config.rs` to prioritize `.pebble` over `.beads`.
- [x] **Task: Update xtask and justfile** cd3b841
    - [x] Preserve `xtask check-beads` as it verifies forbidden words.
    - [x] Ensure the xtask still correctly reads `.bead-whitelist`.
- [x] **Task: Conductor - User Manual Verification 'Phase 1: Environment & Directory Renaming' (Protocol in workflow.md)** cd3b841

## Phase 2: Code & Internal Reference Updates
This phase updates internal strings, error messages, and project documentation.

- [~] **Task: Update internal string references**
    - [ ] Write unit tests for CLI help output and error messages that contain "beads".
    - [ ] Update string literals in `crates/pebble/src` from "beads" to "pebble" (where appropriate).
- [ ] **Task: Update project documentation**
    - [ ] Update `AGENTS.md` and `implementation_plan.md` (root version) to reflect the new naming conventions.
    - [ ] Ensure all mentions of the original tool as "beads" are preserved as per user instructions.
- [ ] **Task: Conductor - User Manual Verification 'Phase 2: Code & Internal Reference Updates' (Protocol in workflow.md)**

## Phase 3: Branch & Worktree Manager Updates
This phase updates the synchronization logic to use the new branch and worktree directory names.

- [ ] **Task: Update WorktreeManager**
    - [ ] Write tests in `crates/pebble/src/worktree.rs` to verify that worktrees are created in `.git/pebble-worktrees`.
    - [ ] Update `WorktreeManager` to use the new directory and the branch name `pebble-sync` as default.
- [ ] **Task: Update sync command**
    - [ ] Write an integration test in `crates/pebble/tests/cli_tests.rs` for `pebble sync` that verifies the use of `pebble-sync`.
    - [ ] Update the `sync` command implementation in `crates/pebble/src/command.rs`.
- [ ] **Task: Conductor - User Manual Verification 'Phase 3: Branch & Worktree Manager Updates' (Protocol in workflow.md)**
