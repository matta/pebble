use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_init_sync_branch_starting_with_dash_fails() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Initialize a git repo first
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .status()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(root)
        .status()
        .unwrap();

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        // Use = to force clap to treat -bad as value, not flag
        .args(["init", "--sync-branch=-bad"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "sync-branch cannot start with '-'",
        ));
}
