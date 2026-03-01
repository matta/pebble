import re

filepath = "crates/pebble/tests/cli_json_purity_extended.rs"
with open(filepath, 'r') as f:
    content = f.read()

# For tests that do not have `TestEnv` (e.g., they just use `tempfile::tempdir()` directly):
# They need to revert to `assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found")`
content = re.sub(
    r'let dir = tempfile::tempdir\(\)\.expect\("temp directory should be created"\);\n\s*let output = env\.pebble\(\)',
    r'let dir = tempfile::tempdir().expect("temp directory should be created");\n    let output = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found").current_dir(dir.path())',
    content
)

# test_help_json_stdout_only has no `env` created at all.
content = re.sub(
    r'fn test_help_json_stdout_only\(\) {\n\s*let output = env\.pebble\(\)',
    r'fn test_help_json_stdout_only() {\n    let output = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found")',
    content
)

with open(filepath, 'w') as f:
    f.write(content)
