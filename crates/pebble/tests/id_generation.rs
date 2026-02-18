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
    // With N=1 issue (including this one), the dynamic length calculation:
    // P=1.0e-12, k=1.0
    // required_pool_size = 1.0 / 2.0e-12 = 5.0e11
    // length = log36(5.0e11) = log10(5.0e11)/log10(36) = 11.69/1.55 ~= 7.5
    // ceil(7.5) = 8
    // wait, population is BEFORE adding the new one.
    // In test:
    // 1. `add "Test Issue"` -> issues.len() is 0. suffix_len = 1.
    //
    // Let's re-verify the logic in `add.rs`:
    // let existing_issues = store.read_issues()?;
    // let suffix_length = recommended_id_length(existing_issues.len() as u64);
    //
    // When adding the FIRST issue, len is 0.
    // recommended_id_length(0) returns 1.
    // So the suffix should be 1 char long.
    assert_eq!(
        parts[1].len(),
        1,
        "ID suffix should be 1 char for the first issue"
    );
}
