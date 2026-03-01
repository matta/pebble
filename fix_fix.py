import re

filepath = "crates/pebble/tests/cli_fix.rs"
with open(filepath, 'r') as f:
    content = f.read()

content = content.replace("run_fix(&env.root, ", "run_fix(&env, ")

with open(filepath, 'w') as f:
    f.write(content)
