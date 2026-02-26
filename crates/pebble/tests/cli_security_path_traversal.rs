use assert_cmd::Command;
use std::fs;

#[test]
#[expect(clippy::expect_used, reason = "test assertions rely on panics")]
fn test_list_path_traversal_prevention() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let root = temp.path();

    // Create a legitimate project directory
    let project_dir = root.join("project");
    fs::create_dir(&project_dir).expect("project dir should be created");

    // Initialize legitimate project
    #[allow(deprecated)]
    let mut init_cmd = Command::cargo_bin("pebble").expect("pebble binary");
    init_cmd
        .current_dir(&project_dir)
        .arg("init")
        .assert()
        .success();

    // Create a directory outside the project with a fake task
    let outside_dir = root.join("outside");
    fs::create_dir(&outside_dir).expect("outside dir should be created");

    // Create a fake task in the outside directory
    let outside_task = outside_dir.join("secret-task.md");
    let task_content = r#"+++
id = "SECRET-1"
title = "Secret Task"
status = "todo"
created_at = 2024-01-01T00:00:00Z
needs = []
tags = []
[extra]
+++

This is a secret task outside the project.
"#;
    fs::write(&outside_task, task_content).expect("should write secret task");

    // Try to list tasks from the outside directory using path traversal
    #[allow(deprecated)]
    let mut list_cmd = Command::cargo_bin("pebble").expect("pebble binary");
    let output = list_cmd
        .current_dir(&project_dir)
        .args(["list", "--dir", "../outside"])
        .output()
        .expect("pebble list should execute");

    // Should FAIL if protection is in place.
    // If it succeeds (exit code 0), then the vulnerability exists.
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Secret Task") {
            panic!(
                "Vulnerability confirmed: 'pebble list --dir ../outside' accessed external files!"
            );
        }
    } else {
        // If it failed, check the error message
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("tasks-dir") && stderr.contains("parent directory components") {
            // This is the desired behavior (once fixed)
        }
    }
}
