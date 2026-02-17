use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

struct TestEnv {
    _temp_dir: TempDir,
    root: PathBuf,
}

impl TestEnv {
    fn setup() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // git init
        std::process::Command::new("git")
            .arg("init")
            .arg("-b")
            .arg("main")
            .current_dir(&root)
            .status()
            .unwrap();

        // .beads/config.yaml
        std::fs::create_dir(root.join(".beads")).unwrap();
        std::fs::write(
            root.join(".beads/config.yaml"),
            "sync-branch: beads-sync\nissue-prefix: test\n",
        )
        .unwrap();

        // initial commit
        std::fs::write(root.join("README.md"), "test").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&root)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&root)
            .status()
            .unwrap();

        // Create sync branch with issues.jsonl
        std::process::Command::new("git")
            .args(["checkout", "-b", "beads-sync"])
            .current_dir(&root)
            .status()
            .unwrap();
        std::fs::write(root.join(".beads/issues.jsonl"), "").unwrap();
        std::process::Command::new("git")
            .args(["add", ".beads/issues.jsonl"])
            .current_dir(&root)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "sync init"])
            .current_dir(&root)
            .status()
            .unwrap();

        // switch back to main
        std::process::Command::new("git")
            .args(["checkout", "main"])
            .current_dir(&root)
            .status()
            .unwrap();

        Self {
            _temp_dir: temp_dir,
            root,
        }
    }

    fn pebble(&self) -> Command {
        let mut cmd = Command::cargo_bin("pebble").unwrap();
        cmd.current_dir(&self.root);
        cmd
    }
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("pebble").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pebble 0.1.0"));
}

#[test]
fn test_config_get_sync_branch() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("beads-sync"));
}

#[test]
fn test_sync_fail_no_config() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pebble").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sync")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read config file"));
}

#[test]
fn test_list_issues_empty() {
    let env = TestEnv::setup();
    env.pebble()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found."));
}

#[test]
fn test_add_issue() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["add", "New Test Issue", "--description", "This is a test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added issue test-"));
}

#[test]
fn test_add_and_show_issue() {
    let env = TestEnv::setup();
    // 1. Add issue
    let output = env
        .pebble()
        .args(["add", "Show Test Issue", "--description", "Showing this"])
        .output()
        .expect("Failed to run add");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .last()
        .expect("Failed to get ID from output");

    // 2. Show issue
    env.pebble()
        .args(["show", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test Issue"))
        .stdout(predicate::str::contains("Showing this"));
}

#[test]
fn test_edit_issue() {
    let env = TestEnv::setup();
    // 1. Add issue
    let output = env
        .pebble()
        .args(["add", "Edit Test Issue"])
        .output()
        .expect("Failed to run add");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .last()
        .expect("Failed to get ID from output");

    // 2. Edit issue
    env.pebble()
        .args(["edit", id, "--title", "New Edited Title"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated issue"));

    // 3. Verify edit
    env.pebble()
        .args(["show", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("New Edited Title"));
}
