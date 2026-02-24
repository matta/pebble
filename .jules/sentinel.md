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

## 2025-05-23 - TOCTOU Race Condition in File Creation
**Vulnerability:** The `run_add` command used a check-then-act pattern (`if !exists { write }`) to find an available filename. This race condition (Time-of-Check to Time-of-Use) allowed concurrent processes to select the same filename and overwrite each other's data.
**Learning:** Checking for file existence before creation is never atomic. Filesystem state can change between the check and the write operation.
**Prevention:** Always use atomic file creation primitives like `std::fs::File::create_new(true)` (which maps to `O_EXCL` on POSIX) when the goal is to create a file only if it does not exist.
