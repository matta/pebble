---
id: pebl-c1wp5f4hlc
title: Centralize config keys for help text
status: todo
created_at: 2026-02-24T05:41:50.195555+00:00
---
Hardcoding the list of configuration keys in the help text can lead to outdated documentation as new keys are added. To improve maintainability, consider defining the keys in a central `pub const` array. You could then use a build script (`build.rs`) or similar mechanism to dynamically generate this help string from the constant, ensuring it's always synchronized with the implementation.

Tracked from the following code review feedback:

**`crates/pebble/src/cli.rs:220`**
> Hardcoding the list of configuration keys in the help text can lead to outdated documentation as new keys are added. To improve maintainability, consider defining the keys in a central `pub const` array. You could then use a build script (`build.rs`) to dynamically generate this help string from the constant, ensuring it's always synchronized with the implementation.

**`crates/pebble/tests/cli_help_completeness.rs:62`**
> This test hardcodes the configuration keys it checks for in the help output. This means the test won't fail if a new key is added to the `Config` struct but not to the help text; it only protects against removing existing ones. If the list of keys is centralized into a `const` array (as suggested for `cli.rs`), this test could iterate over that constant to ensure all supported keys are documented. This would make the test more robust against future changes.
