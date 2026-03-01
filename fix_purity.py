import re

filepath = "crates/pebble/tests/cli_json_purity_extended.rs"
with open(filepath, 'r') as f:
    content = f.read()

content = re.sub(
    r'let output = env\.pebble\(\)',
    r'let output = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found")',
    content
)

with open(filepath, 'w') as f:
    f.write(content)
