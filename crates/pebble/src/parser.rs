use crate::models::{TaskFrontmatter, TaskNode};
use color_eyre::eyre::{Result, eyre};
use std::path::Path;

/// Parses a Markdown file with YAML frontmatter into a TaskNode.
pub fn parse_task_file(path: &Path, content: &str) -> Result<TaskNode> {
    // 1. Ensure file starts with "---"
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(eyre!(
            "Missing or invalid YAML frontmatter: file must start with '---'"
        ));
    }

    // 2. Find the closing "---"
    // We look for a newline followed by "---" to ensure it's on its own line.
    // This handles both LF (\n---) and CRLF (\r\n---).
    let remainder = &content[3..];
    let end_offset = remainder
        .find("\n---")
        .ok_or_else(|| eyre!("Missing closing '---' for YAML frontmatter"))?;

    // 3. Extract and parse frontmatter (zero allocation slice)
    let yaml_slice = &remainder[..end_offset];
    let frontmatter: TaskFrontmatter = serde_yaml::from_str(yaml_slice)
        .map_err(|e| eyre!("Failed to parse YAML frontmatter: {}", e))?;

    // 4. Extract body
    // The body starts after the closing "---" (3 chars) and potentially a newline.
    // end_offset points to the \n before ---.
    // So the closing delimiter starts at end_offset + 1 (the "-").
    // Its length is 3. So end of delimiter is end_offset + 1 + 3 = end_offset + 4.
    let body_start = end_offset + 4;
    let body_slice = &remainder[body_start..];

    // Normalize body: trim start and normalize CRLF to LF
    let body = body_slice.trim_start().replace("\r\n", "\n");

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

    #[test]
    fn test_parse_crlf_content() {
        let content = "---\r\nid: issue-crlf\r\ntitle: CRLF Test\r\nstatus: todo\r\ncreated_at: 2026-02-21T17:00:00Z\r\n---\r\n\r\n# Body\r\nThis has CRLF.\r\n";
        let node = parse_task_file(Path::new("crlf.md"), content).unwrap();
        assert_eq!(node.frontmatter.id, "issue-crlf");
        // Current implementation normalizes body newlines to \n
        assert_eq!(node.body, "# Body\nThis has CRLF.\n");
    }
}
