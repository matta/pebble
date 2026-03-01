import re

filepath = "crates/pebble/tests/cli_check.rs"
with open(filepath, 'r') as f:
    content = f.read()

content = content.replace("use std::path::Path;\n", "")

with open(filepath, 'w') as f:
    f.write(content)
