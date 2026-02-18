use assert_cmd::Command;
use assert_cmd::cargo_bin;
use pebble::store::Issue;
use std::fs;
use std::process::Command as std_command;
use tempfile::TempDir;

fn setup_pebble_repo(path: &std::path::Path) {
    // 1. git init
    std_command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .status()
        .unwrap();
    std_command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .status()
        .unwrap();
    std_command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .status()
        .unwrap();
    fs::write(path.join("README.md"), "test").unwrap();
    std_command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .status()
        .unwrap();
    std_command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(path)
        .status()
        .unwrap();

    // 2. pebble init
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(path).arg("init").assert().success();
}

#[test]
fn test_import_basic() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    setup_pebble_repo(root);

    // Create an external JSONL file to import
    let import_file = root.join("external.jsonl");
    let issue = Issue {
        id: "EXT-1".to_string(),
        title: "Imported Issue".to_string(),
        description: "External desc".to_string(),
        status: "open".to_string(),
        priority: 1,
        issue_type: "task".to_string(),
        owner: "external@example.com".to_string(),
        created_at: "2026-01-01T10:00:00Z".to_string(),
        created_by: "External".to_string(),
        updated_at: "2026-01-01T10:00:00Z".to_string(),
        closed_at: None,
        close_reason: None,
    };
    let json = serde_json::to_string(&issue).unwrap();
    fs::write(
        &import_file,
        format!(
            "{}
",
            json
        ),
    )
    .unwrap();

    // Run 'pebble import external.jsonl'
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["import", import_file.to_str().unwrap()])
        .assert()
        .success();

    // Verify it was imported
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(root)
        .args(["show", "EXT-1"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Imported Issue"));
}
