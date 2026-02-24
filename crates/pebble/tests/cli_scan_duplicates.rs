mod support;

use assert_cmd::cargo_bin;
use std::process::Command;
use serde_json::Value;
use std::fs;
use support::setup_test_env;

#[test]
fn test_list_json_includes_nested_markdown_tasks() {
    let env = setup_test_env();
    let nested = env.tasks_dir.join("a").join("b");
    fs::create_dir_all(&nested).unwrap();

    let content = r#"+++
id = "PROJ-NESTED"
title = "Nested Task"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    fs::write(nested.join("nested-task.md"), content).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(ids.contains(&"PROJ-NESTED"));
}

#[test]
fn test_show_json_finds_nested_markdown_task() {
    let env = setup_test_env();
    let nested = env.tasks_dir.join("a").join("b");
    fs::create_dir_all(&nested).unwrap();

    let content = r#"+++
id = "PROJ-SHOW-NESTED"
title = "Nested Show Task"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    fs::write(nested.join("show-task.md"), content).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["show", "PROJ-SHOW-NESTED", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["id"].as_str(), Some("PROJ-SHOW-NESTED"));
}

#[test]
fn test_list_json_skips_all_duplicate_ids_and_warns() {
    let env = setup_test_env();
    let nested = env.tasks_dir.join("nested");
    fs::create_dir_all(&nested).unwrap();

    let dup_a = r#"+++
id = "PROJ-DUP"
title = "Duplicate A"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    let dup_b = r#"+++
id = "PROJ-DUP"
title = "Duplicate B"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    let unique = r#"+++
id = "PROJ-UNIQUE"
title = "Unique"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;

    fs::write(env.tasks_dir.join("dup-a.md"), dup_a).unwrap();
    fs::write(nested.join("dup-b.md"), dup_b).unwrap();
    fs::write(env.tasks_dir.join("unique.md"), unique).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(ids.contains(&"PROJ-UNIQUE"));
    assert!(!ids.contains(&"PROJ-DUP"));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Duplicate task ID"));
    assert!(stderr.contains("PROJ-DUP"));
}

#[test]
fn test_show_duplicate_id_is_treated_as_not_found() {
    let env = setup_test_env();
    let nested = env.tasks_dir.join("nested");
    fs::create_dir_all(&nested).unwrap();

    let dup = r#"+++
id = "PROJ-DUP-SHOW"
title = "Duplicate"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    fs::write(env.tasks_dir.join("dup-a.md"), dup).unwrap();
    fs::write(nested.join("dup-b.md"), dup).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["show", "PROJ-DUP-SHOW", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Task 'PROJ-DUP-SHOW' not found"));
    assert!(stderr.contains("Duplicate task ID"));
    assert!(stderr.contains("PROJ-DUP-SHOW"));
}

#[test]
fn test_update_duplicate_id_fails_with_runtime_error_and_empty_stdout() {
    let env = setup_test_env();
    let nested = env.tasks_dir.join("nested");
    fs::create_dir_all(&nested).unwrap();

    let dup = r#"+++
id = "PROJ-DUP-UPDATE"
title = "Duplicate"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    fs::write(env.tasks_dir.join("dup-a.md"), dup).unwrap();
    fs::write(nested.join("dup-b.md"), dup).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "update",
            "PROJ-DUP-UPDATE",
            "--status",
            "done",
            "--json",
            "--dir",
            "tasks",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Duplicate task ID"));
    assert!(stderr.contains("PROJ-DUP-UPDATE"));
}

#[test]
fn test_show_unique_id_still_succeeds_when_other_duplicate_exists() {
    let env = setup_test_env();
    let nested = env.tasks_dir.join("nested");
    fs::create_dir_all(&nested).unwrap();

    let dup = r#"+++
id = "PROJ-DUP-OTHER"
title = "Duplicate"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    let unique = r#"+++
id = "PROJ-UNIQUE-SHOW"
title = "Unique"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;

    fs::write(env.tasks_dir.join("dup-a.md"), dup).unwrap();
    fs::write(nested.join("dup-b.md"), dup).unwrap();
    fs::write(env.tasks_dir.join("unique.md"), unique).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["show", "PROJ-UNIQUE-SHOW", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["id"].as_str(), Some("PROJ-UNIQUE-SHOW"));
}

#[test]
fn test_duplicate_warning_mentions_every_file_for_each_duplicate_id() {
    let env = setup_test_env();
    let nested_a = env.tasks_dir.join("nested-a");
    let nested_b = env.tasks_dir.join("nested-b");
    fs::create_dir_all(&nested_a).unwrap();
    fs::create_dir_all(&nested_b).unwrap();

    let dup_one = r#"+++
id = "PROJ-DUP-ONE"
title = "Duplicate One"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;
    let dup_two = r#"+++
id = "PROJ-DUP-TWO"
title = "Duplicate Two"
status = "todo"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#;

    let dup_one_a_path = env.tasks_dir.join("dup-one-a.md");
    let dup_one_b_path = nested_a.join("dup-one-b.md");
    let dup_two_a_path = env.tasks_dir.join("dup-two-a.md");
    let dup_two_b_path = nested_b.join("dup-two-b.md");

    fs::write(&dup_one_a_path, dup_one).unwrap();
    fs::write(&dup_one_b_path, dup_one).unwrap();
    fs::write(&dup_two_a_path, dup_two).unwrap();
    fs::write(&dup_two_b_path, dup_two).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("PROJ-DUP-ONE"));
    assert!(stderr.contains(&dup_one_a_path.display().to_string()));
    assert!(stderr.contains(&dup_one_b_path.display().to_string()));

    assert!(stderr.contains("PROJ-DUP-TWO"));
    assert!(stderr.contains(&dup_two_a_path.display().to_string()));
    assert!(stderr.contains(&dup_two_b_path.display().to_string()));
}
