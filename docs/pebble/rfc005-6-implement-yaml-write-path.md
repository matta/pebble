+++
id = "pebl-jfi8jsyoai"
title = "RFC005-6 Implement YAML write path"
status = "todo"
priority = 0
created_at = 2026-03-01T16:41:50.674510836+00:00
needs = ["pebl-ombr9kv475"]
tags = ["planning", "rfc005"]
+++
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
