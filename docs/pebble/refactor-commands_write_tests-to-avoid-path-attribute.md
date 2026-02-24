+++
id = "pebl-p3k8qhfwqu"
title = "Refactor commands_write_tests to avoid #[path] attribute"
status = "todo"
created_at = 2026-02-24T04:22:23.907421+00:00
modified_at = 2026-02-24T04:22:44.43626+00:00
needs = []
tags = ["review_followup", "self_hosted"]
+++
The #[path] attribute on commands_write_tests module is non-idiomatic. Refactor to use either the modern module structure (commands_write/ directory) or keep tests inline. The current approach was a pragmatic workaround for xtask token limits.