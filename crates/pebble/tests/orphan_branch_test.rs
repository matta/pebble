use pebble::worktree::WorktreeManager;
use std::process::Command;
use tempfile::TempDir;

fn setup_git_repo(path: &std::path::Path) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .status()
        .unwrap();
    
    // Create initial commit
    std::fs::write(path.join("README.md"), "Initial").unwrap();
    Command::new("git").args(["add", "."]).current_dir(path).status().unwrap();
    Command::new("git").args(["commit", "-m", "Initial"]).current_dir(path).status().unwrap();
}

#[test]
fn test_create_orphaned_branch() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    setup_git_repo(root);
    
    let sync_branch = "pebble-data";
    let manager = WorktreeManager::new(root.to_path_buf(), sync_branch.to_string());
    
    manager.create_orphaned_sync_branch().expect("Failed to create orphaned branch");
    
    // Verify the branch exists
    let output = Command::new("git")
        .args(["rev-parse", "--verify", sync_branch])
        .current_dir(root)
        .output()
        .expect("Failed to run git rev-parse");
    assert!(output.status.success(), "Branch should exist");
    
    // Verify the commit message
    let output = Command::new("git")
        .args(["log", "--format=%s", "-n", "1", sync_branch])
        .current_dir(root)
        .output()
        .expect("Failed to run git log");
    let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(message, "Pebble database tracking branch initial commit");
}
