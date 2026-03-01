+++
id = "pebl-ombr9kv475"
title = "RFC005-5 Write-path tests first (YAML emit)"
status = "done"
priority = 0
created_at = 2026-03-01T16:41:38.740487853+00:00
modified_at = 2026-03-01T17:14:15.926528050+00:00
resolved_at = 2026-03-01T17:14:15.926528050+00:00
needs = ["pebl-gr537c9can"]
tags = ["planning", "rfc005"]
+++
Execution Note (Manual Tracking During YAML Migration):
- `cargo pebble` task reads are temporarily non-functional until RFC005-8 converts all `docs/pebble/*.md` files to YAML frontmatter.
- During this window, mark task state changes by directly editing frontmatter fields in task files (`status`, `modified_at`, `resolved_at`) instead of using `cargo pebble update`.

Goal:
Add failing tests for write commands to define YAML output before implementation changes.

Do exactly this:
1. Update/add tests for `add`, `update`, and any writer paths that emit task files.
2. Add failing assertions for:
   - frontmatter delimiter is `---`
   - frontmatter syntax is YAML, not TOML
   - expected optional fields remain omitted when empty (same semantics as before unless RFC005 says otherwise)
3. Keep this task test-only.
4. Record failing test names in this task body.

Acceptance Criteria:
- New/updated tests fail before write-path code changes.
- No production writer logic is changed in this task.

Failing tests recorded before implementation:
- `commands_test::test_add_writes_yaml_frontmatter`
- `commands_test::test_update_writes_yaml_frontmatter`
- `models::tests::test_task_node_disk_content_uses_yaml_frontmatter`
