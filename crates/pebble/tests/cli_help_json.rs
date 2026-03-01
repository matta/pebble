#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use assert_cmd::cargo_bin;
use serde_json::Value;
use std::process::Command;

#[test]
fn test_help_json_emits_valid_schema_to_stdout_only() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["name"].as_str(), Some("pebble"));
    assert!(value["global_options"].is_array());
    assert!(value["commands"].is_array());
}

#[test]
fn test_help_json_lists_core_commands() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let commands = value["commands"]
        .as_array()
        .expect("commands should be an array");

    let names: Vec<&str> = commands
        .iter()
        .filter_map(|cmd| cmd["name"].as_str())
        .collect();

    assert!(names.contains(&"list"));
    assert!(names.contains(&"next"));
    assert!(names.contains(&"search"));
    assert!(names.contains(&"show"));
    assert!(names.contains(&"config"));
}

#[test]
fn test_help_json_treats_help_json_as_command_not_global_flag() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");

    let global_option_names: Vec<&str> = value["global_options"]
        .as_array()
        .expect("global_options should be an array")
        .iter()
        .filter_map(|opt| opt["name"].as_str())
        .collect();
    assert!(!global_option_names.contains(&"--help-json"));

    let command_names: Vec<&str> = value["commands"]
        .as_array()
        .expect("commands should be an array")
        .iter()
        .filter_map(|cmd| cmd["name"].as_str())
        .collect();
    assert!(command_names.contains(&"help-json"));
}

#[test]
fn test_help_json_includes_command_descriptions() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let commands = value["commands"]
        .as_array()
        .expect("commands should be an array");

    for cmd in commands {
        let name = cmd["name"]
            .as_str()
            .expect("command name should be a string");
        let description = cmd["description"]
            .as_str()
            .expect("command description should be a string");
        assert!(
            !description.trim().is_empty(),
            "Expected non-empty description for command '{}'",
            name
        );
    }
}

#[test]
fn test_help_json_includes_options_for_add_command() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    let commands = value["commands"]
        .as_array()
        .expect("commands should be an array");

    let add_cmd = commands
        .iter()
        .find(|cmd| cmd["name"] == "add")
        .expect("'add' command should be present in help JSON");
    let options = add_cmd["options"]
        .as_array()
        .expect("options should be an array for 'add' command");

    let opt_names: Vec<&str> = options
        .iter()
        .filter_map(|opt| opt["name"].as_str())
        .collect();

    assert!(opt_names.contains(&"--status"));
    assert!(opt_names.contains(&"--priority"));
    assert!(opt_names.contains(&"--need"));
    assert!(opt_names.contains(&"--tag"));
}

#[test]
fn test_help_json_replaces_doctor_and_fix_with_check_flags() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    let commands = value["commands"]
        .as_array()
        .expect("commands should be an array");

    let names: Vec<&str> = commands
        .iter()
        .filter_map(|cmd| cmd["name"].as_str())
        .collect();
    assert!(names.contains(&"check"));
    assert!(!names.contains(&"doctor"));
    assert!(!names.contains(&"fix"));

    let check_options = commands
        .iter()
        .find(|cmd| cmd["name"] == "check")
        .expect("'check' command should exist")["options"]
        .as_array()
        .expect("'check' options should be an array");

    let check_option_names: Vec<&str> = check_options
        .iter()
        .filter_map(|opt| opt["name"].as_str())
        .collect();
    assert!(check_option_names.contains(&"--warn-only"));
    assert!(check_option_names.contains(&"--fix"));

    let check_output = commands
        .iter()
        .find(|cmd| cmd["name"] == "check")
        .expect("'check' command should exist")["output"]
        .as_object()
        .expect("'check' output should be an object");
    assert!(check_output.contains_key("ok"));
    assert!(check_output.contains_key("errors"));
    assert!(check_output.contains_key("fixed_tasks"));
}
