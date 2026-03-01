import re

filepath = "crates/pebble/tests/cli_fix.rs"
with open(filepath, 'r') as f:
    content = f.read()

# fix the unused variable root in `fn run_fix`
content = content.replace("cmd.arg(\"fix\");", "cmd.current_dir(root);\n    cmd.arg(\"fix\");")

with open(filepath, 'w') as f:
    f.write(content)
