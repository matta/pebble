use assert_cmd::Command;
use assert_cmd::cargo_bin;

#[test]
fn test_help_json() {
    let output = Command::new(cargo_bin!("pebble"))
        .arg("--help-json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let data: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    let commands = data["commands"]
        .as_array()
        .expect("commands must be an array");
    let names: Vec<&str> = commands
        .iter()
        .filter_map(|cmd| cmd["name"].as_str())
        .collect();

    assert!(names.contains(&"list"));
    assert!(names.contains(&"add"));
    assert!(names.contains(&"show"));
    assert!(names.contains(&"update"));
    assert!(names.contains(&"sync"));
    assert!(names.contains(&"init"));
    assert!(names.contains(&"import"));
    assert!(names.contains(&"config"));

    assert!(data["schemas"].is_object());
    assert!(
        data["top_level_help"]
            .as_str()
            .unwrap_or("")
            .contains("Usage:")
    );
}
