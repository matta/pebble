---
id: pebl-600Hmb
title: empty needs and tags should be absent from frontmatter
status: done
created_at: 2026-02-23T16:59:56.742176+00:00
modified_at: 2026-03-02T05:15:29.344458472+00:00
resolved_at: 2026-03-02T05:15:29.344446271+00:00
---
When pebble writes the TOML frontmatter, needs and tags lists should be elided when they're empty, and not present in the TOML output.
