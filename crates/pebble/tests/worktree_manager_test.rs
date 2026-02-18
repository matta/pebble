use pebble::worktree::WorktreeManager;
use std::path::PathBuf;
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
    let expected = repo_root.join(".git/x-pebble");
    assert_eq!(manager.get_worktree_path(), expected);
}

#[test]
fn test_get_absolute_jsonl_path() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();

    setup_git_repo(&repo_root);

    std::fs::create_dir(repo_root.join(".beads")).unwrap();
    std::fs::write(repo_root.join(".beads/dummy"), "dummy").unwrap();
    execute_git(&["add", "."], &repo_root);
    execute_git(&["commit", "-m", "Initial"], &repo_root);

    let manager = WorktreeManager::new(repo_root.clone(), "pebble-sync".to_string());
    let expected = repo_root.join(".git/x-pebble/issues.jsonl");

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
