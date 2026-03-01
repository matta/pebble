import glob
import re

for filepath in glob.glob('crates/pebble/tests/*.rs'):
    with open(filepath, 'r') as f:
        content = f.read()

    # Revert files that don't instantiate test_env but had Command::new modified by our naive sed scripts earlier.
    # Actually, we checked them out so they are in pristine state.
    pass
