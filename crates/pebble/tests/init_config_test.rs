use assert_cmd::Command;
use assert_cmd::cargo_bin;
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
fn test_init_writes_config() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    setup_git_repo(root);

    let sync_branch = "pebble-sync-branch";

    // Run 'pebble init --sync-branch pebble-sync-branch'
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["init", "--sync-branch", sync_branch])
        .assert()
        .success();

    // 1. Verify .pebble/config.toml exists
    let config_path = root.join(".pebble/config.toml");
    assert!(config_path.exists(), "Config file should be created");

    // 2. Verify content
    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains(&format!("sync-branch = \"{}\"", sync_branch)));

    // 3. Verify 'pebble config get sync-branch' works
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicates::str::contains(sync_branch));
}
