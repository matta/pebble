import re

filepath = "crates/pebble/tests/cli_check.rs"
with open(filepath, 'r') as f:
    content = f.read()

content = content.replace("run_check(mode: CheckMode, root: &Path, json: bool)", "run_check(mode: CheckMode, env: &TestEnv, json: bool)")
content = content.replace("let mut cmd = assert_cmd::Command::cargo_bin(\"pebble\").expect(\"pebble binary should be found\");", "let mut cmd = env.pebble();")
content = content.replace("cmd.current_dir(root);\n", "")
content = content.replace("run_check(mode, &env.root, ", "run_check(mode, &env, ")
content = content.replace("run_check(CheckMode::WarnOnly, &env_warn.root, ", "run_check(CheckMode::WarnOnly, &env_warn, ")
content = content.replace("run_check(CheckMode::Strict, &env_strict.root, ", "run_check(CheckMode::Strict, &env_strict, ")

with open(filepath, 'w') as f:
    f.write(content)
