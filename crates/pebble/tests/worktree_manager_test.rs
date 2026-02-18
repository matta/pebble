use color_eyre::Result;
use pebble::git_provider::GitProvider;
use pebble::worktree::WorktreeManager;
use pebble::{CONFIG_DIR, ISSUES_FILE, WORKTREE_DIR};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn execute_git(args: &[&str], dir: &std::path::Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("Failed to execute git");
    if !status.success() {
        panic!("Git command failed: git {}", args.join(" "));
    }
}

fn setup_git_repo(path: &std::path::Path) {
    execute_git(&["init", "-b", "main"], path);
    execute_git(&["config", "user.email", "test@example.com"], path);
    execute_git(&["config", "user.name", "Test User"], path);
}

#[test]
fn test_worktree_path_generation() {
    let repo_root = PathBuf::from("/tmp/repo");
    let manager = WorktreeManager::new(repo_root.clone(), "pebble-sync".to_string());
    let expected = repo_root.join(WORKTREE_DIR);
    assert_eq!(manager.get_worktree_path(), expected);
}

#[test]
fn test_get_absolute_jsonl_path() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    setup_git_repo(&repo_root);

    std::fs::create_dir(repo_root.join(CONFIG_DIR)).unwrap();
    std::fs::write(repo_root.join(CONFIG_DIR).join("dummy"), "dummy").unwrap();
    execute_git(&["add", "."], &repo_root);
    execute_git(&["commit", "-m", "Initial"], &repo_root);

    let manager = WorktreeManager::new(repo_root.clone(), "pebble-sync".to_string());
    let expected = repo_root.join(WORKTREE_DIR).join(ISSUES_FILE);

    let path = manager
        .get_absolute_jsonl_path()
        .expect("Failed to get jsonl path");
    assert_eq!(path, expected);
    assert!(path.parent().unwrap().exists());
}

#[test]
fn test_ensure_worktree_creation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    setup_git_repo(&repo_root);

    std::fs::write(repo_root.join("README.md"), "Initial commit").unwrap();
    execute_git(&["add", "."], &repo_root);
    execute_git(&["commit", "-m", "Initial commit"], &repo_root);

    let manager = WorktreeManager::new(repo_root.clone(), "pebble-sync".to_string());

    let worktree_path = manager
        .ensure_worktree()
        .expect("Failed to ensure worktree");

    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(&worktree_path)
        .output()
        .expect("Failed to run git status in worktree");

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_sync_command_operations() {
    let remote_dir = TempDir::new().unwrap();
    let remote_root = remote_dir.path().to_path_buf();

    execute_git(&["init", "--bare"], &remote_root);

    let local_dir = TempDir::new().unwrap();
    let local_root = local_dir.path().to_path_buf();

    setup_git_repo(&local_root);

    std::fs::write(local_root.join("README.md"), "Initial content").unwrap();
    execute_git(&["add", "."], &local_root);
    execute_git(&["commit", "-m", "Initial"], &local_root);

    execute_git(
        &["remote", "add", "origin", remote_root.to_str().unwrap()],
        &local_root,
    );

    execute_git(&["checkout", "-b", "pebble-sync"], &local_root);
    execute_git(&["push", "-u", "origin", "pebble-sync"], &local_root);
    execute_git(&["checkout", "main"], &local_root);

    let manager = WorktreeManager::new(local_root.clone(), "pebble-sync".to_string());
    manager.sync().expect("Sync failed");

    let worktree_path = manager.get_worktree_path();
    assert!(worktree_path.exists());

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&worktree_path)
        .output()
        .expect("Failed to check branch");

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(branch == "HEAD" || branch == "pebble-sync");
}

#[test]
fn test_sync_branch_argument_injection_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    setup_git_repo(&repo_root);

    std::fs::write(repo_root.join("README.md"), "Initial commit").unwrap();
    execute_git(&["add", "."], &repo_root);
    execute_git(&["commit", "-m", "Initial commit"], &repo_root);

    let remote_dir = TempDir::new().unwrap();
    let remote_root = remote_dir.path().to_path_buf();
    execute_git(&["init", "--bare"], &remote_root);
    execute_git(
        &["remote", "add", "origin", remote_root.to_str().unwrap()],
        &repo_root,
    );

    let malicious_branch = "--unknown-option".to_string();
    let manager = WorktreeManager::new(repo_root.clone(), malicious_branch);

    let result = manager.sync();
    assert!(result.is_err());
    let debug_msg = format!("{:?}", result.unwrap_err());

    // Check if error is usage error (129) or fatal error (128/1/etc)
    // If 129 -> Vulnerable (git treated it as flag)
    if debug_msg.contains("129") {
        panic!(
            "VULNERABLE: Git fetch failed with usage error (129). This indicates argument injection."
        );
    }
}

struct MockGit {
    fail_fetch: bool,
    fail_merge: bool,
    fail_push: bool,
}

impl MockGit {
    fn new() -> Self {
        Self {
            fail_fetch: false,
            fail_merge: false,
            fail_push: false,
        }
    }
}

impl GitProvider for MockGit {
    fn run(&self, args: &[&dyn AsRef<OsStr>], _current_dir: &Path) -> Result<()> {
        let has_arg = |target: &str| args.iter().any(|a| a.as_ref() == target);

        if has_arg("fetch") && self.fail_fetch {
            return Err(color_eyre::eyre::eyre!("Simulated fetch failure"));
        }
        if has_arg("merge") && self.fail_merge {
            return Err(color_eyre::eyre::eyre!("Simulated merge failure"));
        }
        if has_arg("push") && self.fail_push {
            return Err(color_eyre::eyre::eyre!("Simulated push failure"));
        }
        Ok(())
    }

    fn output(&self, _args: &[&dyn AsRef<OsStr>], _current_dir: &Path) -> Result<String> {
        Ok(String::new())
    }

    fn status(
        &self,
        args: &[&dyn AsRef<OsStr>],
        _current_dir: &Path,
    ) -> Result<std::process::ExitStatus> {
        let has_arg = |target: &str| args.iter().any(|a| a.as_ref() == target);

        if has_arg("merge") && self.fail_merge {
            // Return a non-zero exit status (code 1 for conflict)
            // We use "false" command to get a status with code 1.
            return Ok(std::process::Command::new("false").status().unwrap());
        }
        Ok(std::process::Command::new("true").status().unwrap())
    }
}
#[test]
fn test_sync_failure_fetch() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    let mut mock_git = MockGit::new();
    mock_git.fail_fetch = true;

    let manager = WorktreeManager::new_with_git(repo_root, "sync-branch".to_string(), mock_git);

    let result = manager.sync();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to execute git fetch"));
}

#[test]
fn test_sync_failure_merge() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    let mut mock_git = MockGit::new();
    mock_git.fail_merge = true;

    let manager = WorktreeManager::new_with_git(repo_root, "sync-branch".to_string(), mock_git);

    let result = manager.sync();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to execute git merge"));
}

#[test]
fn test_sync_failure_push() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    let mut mock_git = MockGit::new();
    mock_git.fail_push = true;

    let manager = WorktreeManager::new_with_git(repo_root, "sync-branch".to_string(), mock_git);

    let result = manager.sync();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to execute git push"));
}
