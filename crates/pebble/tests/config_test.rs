use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_custom_config_path_flag() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let custom_config = root.join("my-config.yaml");

    fs::write(
        &custom_config,
        "sync-branch: custom-sync\nissue-prefix: custom\n",
    )
    .unwrap();

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("--config")
        .arg(&custom_config)
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-sync"));
}

#[test]
fn test_custom_config_path_env() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let custom_config = root.join("env-config.yaml");

    fs::write(&custom_config, "sync-branch: env-sync\nissue-prefix: env\n").unwrap();

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .env("PEBBLE_CONFIG", &custom_config)
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("env-sync"));
}

#[test]
fn test_custom_config_path_relative() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let custom_config = root.join("rel-config.yaml");

    fs::write(&custom_config, "sync-branch: rel-sync\nissue-prefix: rel\n").unwrap();

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .arg("-c")
        .arg("rel-config.yaml")
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rel-sync"));
}
