import re

filepath = "crates/pebble/tests/cli_fix.rs"
with open(filepath, 'r') as f:
    content = f.read()

# in run_fix:
# fn run_fix(root: &Path, json: bool) -> Output {
#    let mut cmd = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found");
# It needs `cmd.current_dir(root);` back, because I removed it previously!
content = content.replace(
    "let mut cmd = assert_cmd::Command::cargo_bin(\"pebble\").expect(\"pebble binary should be found\");\n    cmd.arg(\"fix\");",
    "let mut cmd = assert_cmd::Command::cargo_bin(\"pebble\").expect(\"pebble binary should be found\");\n    cmd.current_dir(root);\n    cmd.arg(\"fix\");"
)

with open(filepath, 'w') as f:
    f.write(content)
