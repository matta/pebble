---
id: pebl-4ype952l5d
title: P0.1 Graph semantics for readiness and blocking
status: done
created_at: 2026-03-02T01:29:28.972049992+00:00
resolved_at: 2026-03-02T01:29:28.972049992+00:00
tags:
  - bootstrap
  - self_hosted
---
Checklist:
- [x] P0.1.a `blocking` includes only **non-terminal** direct dependents.
- [x] P0.1.b Transitive blocking count excludes terminal tasks, excludes self, ignores missing IDs, and is cycle-safe.
