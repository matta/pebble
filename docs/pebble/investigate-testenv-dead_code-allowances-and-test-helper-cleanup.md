---
id: pebl-Vs0xNh
title: Investigate TestEnv dead_code allowances and test helper cleanup
status: done
created_at: 2026-02-23T05:22:43.013058+00:00
modified_at: 2026-03-01T22:52:06.312034+00:00
resolved_at: 2026-03-01T22:52:06.312010+00:00
tags:
  - bug
  - cleanup
---
Observed required dead_code allowances in integration test support:
- crates/pebble/tests/support.rs:9 on TestEnv::dir
- crates/pebble/tests/support.rs:62 on write_task_with_id

Investigate broader TestEnv/test helper API shape and remove or justify allowances by redesigning shared helpers and call sites.

Acceptance:
- Explain root cause for both allowances.
- Refactor helpers and tests so dead_code allowances are not required, or document a justified exception with rationale.



Resolution (2026-03-01):
- Root cause for `TestEnv::dir`: each integration test crate compiles its own `support` module, and some crates never touch the TempDir keeper field directly, so dead_code was raised.
- Root cause for `write_task_with_id`: same per-crate compilation effect; helper is used in some integration tests but not all, so dead_code appeared in crates that did not call it.
- Fix: removed the targeted dead_code allowances; renamed the keeper field to `_dir`; switched test crates from `mod support;` to `pub mod support;` so shared helpers are externally reachable and no per-item dead_code suppression is required.
- Also normalized `cli_errors` to use `env.pebble()` where applicable so helper usage is consistent.
