+++
id = "pebl-gr537c9can"
title = "RFC005-4 Implement YAML read path"
status = "todo"
priority = 0
created_at = 2026-03-01T16:41:29.176343279+00:00
needs = ["pebl-rm0kn1fvli"]
tags = ["planning", "rfc005"]
+++
Goal:
Implement YAML frontmatter parsing per RFC005 so tests from RFC005-3 pass.

Do exactly this:
1. Add `serde-saphyr` dependency and required wiring for task frontmatter deserialization.
2. Replace TOML frontmatter extraction logic with YAML delimiter scanning (`---` ... `---`).
3. Deserialize frontmatter into existing task structs with YAML input.
4. Preserve behavior for files without YAML frontmatter: treated as missing metadata and skipped/warned by existing read-path behavior.
5. Keep implementation focused only on read/parse path.

Acceptance Criteria:
- RFC005-3 tests pass.
- No code path adds explicit TOML-frontmatter rejection logic.
