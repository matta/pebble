#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use assert_cmd::Command;

use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Output;

mod support;
use support::{TestEnv, setup_test_env};

#[derive(Clone, Copy, Debug)]
enum CheckMode {
    Strict,
    WarnOnly,
}

impl CheckMode {
    fn label(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::WarnOnly => "warn_only",
        }
    }

    fn apply_args(self, cmd: &mut Command) {
        cmd.arg("check");
        if let Self::WarnOnly = self {
            cmd.arg("--warn-only");
        }
    }

    fn expects_failure_on_issues(self) -> bool {
        matches!(self, Self::Strict)
    }
}

const CHECK_MODES: [CheckMode; 2] = [CheckMode::Strict, CheckMode::WarnOnly];

fn run_check(mode: CheckMode, root: &Path, json: bool) -> Output {
    let mut cmd = support::pebble(root);
    mode.apply_args(&mut cmd);
    if json {
        cmd.arg("--json");
    }
    cmd.output().expect("pebble check command should execute")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn write_file(env: &TestEnv, name: &str, body: &str) {
    fs::write(env.tasks_dir.join(name), body).expect("task file should be written");
}

fn assert_issue_exit(mode: CheckMode, output: &Output) {
    if mode.expects_failure_on_issues() {
        assert_eq!(
            output.status.code(),
            Some(1),
            "expected non-zero exit for {} mode",
            mode.label()
        );
    } else {
        assert!(
            output.status.success(),
            "expected success exit for {} mode",
            mode.label()
        );
    }
}

fn assert_issue_stderr(
    mode: CheckMode,
    stderr: &str,
    expected_messages: &[&str],
    expected_issue_count: usize,
) {
    assert!(
        stderr.contains("Graph is unhealthy."),
        "expected overall unhealthy summary for {} mode; stderr was: {}",
        mode.label(),
        stderr
    );
    for message in expected_messages {
        assert!(
            stderr.contains(message),
            "missing message '{}' for {} mode; stderr was: {}",
            message,
            mode.label(),
            stderr
        );
    }
    assert!(
        stderr.contains(&format!("Found {} issue(s).", expected_issue_count)),
        "expected issue count summary for {} mode; stderr was: {}",
        mode.label(),
        stderr
    );

    if mode.expects_failure_on_issues() {
        assert!(stderr.contains("Runtime error:"));
        assert!(stderr.contains("Check failed: graph has issues."));
    } else {
        assert!(!stderr.contains("Runtime error:"));
        assert!(!stderr.contains("Check failed: graph has issues."));
    }
}

fn parse_json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn setup_healthy_graph(env: &TestEnv) {
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
    write_file(env, "A.md", a);
    write_file(env, "B.md", b);
}

fn setup_unknown_keys_graph(env: &TestEnv) {
    let x = r#"+++
id = "issue-X"
title = "X"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = []
weird_key = "abc"
other_key = 123
+++
Body"#;
    write_file(env, "X.md", x);
}

fn setup_dangling_need_graph(env: &TestEnv) {
    let b = r#"+++
id = "B"
title = "B"
status = "todo"
created_at = 2026-03-01T00:00:00Z
needs = ["MISSING_TASK"]
+++
"#;
    write_file(env, "B.md", b);
}

fn setup_cycle_graph(env: &TestEnv) {
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
    write_file(env, "A.md", a);
    write_file(env, "B.md", b);
}

fn setup_missing_created_at_graph(env: &TestEnv) {
    let a = r#"+++
id = "A"
title = "A"
status = "todo"
needs = []
+++
"#;
    write_file(env, "A.md", a);
}

#[test]
fn test_check_modes_healthy_human_output() {
    for mode in CHECK_MODES {
        let env = setup_test_env();
        setup_healthy_graph(&env);

        let output = run_check(mode, &env.root, false);
        assert!(output.status.success(), "mode {}", mode.label());
        assert_eq!(stdout_text(&output), "Graph is healthy. No issues found.\n");
        assert_eq!(stderr_text(&output), "");
    }
}

#[test]
fn test_check_modes_healthy_json_output_is_identical() {
    let env_warn = setup_test_env();
    setup_healthy_graph(&env_warn);
    let warn_output = run_check(CheckMode::WarnOnly, &env_warn.root, true);

    let env_strict = setup_test_env();
    setup_healthy_graph(&env_strict);
    let strict_output = run_check(CheckMode::Strict, &env_strict.root, true);

    assert!(warn_output.status.success());
    assert!(strict_output.status.success());
    assert_eq!(stderr_text(&warn_output), "");
    assert_eq!(stderr_text(&strict_output), "");

    let warn_json = parse_json_stdout(&warn_output);
    let strict_json = parse_json_stdout(&strict_output);
    assert_eq!(warn_json, strict_json);
    assert_eq!(warn_json["ok"].as_bool(), Some(true));
    assert_eq!(
        warn_json["errors"]
            .as_array()
            .expect("errors should be an array")
            .len(),
        0
    );
}

#[test]
fn test_check_modes_unknown_keys_human_output() {
    for mode in CHECK_MODES {
        let env = setup_test_env();
        setup_unknown_keys_graph(&env);

        let output = run_check(mode, &env.root, false);
        assert_issue_exit(mode, &output);
        assert_eq!(stdout_text(&output), "");
        assert_issue_stderr(
            mode,
            &stderr_text(&output),
            &[
                "Unknown frontmatter key: 'weird_key'",
                "Unknown frontmatter key: 'other_key'",
            ],
            2,
        );
    }
}

#[test]
fn test_check_modes_dangling_need_human_output() {
    for mode in CHECK_MODES {
        let env = setup_test_env();
        setup_dangling_need_graph(&env);

        let output = run_check(mode, &env.root, false);
        assert_issue_exit(mode, &output);
        assert_eq!(stdout_text(&output), "");
        assert_issue_stderr(
            mode,
            &stderr_text(&output),
            &["Dangling reference in 'needs': 'MISSING_TASK' not found"],
            1,
        );
    }
}

#[test]
fn test_check_modes_cycle_human_output() {
    for mode in CHECK_MODES {
        let env = setup_test_env();
        setup_cycle_graph(&env);

        let output = run_check(mode, &env.root, false);
        assert_issue_exit(mode, &output);
        assert_eq!(stdout_text(&output), "");
        assert_issue_stderr(
            mode,
            &stderr_text(&output),
            &["Dependency cycle detected: A, B"],
            2,
        );
    }
}

#[test]
fn test_check_modes_missing_created_at_human_output() {
    for mode in CHECK_MODES {
        let env = setup_test_env();
        setup_missing_created_at_graph(&env);

        let output = run_check(mode, &env.root, false);
        assert_issue_exit(mode, &output);
        assert_eq!(stdout_text(&output), "");
        assert_issue_stderr(
            mode,
            &stderr_text(&output),
            &["Missing required frontmatter key: 'created_at'"],
            1,
        );
    }
}

#[test]
fn test_check_modes_issue_json_payload_is_identical() {
    let env_warn = setup_test_env();
    setup_unknown_keys_graph(&env_warn);
    let warn_output = run_check(CheckMode::WarnOnly, &env_warn.root, true);

    let env_strict = setup_test_env();
    setup_unknown_keys_graph(&env_strict);
    let strict_output = run_check(CheckMode::Strict, &env_strict.root, true);

    assert!(warn_output.status.success());
    assert_eq!(strict_output.status.code(), Some(1));

    let warn_json = parse_json_stdout(&warn_output);
    let strict_json = parse_json_stdout(&strict_output);
    assert_eq!(warn_json, strict_json);
    assert_eq!(warn_json["ok"].as_bool(), Some(false));

    let errors = warn_json["errors"]
        .as_array()
        .expect("errors should be an array");
    assert_eq!(errors.len(), 2);

    let warn_stderr = stderr_text(&warn_output);
    let strict_stderr = stderr_text(&strict_output);
    assert_eq!(warn_stderr, "");
    assert!(strict_stderr.contains("Runtime error:"));
    assert!(strict_stderr.contains("Check failed: graph has issues."));
}

#[test]
fn test_legacy_doctor_command_is_no_longer_available() {
    let env = setup_test_env();

    env.pebble()
        .arg("doctor")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unrecognized subcommand 'doctor'"));
}

#[test]
fn test_legacy_fix_command_is_no_longer_available() {
    let env = setup_test_env();

    env.pebble()
        .arg("fix")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unrecognized subcommand 'fix'"));
}

#[test]
fn test_check_warn_only_and_fix_are_mutually_exclusive() {
    let env = setup_test_env();

    env.pebble()
        .args(["check", "--warn-only", "--fix"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '--warn-only' cannot be used with '--fix'",
        ));
}
