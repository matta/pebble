use assert_cmd::Command;
use assert_cmd::cargo_bin;
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

        // git config
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&root)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
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

    fn add_issue_to_worktree(&self, issue: &serde_json::Value) {
        let worktree_path = self.root.join(".git/beads-worktrees/beads-sync");
        let issues_dir = worktree_path.join(".beads");
        std::fs::create_dir_all(&issues_dir).unwrap();
        let issues_path = issues_dir.join("issues.jsonl");

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&issues_path)
            .unwrap();

        let json = serde_json::to_string(issue).unwrap();
        use std::io::Write;
        writeln!(file, "{}", json).unwrap();
    }

    fn pebble(&self) -> Command {
        let mut cmd = Command::new(cargo_bin!("pebble"));
        cmd.current_dir(&self.root);
        cmd
    }
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::new(cargo_bin!("pebble"));
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
    let mut cmd = Command::new(cargo_bin!("pebble"));
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
        .stdout(predicate::str::contains("Using database:"))
        .stdout(predicate::str::contains("No issues found."));
}

#[test]
fn test_list_issues_with_data() {
    let env = TestEnv::setup();
    let issue = serde_json::json!({
        "id": "test-123",
        "title": "Fixture Issue",
        "status": "open",
        "priority": 0,
        "issue_type": "task",
        "owner": "test@example.com",
        "created_at": "2026-01-01T00:00:00Z",
        "created_by": "Tester",
        "updated_at": "2026-01-01T00:00:00Z",
        "description": "A test fixture issue"
    });
    env.add_issue_to_worktree(&issue);

    env.pebble()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Using database:"))
        .stdout(predicate::str::contains("test-123 [open] Fixture Issue"));
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
fn test_show_non_existent_issue() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["show", "non-existent-id"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Issue non-existent-id not found"));
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

#[test]
fn test_edit_non_existent_issue() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["edit", "non-existent-id", "--title", "New Title"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Issue non-existent-id not found"));
}

#[test]
fn test_directory_flag() {
    let env = TestEnv::setup();
    let temp_dir = TempDir::new().unwrap();

    // Run pebble from a completely different directory, pointing to env.root with -C
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(temp_dir.path())
        .arg("-C")
        .arg(&env.root)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Using database:"))
        .stdout(predicate::str::contains("No issues found."));
}
