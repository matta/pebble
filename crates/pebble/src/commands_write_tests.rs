use super::*;

#[test]
fn test_slugify_basic() {
    assert_eq!(slugify("Implement Task Node"), "implement-task-node");
    assert_eq!(slugify("  Lots   of  Spaces  "), "lots-of-spaces");
    assert_eq!(
        slugify("Punctuation! (is) removed?"),
        "punctuation-is-removed"
    );
    assert_eq!(slugify("Already-Slugified"), "already-slugified");
}

#[test]
fn test_slugify_mixed_separators() {
    assert_eq!(
        slugify("mix_of_dashes-and_underscores"),
        "mix_of_dashes-and_underscores"
    );
    assert_eq!(slugify("---Trim-Repeating---"), "trim-repeating");
    assert_eq!(slugify("123-Numbers-456"), "123-numbers-456");
}

#[test]
fn test_slugify_empty_fallback() {
    assert_eq!(slugify(""), "task");
    assert_eq!(slugify("!!!"), "task");
}

#[test]
fn test_slugify_reserved_chars() {
    // Strict character set tests (reserved characters become delimiters)
    assert_eq!(slugify("Windows: < > : \" / \\ | ? *"), "windows");
    assert_eq!(slugify("macOS: / and :"), "macos-and");
    assert_eq!(slugify("Linux/Unix: \0 and /"), "linux-unix-and");
}

#[test]
fn test_run_add_collision_logic() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let tasks_dir = temp_dir.path().to_path_buf();
    let ctx = RunContext {
        project_root: None,
        config: Default::default(),
        tasks_dir: tasks_dir.clone(),
        json: false,
    };

    // First add
    run_add(
        &ctx,
        "My Task".to_string(),
        None,
        None,
        None,
        vec![],
        vec![],
    )?;
    assert!(tasks_dir.join("my-task.md").exists());

    // Second add (collision)
    run_add(
        &ctx,
        "My Task".to_string(),
        None,
        None,
        None,
        vec![],
        vec![],
    )?;
    assert!(tasks_dir.join("my-task-2.md").exists());

    // Third add (collision)
    run_add(
        &ctx,
        "My Task".to_string(),
        None,
        None,
        None,
        vec![],
        vec![],
    )?;
    assert!(tasks_dir.join("my-task-3.md").exists());

    Ok(())
}
