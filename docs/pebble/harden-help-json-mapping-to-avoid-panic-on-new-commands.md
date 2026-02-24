+++
id = "pebl-exvts01y2i"
title = "Harden help-json mapping to avoid panic on new commands"
status = "todo"
created_at = 2026-02-24T04:10:09.11822+00:00
modified_at = 2026-02-24T05:26:04.179603+00:00
needs = []
tags = ["self_hosted", "review_followup"]
+++
Prevent runtime panic when command surface grows and mapping lags. Return a safe error or enforce completeness without panic.