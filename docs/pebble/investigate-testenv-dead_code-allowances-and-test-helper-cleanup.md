---
id: "pebl-Vs0xNh"
title: "Investigate TestEnv dead_code allowances and test helper cleanup"
status: "todo"
created_at: "2026-02-23T05:22:43.013058+00:00"
needs: []
tags: ["bug", "cleanup"]
---
Observed required dead_code allowances in integration test support:
- crates/pebble/tests/support.rs:9 on TestEnv::dir
- crates/pebble/tests/support.rs:62 on write_task_with_id

Investigate broader TestEnv/test helper API shape and remove or justify allowances by redesigning shared helpers and call sites.

Acceptance:
- Explain root cause for both allowances.
- Refactor helpers and tests so dead_code allowances are not required, or document a justified exception with rationale.
