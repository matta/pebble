# Implementation Plan - Pebble

This document outlines the step-by-step plan to implement `pebble`, a streamlined CLI task tracker.
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
- [x] Implement `pebble update` (writes to worktree)

## Phase 4.5: Extended Issue Fields (In Progress)
- [x] Consolidate mutations into `pebble update` (remove redundant edit command).
- [x] Golden fixture: move to `crates/pebble/tests/fixtures/golden.jsonl` and add round-trip test.
- [x] Forbidden-words xtask: ignore the golden fixture.
- [x] Schema cleanup: remove `comments` and convert `notes` to a list of plain strings.
- [x] Update help JSON schema for the `notes` list and `comments` removal.
- [x] Codify list/set update semantics in `docs/cli-contract.md` (repeatable add/remove flags, explicit set).
- [ ] Finalize remaining field decisions (dependency relation semantics, issue_type update).
- [x] Ratified: `owner` settable via `add` and `update`.
- [x] Ratified: `status` defaults to `open` on `add`; changes only via `update`.
- [x] Ratified: `priority` settable via `add` and `update`.
- [x] Ratified: `issue_type` defaults to `task`; `add` can override (update behavior TBD).
- [x] Ratified: `close_reason` set via `update` when closing; `closed_at` auto-set (read-only).
- [x] Ratified: `acceptance_criteria` supported via `add` and `update`.
- [x] Ratified: `labels` supported via incremental flags.
- [x] Ratified: `notes` is a list of plain strings.
- [x] Ratified: `defer_until` supported via `add` and `update`.
- [x] Ratified: `deleted_*` fields are read-only/internal.
- [ ] Pending: dependency relation semantics and CLI shape.
- [ ] Pending: confirm whether `issue_type` can be updated.
- [ ] Add `pebble add` support for the chosen editable fields (flags + parsing + validation).
- [ ] Add `pebble update` support for the chosen editable fields (flags + parsing + validation).
- [ ] Implement incremental list/set flags in `update` for every list/set field (consistent `--add-*` / `--remove-*`).
- [ ] Add tests that assert:
    - [ ] `pebble add` can set allowed fields.
    - [ ] `pebble add` rejects attempts to set read-only fields with a usage error (exit code 2).
    - [ ] `pebble update` can modify allowed fields.
    - [ ] `pebble update` rejects attempts to modify read-only fields with a usage error (exit code 2).
- [ ] Validate `pebble show` outputs new fields properly:
    - [ ] JSON output includes all stored fields.
    - [ ] Human output includes any new fields we decide to surface (and omits empty ones).
- [ ] Validate `pebble list` supports the new fields properly:
    - [ ] JSON output includes all stored fields.
    - [ ] Human output remains stable (or add a new “long”/expanded view if needed).
- [ ] Update CLI help/examples to reflect the supported new fields.

## Phase 5: Renaming & Cleanup (Eliminate legacy naming where appropriate) (Completed)
- [x] Rename `.beads` directory to `.pebble` (while maintaining fallback)
- [x] Rename `beads-sync` branch references to `pebble-sync`
- [x] Update `WorktreeManager` to use `.git/pebble-worktrees`
- [x] Update all code, comments, and tests to use `pebble` instead of `beads` where appropriate
- [x] Preserve `.forbidden-word-whitelist` and `xtask check-forbidden-words` as per project requirements

## Phase 6: Agent UX & Issue Lifecycle (MVP)
Sequencing: P6-1 → P6-2 → P6-3 → P6-4 → P6-5 → P6-6 → P6-7 → P6-8 → P6-9 → P6-10
- [x] P6-1 Define CLI I/O contract: `stdout` data, `stderr` diagnostics, stable error codes, and exit code map (`0/1/2`).
- [x] P6-2 Implement `--json` on **all** commands (add/update/search/list/show/sync/init/import/config).
- [x] P6-3 Add `--help-json` (or `pebble help --json`) with output schemas.
- [x] P6-4 Update `--help` with concrete examples for core workflows.
- [ ] P6-5 Add list filters/sorting (`--status`, `--owner`, `--type`, `--priority`, `--updated`).
  - [x] Filters: `--status`, `--owner`, `--priority`.
  - [x] Filters: `--type`.
  - [ ] Filters: `--updated`.
  - [ ] Sorting.
- [ ] P6-6 Add `pebble search` (full-text + filters: status, owner, type, priority).
  - [x] Full-text search on title/description.
  - [x] Filters: status, owner, type, priority.
- [ ] P6-7 Add `pebble update` for status/priority/owner/type/close fields.
  - [x] Update: status/priority/owner/type.
  - [x] Close behavior: `close_reason` validation + `closed_at` auto-set.
- [ ] P6-8 Remove interactive prompts; require `--yes` / `--force` for destructive ops.
- [ ] P6-9 Disable color/formatting in structured mode; respect `NO_COLOR` and `isatty()`.
- [ ] P6-10 Tests: idempotency, exit codes, and stdout/stderr separation.

## Phase 7: Deterministic Merge & Storage Redesign (CRDT-Friendly)
- [ ] Decide storage layout (no backward-compat required)
    - [ ] Evaluate CRDT operation log vs. per-issue files vs. single JSONL snapshot
    - [ ] Consider Markdown + TOML frontmatter for human-readable per-issue storage
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
- [ ] File locking to prevent concurrent add/update collisions
- [ ] Crash recovery: detect dirty worktree and auto-repair on next run

## Rules
1. **One Fail at a Time**: don't write multiple failing tests.
2. **Refactor**: Refactor after passing.
3. **No Daemon**: Ensure no daemon logic creeps in.
4. **Regular Gate**: Run `just check` and `just test` regularly to confirm the workspace stays green.
