use assert_cmd::cargo_bin;
use serde_json::Value;
use std::process::Command;

#[test]
fn test_help_json_emits_valid_schema_to_stdout_only() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("Failed to execute help-json command");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse help JSON");
    assert_eq!(value["name"].as_str(), Some("pebble"));
    assert!(value["global_options"].is_array());
    assert!(value["commands"].is_array());
}

#[test]
fn test_help_json_lists_core_commands() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("Failed to execute help-json command");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse help JSON");
    let commands = value["commands"]
        .as_array()
        .expect("Expected commands to be an array");

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
        .expect("Failed to execute help-json command");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse help JSON");

    let global_option_names: Vec<&str> = value["global_options"]
        .as_array()
        .expect("Expected global_options to be an array")
        .iter()
        .filter_map(|opt| opt["name"].as_str())
        .collect();
    assert!(!global_option_names.contains(&"--help-json"));

    let command_names: Vec<&str> = value["commands"]
        .as_array()
        .expect("Expected commands to be an array")
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
        .expect("Failed to execute help-json command");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse help JSON");
    let commands = value["commands"]
        .as_array()
        .expect("Expected commands to be an array");

    for cmd in commands {
        let name = cmd["name"]
            .as_str()
            .expect("Expected command name to be a string");
        let description = cmd["description"]
            .as_str()
            .expect("Expected command description to be a string");
        assert!(
            !description.trim().is_empty(),
            "Expected non-empty description for command '{}'",
            name
        );
    }
}
