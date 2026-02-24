+++
id = "pebl-exvts01y2i"
title = "Harden help-json mapping to avoid panic on new commands"
status = "canceled"
created_at = 2026-02-24T04:10:09.11822+00:00
modified_at = 2026-02-24T05:31:21.377763+00:00
resolved_at = 2026-02-24T05:31:21.377759+00:00
needs = []
tags = ["self_hosted", "review_followup"]
+++
Prevent runtime panic when command surface grows and mapping lags. Return a safe error or enforce completeness without panic.

**Resolution**: Canceled without changes. We rely on the existing unit test (`test_all_commands_have_help_json_output_schema`) to validate the exhaustiveness of the command mapping at compile/test time. The panic acts as intended to ensure CI fails if new commands are missing schema mappings.