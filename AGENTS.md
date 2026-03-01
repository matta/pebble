# Project: Pebble

This project is a CLI task tracker written in Rust, built upon a Markdown-native graph design.

## Pebble is self hosting

Read @.pebble/AGENTS.md to understand tool usage.

## Project Goals & Immutable Invariants

1. **Markdown-Native Storage**: 
    - Task data is stored entirely in Markdown files with TOML frontmatter.
    - There is no JSONL database, no SQLite database, and no hidden Git worktrees.
    - The files themselves act as the directed graph.

2. **Strict Project Invariants**:
    - **The `id` field is immutable**: The CLI must never change an ID after generation.
    - **No Preemptive Cycle Prevention**: Do **NOT** write cycle-prevention or deadlock-handling logic in the storage/write layer. The read layer naturally handles cycles by refusing to consider a cyclical task as "ready".
    - **One True Edge (`needs`)**: Hierarchy and temporal ordering are entirely collapsed into a single edge: `needs`. Do not re-introduce `parent`/`child` relationships.
    - **Absolute Readiness**: A task is ready if and only if all of its `needs` exist and have a terminal status. Dangling pointers structurally block readiness forever without panicking.

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
- [Historical RFCs](docs/rfcs/) - Frozen design proposals preserved as historical records. These are expected to diverge from current code and normative specs. Never use for active implementation details or to flag code inconsistencies.

## Constraints

- Strictly adhere to TDD.
- **Style Guide**: Adhere to the [.gemini/styleguide.md](.gemini/styleguide.md).
- **Docs Discoverability**: Any new documentation added to the repository must be linked from this file. Exception: files under `docs/pebble/` are Pebble task files managed by the tool and do not require individual index entries.
- **Clippy Changes Require Approval**: Any modifications to clippy configuration or suppression of clippy warnings must be explicitly discussed and approved by the operator before applying.
- **Rust Language Baseline**: Before flagging Rust syntax compatibility concerns, check `edition` and `rust-version` in `Cargo.toml`. This repository treats Rust 2024 syntax as canonical.

## Workflows

### Self-Hosted Planning Protocol
Pebble task tracking is the execution mechanism for Pebble's own development, with `implementation_plan.md` as the governance driver.

1. **Driver authority**:
    - `implementation_plan.md` is canonical for phase structure, process Rules, and completion state.
2. **Task execution**:
    - Default graph shape is root task + phase tasks.
    - Implement phase details as markdown checklists in phase task bodies.
    - Promote checklist items to child Pebble tasks only when Adaptive Task Decomposition criteria are met.
    - Use `needs` to model sequencing between phases and any promoted child tasks.
3. **Sync discipline**:
    - Update `implementation_plan.md` checkboxes whenever task state changes (`[ ]` -> `[-]` -> `[x]`).
    - Keep the plan's "Task ID Index" aligned with actual Pebble IDs (root/phase always; child IDs only when promoted).
4. **Policy vs graph**:
    - Process requirements (TDD, gauntlets, push gates) are policy gates and must be followed even when not represented as `needs`.

### Temporary RFC005 Migration Override (Highest Priority)
This override is active during the YAML frontmatter migration window.

1. **Why override exists**:
    - Read-path behavior is now YAML-only while many repository task files are still TOML frontmatter.
    - As a result, `cargo pebble next` and the `next-pebble` skill are not authoritative for prioritization until migration conversion is complete.
2. **What is highest priority**:
    - Treat the RFC005 chain as the most important work in the repository.
    - Source of truth: `docs/pebble/plan-rfc-005-yaml-frontmatter-migration.md`.
    - Execute remaining RFC005 tasks strictly in listed order unless the operator explicitly reprioritizes.
3. **How to pick the next task while override is active**:
    - Read RFC005 task files directly in `docs/pebble/`.
    - Determine readiness from frontmatter `status` + `needs` manually.
    - Do not rely on `cargo pebble next` output during this window.
4. **How to mark progress while override is active**:
    - Update task state by editing task-file frontmatter directly (`status`, `modified_at`, `resolved_at`).
    - Keep `implementation_plan.md` and RFC005 parent-task checklist synced to match manual state changes.
5. **When override ends**:
    - End this override immediately after RFC005-8 completes and all `docs/pebble/*.md` files are converted to YAML frontmatter.
    - After that point, resume normal `cargo pebble` and `next-pebble` driven task selection.

### Adaptive Task Decomposition Policy
Agents must avoid unnecessary task explosion while still exposing meaningful graph structure.

1. **Default**:
    - Keep sub-steps as markdown checklist items in the parent task body.
2. **Promote checklist item to child Pebble task when**:
    - `MUST`: it has independent `needs` or blocks other work.
    - `MUST`: it requires independent status tracking for planning value.
    - `MUST`: it likely spans multiple sessions or PRs.
    - `SHOULD`: it touches multiple subsystems or high-risk surfaces.
    - `SHOULD`: it requires design/spike/uncertainty reduction.
    - `SHOULD`: it exceeds one focused implementation session.
    - Rule: promote on any `MUST`, or at least two `SHOULD` conditions.
3. **Do not split by default**:
    - Do not create Pebble tasks merely to mirror every checklist line.
    - Never auto-expand a full phase checklist into one-task-per-checkmark.
4. **Recursive decomposition**:
    - Re-assess remaining checklist items after each child completion.
    - Further split only where criteria are still met.
5. **Parent traceability**:
    - When decomposition occurs, keep a `Child Tasks` mapping in the parent task body.

### Just Gauntlet
This workflow prepares the codebase for a push:
1. `just fix` -- optional, fixes formatting and some clippy warnings.
2. `just check` -- required, ensures code is free of errors.
3. `just test` -- required, ensures tests pass.

### Push Gauntlet
This workflow requires the following steps to be done in order:
1. Run the just gauntlet.
2. `git commit`.
3. `git push`.

## Gates

### Push Gate
- **Requirement**: `just check` must pass cleanly.
- **Enforcement**: Agents must run `just check` and ensure it exits with code 0 before pushing any code to the repository.

### Process Gate Enforcement
Before marking a Pebble task complete for implementation work, agents must verify:
1. A failing test was written first (TDD) when behavior changed.
2. `just check` and `just test` pass locally.
3. `implementation_plan.md` status and task linkage are updated.

## Documentation Index
- .agents/README.md
- .agents/checks/gemini-styleguide.md
- .agents/checks/rust-language-baseline.md
- .agents/checks/rust-api-docs.md
- .agents/checks/specifications.md
- .agents/checks/docs-discoverability.md
- .agents/checks/rust-module-structure.md
- .agents/checks/warning-suppression-review.md
- .agents/checks/test-module-placement.md
- docs/rust-api-docs.md
- implementation_plan.md
- docs/pebble/ (Pebble task files; exempt from individual index entries)
