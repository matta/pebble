---
id: issue-dLmIez
title: default AGENTS.md is weak
status: done
created_at: 2026-02-22T20:19:57.938979+00:00
modified_at: 2026-03-01T22:14:55.221356+00:00
resolved_at: 2026-03-01T22:14:55.221355+00:00
---
The default AGENTS.md installed by pebble init is minimal. It should contain suitable instructions for agents to use the pebble tool for task tracking

Implemented in code:
- Added stronger init AGENTS.md expectations in cli_init integration test (TDD-first).
- Expanded run_init AGENTS.md template with --json guidance, next-task workflow, and configured tasks-dir path.

Verification:
- just check
- just test
