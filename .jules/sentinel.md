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
## 2025-03-01 - TOCTOU Race Condition in File Creation
**Vulnerability:** A Time-of-Check to Time-of-Use (TOCTOU) vulnerability existed in `run_add` when creating new task files. The code checked if a file existed (`filepath.exists()`) and then later created/wrote to it, leaving a window where a malicious process could create the file or a symlink, leading to unintended overwrites.
**Learning:** Using separate check and write operations for file creation is inherently unsafe in concurrent or multi-user environments.
**Prevention:** Always use atomic operations for file creation when uniqueness is required. In Rust, this means using `std::fs::OpenOptions::new().write(true).create_new(true).open(...)` to atomically ensure the file is created only if it does not already exist, and handling the resulting `AlreadyExists` error appropriately (e.g., via a retry loop).

## 2026-03-03 - False Positive: Stack Traces in Open-Source CLI Tools
**Vulnerability:** Initially identified the use of the Debug formatter (`{:?}`) for `color_eyre::eyre::Result` errors as an information disclosure vulnerability because it leaked stack traces and file paths to stderr.
**Learning:** For a local, open-source CLI tool that processes data from the user's local disk, internal implementation file paths and stack traces are non-secrets. The codebase is already public. Leaking this information is a UX concern (overly verbose errors), not a security vulnerability.
**Prevention:** When evaluating potential information disclosure vulnerabilities, consider the application's execution context (local vs. server), the data it processes, and whether the "leaked" information is actually secret or sensitive. Do not flag stack traces as security issues in local, open-source tools.
