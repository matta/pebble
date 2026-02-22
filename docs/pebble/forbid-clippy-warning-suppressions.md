+++
id = "pebl-FU-FSN"
title = "Forbid clippy warning suppressions"
status = "todo"
created_at = 2026-02-22T23:14:28.82665+00:00
deps = []
tags = []
+++
We should forbid circumvention of key clippy warnings (cognitive_complexity, type_complexity, too_many_arguments, too_many_lines, large_enum_variant, struct_excessive_bools). Suggestion: pass these as -F flags to the cargo clippy invocation in justfile so suppressions are treated as warnings to fail CI.