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
fn test_ensure_worktree_path_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path().to_path_buf();
    setup_git_repo(&repo_root);

    // Try to break out of the repo directory
    // We use a relative path that goes up and into a new dir inside temp_dir to keep it clean
    let unique_name = "pwned_dir";
    let malicious_dir = temp_dir.path().join(unique_name);
    // construct relative path from .git/x-pebble (which is inside repo_root) to malicious_dir
    // repo_root/.git/x-pebble -> malicious_dir
    // .git/x-pebble is 2 levels deep from repo_root.
    // So ../../ goes to repo_root.
    // Then ../ goes to temp_dir (parent of repo_root since repo_root IS temp_dir path).
    // Wait, repo_root = temp_dir.path().
    // So repo_root/.git/x-pebble is temp_dir/.git/x-pebble.
    // ../../ is temp_dir.
    // So we need ../../unique_name to target temp_dir/unique_name.

    let malicious_branch = format!("../../{}", unique_name);
    let manager = WorktreeManager::new(repo_root.clone(), malicious_branch);

    // This should fail
    let result = manager.ensure_worktree();

    assert!(
        result.is_err(),
        "ensure_worktree should return error for invalid branch"
    );

    // BUT CRITICALLY: It should NOT have created the directory
    assert!(
        !malicious_dir.exists(),
        "Worktree directory was created despite invalid branch name! Path traversal vulnerability confirmed."
    );
}
