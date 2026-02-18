use assert_cmd::Command;
use assert_cmd::cargo_bin;
use pebble::config::Config;
use pebble::CONFIG_DIR;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_uninitialized_error_message() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Run 'pebble list' in an empty directory
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("list")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error: Pebble is not initialized in this repository. Run 'pebble init' to get started."));
}

#[test]
fn test_uninitialized_error_message_add() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Run 'pebble add' in an empty directory
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["add", "test issue"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error: Pebble is not initialized in this repository. Run 'pebble init' to get started."));
}

#[test]
fn test_initialized_no_error() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    use std::fs;

    // Create a .pebble directory and a dummy config
    fs::create_dir(root.join(CONFIG_DIR)).unwrap();
    fs::write(
        Config::default_path(root),
        "sync-branch = \"pebble-sync\"\n",
    )
    .unwrap();

    // Run 'pebble list' - it should NOT show the initialization error
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: Pebble is not initialized in this repository. Run 'pebble init' to get started.").not());
}
