import re
import glob

files_to_fix = ["crates/pebble/tests/cli_help_json.rs", "crates/pebble/tests/cli_fix.rs"]

for filepath in files_to_fix:
    with open(filepath, 'r') as f:
        content = f.read()

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

# We should also replace the remaining cargo_bin usages to env.pebble() where an env is actually available.
# But looking at what failed, it was because they didn't have `env` available in those tests, they were manually using `assert_cmd::Command::cargo_bin(...)`
# However, the goal of the plan is: Replace them with calls to env.pebble().
# For the tests that DON'T have `env`, we should initialize `env` instead of manually initializing `Command`.
# But wait, does `TestEnv` initialize a full workspace which might not be needed for `help` output tests?
# Yes. But the prompt said: "Find all places in tests where Command::new(cargo_bin!("pebble")) ... and replace them with env.pebble()".
# For those without `setup_test_env`, maybe they shouldn't be touched. The ones I reverted to `assert_cmd::Command::cargo_bin` are passing now, meaning they don't have `current_dir` anyway.
# Actually, the user asked to replace "Command::new(cargo_bin!("pebble"))" that also has `.current_dir(&env.root)` with `env.pebble()`.
