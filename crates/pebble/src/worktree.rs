use crate::command::CommandExt;
use color_eyre::Result;
use color_eyre::eyre::Context;
use std::path::{Path, PathBuf};
use std::process::Command;

pub trait GitProvider {
    fn run(&self, args: &[&str], current_dir: &Path) -> Result<()>;
    fn output(&self, args: &[&str], current_dir: &Path) -> Result<String>;
}

pub struct RealGit;

impl GitProvider for RealGit {
    fn run(&self, args: &[&str], current_dir: &Path) -> Result<()> {
        Command::new("git")
            .args(args)
            .current_dir(current_dir)
            .check_run()
            .map_err(Into::into)
    }

    fn output(&self, args: &[&str], current_dir: &Path) -> Result<String> {
        Command::new("git")
            .args(args)
            .current_dir(current_dir)
            .check_output()
            .map_err(Into::into)
    }
}

pub struct WorktreeManager<G: GitProvider = RealGit> {
    repo_root: PathBuf,
    sync_branch: String,
    git: G,
}

impl WorktreeManager<RealGit> {
    pub fn new(repo_root: PathBuf, sync_branch: String) -> Self {
        Self {
            repo_root,
            sync_branch,
            git: RealGit,
        }
    }
}

impl<G: GitProvider> WorktreeManager<G> {
    pub fn new_with_git(repo_root: PathBuf, sync_branch: String, git: G) -> Self {
        Self {
            repo_root,
            sync_branch,
            git,
        }
    }

    /// Checks if the current directory is inside a Git repository.
    pub fn is_inside_git_repo(path: &std::path::Path) -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Creates an orphaned branch for synchronization with no shared history.
    pub fn create_orphaned_sync_branch(&self) -> Result<()> {
        // git checkout --orphan <sync_branch>
        // We do this by creating a temp directory to avoid messing with the current worktree
        // Actually, we can use a more efficient approach with plumbing commands or just work with current worktree if it's clean enough
        // But the safest way in a script is often to create a new index or use a temporary worktree.
        // For simplicity and matching requirements:
        // 1. Create an orphan branch (without switching to it in the main repo)
        // 2. We can use a lower-level git command or just do it in the main repo if we are careful.

        // Let's use the current repository to create the branch
        // We first need to check if the branch already exists.
        let output = Command::new("git")
            .args(["rev-parse", "--verify", &self.sync_branch])
            .current_dir(&self.repo_root)
            .output()?;

        if output.status.success() {
            // Branch already exists, nothing to do for "create"
            return Ok(());
        }

        // To create a truly orphaned branch with NO history:
        // git checkout --orphan <branch>
        // git rm -rf .
        // git commit --allow-empty -m "Initial orphaned branch"
        // git checkout <original_branch>

        // But we don't want to disrupt the user's current worktree.
        // Alternative: Use a temporary directory as a git repository to create the commit and push it to the main repo.
        // Or even better: Use `git hash-object` and `git update-ref` to create a commit with no parent.

        // 1. Create an empty tree
        let empty_tree_hash = Command::new("git")
            .args(["mktree"])
            .stdin(std::process::Stdio::null())
            .current_dir(&self.repo_root)
            .check_output()?;
        let empty_tree_hash = empty_tree_hash.trim();

        // 2. Create a commit from that tree (with no parents)
        let commit_hash = Command::new("git")
            .args([
                "commit-tree",
                empty_tree_hash,
                "-m",
                "Pebble database tracking branch initial commit",
            ])
            .current_dir(&self.repo_root)
            .check_output()?;
        let commit_hash = commit_hash.trim();

        // 3. Update the reference to point to this commit
        Command::new("git")
            .args([
                "update-ref",
                &format!("refs/heads/{}", self.sync_branch),
                commit_hash,
            ])
            .current_dir(&self.repo_root)
            .check_run()?;

        Ok(())
    }

    /// Initializes a worktree at the given path linked to the sync branch.
    pub fn init_worktree(&self, path: &std::path::Path) -> Result<()> {
        if path.exists() {
            return Err(color_eyre::eyre::eyre!(
                "Worktree path {:?} already exists",
                path
            ));
        }

        // git worktree add <path> <branch>
        Command::new("git")
            .args(["worktree", "add", path.to_str().unwrap(), &self.sync_branch])
            .current_dir(&self.repo_root)
            .check_run()
            .with_context(|| format!("Failed to add git worktree at {:?}", path))?;

        Ok(())
    }

    /// Checks if the worktree has uncommitted changes.
    pub fn is_dirty(&self) -> Result<bool> {
        let path = self.get_worktree_path();
        if !path.exists() {
            return Ok(false);
        }

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&path)
            .check_output()?;

        Ok(!output.trim().is_empty())
    }

    /// Stages all changes and commits them in the worktree.
    pub fn commit_all(&self, message: &str) -> Result<()> {
        let path = self.get_worktree_path();

        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&path)
            .check_run()?;

        Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&path)
            .check_run()?;

        Ok(())
    }

    pub fn get_worktree_path(&self) -> PathBuf {
        self.repo_root.join(".git/x-pebble")
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

        // Check if sync_branch exists locally
        let has_local = Command::new("git")
            .args(["rev-parse", "--verify", &self.sync_branch])
            .current_dir(&self.repo_root)
            .check_run()
            .is_ok();

        let target_branch = if has_local {
            Some(self.sync_branch.clone())
        } else {
            // Check if 'origin' remote exists
            // TODO: Make 'origin' configurable
            let has_origin = Command::new("git")
                .args(["remote", "get-url", "origin"])
                .current_dir(&self.repo_root)
                .check_run()
                .is_ok();

            if has_origin {
                // Try to fetch origin to update remote refs
                Command::new("git")
                    .args(["fetch", "origin"])
                    .current_dir(&self.repo_root)
                    .check_run()
                    .with_context(|| "Failed to fetch from origin")?;

                // Check if origin/sync_branch exists
                let remote_ref = format!("origin/{}", self.sync_branch);
                let has_remote = Command::new("git")
                    .args(["rev-parse", "--verify", &remote_ref])
                    .current_dir(&self.repo_root)
                    .check_run()
                    .is_ok();

                if has_remote { Some(remote_ref) } else { None }
            } else {
                None
            }
        };

        if let Some(target) = target_branch {
            self.git
                .run(
                    &[
                        "worktree",
                        "add",
                        "--detach",
                        path.to_str().unwrap(),
                        &target,
                    ],
                    &self.repo_root,
                )
                .with_context(|| "Failed to execute git worktree add")?;
        } else {
            // Initialize as orphan branch
            // First create worktree without checking out anything (to avoid huge checkout)
            self.git
                .run(
                    &[
                        "worktree",
                        "add",
                        "--detach",
                        "--no-checkout",
                        path.to_str().unwrap(),
                    ],
                    &self.repo_root,
                )
                .with_context(|| "Failed to execute git worktree add (orphan)")?;

            // Create orphan branch inside worktree
            self.git
                .run(&["checkout", "--orphan", &self.sync_branch], &path)
                .with_context(|| "Failed to create orphan branch")?;

            // Ensure index is empty
            let _ = self.git.run(&["rm", "-rf", "."], &path);
        }

        Ok(path)
    }

    pub fn get_absolute_jsonl_path(&self) -> Result<PathBuf> {
        let worktree_path = self.ensure_worktree()?;
        Ok(worktree_path.join("issues.jsonl"))
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
        self.git
            .run(
                &["fetch", "origin", "--", &self.sync_branch],
                &worktree_path,
            )
            .with_context(|| "Failed to execute git fetch")?;

        // git merge origin/<sync_branch>
        // We use --ff-only to fail if there are conflicts for now (Phase 1)
        // Ideally we would support 3-way merge but let's start simple
        self.git
            .run(
                &[
                    "merge",
                    "--ff-only",
                    &format!("origin/{}", self.sync_branch),
                ],
                &worktree_path,
            )
            .with_context(|| "Failed to execute git merge")?;

        // git push origin <sync_branch>
        self.git
            .run(&["push", "origin", "--", &self.sync_branch], &worktree_path)
            .with_context(|| "Failed to execute git push")?;

        Ok(())
    }
}
