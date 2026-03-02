---
id: pebl-4ssy3fsyds
title: "Feature request: add reverse dependency link at add time"
status: done
created_at: 2026-02-24T04:10:09.372775+00:00
modified_at: 2026-02-24T16:25:55.086038+00:00
resolved_at: 2026-02-24T16:25:55.086011+00:00
tags:
  - self_hosted
  - review_followup
---
Support add-time reverse linking via `--blocks`. Command should update needs on referenced task IDs as appropriate while preserving one true edge semantics.

Implemented: added repeatable `--blocks` to `pebble add`; target task IDs receive the new task ID in `needs` without duplicates; missing/duplicate target IDs fail fast. Added CLI tests for success and missing target failure.
