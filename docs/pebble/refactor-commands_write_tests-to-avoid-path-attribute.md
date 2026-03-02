---
id: pebl-p3k8qhfwqu
title: "Refactor commands_write_tests to avoid #[path] attribute"
status: done
created_at: 2026-02-24T04:22:23.907421+00:00
modified_at: 2026-02-25T03:09:04.247867+00:00
resolved_at: 2026-02-25T03:09:04.247859+00:00
tags:
  - review_followup
  - self_hosted
---

The #[path] attribute on commands_write_tests module is non-idiomatic. Refactor to use either the modern module structure (commands_write/ directory) or keep tests inline. The current approach was a pragmatic workaround for xtask token limits.
