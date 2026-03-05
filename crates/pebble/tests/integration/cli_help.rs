use super::support;
use predicates::prelude::*;

fn commands_section_rows(help_stdout: &str) -> Vec<&str> {
    let mut in_commands = false;
    let mut rows = Vec::new();

    for line in help_stdout.lines() {
        if !in_commands {
            if line.trim() == "Commands:" {
                in_commands = true;
            }
            continue;
        }

        if line.trim().is_empty() {
            break;
        }

        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        rows.push(trimmed);
    }

    rows
}

#[test]
fn test_help_output_contains_show() {
    let mut cmd = support::pebble_cli();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("Show one task by ID"));
}

#[test]
fn test_show_help_output_contains_path_only_flag() {
    let mut cmd = support::pebble_cli();
    cmd.arg("show")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--path-only"))
        .stdout(predicate::str::contains(
            "Output only the file path relative to tasks-dir",
        ));
}

#[test]
fn test_check_help_includes_warn_only_flag() {
    let mut cmd = support::pebble_cli();
    cmd.arg("check")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--warn-only"))
        .stdout(predicate::str::contains("--fix"))
        .stdout(predicate::str::contains("exit with status code 0"));
}

#[test]
fn test_top_level_help_commands_have_one_line_summaries() {
    let output = match support::pebble_cli().arg("--help").output() {
        Ok(output) => output,
        Err(error) => panic!("pebble --help should execute: {error}"),
    };
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let rows = commands_section_rows(&stdout);
    assert!(!rows.is_empty(), "expected a commands section in --help");

    for row in rows {
        let mut parts = row.split_whitespace();
        let command_name = parts.next().unwrap_or_default();
        let summary = parts.collect::<Vec<_>>().join(" ");
        assert!(
            !command_name.is_empty(),
            "expected command name in row: `{row}`"
        );
        assert!(
            !summary.is_empty(),
            "missing one-line summary for command `{command_name}` in row: `{row}`"
        );
    }
}

#[test]
fn test_config_help_commands_have_one_line_summaries() {
    let output = match support::pebble_cli()
        .args(["config", "--help"])
        .output()
    {
        Ok(output) => output,
        Err(error) => panic!("pebble config --help should execute: {error}"),
    };
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let rows = commands_section_rows(&stdout);
    assert!(
        !rows.is_empty(),
        "expected a commands section in `pebble config --help`"
    );

    for row in rows {
        let mut parts = row.split_whitespace();
        let command_name = parts.next().unwrap_or_default();
        let summary = parts.collect::<Vec<_>>().join(" ");
        assert!(
            !command_name.is_empty(),
            "expected command name in row: `{row}`"
        );
        assert!(
            !summary.is_empty(),
            "missing one-line summary for subcommand `{command_name}` in row: `{row}`"
        );
    }
}
