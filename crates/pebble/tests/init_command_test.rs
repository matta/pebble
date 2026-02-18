use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_init_help() {
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize a new Pebble repository"));
}

#[test]
fn test_init_basic_execution() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Run 'pebble init' - currently just a skeleton
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing pebble..."));
}
