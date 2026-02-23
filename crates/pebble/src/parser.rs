use crate::models::{TaskFrontmatter, TaskNode};
use color_eyre::eyre::{Result, eyre};
use std::path::Path;

/// Parses a Markdown file with TOML frontmatter into a TaskNode.
pub fn parse_task_file(path: &Path, content: &str) -> Result<TaskNode> {
    // Determine where the TOML content starts (after the first line)
    let first_newline = content.find('\n');
    let first_line_len = first_newline.unwrap_or(content.len());
    let first_line = &content[..first_line_len];

    // Frontmatter must start on the first line.
    if first_line.trim() != "+++" {
        return Err(eyre!(
            "Missing or invalid TOML frontmatter: file must start with '+++'"
        ));
    }

    let toml_start = if let Some(idx) = first_newline {
        idx + 1
    } else {
        // Only one line "+++", which is not valid but let the loop handle missing closing delimiter
        first_line_len
    };

    // Find the closing "+++"
    // It must be at the start of a line, so we look for "\n+++"
    let mut search_start = toml_start;
    let mut end_offset = None;

    while let Some(relative_idx) = content[search_start..].find("\n+++") {
        let newline_idx = search_start + relative_idx;
        let line_start = newline_idx + 1;
        let rest = &content[line_start..];

        // Find end of this line to check if it's exactly "+++" (ignoring whitespace)
        let line_len = rest.find('\n').unwrap_or(rest.len());
        let line_str = &rest[..line_len];

        if line_str.trim() == "+++" {
            // Found the delimiter
            let body_start = if line_start + line_len < content.len() {
                line_start + line_len + 1 // Skip the newline after +++
            } else {
                content.len()
            };
            end_offset = Some((newline_idx, body_start));
            break;
        }

        // Not the delimiter, advance search
        search_start = line_start + line_len;
    }

    let (toml_end, body_start) =
        end_offset.ok_or_else(|| eyre!("Missing closing '+++' for TOML frontmatter"))?;

    let toml_str = &content[toml_start..toml_end];
    let frontmatter: TaskFrontmatter =
        toml::from_str(toml_str).map_err(|e| eyre!("Failed to parse TOML frontmatter: {}", e))?;

    // Extract body
    let raw_body = &content[body_start..];
    let trimmed_body = raw_body.trim_start();
    let body = if trimmed_body.contains("\r\n") {
        trimmed_body.replace("\r\n", "\n")
    } else {
        trimmed_body.to_string()
    };

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
        let content = r#"+++
id = "issue-1"
title = "Test"
status = "todo"
created_at = 2026-02-21T17:00:00Z
+++

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
        assert!(err.to_string().contains("must start with '+++'"));
    }

    #[test]
    fn test_parse_unclosed_frontmatter() {
        let content = r#"+++
id = "issue-1"
title = "Test"
status = "todo"
created_at = 2026-02-21T17:00:00Z
"#;
        let err = parse_task_file(Path::new("file.md"), content).unwrap_err();
        assert!(err.to_string().contains("Missing closing '+++'"));
    }

    #[test]
    fn test_parse_invalid_toml() {
        let content = r#"+++
id = "issue-1"
title = "Test"
status = "invalid_status"
created_at = 2026-02-21T17:00:00Z
+++"#;
        let err = parse_task_file(Path::new("file.md"), content).unwrap_err();
        assert!(err.to_string().contains("Failed to parse TOML frontmatter"));
    }
}
