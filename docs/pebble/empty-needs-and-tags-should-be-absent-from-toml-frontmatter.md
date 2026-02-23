+++
id = "pebl-600Hmb"
title = "empty needs and tags should be absent from TOML frontmatter"
status = "todo"
created_at = 2026-02-23T16:59:56.742176+00:00
needs = []
tags = []
+++
When pebble writes the TOML frontmatter, needs and tags lists should be elided when they're empty, and not present in the TOML output.
