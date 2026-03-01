#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
pub mod support;

use serde_json::Value;
use std::fs;
use std::path::Path;
use support::setup_test_env;

/// Write a task file with an explicit `resolved_at` field.
fn write_resolved_task(tasks_dir: &Path, id: &str, status: &str, resolved_at: Option<&str>) {
    let resolved_line = match resolved_at {
        Some(ts) => format!("resolved_at: \"{ts}\"\n"),
        None => String::new(),
    };
    let content = format!(
        "---\n\
         id: \"{id}\"\n\
         title: \"{id}\"\n\
         status: \"{status}\"\n\
         created_at: \"2024-01-01T00:00:00Z\"\n\
         {resolved_line}\
         ---\n\
         Body\n"
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).expect("task file should be written");
}

#[test]
fn test_archive_moves_old_resolved_task() {
    let env = setup_test_env();
    write_resolved_task(
        &env.tasks_dir,
        "PROJ-OLD",
        "done",
        Some("2024-01-01T00:00:00Z"),
    );

    let output = env
        .pebble()
        .args(["archive", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let archived = json["archived"]
        .as_array()
        .expect("archived should be an array");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0]["id"], "PROJ-OLD");

    assert!(!env.tasks_dir.join("PROJ-OLD.md").exists());
    assert!(env.tasks_dir.join("archive/PROJ-OLD.md").exists());
}

#[test]
fn test_archive_skips_recently_resolved_task() {
    let env = setup_test_env();
    write_resolved_task(
        &env.tasks_dir,
        "PROJ-RECENT",
        "done",
        Some("2026-03-01T00:00:00Z"),
    );

    let output = env
        .pebble()
        .args(["archive", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let archived = json["archived"]
        .as_array()
        .expect("archived should be an array");
    assert!(archived.is_empty());

    assert!(env.tasks_dir.join("PROJ-RECENT.md").exists());
}

#[test]
fn test_archive_skips_non_terminal_task() {
    let env = setup_test_env();
    write_resolved_task(&env.tasks_dir, "PROJ-TODO", "todo", None);

    let output = env
        .pebble()
        .args(["archive", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let archived = json["archived"]
        .as_array()
        .expect("archived should be an array");
    assert!(archived.is_empty());

    assert!(env.tasks_dir.join("PROJ-TODO.md").exists());
}

#[test]
fn test_archive_collision_appends_numeric_suffix() {
    let env = setup_test_env();
    write_resolved_task(
        &env.tasks_dir,
        "PROJ-COLL",
        "done",
        Some("2024-01-01T00:00:00Z"),
    );

    let archive_dir = env.tasks_dir.join("archive");
    fs::create_dir_all(&archive_dir).expect("archive directory should be created");
    fs::write(archive_dir.join("PROJ-COLL.md"), "already here")
        .expect("archive file should be written");

    let output = env
        .pebble()
        .args(["archive", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let archived = json["archived"]
        .as_array()
        .expect("archived should be an array");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0]["id"], "PROJ-COLL");

    let moved_to = archived[0]["moved_to"]
        .as_str()
        .expect("moved_to should be a string");
    assert!(
        moved_to.contains("-2"),
        "Expected moved_to to contain '-2', got: {moved_to}"
    );

    assert!(!env.tasks_dir.join("PROJ-COLL.md").exists());
    assert!(archive_dir.join("PROJ-COLL-2.md").exists());
}
