#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use assert_cmd::Command;

mod support;
use support::setup_test_env;

#[test]
fn test_cli_dir_override_path_traversal_prevention() {
    let env = setup_test_env();

    #[allow(deprecated)] // TODO: Migrate to cargo_bin_cmd! when feasible
    let mut cmd = Command::cargo_bin("pebble").expect("pebble binary should be found");
    let output = cmd
        .current_dir(&env.root)
        .args(["list", "--dir", "../outside"])
        .output()
        .expect("pebble command should execute");

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
}
