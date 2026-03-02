---
id: pebl-z5v17c3486
title: "P5.2 Graph Behavior: Cycles"
status: done
created_at: 2026-03-02T01:29:35.690681306+00:00
resolved_at: 2026-03-02T01:29:35.690681306+00:00
tags:
  - bootstrap
  - self_hosted
---

Checklist:
- [x] P5.2.a `list`: tasks in cycles are never ready.
- [x] P5.2.b `list`: default topological sort groups cycles.
- [x] P5.2.c `next`: tasks in cycles are never ready.
- [x] P5.2.d `show`: `blocking` and `transitive_blocking_count` are cycle-safe.
- [x] P5.2.e `check --warn-only`: reports dependency cycles.
- [x] P5.2.f `check`: reports dependency cycles and exits non-zero.
