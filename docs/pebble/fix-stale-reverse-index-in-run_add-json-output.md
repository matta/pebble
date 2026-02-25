+++
id = "pebl-uj0hll5buc"
title = "Fix stale reverse index in run_add JSON output"
status = "done"
created_at = 2026-02-24T04:22:30.29217+00:00
modified_at = 2026-02-25T03:25:19.371601+00:00
resolved_at = 2026-02-25T03:25:19.371595+00:00
needs = []
tags = ["review_followup", "self_hosted"]
+++
run_add inserts the new node into graph.nodes but does not rebuild the blocking reverse index. TaskObject::from_node uses the stale index, so blocking/blocked_by may be incorrect in the JSON output for pebble add --json. Either rebuild the index after insert or reload from disk.