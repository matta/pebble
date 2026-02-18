---
name: gh-pr-fixer
description: Fix GitHub Pull Request CI failures and resolve merge conflicts. Use when a PR is reported as failing CI or having conflicts. Strongly prefer the `gh` CLI tool over browser-based tools for speed and automation.
---

# GH PR Fixer Workflow

## Tooling Preferences

- **Mandatory Tool**: Use the `gh` (GitHub CLI) tool for all PR discovery, checkout, and status viewing.
- **Avoid Browser Tools**: Do not use `browser_subagent` or `read_url_content` for PR tasks unless specifically instructed by the user or if `gh` is unavailable. The `gh` tool is faster, more precise, and better suited for automation.

## Phase 1: Preparation & Checkout

1.  **Verify Pristine Workspace**: Ensure no uncommitted changes in the current directory.
    - `git status`
    - Failing this, report and stop.
2.  **Checkout PR**: Use GitHub CLI to switch to the PR's branch.
    - `gh pr checkout <PR_NUMBER>`

## Phase 2: Diagnosis

Identify why the PR is blocked.

1.  **Check Mergeability**:
    - `gh pr view <PR_NUMBER> --json mergeable,mergeStateStatus`
    - Look for `mergeable: "CONFLICTING"` or `mergeStateStatus: "DIRTY"`.
2.  **Check CI Results**:
    - `gh pr checks <PR_NUMBER>`
    - If checks are failing, list the failing jobs and analyze logs (if available via `gh run view`).

## Phase 3: Resolution

### Resolving Merge Conflicts
1.  **Fetch Latest Main**: `git fetch origin main`
2.  **Merge into PR Branch**: `git merge origin/main`
3.  **Resolve Conflicts**:
    - Identify conflicted files: `git status` or `git diff --name-only --diff-filter=U`
    - Read conflicted files and apply resolutions (e.g., combining changes, deleting redundant code).
    - Mark resolved: `git add <FILE>`
    - Finalize: `git commit --no-edit`

### Fixing CI Failures
1.  **Reproduce Locally**: Run tests or build commands associated with the failure.
    - `cargo test` (for Rust)
    - `npm test` (for JS/TS)
    - `pytest` (for Python)
2.  **Analyze and Fix**: Apply targeted code changes to address the root cause of the failure.
3.  **Verify Fix**: Re-run local validation.

## Phase 4: Submission

1.  **Commit Changes**: (If not already committed during merge resolution).
    - `git add .`
    - `git commit -m "Fix CI/Merge conflicts in PR #<PR_NUMBER>"`
2.  **Push**:
    - `git push origin HEAD`
3.  **Verify PR Status**:
    - `gh pr view <PR_NUMBER> --json mergeable,mergeStateStatus`
