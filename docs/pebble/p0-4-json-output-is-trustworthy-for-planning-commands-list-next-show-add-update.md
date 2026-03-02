---
id: pebl-aije2g1xkz
title: >-
  P0.4 JSON output is trustworthy for planning commands (`list`, `next`, `show`,
  `add`, `update`)
status: done
created_at: 2026-03-02T01:29:29.670837672+00:00
resolved_at: 2026-03-02T01:29:29.670837672+00:00
tags:
  - bootstrap
  - self_hosted
---

Checklist:
- [x] P0.4.a `--json` emits valid JSON to `stdout` and nothing else.
- [x] P0.4.b Errors and diagnostics go to `stderr` only; exit codes follow `0/1/2`.
- [x] P0.4.c Tests validating JSON purity and stdout/stderr separation for these commands.
