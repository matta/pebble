use assert_cmd::Command;
use assert_cmd::cargo_bin;
use pebble::command::CommandExt;
use pebble::config::Config;
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
