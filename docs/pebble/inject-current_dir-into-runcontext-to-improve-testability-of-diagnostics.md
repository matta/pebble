+++
id = "pebl-cug7mpg7cz"
title = "Inject current_dir into RunContext to improve testability of diagnostics"
status = "todo"
created_at = 2026-02-25T17:55:42.855684+00:00
needs = ["pebl-hRuKk1"]
tags = []
+++
Calling env::current_dir() introduces a side-effect, making this function harder to test in isolation. The RunContext is designed to encapsulate environmental details. Consider adding current_dir to RunContext during its creation in main.rs and using ctx.current_dir here instead. This would improve testability and align with the 'inject dependencies' principle.