use anyhow::Result;
use predicates::prelude::*;
use std::fs;

use super::support::setup_test_env;

#[test]
fn test_check_fix_normalizes_frontmatter() -> Result<()> {
    let env = setup_test_env();

    // Create a "dirty" task file manually:
    // - empty needs/tags lists
    // - no trailing newline
    let dirty_content = "---
id: pebl-dirty
title: Dirty Task
status: todo
created_at: 2026-03-01T00:00:00+00:00
needs: []
tags: []
---
Body content"; // No trailing newline here

    let task_path = env.tasks_dir.join("dirty-task.md");
    fs::write(&task_path, dirty_content)?;

    // 1. pebble check should fail
    env.pebble()
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Task file is not canonical"));

    // 2. pebble check --fix should succeed
    env.pebble()
        .args(["check", "--fix"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Fixed 1 task(s)"));

    // 3. Verify file content is now normalized
    let normalized_content = fs::read_to_string(&task_path)?;
    assert!(!normalized_content.contains("needs: []"));
    assert!(!normalized_content.contains("tags: []"));
    assert!(
        normalized_content.ends_with('\n'),
        "file should end with exactly one newline"
    );
    assert!(
        !normalized_content.ends_with("\n\n"),
        "file should not end with multiple newlines"
    );
    assert!(normalized_content.contains("---"));

    // 4. pebble check should now pass
    env.pebble()
        .arg("check")
        .assert()
        .success()
        .stderr(predicate::str::contains("Graph is healthy"));

    Ok(())
}

#[test]
fn test_check_fix_normalizes_missing_newline() -> Result<()> {
    let env = setup_test_env();

    // Create a task file with correct frontmatter but missing trailing newline
    let content = "---
id: pebl-nonl
title: No Newline
status: todo
created_at: 2026-03-01T00:00:00+00:00
---
Body without newline";

    let task_path = env.tasks_dir.join("nonl.md");
    fs::write(&task_path, content)?;

    // 1. pebble check should fail due to missing newline
    env.pebble()
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Task file is not canonical"));

    // 2. pebble check --fix should succeed
    env.pebble().args(["check", "--fix"]).assert().success();

    // 3. Verify file content now has a newline and is trimmed
    let normalized_content = fs::read_to_string(&task_path)?;
    assert!(
        normalized_content.ends_with('\n'),
        "file should have a trailing newline"
    );
    // The canonical output will have the body trimmed and separated by a blank line.
    assert!(
        normalized_content.contains("---\n\nBody without newline\n"),
        "body should be separated by a blank line and followed by a newline"
    );

    Ok(())
}

#[test]
fn test_check_fix_normalizes_extra_newlines() -> Result<()> {
    let env = setup_test_env();

    // Create a task file with multiple trailing newlines
    let content = "---
id: pebl-extra-nl
title: Extra Newlines
status: todo
created_at: 2026-03-01T00:00:00+00:00
---
Body with extra newlines\n\n\n";

    let task_path = env.tasks_dir.join("extra-nl.md");
    fs::write(&task_path, content)?;

    // Verify written content is actually dirty
    let written = fs::read_to_string(&task_path)?;
    assert!(
        written.ends_with("\n\n\n"),
        "test setup should create extra newlines"
    );

    // 1. pebble check should fail
    env.pebble()
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Task file is not canonical"));

    // 2. pebble check --fix should succeed
    env.pebble().args(["check", "--fix"]).assert().success();

    // 3. Verify file content now has exactly one newline
    let normalized_content = fs::read_to_string(&task_path)?;
    assert!(
        normalized_content.ends_with('\n'),
        "file should end with exactly one newline"
    );
    assert!(
        !normalized_content.ends_with("\n\n"),
        "file should not have extra newlines"
    );

    Ok(())
}
