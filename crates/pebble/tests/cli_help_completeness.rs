use assert_cmd::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_all_subcommands_include_examples_section() {
    let cases: &[(&[&str], &[&str])] = &[
        (&["list", "--help"], &["Examples:", "pebble list"]),
        (&["next", "--help"], &["Examples:", "pebble next"]),
        (&["search", "--help"], &["Examples:", "pebble search"]),
        (&["show", "--help"], &["Examples:", "pebble show"]),
        (&["add", "--help"], &["Examples:", "pebble add"]),
        (&["update", "--help"], &["Examples:", "pebble update"]),
        (&["archive", "--help"], &["Examples:", "pebble archive"]),
        (&["init", "--help"], &["Examples:", "pebble init"]),
        (
            &["config", "get", "--help"],
            &["Examples:", "pebble config get"],
        ),
    ];

    for (args, contains_all) in cases {
        let output = Command::new(cargo_bin!())
            .args(*args)
            .output()
            .expect("pebble command should execute successfully");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for needle in *contains_all {
            assert!(
                stdout.contains(needle),
                "Expected '{}' in help output for args {:?}. Output: {}",
                needle,
                args,
                stdout
            );
        }
    }
}

#[test]
fn test_list_help_describes_combined_filter_semantics() {
    let mut cmd = Command::new(cargo_bin!());
    cmd.args(["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OR"))
        .stdout(predicate::str::contains("AND"))
        .stdout(predicate::str::contains("default omits"))
        .stdout(predicate::str::contains("--sort"));
}
#[test]
fn test_config_get_help_documents_keys() {
    let mut cmd = Command::new(cargo_bin!());
    cmd.args(["config", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration key to fetch"))
        .stdout(predicate::str::contains("issue-prefix"))
        .stdout(predicate::str::contains("tasks-dir"));
}
