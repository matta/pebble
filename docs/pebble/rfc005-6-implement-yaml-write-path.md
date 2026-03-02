---
id: pebl-jfi8jsyoai
title: RFC005-6 Implement YAML write path
status: done
priority: 0
created_at: 2026-03-01T16:41:50.674510836+00:00
modified_at: 2026-03-01T17:16:07.128622177+00:00
resolved_at: 2026-03-01T17:16:07.128622177+00:00
needs:
  - pebl-ombr9kv475
tags:
  - planning
  - rfc005
---

Execution Note (Manual Tracking During YAML Migration):
- `cargo pebble` task reads are temporarily non-functional until RFC005-8 converts all `docs/pebble/*.md` files to YAML frontmatter.
- During this window, mark task state changes by directly editing frontmatter fields in task files (`status`, `modified_at`, `resolved_at`) instead of using `cargo pebble update`.

Goal:
Switch task-file writing from TOML frontmatter to YAML frontmatter so RFC005-5 tests pass.

Do exactly this:
1. Update serialization/writer code used by `add`, `update`, and related mutation flows to emit YAML frontmatter between `---` delimiters.
2. Ensure generated YAML maps to the same task schema fields and ordering expectations used by tests.
3. Preserve invariant behavior (`id` immutable, `needs` semantics unchanged, readiness unaffected).
4. Keep scope limited to writing/emission behavior.

Acceptance Criteria:
- RFC005-5 tests pass.
- Newly written task files use YAML frontmatter exclusively.

Implementation Evidence:
- Updated `TaskNode` disk serialization to emit YAML frontmatter between `---` delimiters.
- Added frontmatter conversion for preserved unknown keys (`extra`) so write-backs remain lossless for supported scalar/array/object values.
- Verified targeted tests pass:
  - `cargo test -p pebble 'models::tests::test_task_node_disk_content_uses_yaml_frontmatter' -- --exact`
  - `cargo test -p pebble 'commands_test::test_add_writes_yaml_frontmatter' -- --exact`
  - `cargo test -p pebble 'commands_test::test_update_writes_yaml_frontmatter' -- --exact`
