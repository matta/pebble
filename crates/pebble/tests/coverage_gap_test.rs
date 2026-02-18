use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_config_get_issue_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // 1. Initialize Git
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .assert()
        .success();

    // 2. Initialize Pebble
    let mut cmd = Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.current_dir(root).arg("init").assert().success();

    // 3. Set a custom prefix in config
    let config_path = root.join(".pebble/config.toml");
    std::fs::write(
        config_path,
        r#"issue-prefix = "PROJ"
sync-branch = "pebble-data"
"#,
    )
    .unwrap();

    // 4. Test 'pebble config get issue-prefix'
    let mut cmd = Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["config", "get", "issue-prefix"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PROJ"));
}

#[test]
fn test_import_no_changes() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // 1. Initialize Git & Pebble
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .assert()
        .success();
    let mut cmd = Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.current_dir(root).arg("init").assert().success();

    // 2. Create an issue
    let mut cmd = Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["add", "Test Issue"])
        .assert()
        .success();

    // 3. Get the JSONL content and write to an external file
    let issues_path = pebble::worktree::generate_worktree_path(root, "pebble-data").join("issues.jsonl");
    let content = std::fs::read_to_string(issues_path).unwrap();
    let ext_path = root.join("ext.jsonl");
    std::fs::write(&ext_path, content).unwrap();

    // 4. Import the same file - should report "No changes"
    let mut cmd = Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["import", ext_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Import complete: No changes."));
}

#[test]
fn test_config_validation_fail() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // 1. Initialize Git & Pebble
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .assert()
        .success();
    let mut cmd = Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.current_dir(root).arg("init").assert().success();

    // 2. Create an INVALID config (missing sync-branch)
    let config_path = root.join(".pebble/config.toml");
    std::fs::write(
        config_path,
        r#"issue-prefix = "PROJ"
"#,
    )
    .unwrap();

    // 3. Any command should fail validation
    let mut cmd = Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "sync-branch is required in configuration",
        ));
}
