---
id: pebl-ouajn82jhl
title: Resolve init output contract inconsistency in cli-contract
status: done
created_at: 2026-02-24T04:10:08.613662+00:00
modified_at: 2026-02-24T05:11:43.150123+00:00
resolved_at: 2026-02-24T05:11:43.150119+00:00
tags:
  - self_hosted
  - review_followup
---
cli-contract currently states both no init output and structured JSON output. Make the normative contract internally consistent for human and --json modes.
