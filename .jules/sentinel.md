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

## 2025-02-24 - CLI Override Validation Gap in RunContext
**Vulnerability:** While `pebble init` was secured, `RunContext::load` (used by `list`, `add`, etc.) still allowed `tasks-dir` override via `--dir` to contain `..`, bypassing the `tasks-dir` safety invariant.
**Learning:** When multiple code paths (e.g., initialization vs runtime loading) handle the same configuration override, security validation must be applied to ALL of them. Centralizing validation logic is not enough; you must ensure every *call site* uses it.
**Prevention:**
1. Audit all entry points that accept configuration overrides (CLI flags, env vars) and ensure they invoke the centralized validation function.
