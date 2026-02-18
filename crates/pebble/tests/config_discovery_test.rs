use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_default_config_discovery_pebble_over_beads() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create both .pebble and .beads
    fs::create_dir(root.join(".pebble")).unwrap();
    fs::create_dir(root.join(".beads")).unwrap();

    fs::write(
        root.join(".pebble/config.yaml"),
        "sync-branch: pebble-sync
",
    )
    .unwrap();

    fs::write(
        root.join(".beads/config.yaml"),
        "sync-branch: beads-sync
",
    )
    .unwrap();

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pebble-sync"));
}

#[test]
fn test_default_config_discovery_beads_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create only .beads
    fs::create_dir(root.join(".beads")).unwrap();

    fs::write(
        root.join(".beads/config.yaml"),
        "sync-branch: beads-fallback-sync
",
    )
    .unwrap();

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("beads-fallback-sync"));
}
