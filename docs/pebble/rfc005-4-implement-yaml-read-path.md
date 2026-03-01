---
id: "pebl-gr537c9can"
title: "RFC005-4 Implement YAML read path"
status: "done"
priority: 0
created_at: "2026-03-01T16:41:29.176343279+00:00"
modified_at: "2026-03-01T17:07:37.368467321+00:00"
resolved_at: "2026-03-01T17:07:37.368467321+00:00"
needs: ["pebl-rm0kn1fvli"]
tags: ["planning", "rfc005"]
---
Execution Note (Manual Tracking During YAML Migration):
- `cargo pebble` read operations are currently non-functional against the repository task set because the read path is now YAML-only while most `docs/pebble/*.md` files are still TOML frontmatter.
- Until RFC005-8 completes full task-file conversion, update task completion state by directly editing task frontmatter (`status`, `modified_at`, `resolved_at`) in the markdown files.
- Do not treat legacy TOML frontmatter as an explicit parser error; it remains equivalent to missing YAML frontmatter.

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

Implementation Evidence:
- Added `serde-saphyr` dependency and wired YAML deserialization in the read path.
- Replaced parser frontmatter delimiter handling from `+++` to `---`.
- Updated graph directory loading to parse only YAML-frontmatter task files and skip non-YAML files.
- Confirmed RFC005-3 test set passes:
  - `cargo test -p pebble parser::tests:: -- --nocapture`
  - `cargo test -p pebble graph::tests::test_load_from_dir_prefers_yaml_frontmatter_and_ignores_non_yaml -- --nocapture`
