mod support;

use assert_cmd::cargo_bin;
use serde_json::Value;
use std::process::Command;
use support::setup_test_env;

#[test]
fn test_add_generates_id_with_lowercase_alphanumeric_suffix() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "New Task", "--json"])
        .output()
        .expect("Failed to execute pebble add");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let id = json["id"].as_str().expect("id should be a string");

    // Format should be PROJ-<random_part>
    let random_part = id
        .strip_prefix("PROJ-")
        .expect("ID should start with PROJ-");

    // Check alphabet: a-z0-9
    for c in random_part.chars() {
        assert!(
            c.is_ascii_lowercase() || c.is_ascii_digit(),
            "Char '{}' in random part '{}' is not a-z0-9",
            c,
            random_part
        );
    }
}

#[test]
fn test_add_id_suffix_length_is_at_least_8() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "New Task", "--json"])
        .output()
        .expect("Failed to execute pebble add");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let id = json["id"].as_str().expect("id should be a string");
    let random_part = id
        .strip_prefix("PROJ-")
        .expect("ID should start with PROJ-");

    // Based on collision probability < 1e-12, length should be at least 8 for n=1
    assert!(
        random_part.len() >= 8,
        "Random ID length {} is too short",
        random_part.len()
    );
}

#[test]
fn test_add_id_suffix_length_scales_with_task_count() {
    let env = setup_test_env();

    // Generate 10 tasks to push n to 10
    // log36(10^2 * 1e12 / 2) = log36(5e13) \approx 8.8 -> length 9
    for i in 0..10 {
        Command::new(cargo_bin!())
            .current_dir(&env.root)
            .args(["add", &format!("Task {}", i), "--dir", "tasks"])
            .output()
            .unwrap();
    }

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "New Task", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();
    let random_part = id
        .strip_prefix("PROJ-")
        .expect("ID should start with PROJ-");

    assert!(
        random_part.len() >= 9,
        "Random ID length {} should be at least 9 for n=10",
        random_part.len()
    );
}
