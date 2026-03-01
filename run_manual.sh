#!/bin/bash
sed -i -e 's/let mut cmd = Command::new(cargo_bin!("pebble"));/let mut cmd = env.pebble();/g' crates/pebble/tests/cli_fix.rs
sed -i -e 's/let mut cmd = Command::new(cargo_bin!());/let mut cmd = env.pebble();/g' crates/pebble/tests/cli_ux.rs
sed -i -e 's/let output = Command::new(cargo_bin!())/let output = env.pebble()/g' crates/pebble/tests/cli_json_purity_extended.rs
sed -i -e 's/Command::new(cargo_bin!())/env.pebble()/g' crates/pebble/tests/cli_renaming.rs
sed -i -e '/\.current_dir/d' crates/pebble/tests/cli_fix.rs crates/pebble/tests/cli_ux.rs crates/pebble/tests/cli_json_purity_extended.rs crates/pebble/tests/cli_renaming.rs

for file in crates/pebble/tests/cli_fix.rs crates/pebble/tests/cli_ux.rs crates/pebble/tests/cli_json_purity_extended.rs crates/pebble/tests/cli_renaming.rs; do
  sed -i -e '/use assert_cmd::cargo_bin;/d' "$file"
  sed -i -e '/use std::process::Command;/d' "$file"
  sed -i -e '/use assert_cmd::Command;/d' "$file"
  sed -i -e '/use assert_cmd::prelude::\*;/d' "$file"
done
