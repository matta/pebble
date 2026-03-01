import re

for filepath in ["crates/pebble/tests/cli_errors.rs", "crates/pebble/tests/cli_ux.rs"]:
    with open(filepath, 'r') as f:
        content = f.read()

    # Some tests use `Command::new(cargo_bin!())` without `env` initialized!
    # They should use `assert_cmd::Command::cargo_bin("pebble").expect(...)`
    # or we can leave them as Command::new(cargo_bin!()) but we removed the imports!

    # Let's just fix it by providing an `env` or reverting it.

    # Wait, in cli_errors.rs:78 and 94 it's `let output = env.pebble().arg...` without `env`?
    # Actually, in cli_errors.rs:
    content = re.sub(
        r'let output = env\.pebble\(\)',
        r'let output = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found")',
        content
    )
    content = re.sub(
        r'let mut cmd = env\.pebble\(\);',
        r'let mut cmd = assert_cmd::Command::cargo_bin("pebble").expect("pebble binary should be found");',
        content
    )

    with open(filepath, 'w') as f:
        f.write(content)
