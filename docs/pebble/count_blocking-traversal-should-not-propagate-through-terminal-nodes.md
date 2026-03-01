---
id: pebl-uekcwU
title: count_blocking traversal should not propagate through terminal nodes
status: done
created_at: 2026-02-22T23:50:49.737541+00:00
modified_at: 2026-03-01T22:49:33.891651+00:00
resolved_at: 2026-03-01T22:49:33.891637+00:00
tags:
  - bug
---
In graph.rs, count_blocking() continues DFS traversal through closed (Done/Canceled) nodes. This inflates blocking counts incorrectly.

Example: A(Todo) → B(Done) → C(Todo). A does not actually block C because B is already Done and breaks the dependency chain. But the current code pushes B onto the stack regardless of status, eventually visiting and counting C.

Fix: move stack.push inside the is_actionable() guard so traversal stops at terminal nodes.
