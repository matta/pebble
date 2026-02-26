#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use std::fs;

mod support;
use support::setup_test_env;

#[test]
fn test_check_healthy_graph() {
    let env = setup_test_env();
    let a = r#"+++
id = "A"
title = "A"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = []
+++
"#;
    fs::write(env.tasks_dir.join("A.md"), a).expect("task file A.md should be written");

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(&env.root)
        .arg("check")
        .assert()
        .success()
        .stdout(
            "Graph is healthy. No issues found.
",
        )
        .stderr("");
}

#[test]
fn test_check_finds_unknown_keys_and_fails() {
    let env = setup_test_env();
    let frontmatter = r#"+++
id = "issue-X"
title = "X"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = []
weird_key = "abc"
+++
Body"#;
    fs::write(env.tasks_dir.join("X.md"), frontmatter).expect("task file X.md should be written");

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(&env.root)
        .arg("check")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Unknown frontmatter key: 'weird_key'",
        ));
}

#[test]
fn test_check_finds_dangling_needs_and_fails() {
    let env = setup_test_env();
    let b = r#"+++
id = "B"
title = "B"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = ["MISSING_TASK"]
+++
"#;
    fs::write(env.tasks_dir.join("B.md"), b).expect("task file B.md should be written");

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(&env.root)
        .arg("check")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Dangling reference in 'needs': 'MISSING_TASK' not found",
        ));
}

#[test]
fn test_check_detects_cycle_and_fails() {
    let env = setup_test_env();
    let a = r#"+++
id = "A"
title = "A"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = ["B"]
+++
"#;
    let b = r#"+++
id = "B"
title = "B"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = ["A"]
+++
"#;
    fs::write(env.tasks_dir.join("A.md"), a).expect("task file A.md should be written");
    fs::write(env.tasks_dir.join("B.md"), b).expect("task file B.md should be written");

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(&env.root)
        .arg("check")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Dependency cycle detected: A, B"));
}
