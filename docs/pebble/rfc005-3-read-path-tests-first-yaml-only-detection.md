---
id: pebl-rm0kn1fvli
title: RFC005-3 Read-path tests first (YAML-only detection)
status: done
priority: 0
created_at: 2026-03-01T16:41:06.656429171+00:00
modified_at: 2026-03-01T16:55:54.908955326+00:00
resolved_at: 2026-03-01T16:55:54.908944206+00:00
needs:
  - pebl-zjpn6fbfmp
tags:
  - planning
  - rfc005
---

Goal:
Write failing tests that define YAML-only frontmatter detection semantics before parser implementation changes.

Do exactly this:
1. Update/add parser/read tests to use YAML delimiters `---`.
2. Add failing tests for:
   - valid YAML frontmatter loads successfully
   - missing YAML frontmatter is treated as missing metadata and file is skipped/warned as current behavior dictates
   - legacy TOML frontmatter (`+++`) is not specially rejected; it is treated the same as missing YAML frontmatter
3. Record exact failing test names in this task body.
4. Do not change production code in this task.

Acceptance Criteria:
- Tests fail before implementation.
- No explicit TOML-rejection behavior is introduced in tests or expectations.

Failing Tests (recorded before implementation):
- `parser::tests::test_parse_valid_yaml_task`
- `parser::tests::test_parse_missing_frontmatter`
- `parser::tests::test_parse_unclosed_yaml_frontmatter`
- `parser::tests::test_parse_invalid_yaml_frontmatter`
- `parser::tests::test_parse_legacy_toml_frontmatter_treated_as_missing_yaml`
- `graph::tests::test_load_from_dir_prefers_yaml_frontmatter_and_ignores_non_yaml`
