## 2025-01-20 - Git Argument Injection via Branch Names
**Vulnerability:** User-controlled branch names starting with `-` were interpreted as flags by `git` commands (e.g., `git checkout --orphan -bad`), bypassing intended logic and creating potentially confusing or harmful repository states.
**Learning:** `std::process::Command` prevents shell injection but NOT argument injection. Git commands parse arguments starting with `-` as flags unless stopped by `--`. However, some git commands (like `checkout --orphan`) do not support `--` to separate the new branch name.
**Prevention:**
1. Validate all user-supplied git refs (branches, tags) to ensure they do not start with `-`.
2. Use `--` delimiter in git commands wherever supported (e.g., `git worktree add -- <branch>`).

## 2025-01-23 - Path Traversal in Configuration Parsing
**Vulnerability:** The `tasks-dir` configuration option allowed `..` components, enabling path traversal outside the project root (e.g., `tasks-dir = "../../../etc"`). This bypassed the `is_absolute()` check and allowed reading arbitrary `.md` files on the system if a user loaded a malicious config.
**Learning:** `Path::is_absolute()` is insufficient to sandbox file access to a directory. Relative paths starting with `..` are not absolute but can still traverse upwards. Rust's `Path::components()` iterator provides a reliable way to detect `ParentDir` components.
**Prevention:**
1. Explicitly check for and reject `std::path::Component::ParentDir` in configuration paths intended to be sandbox-relative.
2. Consider canonicalizing paths (resolving symlinks and `..`) and verifying they start with the intended root prefix, though this can be complex with symlinks.

## 2025-02-21 - Inconsistent Validation between Config and CLI
**Vulnerability:** While `tasks-dir` was validated in `config.rs` to prevent path traversal, the `pebble init --dir` command argument bypassed this check, allowing users to initialize repositories with unsafe paths.
**Learning:** Security validation logic located in configuration parsing is easily bypassed by CLI argument overrides if not explicitly shared and reused.
**Prevention:**
1. Centralize validation logic (e.g., `validate_tasks_dir`) and call it from both configuration parsing and CLI argument handling.
2. Ensure CLI arguments that override configuration values undergo the exact same security checks as the configuration values themselves.

## 2025-02-22 - Path Traversal via Missing Validation in Global CLI Override
**Vulnerability:** A path traversal vulnerability was discovered in the CLI's processing of the global `--dir` flag. Although `pebble init` validated the `--dir` argument against parent components (`..`), the common command evaluation (`RunContext::load`) failed to perform this validation on `cli_dir_override`. A malicious user could bypass boundary checks completely (e.g., `pebble list --dir ../../etc`).
**Learning:** Validation MUST be applied holistically at the data resolution layer (`RunContext::load`), not merely within specific command sub-handlers. This ensures all consumers of configuration paths benefit from sandboxing guarantees.
**Prevention:**
1. Centralize the enforcement of `validate_tasks_dir` wherever an override path is consumed (such as `RunContext::load`), establishing an airtight boundary constraint for directory traversal.
2. Adopt a "validate at the edges" architecture where raw user inputs are aggressively sanitized prior to propagating through the system.
