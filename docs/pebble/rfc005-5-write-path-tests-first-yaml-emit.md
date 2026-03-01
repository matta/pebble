+++
id = "pebl-ombr9kv475"
title = "RFC005-5 Write-path tests first (YAML emit)"
status = "todo"
priority = 0
created_at = 2026-03-01T16:41:38.740487853+00:00
needs = ["pebl-gr537c9can"]
tags = ["planning", "rfc005"]
+++
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
