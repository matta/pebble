use assert_cmd::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_all_subcommands_include_examples_section() {
    let cases: &[(&[&str], &[&str])] = &[
        (&["list", "--help"], &["Examples:", "list tasks"]),
        (&["next", "--help"], &["Examples:", "next task"]),
        (&["search", "--help"], &["Examples:", "search"]),
        (&["show", "--help"], &["Examples:", "show"]),
        (&["add", "--help"], &["Examples:", "add"]),
        (&["update", "--help"], &["Examples:", "update"]),
        (&["archive", "--help"], &["Examples:", "archive"]),
        (&["init", "--help"], &["Examples:", "init"]),
        (&["config", "get", "--help"], &["Examples:", "config get"]),
    ];

    for (args, contains_all) in cases {
        let output = Command::new(cargo_bin!())
            .args(*args)
            .output()
            .expect("Failed to execute help command");
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
