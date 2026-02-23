+++
id = "pebl-7Rnb6B"
title = "ID generation uses nanoid SAFE alphabet instead of lowercase alphanumeric"
status = "todo"
created_at = 2026-02-23T05:18:34.142651+00:00
needs = []
tags = ["bug"]
+++
The `nanoid::alphabet::SAFE` character set includes uppercase letters and possibly
symbols (e.g. `_`, `-`), but `cli-contract.md` specifies that ID suffixes must use
only `a-z0-9`. Replace the SAFE alphabet with a custom `0123456789abcdefghijklmnopqrstuvwxyz` alphabet.
