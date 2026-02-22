use assert_cmd::Command;
use assert_cmd::cargo_bin;
use pebble::config::Config;
use pebble::worktree::generate_worktree_path;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_init_help() {
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Initialize a new Pebble repository",
        ));
}

#[test]
fn test_init_basic_execution() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Initialize a git repo
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
    // commit something so HEAD exists
    std::fs::write(root.join("README.md"), "test").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .status()
        .unwrap();

    // Run 'pebble init'
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Creating orphaned sync branch: pebble-data...",
        ))
        .stdout(predicate::str::contains("Pebble initialized successfully!"));
}

#[test]
fn test_init_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

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
    std::fs::write(root.join("README.md"), "test").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .status()
        .unwrap();

    let output = Command::new(cargo_bin!("pebble"))
        .current_dir(root)
        .args(["init", "--sync-branch", "json-sync", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let data: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    fn canonicalize_display(path: &std::path::Path) -> String {
        std::fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string()
    }

    fn canonicalize_value(value: &serde_json::Value) -> String {
        let raw = value.as_str().unwrap_or_default();
        canonicalize_display(std::path::Path::new(raw))
    }

    assert_eq!(data["sync_branch"], "json-sync");
    assert_eq!(
        canonicalize_value(&data["config_path"]),
        canonicalize_display(&Config::default_path(root))
    );
    assert_eq!(
        canonicalize_value(&data["worktree_path"]),
        canonicalize_display(&generate_worktree_path(root, "json-sync"))
    );
}

#[test]
fn test_init_fails_in_non_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Run 'pebble init' in a directory without git
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("init")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Error: 'pebble init' must be run inside a Git repository.",
        ));
}
