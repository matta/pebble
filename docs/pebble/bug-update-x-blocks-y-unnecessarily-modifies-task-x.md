---
id: pebl-40zkfmonlr
title: "Bug: `update X --blocks Y` unnecessarily modifies task X"
status: done
created_at: 2026-02-25T01:58:34.344913+00:00
modified_at: 2026-03-01T22:35:18.230983+00:00
resolved_at: 2026-03-01T22:35:18.230971+00:00
tags:
  - bug
---

When running `pebble update X --blocks Y`, task Y is correctly updated to "need" X. However, task X is also written back to disk with an updated "modified_at" timestamp, even if no fields on X were actually changed. Pebble should only write tasks and update "modified_at" if the command actually modifies that specific task's content or frontmatter.
