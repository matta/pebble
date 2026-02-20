use pebble::worktree::WorktreeManager;
use std::process::Command;
use tempfile::TempDir;

fn setup_git_repo(path: &std::path::Path) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .expect("git init failed");
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .expect("git config failed");
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .expect("git config failed");
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "initial"])
        .current_dir(path)
        .output()
        .expect("git commit failed");
}

#[test]
fn test_worktree_path_traversal_via_sync_branch() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().join("repo");
    std::fs::create_dir(&repo_root).unwrap();
    setup_git_repo(&repo_root);

    // Attempt to break out of the repo root using ".."
    // WORKTREE_DIR is .git/x-pebble
    // If sync_branch is "../../../hacked", the path becomes:
    // repo/.git/x-pebble/../../../hacked -> repo/../hacked -> temp_dir/hacked

    let malicious_branch = "../../../hacked".to_string();

    let manager = WorktreeManager::new(repo_root.clone(), malicious_branch);

    // ensure_worktree should now fail due to validation
    let result = manager.ensure_worktree();

    // Assert that it returns an error
    assert!(result.is_err(), "ensure_worktree should fail for invalid branch name");

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("sync-branch cannot contain '..'"), "Unexpected error message: {}", error_message);

    let hacked_path = temp_dir.path().join("hacked");

    // Assert that the directory does NOT exist.
    assert!(!hacked_path.exists(), "Directory created outside of expected location! Vulnerability confirmed.");
}
