import re

filepath = "crates/pebble/tests/cli_ux.rs"
with open(filepath, 'r') as f:
    content = f.read()

# I removed the `let env = setup_test_env();` and `.current_dir(&env.root)` from `cli_ux.rs` earlier!
# `pebble next` was executed directly in the project root, so it's picking up actual tasks from the current working directory! That's why it succeeded with `issue-10ydgv4b Clean Task\n` because it found an actual task from my earlier tests that leaked!
# We need to reinstate the test env for tests that touch data!
#
# Actually, the `pebble next` test specifically wants to run in an EMPTY environment (no tasks). So it MUST use `setup_test_env` to get an isolated directory.
# Similarly for `add`! If it runs in root, it drops files in the real directory.

content = content.replace("fn test_next_stdout_is_clean_when_no_tasks() {\n    let mut cmd = assert_cmd::Command::new(assert_cmd::cargo_bin!(\"pebble\"));", "fn test_next_stdout_is_clean_when_no_tasks() {\n    let env = support::setup_test_env();\n    let mut cmd = env.pebble();")
content = content.replace("fn test_add_stdout_is_clean_non_json() {\n    let mut cmd = assert_cmd::Command::new(assert_cmd::cargo_bin!(\"pebble\"));", "fn test_add_stdout_is_clean_non_json() {\n    let env = support::setup_test_env();\n    let mut cmd = env.pebble();")

with open(filepath, 'w') as f:
    f.write(content)
