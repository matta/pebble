# Implementation Plan - Pebble

This document outlines the step-by-step plan to implement `pebble`, a simplified Rust version of `beads`.
The approach is strict TDD. We will write a failing test, then implement the code to pass it.

## Phase 1: Foundation & Configuration (Completed)
- [x] Create project structure
- [x] Implement basic config parsing (TDD)
- [x] Implement command-line argument parsing (clap)
    - [x] Test `pebble --version`
    - [x] Test `pebble config get sync.branch`
- [x] Confirm JSONL-only storage (no SQLite)

## Phase 1.5: Tooling & CI (Completed)
- [x] Restructure workspace (pebble + xtask)
- [x] Implement check-beads xtask command
- [x] Hook into `just check`

## Phase 2: Core Data Model & JSONL Store (Completed)
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

## Phase 5: Renaming & Cleanup (Eliminate legacy naming where appropriate) (Completed)
- [x] Rename `.beads` directory to `.pebble` (while maintaining fallback)
- [x] Rename `beads-sync` branch references to `pebble-sync`
- [x] Update `WorktreeManager` to use `.git/pebble-worktrees`
- [x] Update all code, comments, and tests to use `pebble` instead of `beads` where appropriate
- [x] Preserve `.forbidden-word-whitelist` and `xtask check-forbidden-words` as per project requirements

## Phase 6: Agent UX & Issue Lifecycle (MVP)
Sequencing: P6-1 → P6-2 → P6-3 → P6-4 → P6-5 → P6-6 → P6-7 → P6-8 → P6-9 → P6-10
- [ ] P6-1 Define CLI I/O contract: `stdout` data, `stderr` diagnostics, stable error codes, and exit code map (`0/1/2`).
- [ ] P6-2 Implement `--json` on **all** commands (add/edit/update/search/list/show/sync/init/import/config).
- [ ] P6-3 Add `--help-json` (or `pebble help --json`) with output schemas.
- [ ] P6-4 Update `--help` with concrete examples for core workflows.
- [ ] P6-5 Add list filters/sorting (`--status`, `--owner`, `--type`, `--priority`, `--updated`).
- [ ] P6-6 Add `pebble search` (full-text + filters: status, owner, type, priority).
- [ ] P6-7 Add `pebble update` for status/priority/owner/type/close fields.
- [ ] P6-8 Remove interactive prompts; require `--yes` / `--force` for destructive ops.
- [ ] P6-9 Disable color/formatting in structured mode; respect `NO_COLOR` and `isatty()`.
- [ ] P6-10 Tests: idempotency, exit codes, and stdout/stderr separation.

## Phase 7: Deterministic Merge & Storage Redesign (CRDT-Friendly)
- [ ] Decide storage layout (no backward-compat required)
    - [ ] Evaluate CRDT operation log vs. per-issue files vs. single JSONL snapshot
    - [ ] Consider Markdown + YAML frontmatter for human-readable per-issue storage
    - [ ] Pick one and document deterministic merge semantics
- [ ] Define schema v2 (supports children + ordered checklists)
- [ ] Implement conflict-free merge (no prompts)
    - [ ] LWW for scalar fields with per-field timestamps
    - [ ] OR-Set for sets (e.g., tags/children)
    - [ ] Ordered list CRDT for checklists (RGA/Logoot-style)
- [ ] Implement `pebble merge` command (git merge driver entrypoint)
- [ ] Update `pebble sync` to use deterministic merge (no interactive editor)
- [ ] TDD coverage for concurrent edits, adds, deletes, and checklist merges

## Phase 8: Auto-Sync + Clean Worktree Invariant
- [ ] Preflight: verify clean worktree or auto-commit before any write
- [ ] Auto-sync on command entry/exit with debounce state file
- [ ] Ensure **every** write path commits immediately to sync worktree
- [ ] Atomic file writes (temp + rename) to avoid partial JSONL writes
- [ ] File locking to prevent concurrent add/edit/update collisions
- [ ] Crash recovery: detect dirty worktree and auto-repair on next run

## Rules
1. **One Fail at a Time**: don't write multiple failing tests.
2. **Refactor**: Refactor after passing.
3. **No Daemon**: Ensure no daemon logic creeps in.
4. **Regular Gate**: Run `just check` and `just test` regularly to confirm the workspace stays green.
