import re

filepath = "crates/pebble/tests/cli_fix.rs"
with open(filepath, 'r') as f:
    content = f.read()

content = content.replace("use std::path::Path;\n", "")

with open(filepath, 'w') as f:
    f.write(content)


filepath = "crates/pebble/tests/support.rs"
with open(filepath, 'r') as f:
    content = f.read()

content = content.replace("pub root: PathBuf,", "#[allow(dead_code)]\n    pub root: PathBuf,")
content = content.replace("pub fn pebble(&self) -> assert_cmd::Command {", "#[allow(dead_code)]\n    pub fn pebble(&self) -> assert_cmd::Command {")

with open(filepath, 'w') as f:
    f.write(content)
