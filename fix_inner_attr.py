import re

filepath = "crates/pebble/tests/cli_check.rs"
with open(filepath, 'r') as f:
    content = f.read()

# Remove the incorrectly placed inner attribute `use assert_cmd::Command;` was placed before `#![expect(...)]`
# Wait, I added `use assert_cmd::Command;\n` at the start of the file in `fix_init_check.py`.
# I should move `#![expect(...)]` to be the very first line.

content = content.replace("use assert_cmd::Command;\n#![expect(clippy::expect_used, reason = \"TODO: remove all calls to expect\")]\n", "#![expect(clippy::expect_used, reason = \"TODO: remove all calls to expect\")]\nuse assert_cmd::Command;\n")

with open(filepath, 'w') as f:
    f.write(content)
