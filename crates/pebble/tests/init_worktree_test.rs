use assert_cmd::Command;
use assert_cmd::cargo_bin;
use pebble::WORKTREE_DIR;
use std::process::Command as std_command;
use tempfile::TempDir;

fn setup_git_repo(path: &std::path::Path) {
    std_command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .status()
        .unwrap();
    std_command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .status()
        .unwrap();
    std_command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .status()
        .unwrap();

    // Create initial commit
    std::fs::write(path.join("README.md"), "Initial").unwrap();
    std_command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .status()
        .unwrap();
    std_command::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(path)
        .status()
        .unwrap();
}

#[test]
fn test_init_creates_worktree() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    setup_git_repo(root);

    let sync_branch = "pebble-sync-test";

    // Run 'pebble init --sync-branch pebble-sync-test'
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["init", "--sync-branch", sync_branch])
        .assert()
        .success();

    // 1. Verify worktree exists
    let pebble_dir = root.join(WORKTREE_DIR).join(sync_branch);
    assert!(pebble_dir.exists(), "Worktree directory should be created");

    // 2. Verify it's a worktree and on the correct branch
    let output = std_command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&pebble_dir)
        .output()
        .expect("Failed to run git rev-parse");
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(branch, sync_branch, "Worktree should be on the sync branch");

    // 3. Verify it's linked to the main repo
    let output = std_command::new("git")
        .args(["worktree", "list"])
        .current_dir(root)
        .output()
        .expect("Failed to run git worktree list");
    let worktree_list = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        worktree_list.contains(WORKTREE_DIR.split('/').next_back().unwrap()),
        "Worktree should be listed by git"
    );
}
