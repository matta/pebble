#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use std::fs;
use std::process::Output;

use super::support::{TestEnv, setup_test_env};

fn run_fix(env: &TestEnv, json: bool) -> Output {
    let mut cmd = env.pebble();
    cmd.arg("check").arg("--fix");
    if json {
        cmd.arg("--json");
    }
    cmd.output()
        .expect("pebble check --fix command should execute")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn write_file(env: &TestEnv, name: &str, body: &str) {
    fs::write(env.tasks_dir.join(name), body).expect("task file should be written");
}

#[test]
fn test_fix_backfills_created_at() {
    let env = setup_test_env();
    // Task missing created_at
    let content = r#"---
id: "A"
title: "A"
status: "todo"
---
Body"#;
    write_file(&env, "A.md", content);

    let output = run_fix(&env, false);
    assert!(
        output.status.success(),
        "fix should succeed even if it modifies files"
    );
    assert_eq!(stdout_text(&output), "Fixed 1 task(s).\n");

    let updated_content =
        fs::read_to_string(env.tasks_dir.join("A.md")).expect("file should be readable");
    assert!(
        updated_content.contains("created_at:"),
        "created_at should be backfilled"
    );
}

#[test]
fn test_fix_warns_on_unknown_keys_but_preserves_them() {
    let env = setup_test_env();
    let content = r#"---
id: "A"
title: "A"
status: "todo"
created_at: "2026-03-01T00:00:00Z"
weird_key: "abc"
---
Body"#;
    write_file(&env, "A.md", content);

    let output = run_fix(&env, false);
    assert_eq!(output.status.code(), Some(1));

    let stderr = stderr_text(&output);
    assert!(stderr.contains("Unknown frontmatter key: 'weird_key'"));

    let updated_content =
        fs::read_to_string(env.tasks_dir.join("A.md")).expect("file should be readable");
    assert!(
        updated_content.contains("weird_key: abc"),
        "unknown keys should be preserved"
    );
}

#[test]
fn test_fix_json_output() {
    let env = setup_test_env();
    let content = r#"---
id: "A"
title: "A"
status: "todo"
---
Body"#;
    write_file(&env, "A.md", content);

    let output = run_fix(&env, true);
    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("file should be readable");
    assert_eq!(json["ok"].as_bool(), Some(true));
    assert_eq!(
        json["fixed_tasks"].as_array().map(|items| items.len()),
        Some(1)
    );
    assert_eq!(json["errors"].as_array().map(|items| items.len()), Some(0));
}

#[test]
fn test_fix_json_still_warns_on_unknown_keys_to_stderr() {
    let env = setup_test_env();
    let content = r#"---
id: "A"
title: "A"
status: "todo"
created_at: "2026-03-01T00:00:00Z"
weird_key: "abc"
---
Body"#;
    write_file(&env, "A.md", content);

    let output = run_fix(&env, true);
    assert_eq!(output.status.code(), Some(1));

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(json["ok"].as_bool(), Some(false));
    assert_eq!(
        json["fixed_tasks"].as_array().map(|items| items.len()),
        Some(1)
    );
    assert_eq!(json["errors"].as_array().map(|items| items.len()), Some(1));

    let stderr = stderr_text(&output);
    assert!(stderr.contains("Unknown frontmatter key: 'weird_key'"));
}

#[test]
fn test_fix_fails_when_non_repairable_findings_remain() {
    let env = setup_test_env();
    let content = r#"---
id: "A"
title: "A"
status: "todo"
created_at: "2026-03-01T00:00:00Z"
needs: ["MISSING_TASK"]
---
Body"#;
    write_file(&env, "A.md", content);

    let output = run_fix(&env, false);
    assert_eq!(output.status.code(), Some(1));

    let stderr = stderr_text(&output);
    assert!(stderr.contains("Dangling reference in 'needs': 'MISSING_TASK' not found"));
}
