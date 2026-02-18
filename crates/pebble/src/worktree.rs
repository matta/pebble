use crate::command::CommandExt;
use color_eyre::Result;
use color_eyre::eyre::Context;
use std::path::PathBuf;
use std::process::Command;

pub struct WorktreeManager {
    repo_root: PathBuf,
    sync_branch: String,
}

impl WorktreeManager {
    pub fn new(repo_root: PathBuf, sync_branch: String) -> Self {
        Self {
            repo_root,
            sync_branch,
        }
    }

    pub fn get_worktree_path(&self) -> PathBuf {
        self.repo_root
            .join(".git/beads-worktrees")
            .join(&self.sync_branch)
    }

    pub fn ensure_worktree(&self) -> Result<PathBuf> {
        let path = self.get_worktree_path();
        if path.exists() {
            // Verify it's actually a worktree? For now just return path
            return Ok(path);
        }

        // Create the directory for worktrees if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create worktree parent directory: {:?}", parent)
            })?;
        }

        // Run git worktree add
        // We use --detach because we don't need a local branch, just the checked output
        // actually, we probably want to check out the specific branch
        // but if the branch doesn't exist locally, we might need to fetch it first.
        // For phase 1, let's assume we can just add the worktree.

        Command::new("git")
            .arg("worktree")
            .arg("add")
            .arg("--detach") // use detach to avoid branch conflicts for now
            .arg(&path)
            .current_dir(&self.repo_root)
            .check_output()
            .with_context(|| "Failed to execute git worktree add")?;

        Ok(path)
    }

    pub fn get_absolute_jsonl_path(&self) -> Result<PathBuf> {
        let worktree_path = self.ensure_worktree()?;
        Ok(worktree_path.join(".beads/issues.jsonl"))
    }

    /// Synchronizes the local worktree with the remote repository.
    ///
    /// This method performs a sequence of Git operations to ensure the worktree
    /// is up-to-date and local changes are pushed. Specifically, it:
    /// 1. Ensures the worktree exists (creating it if necessary).
    /// 2. Fetches the latest changes from the remote `origin` for the configured sync branch.
    /// 3. Merges the remote changes into the local worktree using `--ff-only`.
    /// 4. Pushes the local worktree state back to `origin`.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if:
    /// * The worktree creation/access fails.
    /// * Any of the Git commands (`fetch`, `merge`, `push`) fail (return a non-zero exit code).
    /// * The merge requires conflict resolution (since `--ff-only` is used).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pebble::worktree::WorktreeManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = WorktreeManager::new(
    ///     PathBuf::from("/path/to/repo"),
    ///     "my-sync-branch".to_string()
    /// );
    ///
    /// // Requires a valid git environment and remote
    /// if let Err(e) = manager.sync() {
    ///     eprintln!("Sync failed: {}", e);
    /// }
    /// ```
    pub fn sync(&self) -> Result<()> {
        let worktree_path = self.ensure_worktree()?;

        // git fetch origin <sync_branch>
        Command::new("git")
            .args(["fetch", "origin", "--", &self.sync_branch])
            .current_dir(&worktree_path)
            .check_run()
            .with_context(|| "Failed to execute git fetch")?;

        // git merge origin/<sync_branch>
        // We use --ff-only to fail if there are conflicts for now (Phase 1)
        // Ideally we would support 3-way merge but let's start simple
        Command::new("git")
            .args([
                "merge",
                "--ff-only",
                &format!("origin/{}", self.sync_branch),
            ])
            .current_dir(&worktree_path)
            .check_run()
            .with_context(|| "Failed to execute git merge")?;

        // git push origin <sync_branch>
        Command::new("git")
            .args(["push", "origin", "--", &self.sync_branch])
            .current_dir(&worktree_path)
            .check_run()
            .with_context(|| "Failed to execute git push")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn run_git(args: &[&str], dir: &std::path::Path) {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .check_run()
            .unwrap_or_else(|e| panic!("Failed to execute git {}: {}", args.join(" "), e));
    }

    fn setup_git_repo(path: &std::path::Path) {
        run_git(&["init", "-b", "main"], path);
        run_git(&["config", "user.email", "test@example.com"], path);
        run_git(&["config", "user.name", "Test User"], path);
    }

    #[test]
    fn test_worktree_path_generation() {
        let repo_root = PathBuf::from("/tmp/repo");
        let manager = WorktreeManager::new(repo_root.clone(), "beads-sync".to_string());
        let expected = repo_root.join(".git/beads-worktrees/beads-sync");
        assert_eq!(manager.get_worktree_path(), expected);
    }

    #[test]
    fn test_get_absolute_jsonl_path() {
        // Setup a dummy git repo (similar to ensure_worktree_creation)
        // Since get_absolute_jsonl_path calls ensure_worktree, we need a valid git repo
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().to_path_buf();

        setup_git_repo(&repo_root);

        std::fs::create_dir(repo_root.join(".beads")).unwrap();
        std::fs::write(repo_root.join(".beads/dummy"), "dummy").unwrap(); // git needs a file to track dir
        run_git(&["add", "."], &repo_root);
        run_git(&["commit", "-m", "Initial"], &repo_root);

        let manager = WorktreeManager::new(repo_root.clone(), "beads-sync".to_string());
        let expected = repo_root.join(".git/beads-worktrees/beads-sync/.beads/issues.jsonl");

        let path = manager
            .get_absolute_jsonl_path()
            .expect("Failed to get jsonl path");
        assert_eq!(path, expected);
        assert!(path.parent().unwrap().exists()); // Worktree should be created and contain .beads
    }

    #[test]
    fn test_ensure_worktree_creation() {
        // Setup a dummy git repo
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().to_path_buf();

        setup_git_repo(&repo_root);

        // Create initial commit so we have a valid HEAD
        // Worktree creation often requires a valid HEAD or existing branch
        std::fs::write(repo_root.join("README.md"), "Initial commit").unwrap();
        run_git(&["add", "."], &repo_root);
        run_git(&["commit", "-m", "Initial commit"], &repo_root);

        let manager = WorktreeManager::new(repo_root.clone(), "beads-sync".to_string());

        // This should trigger worktree creation logic
        let worktree_path = manager
            .ensure_worktree()
            .expect("Failed to ensure worktree");

        assert!(worktree_path.exists(), "Worktree path does not exist");
        assert!(
            worktree_path.join(".git").exists(),
            "Worktree is not a valid git repo (missing .git file/dir)"
        );

        // Check if git works inside the worktree
        let output = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&worktree_path)
            .check_output()
            .expect("Failed to run git status in worktree");

        assert_eq!(output.trim(), "true", "Not inside a worktree");
    }

    #[test]
    fn test_sync_command_operations() {
        // Setup a dummy "remote" repo
        let remote_dir = TempDir::new().unwrap();
        let remote_root = remote_dir.path().to_path_buf();

        run_git(&["init", "--bare"], &remote_root);

        // Setup the "local" repo
        let local_dir = TempDir::new().unwrap();
        let local_root = local_dir.path().to_path_buf();

        setup_git_repo(&local_root);

        // Create initial content to push to "remote" so we have something to fetch
        std::fs::write(local_root.join("README.md"), "Initial content").unwrap();
        run_git(&["add", "."], &local_root);
        run_git(&["commit", "-m", "Initial"], &local_root);

        // Add remote and push master (which we'll use as sync branch base for this test)
        // Actually, we need to push a branch named 'beads-sync' to the remote
        run_git(
            &["remote", "add", "origin", remote_root.to_str().unwrap()],
            &local_root,
        );

        run_git(&["checkout", "-b", "beads-sync"], &local_root);

        run_git(&["push", "-u", "origin", "beads-sync"], &local_root);

        // Now switch back to main to simulate user state
        run_git(&["checkout", "main"], &local_root);

        // Now test the WorktreeManager
        let manager = WorktreeManager::new(local_root.clone(), "beads-sync".to_string());

        // This should create worktree, fetch, merge, and push
        // Note: push might be a no-op if nothing changed, but command should succeed
        manager.sync().expect("Sync failed");

        let worktree_path = manager.get_worktree_path();
        assert!(worktree_path.exists());

        // Verify we are on the correct branch in worktree
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&worktree_path)
            .check_output()
            .expect("Failed to check branch");

        // Since we use --detach, it might be "HEAD"
        let branch = output.trim().to_string();
        assert!(branch == "HEAD" || branch == "beads-sync");
    }

    #[test]
    fn test_sync_branch_argument_injection_prevention() {
        // Setup a dummy git repo
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().to_path_buf();

        setup_git_repo(&repo_root);

        // Create initial commit so we have a valid HEAD
        std::fs::write(repo_root.join("README.md"), "Initial commit").unwrap();
        run_git(&["add", "."], &repo_root);
        run_git(&["commit", "-m", "Initial commit"], &repo_root);

        // We need a remote to fetch from
        let remote_dir = TempDir::new().unwrap();
        let remote_root = remote_dir.path().to_path_buf();
        run_git(&["init", "--bare"], &remote_root);
        run_git(
            &["remote", "add", "origin", remote_root.to_str().unwrap()],
            &repo_root,
        );

        // Malicious branch name
        let malicious_branch = "--unknown-option".to_string();
        let manager = WorktreeManager::new(repo_root.clone(), malicious_branch);

        // This should fail
        let result = manager.sync();

        assert!(result.is_err(), "Sync should fail for --unknown-option");
        let err = result.unwrap_err();
        let debug_msg = format!("{:?}", err);

        // Check if error is usage error (129) or fatal error (128)
        // If 129 -> Vulnerable (git treated it as flag)
        // If 128 -> Fixed (git treated it as ref)
        if debug_msg.contains("129") {
            panic!(
                "VULNERABLE: Git fetch failed with usage error (129). This indicates argument injection."
            );
        } else if debug_msg.contains("128") {
            // Fixed!
        } else {
            // If we get here, it might be some other error.
            panic!("Unknown error code or message: {}", debug_msg);
        }
    }
}
