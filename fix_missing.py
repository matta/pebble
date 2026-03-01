import re

files_to_fix = [
    "crates/pebble/tests/cli_show.rs",
    "crates/pebble/tests/cli_help.rs",
    "crates/pebble/tests/cli_help_completeness.rs"
]

for filepath in files_to_fix:
    with open(filepath, 'r') as f:
        content = f.read()

    content = re.sub(
        r'let output = env\.pebble\(\)',
        r'let output = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found")',
        content
    )
    content = re.sub(
        r'let mut cmd = env\.pebble\(\);',
        r'let mut cmd = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found");',
        content
    )

    with open(filepath, 'w') as f:
        f.write(content)
