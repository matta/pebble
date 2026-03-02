# Implementation Plan - Pebble (Spec Alignment)

This plan reorganizes work so Pebble can be trusted to manage a hierarchical, phase-based task set **as early as possible**. "Phase Zero" contains only the blockers to using Pebble itself for planning: representing nested tasks via `needs`, and trusting `pebble list` / `pebble next` to surface tasks in correct order.

TDD is mandatory: write a failing test before each behavior change.

## Governance & Process Rules
Pebble is self-hosting. All work execution must be tracked within the Pebble database itself.

### Source of Truth
1.  **Database Authority**: The Pebble database (found in `docs/pebble/`) is the canonical source of truth for task status, metadata, and dependencies.
2.  **Implementation Plan Role**: This file (`implementation_plan.md`) serves as the governance driver and process definition. It defines the "how" and "why," while the database tracks the "what" and "when."
3.  **No Duplicate Tracking**: Do not maintain task checklists or status tracking within this file. Refer to `cargo pebble list` or `cargo pebble next` for active work.
4.  **Task Decomposition**: Use checklist-first decomposition within a task's body and only promote checklist items into child Pebble tasks when complexity justifies it.

### MVP Completion
The initial MVP is defined by the terminal status of the following root task:
- **MVP Root**: `pebl-shaoq1hbwg` (Pebble MVP Completion)

Completion of the MVP requires that `pebl-shaoq1hbwg` and all of its transitive dependencies have reached a terminal status (`done` or `canceled`).

### Adaptive Task Decomposition
Use checklist-first decomposition within a task's body and only promote checklist items into child Pebble tasks when complexity justifies it.

1.  **Default behavior**: Keep sub-steps as markdown checklist items in the parent task body.
2.  **Split criteria**:
    -   `MUST`: the item has independent `needs` or blocks other work.
    -   `MUST`: the item requires independent status tracking (`todo`/`in_progress`/`done`) for planning value.
    -   `MUST`: the item is expected to span multiple sessions or PRs.
    -   `SHOULD`: the item touches multiple subsystems or high-risk surfaces.
    -   `SHOULD`: the item requires a design decision, spike, or uncertainty reduction step.
    -   `SHOULD`: estimated effort exceeds a single focused implementation session.
    -   Promote when any `MUST` is true, or when at least two `SHOULD` criteria are true.
3.  **Traceability**: When decomposition occurs, update the parent task's `needs` to include the new child tasks.

### Synchronization
1.  If scope changes, discuss the architectural direction first, then create/update Pebble tasks to match.
2.  Process policies (TDD, gauntlets, gate checks) are enforced by Rules and AGENTS instructions, not by `needs`.

## Rules
1.  **One failing test at a time.**
2.  **Refactor only after green.**
3.  **No preemptive cycle prevention in the write path.**
4.  **Run `just check` and `just test` regularly.**
5.  **Always use `cargo pebble` to query or update task state.**
