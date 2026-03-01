import re

for filepath in ["crates/pebble/tests/cli_init.rs", "crates/pebble/tests/cli_check.rs"]:
    with open(filepath, 'r') as f:
        content = f.read()

    if "cli_init.rs" in filepath:
        content = re.sub(
            r'let mut cmd = env\.pebble\(\)\.expect\("pebble binary should be found"\);',
            r'let mut cmd = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found");',
            content
        )

    if "cli_check.rs" in filepath:
        content = "use assert_cmd::Command;\n" + content

    with open(filepath, 'w') as f:
        f.write(content)
