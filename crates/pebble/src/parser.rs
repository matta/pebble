use crate::models::{TaskFrontmatter, TaskNode};
use color_eyre::eyre::{Result, eyre};
use std::path::Path;

/// Parses a Markdown file with TOML frontmatter into a [`TaskNode`].
///
/// The file must start with a TOML frontmatter block delimited by `+++` on the first
/// line and another `+++` on a subsequent line. The content after the second delimiter
/// is treated as the task body.
///
/// # Errors
///
/// Returns an error if:
/// * The file does not start with `+++`.
/// * The closing `+++` delimiter is missing.
/// * The TOML content cannot be parsed into [`TaskFrontmatter`].
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use pebble::parser::parse_task_file;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let content = r#"+++
/// id = "issue-1"
/// title = "Fix bug"
/// status = "todo"
/// created_at = 2023-01-01T00:00:00Z
/// +++
///
/// Description of the bug."#;
///
/// let node = parse_task_file(Path::new("issue-1.md"), content)?;
/// assert_eq!(node.frontmatter.title, "Fix bug");
/// assert_eq!(node.body.trim(), "Description of the bug.");
/// # Ok(())
/// # }
/// ```
pub fn parse_task_file(path: &Path, content: &str) -> Result<TaskNode> {
    let lines: Vec<&str> = content.lines().collect();

    // Frontmatter must start on the first line.
    if lines.is_empty() || lines[0].trim() != "+++" {
        return Err(eyre!(
            "Missing or invalid TOML frontmatter: file must start with '+++'"
        ));
    }

    // Find the end of the frontmatter.
    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "+++" {
            end_idx = Some(i);
            break;
        }
    }

    let end_idx = end_idx.ok_or_else(|| eyre!("Missing closing '+++' for TOML frontmatter"))?;

    // Extract frontmatter string.
    let toml_str = lines[1..end_idx].join("\n");
    let frontmatter: TaskFrontmatter =
        toml::from_str(&toml_str).map_err(|e| eyre!("Failed to parse TOML frontmatter: {}", e))?;

    // Extract body, stripping leading newlines after the closing '+++'.
    let body_lines = &lines[end_idx + 1..];

    // We want to reconstruct the body. We can use `join("\n")`,
    // but if the file ended with a newline we might want to be faithful.
    // For now, joining lines is standard.
    let mut body = body_lines.join("\n");
    if !body.is_empty() && content.ends_with('\n') {
        body.push('\n');
    }

    // Trim leading whitespace/newlines from the body to drop the immediate gap after ---
    let body = body.trim_start().to_string();

    Ok(TaskNode {
        path: path.to_path_buf(),
        frontmatter,
        body,
    })
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests {
    use super::*;
    use crate::models::TaskStatus;

    #[test]
    fn test_parse_valid_yaml_task() {
        let content = r#"---
id: issue-1
title: Test
status: todo
created_at: 2026-02-21T17:00:00Z
---

# Body
This is the body.
"#;
        let node = parse_task_file(Path::new("issue-1.md"), content)
            .expect("Should parse valid task file");
        assert_eq!(node.frontmatter.id, "issue-1");
        assert_eq!(node.frontmatter.title, "Test");
        assert_eq!(node.frontmatter.status, TaskStatus::Todo);
        assert_eq!(node.body, "# Body\nThis is the body.\n");
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# Just a markdown file";
        let err = parse_task_file(Path::new("file.md"), content)
            .expect_err("Should fail when frontmatter is missing");
        assert!(err.to_string().contains("must start with '---'"));
    }

    #[test]
    fn test_parse_unclosed_yaml_frontmatter() {
        let content = r#"---
id: issue-1
title: Test
status: todo
created_at: 2026-02-21T17:00:00Z
"#;
        let err = parse_task_file(Path::new("file.md"), content)
            .expect_err("Should fail when frontmatter is unclosed");
        assert!(err.to_string().contains("Missing closing '---'"));
    }

    #[test]
    fn test_parse_invalid_yaml_frontmatter() {
        let content = r#"---
id: issue-1
title: Test
status: invalid_status
created_at: 2026-02-21T17:00:00Z
---"#;
        let err = parse_task_file(Path::new("file.md"), content)
            .expect_err("Should fail when frontmatter is invalid YAML");
        assert!(err.to_string().contains("Failed to parse YAML frontmatter"));
    }

    #[test]
    fn test_parse_legacy_toml_frontmatter_treated_as_missing_yaml() {
        let content = r#"+++
id = "issue-1"
title = "Test"
status = "todo"
created_at = 2026-02-21T17:00:00Z
+++"#;
        let err = parse_task_file(Path::new("file.md"), content)
            .expect_err("Legacy TOML frontmatter should be treated as missing YAML frontmatter");
        assert!(err.to_string().contains("must start with '---'"));
    }
}
