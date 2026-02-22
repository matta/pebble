use crate::models::{TaskFrontmatter, TaskNode};
use color_eyre::eyre::{Result, eyre};
use std::path::Path;

/// Parses a Markdown file with YAML frontmatter into a TaskNode.
pub fn parse_task_file(path: &Path, content: &str) -> Result<TaskNode> {
    // Zero-allocation parsing strategy:
    // 1. Check start line.
    // 2. Scan for closing "---" using slicing instead of splitting all lines.

    // 1. Check start
    let first_line_end = content.find('\n').unwrap_or(content.len());
    let first_line = &content[0..first_line_end];

    if first_line.trim() != "---" {
        return Err(eyre!(
            "Missing or invalid YAML frontmatter: file must start with '---'"
        ));
    }

    let rest_start = if first_line_end < content.len() {
        first_line_end + 1
    } else {
        content.len()
    };

    // 2. Find closing "---"
    let mut fm_end_start = 0;
    let mut fm_end_end = 0;
    let mut found = false;
    let mut search_idx = rest_start;

    while let Some(offset) = content[search_idx..].find('\n') {
        let newline_idx = search_idx + offset;
        let line = &content[search_idx..newline_idx];

        if line.trim() == "---" {
            fm_end_start = search_idx;
            fm_end_end = newline_idx;
            found = true;
            break;
        }

        search_idx = newline_idx + 1;
    }

    if !found {
        // Check last line (EOF case)
        let last_line = &content[search_idx..];
        if last_line.trim() == "---" {
            fm_end_start = search_idx;
            fm_end_end = content.len();
            found = true;
        }
    }

    if !found {
        return Err(eyre!("Missing closing '---' for YAML frontmatter"));
    }

    // Extract frontmatter
    let yaml_slice = &content[rest_start..fm_end_start];
    let frontmatter: TaskFrontmatter = serde_yaml::from_str(yaml_slice)
        .map_err(|e| eyre!("Failed to parse YAML frontmatter: {}", e))?;

    // Extract body
    let body_start = if fm_end_end < content.len() {
        fm_end_end + 1
    } else {
        content.len()
    };
    let body_slice = &content[body_start..];

    // Optimize: trim first to reduce size, then normalize newlines.
    // Normalization (CRLF -> LF) ensures consistency with previous behavior and tests.
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
