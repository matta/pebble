---
id: pebl-n6xopu3hq3
title: "P5.4 Graph Behavior: Missing IDs (Dangling Pointers)"
status: done
created_at: 2026-03-02T01:29:36.133993048+00:00
resolved_at: 2026-03-02T01:29:36.133993048+00:00
tags:
  - bootstrap
  - self_hosted
---

Checklist:
- [x] P5.4.a `list`: tasks with missing `needs` are never ready.
- [x] P5.4.b `next`: tasks with missing `needs` are never ready.
- [x] P5.4.c `show`: `blocked_by` includes missing IDs.
- [x] P5.4.d `add`: `--blocks` fails if target ID is missing.
- [x] P5.4.e `update`: `--blocks` fails if target ID is missing.
- [x] P5.4.f `check --warn-only`: reports dangling `needs`.
- [x] P5.4.g `check`: reports dangling `needs` and exits non-zero.
