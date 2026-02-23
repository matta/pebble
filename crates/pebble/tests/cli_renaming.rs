mod support;

use assert_cmd::Command;
use serde_json::Value;
use support::setup_test_env;

#[test]
fn test_cli_renamed_flags_and_roundtrip() {
    let env = setup_test_env();

    // 1. Test 'pebble add --need' (renamed from --dep)
    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["add", "Child Task", "--need", "parent", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success(), "pebble add --need failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let child_id = value.get("id").unwrap().as_str().unwrap().to_string();
    
    let needs = value.get("needs").unwrap().as_array().unwrap();
    assert_eq!(needs[0], "parent");

    // 2. Test 'pebble update --add-need' (renamed from --add-dep)
    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["update", &child_id, "--add-need", "another-parent", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success(), "pebble update --add-need failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let needs = value.get("needs").unwrap().as_array().unwrap();
    let mut needs_strs: Vec<&str> = needs.iter().map(|v| v.as_str().unwrap()).collect();
    needs_strs.sort();
    assert_eq!(needs_strs, vec!["another-parent", "parent"]);

    // 3. Test 'pebble update --remove-need' (renamed from --remove-dep)
    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["update", &child_id, "--remove-need", "parent", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success(), "pebble update --remove-need failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let needs = value.get("needs").unwrap().as_array().unwrap();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0], "another-parent");

    // 4. Test computed fields 'blocked_by' and 'blocking' with real IDs
    // Create the 'another-parent' task so it is not terminal (stays in todo)
    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["add", "Another Parent", "--json", "--dir", "tasks"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let parent_id = value.get("id").unwrap().as_str().unwrap().to_string();

    // Now update 'child' to need this new parent
    Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["update", &child_id, "--add-need", &parent_id, "--dir", "tasks"])
        .assert()
        .success();

    // 'child' needs parent_id. parent_id is todo.
    // So 'child' should be blocked_by parent_id
    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["show", &child_id, "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let blocked_by = value.get("blocked_by").unwrap().as_array().unwrap();
    assert!(blocked_by.iter().any(|v| v.as_str() == Some(&parent_id)));

    // parent_id should be blocking 'child'
    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["show", &parent_id, "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let blocking = value.get("blocking").unwrap().as_array().unwrap();
    assert!(blocking.iter().any(|v| v.as_str() == Some(&child_id)));
}
