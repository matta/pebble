## 2025-01-20 - Git Argument Injection via Branch Names
**Vulnerability:** User-controlled branch names starting with `-` were interpreted as flags by `git` commands (e.g., `git checkout --orphan -bad`), bypassing intended logic and creating potentially confusing or harmful repository states.
**Learning:** `std::process::Command` prevents shell injection but NOT argument injection. Git commands parse arguments starting with `-` as flags unless stopped by `--`. However, some git commands (like `checkout --orphan`) do not support `--` to separate the new branch name.
**Prevention:**
1. Validate all user-supplied git refs (branches, tags) to ensure they do not start with `-`.
2. Use `--` delimiter in git commands wherever supported (e.g., `git worktree add -- <branch>`).

## 2025-01-22 - Path Traversal via Git Branch Names
**Vulnerability:** The `sync-branch` configuration was used directly to construct a filesystem path for a Git worktree. A malicious branch name like `../../../hacked` allowed creating directories outside the repository root.
**Learning:** Git branch names can contain characters like `/` (forward slash), which `PathBuf::join` treats as directory separators. While `..` is invalid in Git refs, validation must occur BEFORE passing the string to filesystem operations, especially when constructing paths. `Config::validate` is insufficient if internal APIs (`WorktreeManager`) are used directly without validation.
**Prevention:**
1. Validate all user-supplied paths/branches to ensure they do not contain `..` or absolute path indicators.
2. Perform validation at the point of use (e.g., in `ensure_worktree`) rather than just at the configuration boundary, to protect internal APIs.
