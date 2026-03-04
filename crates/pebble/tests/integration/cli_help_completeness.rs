#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use assert_cmd::cargo_bin;
use assert_cmd::prelude::*;
use pebble::config::CONFIG_KEYS;
use predicates::prelude::*;
use serde_json::Value;
use std::process::Command;

/// Recursively collect all leaf command paths from the help-json schema.
///
/// For commands with subcommands (like `config`), this returns the
/// subcommand paths (e.g. `["config", "get"]`) rather than the parent.
/// For leaf commands (like `list`), it returns `["list"]`.
fn collect_command_paths(commands: &[Value], prefix: &[String]) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    for cmd in commands {
        let name = cmd["name"]
            .as_str()
            .expect("command name should be a string")
            .to_string();
        let mut path = prefix.to_vec();
        path.push(name);

        if let Some(subs) = cmd.get("subcommands").and_then(Value::as_array) {
            paths.extend(collect_command_paths(subs, &path));
        } else {
            paths.push(path);
        }
    }
    paths
}

#[test]
fn test_all_subcommands_include_examples_section() {
    // Dynamically discover every leaf subcommand from help-json.
    let help_output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("help-json should execute");
    assert!(help_output.status.success());
    let schema: Value =
        serde_json::from_slice(&help_output.stdout).expect("help-json output should be valid JSON");
    let commands = schema["commands"]
        .as_array()
        .expect("commands should be an array");

    let command_paths = collect_command_paths(commands, &[]);
    assert!(
        !command_paths.is_empty(),
        "help-json should report at least one command"
    );

    for path in &command_paths {
        let mut args: Vec<&str> = path.iter().map(String::as_str).collect();
        args.push("--help");

        let output = Command::new(cargo_bin!())
            .args(&args)
            .output()
            .expect("pebble command should execute successfully");
        assert!(output.status.success(), "help failed for {args:?}");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cmd_display = format!("pebble {}", path.join(" "));
        assert!(
            stdout.contains("Examples:"),
            "Missing 'Examples:' section in help for `{cmd_display}`.\nOutput:\n{stdout}"
        );
        assert!(
            stdout.contains(&cmd_display),
            "Help for `{cmd_display}` should include an example starting with `{cmd_display}`.\nOutput:\n{stdout}"
        );
    }
}

#[test]
fn test_list_help_describes_combined_filter_semantics() {
    let mut cmd = Command::new(cargo_bin!());
    cmd.args(["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OR"))
        .stdout(predicate::str::contains("AND"))
        .stdout(predicate::str::contains("default omits"))
        .stdout(predicate::str::contains("--sort"));
}
#[test]
fn test_config_get_help_documents_keys() {
    let mut cmd = Command::new(cargo_bin!());
    let mut assert = cmd
        .args(["config", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration key to fetch"));

    for key in CONFIG_KEYS {
        assert = assert.stdout(predicate::str::contains(*key));
    }
}
