---
name: zipper-merger
description: Perform incremental, commit-by-commit "zipper" merges from a source branch into the current branch to resolve complex conflicts and maintain stability through constant verification.
---

# Zipper Merger

This skill provides a systematic workflow for merging feature branches commit-by-commit. This approach is essential for complex integrations where multiple features, security fixes, and performance optimizations must be reconciled while maintaining a passing test suite at every step.

## Workflow

### 1. Preparation
- Identify the source branch or specific range of commits to merge.
- Find the merge base between the current branch and the source:
  ```bash
  git merge-base HEAD <source-branch>
  ```
- List the commits to be merged in reverse chronological order (oldest first):
  ```bash
  git log --oneline <merge-base>..<source-branch> --reverse
  ```

### 2. Incremental Merge Iteration
For each commit in the list:

#### A. Execute Merge
- Perform the merge without committing:
  ```bash
  git merge --no-commit --no-ff <commit-hash>
  ```
- If the merge fails due to conflicts, resolve them immediately using the **Research -> Strategy -> Execution** cycle.

#### B. Validation (The Gamut)
- Before finalizing the commit, verify the stability of the project.
- Run the project's standard verification suite (e.g., `just fix && just check && just test`).
- Fix any regressions or integration issues introduced by the merge.

#### C. Finalization
- Once verified, finalize the merge commit:
  ```bash
  git add .
  git commit --no-edit
  ```
- Verify the git log to confirm the merge was recorded correctly:
  ```bash
  git log --oneline --graph --decorate -n 5
  ```

### 3. Conclusion
- After all commits are merged, perform a final run of the verification suite.
- Provide a summary of the merged commits and any significant conflict resolutions.

## Best Practices
- **Never Skip Validation**: Running the test suite after *every* incremental commit is the core of this skill. It prevents the compounding cost of diagnosing failures later.
- **Surgical Changes**: Only modify code necessary to resolve conflicts or fix regressions introduced by the merge.
- **Atomic Commits**: Keep the merge commits focused on the integration of the source commit.
