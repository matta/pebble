use assert_cmd::Command;
#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use pebble::{CONFIG_DIR, CONFIG_FILE};
use std::fs;
use tempfile::TempDir;

#[test]
#[allow(deprecated)]
fn test_id_generation_length() {
    let temp = TempDir::new().unwrap();
    let path = temp.path();

    // Initialize git repo
    let status = std::process::Command::new("git")
        .current_dir(path)
        .arg("init")
        .status()
        .unwrap();
    assert!(status.success());

    // Configure git user
    let status = std::process::Command::new("git")
        .current_dir(path)
        .args(["config", "user.name", "Test User"])
        .status()
        .unwrap();
    assert!(status.success());

    let status = std::process::Command::new("git")
        .current_dir(path)
        .args(["config", "user.email", "test@example.com"])
        .status()
        .unwrap();
    assert!(status.success());

    // Create config file and directory BEFORE commit
    let config_dir = path.join(CONFIG_DIR);
    fs::create_dir(&config_dir).unwrap();
    fs::write(config_dir.join(CONFIG_FILE), "sync-branch = \"main\"\n").unwrap();

    // Create an initial commit including .pebble
    fs::write(path.join("README.md"), "# Test Repo").unwrap();
    let status = std::process::Command::new("git")
        .current_dir(path)
        .args(["add", "."])
        .status()
        .unwrap();
    assert!(status.success());
    let status = std::process::Command::new("git")
        .current_dir(path)
        .args(["commit", "-m", "Initial commit"])
        .status()
        .unwrap();
    assert!(status.success());

    // Ensure main branch exists if init created master
    let status = std::process::Command::new("git")
        .current_dir(path)
        .args(["branch", "-m", "main"])
        .status()
        .unwrap();
    assert!(status.success());

    // Add an issue
    let mut cmd = Command::new(cargo_bin("pebble"));
    cmd.current_dir(path)
        .arg("add")
        .arg("Test Issue")
        .assert()
        .success();

    // List issues
    let mut cmd = Command::new(cargo_bin("pebble"));
    let assert = cmd
        .current_dir(path)
        .arg("list")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let issues: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();

    assert_eq!(issues.len(), 1);
    let id = issues[0]["id"].as_str().unwrap();

    // Expected format: issue-XXXXXX
    assert!(id.starts_with("issue-"));
    let parts: Vec<&str> = id.split('-').collect();
    assert_eq!(parts.len(), 2);
    // New behavior is 12 chars
    assert_eq!(parts[1].len(), 12, "ID suffix should be 12 chars");
}
