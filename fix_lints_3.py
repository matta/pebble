import re

filepath = "crates/pebble/tests/cli_ux.rs"
with open(filepath, 'r') as f:
    content = f.read()
content = content.replace("use support::setup_test_env;\n", "")
with open(filepath, 'w') as f:
    f.write(content)

filepath = "crates/pebble/tests/cli_init.rs"
with open(filepath, 'r') as f:
    content = f.read()
content = content.replace("use assert_cmd::Command;\n", "")
with open(filepath, 'w') as f:
    f.write(content)

filepath = "crates/pebble/tests/support.rs"
with open(filepath, 'r') as f:
    content = f.read()
content = content.replace("pub fn setup_test_env() -> TestEnv {", "#[allow(dead_code)]\npub fn setup_test_env() -> TestEnv {")
with open(filepath, 'w') as f:
    f.write(content)
