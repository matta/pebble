---
id: pebl-gkz8szvfpu
title: P0.2 Deterministic default ordering for `list` and `next`
status: done
created_at: 2026-03-02T01:29:29.204427100+00:00
resolved_at: 2026-03-02T01:29:29.204427100+00:00
tags:
  - bootstrap
  - self_hosted
---
Checklist:
- [x] P0.2.a Topological order respecting `needs` (missing needs ignored; cycles grouped, ordered by `created_at` then `id`).
- [x] P0.2.b Then transitive blocking count DESC.
- [x] P0.2.c Then priority ASC (None last).
- [x] P0.2.d Then `created_at` ASC.
- [x] P0.2.e Then `id` ASC.
