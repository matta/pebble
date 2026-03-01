#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use assert_cmd::Command;
use assert_cmd::cargo_bin;
use std::fs;

// These tests cannot use the shared `TestEnv` fixture from `support.rs`
// because `TestEnv::setup_test_env()` creates an already-initialized project
// (with `.pebble/config.toml` and a tasks directory). The `init` command
// requires an un-initialized directory to test against.

#[test]
fn test_init_generates_useful_agents_md() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let root = temp.path();

    let mut cmd = Command::new(cargo_bin!());
    cmd.current_dir(root).args(["init"]).assert().success();

    let agents_content =
        fs::read_to_string(root.join(".pebble/AGENTS.md")).expect("AGENTS.md should exist");

    // Must tell agents about pebble and how to use it.
    assert!(
        agents_content.contains("pebble"),
        "AGENTS.md should mention pebble. Got: {agents_content}"
    );
    assert!(
        agents_content.contains("--json"),
        "AGENTS.md should recommend --json output for agent workflows. Got: {agents_content}"
    );
    assert!(
        agents_content.contains("pebble next --json"),
        "AGENTS.md should include next-task workflow guidance. Got: {agents_content}"
    );
    assert!(
        agents_content.contains("docs/pebble/"),
        "AGENTS.md should mention configured task storage path. Got: {agents_content}"
    );
}

#[test]
fn test_init_path_traversal_prevention() {
    let temp = tempfile::tempdir().expect("temp directory should be created");
    let root = temp.path();
    let subdir = root.join("project");
    fs::create_dir(&subdir).expect("subdir should be created");

    // Try to init inside 'project' but point tasks dir to '../outside'
    // 'outside' would be a sibling of 'project', i.e. directly under 'root'.
    let mut cmd = Command::new(cargo_bin!());
    let output = cmd
        .current_dir(&subdir)
        .args(["init", "--dir", "../outside"])
        .output()
        .expect("pebble command should execute successfully");

    // Must NOT succeed
    assert!(
        !output.status.success(),
        "Command should have failed due to path traversal attempt"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tasks-dir") && stderr.contains("parent directory components"),
        "Error message should mention invalid path components. Got: {}",
        stderr
    );

    // Verify the directory was NOT created
    assert!(
        !root.join("outside").exists(),
        "Directory should NOT have been created via traversal"
    );
}
