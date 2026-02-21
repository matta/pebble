use crate::command::CommandExt;
use crate::git_provider::{GitProvider, RealGit};
use crate::{ISSUES_FILE, WORKTREE_DIR};
use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use std::path::{Path, PathBuf};

use std::process::Command;

/// Generates the path for a worktree given the repository root and sync branch name.
pub fn generate_worktree_path(repo_root: &Path, sync_branch: &str) -> PathBuf {
    repo_root.join(WORKTREE_DIR).join(sync_branch)
}

/// Finds the root of the Git repository from the given path.
pub fn find_git_root(path: &Path) -> Result<PathBuf> {
    let output = RealGit
        .output(&[&"rev-parse", &"--show-toplevel"], path)
        .with_context(|| "Failed to find git root")?;
    Ok(PathBuf::from(output.trim()))
}

#[derive(Debug)]
pub struct WorktreeManager<G: GitProvider = RealGit> {
    repo_root: PathBuf,
    sync_branch: String,
    git: G,
    custom_editor: Option<String>,
}

impl WorktreeManager<RealGit> {
    pub fn new(repo_root: PathBuf, sync_branch: String) -> Self {
        Self {
            repo_root,
            sync_branch,
            git: RealGit,
            custom_editor: None,
        }
    }
}

impl<G: GitProvider> WorktreeManager<G> {
    pub fn new_with_git(repo_root: PathBuf, sync_branch: String, git: G) -> Self {
        Self {
            repo_root,
            sync_branch,
            git,
            custom_editor: None,
        }
    }

    pub fn with_editor(mut self, editor: String) -> Self {
        self.custom_editor = Some(editor);
        self
    }

    /// Checks if the current directory is inside a Git repository.
    pub fn is_inside_git_repo(path: &std::path::Path) -> bool {
        RealGit
            .status_silent(&[&"rev-parse", &"--is-inside-work-tree"], path)
            .map(|s| s.success())
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
        let status = self.git.status_silent(
            &[&"rev-parse", &"--verify", &self.sync_branch],
            &self.repo_root,
        )?;

        if status.success() {
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
        let commit_hash = self.git.output(
            &[
                &"commit-tree",
                &empty_tree_hash,
                &"-m",
                &"Pebble database tracking branch initial commit",
            ],
            &self.repo_root,
        )?;
        let commit_hash = commit_hash.trim();

        // 3. Update the reference to point to this commit
        let ref_path = format!("refs/heads/{}", self.sync_branch);
        self.git
            .run(&[&"update-ref", &ref_path, &commit_hash], &self.repo_root)?;

        Ok(())
    }

    /// Initializes a worktree at the given path linked to the sync branch.
    pub fn init_worktree(&self, path: &std::path::Path) -> Result<()> {
        if path.exists() {
            return Err(eyre!("Worktree path {:?} already exists", path));
        }

        // Ensure the parent directory exists
        if let Some(parent) = path.parent().filter(|p| !p.exists()) {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create worktree parent directory: {:?}", parent)
            })?;
        }

        // git worktree add <path> <branch>
        self.git
            .run_silent(
                &[&"worktree", &"add", &path, &"--", &self.sync_branch],
                &self.repo_root,
            )
            .with_context(|| format!("Failed to add git worktree at {:?}", path))?;

        Ok(())
    }

    /// Checks if the worktree has uncommitted changes.
    pub fn is_dirty(&self) -> Result<bool> {
        let path = self.get_worktree_path();
        if !path.exists() {
            return Ok(false);
        }

        let output = self.git.output(&[&"status", &"--porcelain"], &path)?;

        Ok(!output.trim().is_empty())
    }

    /// Stages all changes and commits them in the worktree.
    pub fn commit_all(&self, message: &str) -> Result<()> {
        let path = self.get_worktree_path();

        self.git.run_quiet(&[&"add", &"-A"], &path)?;

        self.git.run_quiet(&[&"commit", &"-m", &message], &path)?;

        Ok(())
    }

    /// Stages all changes and commits them in the worktree without stdout noise.
    pub fn commit_all_quiet(&self, message: &str) -> Result<()> {
        let path = self.get_worktree_path();

        self.git.run(&[&"add", &"-A"], &path)?;

        self.git.run_quiet(&[&"commit", &"-m", &message], &path)?;

        Ok(())
    }

    pub fn get_worktree_path(&self) -> PathBuf {
        generate_worktree_path(&self.repo_root, &self.sync_branch)
    }

    /// Ensures that a local worktree for the sync branch exists and is ready for use.
    ///
    /// This method manages the lifecycle of the sync worktree. It performs the following steps:
    /// 1. Checks if the worktree directory already exists.
    /// 2. If not, creates the necessary parent directories.
    /// 3. Checks if the sync branch exists locally.
    /// 4. If not local, attempts to fetch it from the `origin` remote.
    /// 5. If it doesn't exist remotely either, it initializes a new orphan branch.
    /// 6. Creates a Git worktree at the target path, detached or checked out as appropriate.
    ///
    /// Returns a `PathBuf` pointing to the root of the initialized worktree.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// * File system operations fail (creating directories).
    /// * Git commands fail (checking status, fetching, adding worktree).
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
        let has_local = self
            .git
            .run_silent(
                &[&"rev-parse", &"--verify", &self.sync_branch],
                &self.repo_root,
            )
            .is_ok();

        let target_branch = if has_local {
            Some(self.sync_branch.clone())
        } else {
            // Check if 'origin' remote exists
            // TODO: Make 'origin' configurable
            let has_origin = self
                .git
                .run_silent(&[&"remote", &"get-url", &"origin"], &self.repo_root)
                .is_ok();

            if has_origin {
                // Try to fetch origin to update remote refs
                self.git
                    .run(&[&"fetch", &"origin"], &self.repo_root)
                    .with_context(|| "Failed to fetch from origin")?;

                // Check if origin/sync_branch exists
                let remote_ref = format!("origin/{}", self.sync_branch);
                let has_remote = self
                    .git
                    .run_silent(&[&"rev-parse", &"--verify", &remote_ref], &self.repo_root)
                    .is_ok();

                if has_remote { Some(remote_ref) } else { None }
            } else {
                None
            }
        };

        if let Some(target) = target_branch {
            self.git
                .run_quiet(
                    &[&"worktree", &"add", &"--detach", &path, &"--", &target],
                    &self.repo_root,
                )
                .with_context(|| "Failed to execute git worktree add")?;
        } else {
            // Initialize as orphan branch
            // First create worktree without checking out anything (to avoid huge checkout)
            self.git
                .run_quiet(
                    &[&"worktree", &"add", &"--detach", &"--no-checkout", &path],
                    &self.repo_root,
                )
                .with_context(|| "Failed to execute git worktree add (orphan)")?;

            // Create orphan branch inside worktree
            self.git
                .run(&[&"checkout", &"--orphan", &self.sync_branch], &path)
                .with_context(|| "Failed to create orphan branch")?;

            // Ensure index is empty
            let _ = self.git.run(&[&"rm", &"-rf", &"."], &path);
        }

        Ok(path)
    }

    pub fn get_absolute_jsonl_path(&self) -> Result<PathBuf> {
        let worktree_path = self.ensure_worktree()?;
        Ok(worktree_path.join(ISSUES_FILE))
    }

    fn commit_local_changes(&self, worktree_path: &Path) -> Result<()> {
        // Stage all changes (including new files)
        self.git
            .run_quiet(&[&"add", &"."], worktree_path)
            .with_context(|| "Failed to stage changes")?;

        // Check if there are changes to commit
        let status = self
            .git
            .output(&[&"status", &"--porcelain"], worktree_path)
            .with_context(|| "Failed to check status")?;

        if !status.trim().is_empty() {
            self.git
                .run_quiet(
                    &[&"commit", &"--no-verify", &"-m", &"Auto-sync"],
                    worktree_path,
                )
                .with_context(|| "Failed to commit changes")?;
        }
        Ok(())
    }

    fn commit_local_changes_quiet(&self, worktree_path: &Path) -> Result<()> {
        self.git
            .run_quiet(&[&"add", &"."], worktree_path)
            .with_context(|| "Failed to stage changes")?;

        let status = self
            .git
            .output(&[&"status", &"--porcelain"], worktree_path)
            .with_context(|| "Failed to check status")?;

        if !status.trim().is_empty() {
            self.git
                .run_quiet(
                    &[&"commit", &"--no-verify", &"-m", &"Auto-sync"],
                    worktree_path,
                )
                .with_context(|| "Failed to commit changes")?;
        }
        Ok(())
    }

    fn resolve_conflicts(&self, worktree_path: &Path) -> Result<()> {
        println!("Conflict detected. Opening editor to resolve...");

        // specific diff-filter=U for unmerged (conflicted) files
        let output = self
            .git
            .output(
                &[&"diff", &"--name-only", &"--diff-filter=U"],
                worktree_path,
            )
            .with_context(|| "Failed to list conflicted files")?;

        if output.trim().is_empty() {
            // Subtle: resolve_conflicts is only called when `git merge` returned non-zero.
            // If there are no unmerged files, the merge failed for some other reason and
            // should not be treated as success.
            return Err(eyre!("No conflicted files found"));
        }

        let files: Vec<&str> = output.lines().collect();
        println!("Conflicted files: {:?}", files);
        let editor = self
            .custom_editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "vi".to_string());
        println!("Launching editor: {}", editor);

        // We use status() to let the editor take over stdin/stdout
        // EDITOR is not a git command, so we still use Command::new here
        let status = Command::new(&editor)
            .args(&files)
            .current_dir(worktree_path)
            .status()
            .with_context(|| format!("Failed to launch editor {}", editor))?;

        if !status.success() {
            return Err(eyre!("Editor exited with error"));
        }

        println!("Editor finished successfully. Staging and committing...");

        // Assume resolved. Stage and commit.
        self.git
            .run(&[&"add", &"."], worktree_path)
            .with_context(|| "Failed to stage resolved files")?;

        self.git
            .run(&[&"commit", &"--no-verify", &"--no-edit"], worktree_path)
            .with_context(|| "Failed to commit merge resolution")?;

        println!("Merge resolution committed.");

        Ok(())
    }

    /// Synchronizes the local worktree with the remote repository.
    ///
    /// This method performs a sequence of Git operations to ensure the worktree
    /// is up-to-date and local changes are pushed. Specifically, it:
    /// 1. Ensures the worktree exists (creating it if necessary).
    /// 2. Fetches the latest changes from the remote `origin` for the configured sync branch.
    /// 3. Merges the remote changes into the local worktree.
    /// 4. Pushes the local worktree state back to `origin`.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if:
    /// * The worktree creation/access fails.
    /// * Any of the Git commands (`fetch`, `merge`, `push`) fail (return a non-zero exit code).
    /// * The merge requires conflict resolution and the interactive resolution fails.
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

        // 1. Commit any local changes first (required for 3-way merge)
        self.commit_local_changes(&worktree_path)
            .with_context(|| "Failed to commit local changes before sync")?;

        // 2. git fetch origin <sync_branch>
        self.git
            .run(
                &[&"fetch", &"origin", &"--", &self.sync_branch],
                &worktree_path,
            )
            .with_context(|| "Failed to execute git fetch")?;

        // 3. git merge origin/<sync_branch> (3-way)
        let merge_ref = format!("origin/{}", self.sync_branch);
        let merge_status = self
            .git
            .status(&[&"merge", &merge_ref], &worktree_path)
            .with_context(|| "Failed to execute git merge command")?;

        if !merge_status.success() {
            if merge_status.code() == Some(1) {
                // Conflict
                // Subtle: a non-zero merge status means the merge failed; we only proceed
                // if we can positively resolve actual conflicts.
                self.resolve_conflicts(&worktree_path)
                    .with_context(|| "Failed to execute git merge")?;
            } else {
                return Err(eyre!("Git merge failed with status: {}", merge_status));
            }
        }

        // 4. git push origin HEAD:<sync_branch>
        let push_ref = format!("HEAD:{}", self.sync_branch);
        self.git
            .run(&[&"push", &"origin", &"--", &push_ref], &worktree_path)
            .with_context(|| "Failed to execute git push")?;

        Ok(())
    }

    pub fn sync_quiet(&self) -> Result<()> {
        let worktree_path = self.ensure_worktree()?;

        self.commit_local_changes_quiet(&worktree_path)
            .with_context(|| "Failed to commit local changes before sync")?;

        self.git
            .run_silent(
                &[&"fetch", &"origin", &"--", &self.sync_branch],
                &worktree_path,
            )
            .with_context(|| "Failed to execute git fetch")?;

        let merge_ref = format!("origin/{}", self.sync_branch);
        let merge_status = self
            .git
            .status_silent(&[&"merge", &merge_ref], &worktree_path)
            .with_context(|| "Failed to execute git merge command")?;

        if !merge_status.success() {
            if merge_status.code() == Some(1) {
                let _ = self.git.run_silent(&[&"merge", &"--abort"], &worktree_path);
                return Err(eyre!(
                    "Merge conflict detected. Run 'pebble sync' interactively to resolve."
                ));
            } else {
                return Err(eyre!("Git merge failed with status: {}", merge_status));
            }
        }

        let push_ref = format!("HEAD:{}", self.sync_branch);
        self.git
            .run_silent(&[&"push", &"origin", &"--", &push_ref], &worktree_path)
            .with_context(|| "Failed to execute git push")?;

        Ok(())
    }
}
