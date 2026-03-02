---
id: pebl-itm1n1sj4n
title: Config get unknown key should be usage error (exit 2)
status: done
created_at: 2026-02-24T04:10:07.581571+00:00
tags:
  - self_hosted
  - review_followup
---

Align runtime behavior with CLI/help contract. Unknown config key should return exit code 2 with stderr diagnostic. Add regression test coverage for status code and stream purity.
