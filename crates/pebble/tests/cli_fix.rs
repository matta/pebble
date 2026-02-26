#![expect(clippy::expect_used, reason = "Test code uses expect for assertions")]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[path = "./support.rs"]
mod support;
use support::*;

#[test]
#[allow(deprecated)]
fn test_fix_backfills_missing_created_at() {
    let test_env = setup_test_env();

    let task_path = test_env.tasks_dir.join("task.md");
    let content = r#"+++
id = "issue-1"
title = "Task without created_at"
status = "todo"
+++

Body content"#;

    fs::write(&task_path, content).expect("Failed to write task file");

    let mut cmd = Command::cargo_bin("pebble").expect("Failed to find binary");
    cmd.current_dir(&test_env.root)
        .arg("fix")
        .assert()
        .success();

    let new_content = fs::read_to_string(&task_path).expect("Failed to read task file");
    assert!(
        new_content.contains("created_at ="),
        "Should have backfilled created_at"
    );
}

#[test]
#[allow(deprecated)]
fn test_fix_warns_on_unknown_keys_but_preserves_them() {
    let test_env = setup_test_env();

    let task_path = test_env.tasks_dir.join("task_unknown.md");
    let content = r#"+++
id = "issue-2"
title = "Task with unknown key"
status = "todo"
created_at = 2023-01-01T00:00:00Z
unknown_key = "value"
+++

Body content"#;

    fs::write(&task_path, content).expect("Failed to write task file");

    let mut cmd = Command::cargo_bin("pebble").expect("Failed to find binary");
    cmd.current_dir(&test_env.root)
        .arg("fix")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Warning: Unknown frontmatter key 'unknown_key'",
        ));

    let new_content = fs::read_to_string(&task_path).expect("Failed to read task file");
    assert!(
        new_content.contains("unknown_key = \"value\""),
        "Should preserve unknown key"
    );
}

#[test]
#[allow(deprecated)]
fn test_fix_rewrites_formatting() {
    let test_env = setup_test_env();

    let task_path = test_env.tasks_dir.join("task_format.md");
    // Badly formatted TOML (but valid)
    let content = r#"+++
id="issue-3"
title="Bad Format"
status="todo"
created_at=2023-01-01T00:00:00Z
+++

Body"#;

    fs::write(&task_path, content).expect("Failed to write task file");

    let mut cmd = Command::cargo_bin("pebble").expect("Failed to find binary");
    cmd.current_dir(&test_env.root)
        .arg("fix")
        .assert()
        .success();

    let new_content = fs::read_to_string(&task_path).expect("Failed to read task file");
    // Check if it looks pretty-printed (e.g. spaces around =)
    assert!(
        new_content.contains("id = \"issue-3\""),
        "Should normalize TOML formatting"
    );
}
