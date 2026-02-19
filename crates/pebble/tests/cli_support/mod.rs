use assert_cmd::Command;
use assert_cmd::cargo_bin;
use pebble::command::CommandExt;
use pebble::config::Config;
use pebble::worktree::generate_worktree_path;
use pebble::{CONFIG_DIR, ISSUES_FILE};
use std::path::PathBuf;
use tempfile::TempDir;

use crate::common::TEST_SYNC_BRANCH;

pub struct TestEnv {
    _temp_dir: TempDir,
    root: PathBuf,
}

impl TestEnv {
    pub fn setup() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        std::process::Command::new("git")
            .arg("init")
            .arg("-b")
            .arg("main")
            .current_dir(&root)
            .check_run()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&root)
            .check_run()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&root)
            .check_run()
            .unwrap();

        std::fs::create_dir(root.join(CONFIG_DIR)).unwrap();
        std::fs::write(
            Config::default_path(&root),
            format!("sync-branch = \"{}\"\n", TEST_SYNC_BRANCH),
        )
        .unwrap();

        std::fs::write(root.join("README.md"), "test").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&root)
            .check_run()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&root)
            .check_run()
            .unwrap();

        std::process::Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg(TEST_SYNC_BRANCH)
            .current_dir(&root)
            .check_run()
            .unwrap();
        std::fs::write(root.join(ISSUES_FILE), "").unwrap();
        std::process::Command::new("git")
            .arg("add")
            .arg(root.join(ISSUES_FILE))
            .current_dir(&root)
            .check_run()
            .unwrap();
        std::process::Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("sync init")
            .current_dir(&root)
            .check_run()
            .unwrap();

        std::process::Command::new("git")
            .args(["checkout", "main"])
            .current_dir(&root)
            .check_run()
            .unwrap();

        Self {
            _temp_dir: temp_dir,
            root,
        }
    }

    pub fn get_worktree_path(&self) -> PathBuf {
        generate_worktree_path(&self.root, TEST_SYNC_BRANCH)
    }

    pub fn add_issue_to_worktree(&self, issue: &serde_json::Value) {
        let worktree_path = self.get_worktree_path();
        std::fs::create_dir_all(&worktree_path).unwrap();
        let issues_path = worktree_path.join(ISSUES_FILE);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&issues_path)
            .unwrap();

        let json = serde_json::to_string(issue).unwrap();
        use std::io::Write;
        writeln!(file, "{}", json).unwrap();
    }

    pub fn pebble(&self) -> Command {
        let mut cmd = Command::new(cargo_bin!("pebble"));
        cmd.current_dir(&self.root);
        cmd
    }

    #[allow(dead_code)]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}

pub fn create_test_issue(id: &str, title: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": title,
        "status": "open",
        "priority": 0,
        "issue_type": "task",
        "owner": "test@example.com",
        "created_at": "2026-01-01T00:00:00Z",
        "created_by": "Tester",
        "updated_at": "2026-01-01T00:00:00Z",
        "description": "A test fixture issue",
        "closed_at": null,
        "close_reason": null
    })
}
