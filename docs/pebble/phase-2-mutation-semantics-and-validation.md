+++
id = "pebl--yb8d4"
title = "Phase 2 Mutation Semantics and Validation"
status = "todo"
created_at = 2026-02-23T01:36:06.025417+00:00
modified_at = 2026-02-24T05:22:57.333688+00:00
needs = ["pebl-cdIZGN", "pebl-itm1n1sj4n", "pebl-efz69mryyt", "pebl-5wuwlwxldk", "pebl-wy9nkoudjt", "pebl-ouajn82jhl", "pebl-8kd42jnhv7", "pebl-exvts01y2i", "pebl-4ssy3fsyds", "pebl-p3k8qhfwqu", "pebl-uj0hll5buc", "pebl-czi45zargx", "pebl-6pv06nwvpl", "pebl-9fvf6xfco3", "pebl-b4ei14wcbe"]
tags = ["bootstrap", "self_hosted"]
+++
Checklist:
- [x] P2.1 ID generation `<issue-prefix>-<suffix>` with alphabet `a-z0-9`, sizing for collision probability < `1e-12`
- [ ] P2.2 Priority validation enforces `0..99` in `add` and `update`
- [ ] P2.3 Status transitions
- [ ] P2.3.a `resolved_at` auto-set when moving to `done`/`canceled`
- [ ] P2.3.b `resolved_at` cleared when leaving terminal states
- [ ] P2.3.c `modified_at` always set on `update`
- [ ] P2.4 `add` output prints relative path from current working directory
- [ ] P2.5 New task files end with a trailing newline
- [ ] P2.6 `archive` behavior per contract (threshold, collision suffixes, JSON path semantics)

Child Tasks:
- None currently. Promote only when Adaptive Decomposition criteria are met.
