#!/bin/bash
for file in crates/pebble/tests/cli_json_purity_extended.rs crates/pebble/tests/cli_help_json.rs crates/pebble/tests/cli_check.rs; do
  sed -i -e 's/Command::new(cargo_bin!("pebble"))/env.pebble()/g' "$file"
  sed -i -e 's/Command::new(cargo_bin!())/env.pebble()/g' "$file"
  sed -i -e '/\.current_dir/d' "$file"
  sed -i -e '/use assert_cmd::cargo_bin;/d' "$file"
  sed -i -e '/use std::process::Command;/d' "$file"
  sed -i -e '/use assert_cmd::Command;/d' "$file"
done
