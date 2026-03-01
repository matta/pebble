+++
id = "pebl-zjpn6fbfmp"
title = "RFC005-2 Spec-to-code inventory"
status = "todo"
priority = 0
created_at = 2026-03-01T16:39:17.452093522+00:00
needs = ["pebl-urd2fpbmfk"]
tags = ["planning", "rfc005"]
+++
Goal:
Create an explicit file-level inventory so implementation work is mechanical.

Do exactly this:
1. Read `docs/rfcs/005-yaml-frontmatter.md`.
2. Create a checklist in this task body with exact files to touch, grouped by category:
   - Parser/read path files
   - Writer/mutation path files
   - Tests/fixtures
   - Docs (`docs/schema.md`, `docs/cli-contract.md`, root `AGENTS.md`)
3. For each file, add one sentence describing the required YAML change.
4. Mark each line `[ ]` (unchecked); do not implement anything yet.

Acceptance Criteria:
- This task body contains a complete, file-by-file action list.
- Another agent can implement from the list without deciding where to edit.
