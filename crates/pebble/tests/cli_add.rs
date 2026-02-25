mod support;

use assert_cmd::cargo_bin;
use serde_json::Value;
use std::process::Command;
use support::{setup_test_env, write_task};

#[test]
fn test_add_generates_id_with_lowercase_alphanumeric_suffix() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "New Task", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

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
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

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
            .expect("pebble command should execute successfully");
    }

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "New Task", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    let id = json["id"].as_str().expect("id should be a string");
    let random_part = id
        .strip_prefix("PROJ-")
        .expect("ID should start with PROJ-");

    assert!(
        random_part.len() >= 9,
        "Random ID length {} should be at least 9 for n=10",
        random_part.len()
    );
}

#[test]
fn test_add_blocks_updates_target_needs_with_new_task_id() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-target", "Target Task", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "add",
            "Precondition Task",
            "--blocks",
            "PROJ-target",
            "--json",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        output.status.success(),
        "pebble add with --blocks failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let created_stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let created_json: Value =
        serde_json::from_str(&created_stdout).expect("stdout should be valid JSON");
    let created_id = created_json["id"].as_str().expect("id should be present");

    let show_output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["show", "PROJ-target", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(show_output.status.success());
    let show_stdout = String::from_utf8(show_output.stdout).expect("stdout should be valid UTF-8");
    let show_json: Value = serde_json::from_str(&show_stdout).expect("stdout should be valid JSON");
    let needs = show_json["needs"]
        .as_array()
        .expect("needs should be an array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();

    assert!(
        needs.contains(&created_id),
        "Expected target needs to include new task ID {created_id}, got {needs:?}"
    );

    let created_needs = created_json["needs"]
        .as_array()
        .expect("created task needs should be an array");
    assert!(
        created_needs.is_empty(),
        "Created task should not gain reverse needs when using --blocks"
    );
}

#[test]
fn test_add_json_includes_blocking_when_blocks_is_used() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-target", "Target Task", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "add",
            "Precondition Task",
            "--blocks",
            "PROJ-target",
            "--json",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        output.status.success(),
        "pebble add with --blocks failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let created_stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let created_json: Value =
        serde_json::from_str(&created_stdout).expect("stdout should be valid JSON");
    let blocking = created_json["blocking"]
        .as_array()
        .expect("blocking should be an array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        blocking,
        vec!["PROJ-target"],
        "Expected created task JSON blocking to include the target task"
    );
}

#[test]
fn test_add_blocks_fails_for_unknown_target_id() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Precondition Task", "--blocks", "PROJ-missing"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        !output.status.success(),
        "Expected add --blocks with missing task to fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "Expected error about missing blocked target, got: {stderr}"
    );
}

#[test]
fn test_add_json_path_is_relative_to_current_working_directory() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Path Visibility Task", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    let path = json["path"].as_str().expect("path should be present");
    assert_eq!(
        path, "tasks/path-visibility-task.md",
        "Expected add --json path to be relative to current working directory"
    );
}

#[test]
fn test_add_human_output_uses_path_relative_to_current_working_directory() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Human Path Task"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains(" at tasks/human-path-task.md"),
        "Expected human add output to include cwd-relative path, got: {stderr}"
    );
    let root = env.root.display().to_string();
    assert!(
        !stderr.contains(&root),
        "Expected human add output to avoid absolute root path, got: {stderr}"
    );
}
