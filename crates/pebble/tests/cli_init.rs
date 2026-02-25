use assert_cmd::Command;
use std::fs;

#[test]
fn test_init_path_traversal_prevention() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let subdir = root.join("project");
    fs::create_dir(&subdir).unwrap();

    // Try to init inside 'project' but point tasks dir to '../outside'
    // 'outside' would be a sibling of 'project', i.e. directly under 'root'.
    #[allow(deprecated)] // TODO: Migrate to cargo_bin_cmd! when feasible
    let mut cmd = Command::cargo_bin("pebble").unwrap();
    let output = cmd
        .current_dir(&subdir)
        .args(["init", "--dir", "../outside"])
        .output()
        .expect("Failed to run init command");

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
