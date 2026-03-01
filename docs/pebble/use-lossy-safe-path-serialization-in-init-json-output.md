---
id: "pebl-czi45zargx"
title: "Use lossy-safe path serialization in init JSON output"
status: "done"
created_at: "2026-02-24T04:22:35.759998+00:00"
modified_at: "2026-02-25T03:36:00.667911+00:00"
resolved_at: "2026-02-25T03:36:00.667902+00:00"
needs: []
tags: ["review_followup", "self_hosted"]
---
init --json uses display() for path serialization, which is lossy on non-UTF-8 systems. Consider to_string_lossy() or erroring on non-UTF-8 paths for strict JSON correctness.