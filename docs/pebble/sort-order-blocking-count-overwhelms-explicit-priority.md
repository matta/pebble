---
id: pebl-buDx2q
title: "Sort order: blocking count overwhelms explicit priority"
status: done
created_at: 2026-02-23T05:46:42.776110+00:00
modified_at: 2026-03-01T23:22:51.870096+00:00
resolved_at: 2026-03-01T23:22:51.870081+00:00
tags:
  - design
  - sort
---
## Problem

The current default sort order is:

1. Topological order (needs)
2. Transitive blocking count DESC
3. Priority ASC (None last)
4. `created_at` ASC
5. `id` ASC

Because blocking count (tier 2) is evaluated before explicit priority (tier 3), a
task with `priority = 0` (highest possible) will sort *below* a task that happens to
have downstream dependents — even if that downstream structure is entirely unrelated.

## Observed Scenario

- `pebl-rWaJHG` ("Rename deps to needs") has `priority = 0`, no needs, no dependents.
  It should intuitively be the first result from `pebble next`.
- `pebl-cdIZGN` ("Phase 1") has no priority set, but has blocking count = 2 because
  P2 and the root program task depend on it.
- `pebble next` returns P1, not the priority-0 task.

The operator explicitly set priority 0 to signal "do this first," but the sort order
ignores that signal in favor of structural depth.

## Why This Happens

The blocking count tiebreaker was designed to prevent starvation from priority
inversion (RFC 001, RFC 002): a low-priority prerequisite that blocks high-priority
work should be surfaced. This is correct in the priority-inversion case. But applying
it unconditionally as a tier above explicit priority means any task forest — even one
with no priority at all — artificially outranks individually prioritized tasks.

In effect, "has dependents" always beats "user said this is urgent." Deep forests win
by sheer structural mass.

## Proposed Direction (for implementation)

Use a simple effective-priority formulation:

- `base_priority(task)`: task `priority` if set, else sentinel `100` (None-last).
- `downstream_min_priority(task)`: minimum `base_priority` among actionable transitive
  downstream dependents (via reverse `needs` traversal), else `100`.
- `effective_priority(task) = min(base_priority(task), downstream_min_priority(task))`.

Interpretation: if a ready task blocks P0 work, it is treated as P0 for ranking.

### Ready-frontier sort (`pebble next`, `pebble list --is-ready`)

Sort key tuple:

1. `effective_priority` ASC
2. `base_priority` ASC
3. Transitive blocking count DESC
4. `created_at` ASC
5. `id` ASC

Rationale:

- Least surprise: explicit urgency remains authoritative.
- Good reason to surface low/no-priority work early: it unlocks more urgent work.
- Deterministic output remains unchanged through existing tie-breakers.

### Default `pebble list` sort

Keep topological tier first, then replace priority-related tiers:

1. Topological order respecting `needs` (existing cycle handling unchanged)
2. `effective_priority` ASC
3. `base_priority` ASC
4. Transitive blocking count DESC
5. `created_at` ASC
6. `id` ASC

## Implementation Checklist (TDD-first)

- [ ] Add failing unit tests in `crates/pebble/src/graph/tests.rs` for:
  - blocker promotion: unset/low-priority blocker of P0 dependent gains
    `effective_priority = 0`
  - no downstream urgency: isolated task keeps `effective_priority = base_priority`
  - tie behavior: when `effective_priority` ties, lower `base_priority` wins
  - deterministic fallback: `created_at` then `id`
- [ ] Add failing integration tests for CLI behavior:
  - `pebble next --json` returns a blocker of urgent downstream work ahead of unrelated
    non-urgent tasks
  - `pebble list --is-ready --json` order matches the new tuple
  - default `pebble list --json` remains dependency-valid while applying new ranking
    tiers within topological constraints
- [ ] Implement scoring helpers in graph layer:
  - `base_priority(task)`
  - `downstream_min_priority(task)` reusing cycle-safe traversal semantics
  - `effective_priority(task)`
- [ ] Update ranking keys used by:
  - `TaskGraph::get_next_tasks`
  - default ordering key construction in `graph/ordering.rs`
- [ ] Update normative docs:
  - `docs/graph-semantics.md`
  - `docs/cli-contract.md`
- [ ] Run verification gauntlet:
  - `just check`
  - `just test`

## Acceptance Criteria

- A ready task with explicit `priority: 0` is not outranked by unrelated structural
  forests.
- A ready task with low/no explicit priority can outrank another task only when it
  unlocks higher-urgency downstream work (captured by `effective_priority`).
- `pebble next` remains equivalent to `pebble list --is-ready --limit 1` under default
  sort semantics.
