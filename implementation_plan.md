# Implementation Plan - Pebble

This document outlines the step-by-step plan to implement `pebble`, a simplified Rust version of `beads`.
The approach is strict TDD. We will write a failing test, then implement the code to pass it.

## Phase 1: Foundation & Configuration (Completed)
- [x] Create project structure
- [x] Implement basic config parsing (TDD)
- [x] Implement command-line argument parsing (clap)
    - [x] Test `pebble --version`
    - [x] Test `pebble config get sync.branch`
- [ ] Implement database connection (rusqlite)
    - [ ] Test creating a fresh database
    - [ ] Test applying migrations (schema creation)

## Phase 1.5: Tooling & CI (Completed)
- [x] Restructure workspace (pebble + xtask)
- [x] Implement check-beads xtask command
- [x] Hook into `just check`

## Phase 2: Core Data Model & JSONL Store
- [x] Remove SQLite code (`src/db.rs`) and dependencies
- [x] Implement `Issue` struct matching `AGENTS.md` schema
    - [x] Add `serde` derives
- [x] Implement JSONL Store (`src/store.rs`)
    - [x] Test reading issues from a sample JSONL file
    - [x] Test writing issues to a JSONL file
    - [x] Test appending a new issue

## Phase 3: Worktree Architecture & Sync (Completed)
- [x] Implement `pebble sync` stub
- [x] Fail if no `sync-branch` is configured
- [x] Implement `WorktreeManager`
    - [x] Logic to find/create git worktree for `sync-branch`
    - [x] Logic to get absolute path to `issues.jsonl` in worktree
- [x] Implement `pebble sync`
    - [x] `git fetch` in worktree
    - [x] `git merge` (fast-forward) in worktree
    - [x] `git push` from worktree
- [x] Update `pebble` to use Worktree Path
    - [x] Ensure `store` uses the worktree path, NOT local file

## Phase 4: CLI Commands (Worktree-Native) (Completed)
- [x] Implement `pebble list` (reads from worktree)
- [x] Implement `pebble add` (writes to worktree)
- [x] Implement `pebble show` (reads from worktree)
- [x] Implement `pebble edit` (writes to worktree)

## Phase 5: Renaming & Cleanup (Eliminate 'beads')
- [ ] Rename `.beads` directory to `.pebble`
- [ ] Rename `beads-sync` branch references to `pebble-sync`
- [ ] Update `WorktreeManager` to use `.git/pebble-worktrees`
- [ ] Rename `xtask check-beads` to `xtask check-pebble`
- [ ] Rename `.bead-whitelist` to `.pebble-whitelist`
- [ ] Update all code, comments, and tests to use `pebble` instead of `beads`

## Phase 6: Intelligent 3-Way Merge for Sync
- [ ] Implement `pebble merge` command (internal tool for git merge driver)
- [ ] Implement field-level 3-way merge algorithm (matching `beads` behavior)
    - [ ] Match issues by (ID, CreatedAt, CreatedBy)
    - [ ] Latest `updated_at` wins for `title` and `description`
    - [ ] `closed` status wins over `open`
    - [ ] Dependencies: removals are authoritative (3-way merge)
    - [ ] Timestamps: `max(left, right)` for `updated_at`
- [ ] Update `pebble sync` to handle merge conflicts
    - [ ] Use `git merge` with custom merge driver if available, or manual 3-way merge logic
- [ ] Add TDD tests for various merge scenarios (conflicts, additions, deletions)

## Rules
1. **One Fail at a Time**: don't write multiple failing tests.
2. **Refactor**: Refactor after passing.
3. **No Daemon**: Ensure no daemon logic creeps in.
