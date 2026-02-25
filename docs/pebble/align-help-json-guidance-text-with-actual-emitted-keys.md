+++
id = "pebl-6pv06nwvpl"
title = "Align help-json guidance text with actual emitted keys"
status = "done"
created_at = 2026-02-24T05:22:50.792978+00:00
modified_at = 2026-02-25T03:39:49.959965+00:00
resolved_at = 2026-02-25T03:39:30.721371+00:00
needs = []
tags = ["self_hosted", "review_followup"]
+++
Current docs mention fields like output_shape and alias metadata that do not match current help-json output. Update guidance to use field names that are actually emitted today, while keeping the section non-normative and defensive.

Completed: updated docs/cli-contract.md help-json guidance to reference emitted field names (including output) and remove alias-metadata wording while keeping non-contract defensive language.