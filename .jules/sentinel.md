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

## 2025-02-21 - Denial of Service via Large Task Files
**Vulnerability:** The application attempted to read and parse the entire contents of any `.md` file in the tasks directory into memory. A malicious or accidentally large file could cause excessive memory consumption (OOM) and crash the application.
**Learning:** `std::fs::read_to_string` loads the entire file at once. When processing user-controlled files in a loop (like loading a graph), always check file metadata (size) before reading content.
**Prevention:**
1. Check `std::fs::metadata(path)?.len()` before reading file content.
2. Enforce a reasonable maximum file size (e.g., 5MB) for user-generated content like task descriptions.
