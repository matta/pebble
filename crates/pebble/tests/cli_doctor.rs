#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use std::fs;

mod support;
use support::setup_test_env;

#[test]
fn test_doctor_healthy_graph() {
    let env = setup_test_env();
    let a = r#"+++
id = "A"
title = "A"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = []
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
        .arg("doctor")
        .assert()
        .success()
        .stdout("Graph is healthy. No issues found.\n")
        .stderr("");
}

#[test]
fn test_doctor_healthy_graph_json() {
    let env = setup_test_env();
    let a = r#"+++
id = "A"
title = "A"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = []
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
    let assert = cmd
        .current_dir(&env.root)
        .arg("doctor")
        .arg("--json")
        .assert()
        .success()
        .stderr("");

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let output: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert_eq!(output["ok"].as_bool(), Some(true));
    assert_eq!(
        output["errors"]
            .as_array()
            .expect("errors should be an array")
            .len(),
        0
    );
}

#[test]
fn test_doctor_finds_dangling_needs() {
    let env = setup_test_env();
    let a = r#"+++
id = "A"
title = "A"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = []
+++
"#;
    let b = r#"+++
id = "B"
title = "B"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = ["A", "MISSING_TASK"]
+++
"#;
    fs::write(env.tasks_dir.join("A.md"), a).expect("task file A.md should be written");
    fs::write(env.tasks_dir.join("B.md"), b).expect("task file B.md should be written");

    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(&env.root)
        .arg("doctor")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Dangling reference in 'needs': 'MISSING_TASK' not found",
        ));
}

#[test]
fn test_doctor_finds_unknown_keys() {
    let env = setup_test_env();
    let frontmatter = r#"+++
id = "issue-X"
title = "X"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = []
weird_key = "abc"
other_key = 123
+++
Body"#;
    fs::write(env.tasks_dir.join("X.md"), frontmatter).expect("task file X.md should be written");

    let mut cmd = Command::new(cargo_bin!("pebble"));
    let assert = cmd
        .current_dir(&env.root)
        .arg("doctor")
        .arg("--json")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let output: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert_eq!(output["ok"].as_bool(), Some(false));

    let errors = output["errors"]
        .as_array()
        .expect("errors should be an array");
    assert_eq!(errors.len(), 2);

    // Sort to make checks stable or just iterate
    let msgs: Vec<&str> = errors
        .iter()
        .filter_map(|e| e["message"].as_str())
        .collect();
    assert!(
        msgs.iter()
            .any(|m| m.contains("Unknown frontmatter key: 'weird_key'"))
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("Unknown frontmatter key: 'other_key'"))
    );
}
