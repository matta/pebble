import re
import glob

files = glob.glob('crates/pebble/tests/*.rs')

for filepath in files:
    with open(filepath, 'r') as f:
        content = f.read()

    # We replace:
    # let mut cmd = Command::new(cargo_bin!("pebble"));
    # cmd.current_dir(&env.root)
    # With:
    # let mut cmd = env.pebble();

    # We replace:
    # let output = Command::new(cargo_bin!())
    #    .current_dir(&env.root)
    # With:
    # let output = env.pebble()

    # Clean up imports
    content = re.sub(r'use assert_cmd::cargo_bin;\n?', '', content)
    content = re.sub(r'use std::process::Command;\n?', '', content)

    # In `support.rs` we shouldn't modify the Command struct or `setup_test_env` implementation except the new method
    if "support.rs" in filepath:
        with open(filepath, 'w') as f:
            f.write(content)
        continue

    # Direct replacements
    content = re.sub(r'Command::new\(cargo_bin!\(\"pebble\"\)\)', 'env.pebble()', content)
    content = re.sub(r'Command::new\(cargo_bin!\(\)\)', 'env.pebble()', content)

    # After doing this, we will have `env.pebble()` followed by `.current_dir(&env.root)`.
    # Let's remove the `.current_dir(&env.root)` only when it immediately follows (or with spaces/newlines)
    # the env.pebble() call or the cmd variable that holds it.

    # If the file has `.current_dir(&env.root)`, let's just blindly remove it because `env.pebble()` handles it.
    # Note: cli_init.rs uses `Command::cargo_bin("pebble")`, we'll let's replace that too.
    content = re.sub(r'Command::cargo_bin\(\"pebble\"\)', 'env.pebble()', content)

    # Remove all `.current_dir(&env.root)` lines/calls in tests.
    content = re.sub(r'\s*\.current_dir\(&env\.root\)', '', content)

    # Also clean up `cmd.current_dir(&env.root);`
    content = re.sub(r'cmd\.current_dir\(&env\.root\);\n?', '', content)

    with open(filepath, 'w') as f:
        f.write(content)
