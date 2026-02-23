+++
id = "pebl-buDx2q"
title = "Sort order: blocking count overwhelms explicit priority"
status = "todo"
created_at = 2026-02-23T05:46:42.77611+00:00
deps = []
tags = ["design", "sort"]
+++

## Problem

The current default sort order is:

1. Topological order (deps)
2. Transitive blocking count DESC
3. Priority ASC (None last)
4. `created_at` ASC
5. `id` ASC

Because blocking count (tier 2) is evaluated before explicit priority (tier 3), a
task with `priority = 0` (highest possible) will sort *below* a task that happens to
have downstream dependents — even if that downstream structure is entirely unrelated.

## Observed Scenario

- `pebl-rWaJHG` ("Rename deps to needs") has `priority = 0`, no deps, no dependents.
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

## Not Blocking

The current sort order is an MVP and is working well enough for self-hosted planning.
This issue captures the suboptimal behavior for future design work. Possible
approaches to explore later:

- Swap tiers 2 and 3 (priority before blocking count), using blocking count only to
  break ties within the same priority level.
- Use a composite score that blends priority and blocking count rather than strict
  lexicographic ordering.
- Only apply the blocking count boost when it actually prevents starvation (i.e., when
  a task's dependents have higher priority than the task itself).
- Weight blocking count logarithmically so deep forests don't get a linear advantage.
