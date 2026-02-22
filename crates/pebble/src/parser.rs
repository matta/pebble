use crate::models::{TaskFrontmatter, TaskNode};
use color_eyre::eyre::{Result, eyre};
use std::path::Path;

/// Parses a Markdown file with YAML frontmatter into a TaskNode.
pub fn parse_task_file(path: &Path, content: &str) -> Result<TaskNode> {
    let lines: Vec<&str> = content.lines().collect();

    // Frontmatter must start on the first line.
    if lines.is_empty() || lines[0].trim() != "---" {
        return Err(eyre!(
            "Missing or invalid YAML frontmatter: file must start with '---'"
        ));
    }

    // Find the end of the frontmatter.
    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = Some(i);
            break;
        }
    }

    let end_idx = end_idx.ok_or_else(|| eyre!("Missing closing '---' for YAML frontmatter"))?;

    // Extract frontmatter string.
    let yaml_str = lines[1..end_idx].join("\n");
    let frontmatter: TaskFrontmatter = serde_yaml::from_str(&yaml_str)
        .map_err(|e| eyre!("Failed to parse YAML frontmatter: {}", e))?;

    // Extract body, stripping leading newlines after the closing '---'.
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
mod tests {
    use super::*;
    use crate::models::TaskStatus;

    #[test]
    fn test_parse_valid_task() {
        let content = r#"---
id: issue-1
title: Test
status: todo
created_at: 2026-02-21T17:00:00Z
---

# Body
This is the body.
"#;
        let node = parse_task_file(Path::new("issue-1.md"), content).unwrap();
        assert_eq!(node.frontmatter.id, "issue-1");
        assert_eq!(node.frontmatter.title, "Test");
        assert_eq!(node.frontmatter.status, TaskStatus::Todo);
        assert_eq!(node.body, "# Body\nThis is the body.\n");
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# Just a markdown file";
        let err = parse_task_file(Path::new("file.md"), content).unwrap_err();
        assert!(err.to_string().contains("must start with '---'"));
    }

    #[test]
    fn test_parse_unclosed_frontmatter() {
        let content = r#"---
id: issue-1
title: Test
status: todo
created_at: 2026-02-21T17:00:00Z
"#;
        let err = parse_task_file(Path::new("file.md"), content).unwrap_err();
        assert!(err.to_string().contains("Missing closing '---'"));
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let content = r#"---
id: issue-1
title: Test
status: invalid_status
created_at: 2026-02-21T17:00:00Z
---"#;
        let err = parse_task_file(Path::new("file.md"), content).unwrap_err();
        assert!(err.to_string().contains("Failed to parse YAML frontmatter"));
    }
}
