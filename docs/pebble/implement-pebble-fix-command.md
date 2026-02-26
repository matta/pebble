+++
id = "pebl-vnywlhmt3y"
title = "Implement pebble fix command"
status = "done"
created_at = 2026-02-26T04:59:06.729312+00:00
modified_at = 2026-02-26T05:15:33.736118+00:00
resolved_at = 2026-02-26T05:15:33.736115+00:00
needs = ["pebl--yb8d4"]
tags = ["feature"]
+++
P3.3 Implement the 'pebble fix' command to apply safe, deterministic repairs to task files.\n\nRequirements:\n- Backfill missing 'created_at' with current UTC time.\n- Warn on unknown frontmatter keys but do not remove or rewrite them.\n- Do not remove or rewrite dependency edges (needs).\n- Support --json output.\n- Follow TDD: write failing tests first.
