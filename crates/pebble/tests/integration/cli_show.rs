#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]

use super::support::{setup_test_env, write_task};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_show_path_only_relative() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-1", "First Task", "todo");

    env.pebble()
        .args(["show", "PROJ-1", "--path-only"])
        .assert()
        .success()
        .stdout(predicate::eq("PROJ-1.md\n"));
}

#[test]
fn test_show_formatted_output() {
    let env = setup_test_env();

    let task_content = r#"---
id: "PROJ-2"
title: "A Formatted Task"
status: "todo"
needs: []
tags: ["frontend"]
created_at: "2024-01-01T00:00:00Z"
---
Body text explaining the task.
"#;
    fs::write(env.tasks_dir.join("PROJ-2.md"), task_content).expect("task file should be written");

    env.pebble()
        .args(["show", "PROJ-2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task: A Formatted Task (PROJ-2)"))
        .stdout(predicate::str::contains("Status: Todo"))
        .stdout(predicate::str::contains("Tags: [\"frontend\"]"))
        .stdout(predicate::str::contains("Body text explaining the task."));
}
