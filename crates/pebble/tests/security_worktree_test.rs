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
fn test_init_worktree_argument_injection() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();
    setup_git_repo(&repo_root);

    // "-f" acts as --force flag if injected.
    // If injected, git worktree add <path> -f -> Succeeds (creates worktree from HEAD)
    // If protected, git worktree add <path> -- -f -> Fails (invalid branch "-f")
    let malicious_branch = "-f".to_string();
    let manager = WorktreeManager::new(repo_root.clone(), malicious_branch);

    let wt_path = repo_root.join("wt_test");

    let result = manager.init_worktree(&wt_path);

    assert!(
        result.is_err(),
        "init_worktree should fail for branch '-f', but it succeeded (likely injection)"
    );
}
