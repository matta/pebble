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
/// use std::path::Path;
/// use pebble::parser::parse_task_file;
///
/// let content = r#"+++
/// id = "issue-1"
/// title = "Fix bug"
/// status = "todo"
/// created_at = 2023-01-01T00:00:00Z
/// +++
///
/// Description of the bug."#;
///
/// let node = parse_task_file(Path::new("issue-1.md"), content).unwrap();
/// assert_eq!(node.frontmatter.title, "Fix bug");
/// assert_eq!(node.body.trim(), "Description of the bug.");
/// ```
pub fn parse_task_file(path: &Path, content: &str) -> Result<TaskNode> {
    // Frontmatter must start on the first line.
    let (first_line, rest) = match content.split_once('\n') {
        Some((l, r)) => (l, r),
        None => (content, ""),
    };

    if first_line.trim() != "+++" {
        return Err(eyre!(
            "Missing or invalid TOML frontmatter: file must start with '+++'"
        ));
    }

    // Find the end of the frontmatter (the second "+++" line).
    let mut toml_end_offset = None;
    let mut search_head = rest;
    let mut relative_offset = 0;

    loop {
        match search_head.split_once('\n') {
            Some((line, remaining)) => {
                if line.trim() == "+++" {
                    toml_end_offset = Some(relative_offset);
                    break;
                }
                relative_offset += line.len() + 1; // +1 for the newline
                search_head = remaining;
            }
            None => {
                // Check the last line (if it doesn't end with a newline)
                if search_head.trim() == "+++" {
                    toml_end_offset = Some(relative_offset);
                }
                break;
            }
        }
    }

    let toml_len = toml_end_offset.ok_or_else(|| eyre!("Missing closing '+++' for TOML frontmatter"))?;

    // Slice the TOML content directly from the source string.
    let toml_str = &rest[..toml_len];

    // Find the start of the body.
    // The closing "+++" line starts at `toml_len`.
    // We need to skip the closing delimiter line to get to the body.
    // We find the newline after the closing delimiter.
    let closing_line_start = toml_len;
    let body_start = match rest[closing_line_start..].find('\n') {
        Some(idx) => closing_line_start + idx + 1,
        None => rest.len(), // EOF after closing +++
    };

    let frontmatter: TaskFrontmatter =
        toml::from_str(toml_str).map_err(|e| eyre!("Failed to parse TOML frontmatter: {}", e))?;

    // Extract body, stripping leading whitespace/newlines.
    // We allocate a String here, but we avoided the intermediate Vec and joins.
    let body = rest[body_start..].trim_start().to_string();

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

    #[test]
    fn test_parse_crlf_task() {
        // Construct a string with CRLF line endings
        let content = "+++\r\nid = \"issue-crlf\"\r\ntitle = \"CRLF Task\"\r\nstatus = \"todo\"\r\ncreated_at = 2026-02-21T17:00:00Z\r\n+++\r\n\r\n# Body\r\nThis is the body.\r\n";
        let node = parse_task_file(Path::new("issue-crlf.md"), content).unwrap();
        assert_eq!(node.frontmatter.id, "issue-crlf");
        assert_eq!(node.frontmatter.title, "CRLF Task");
        assert_eq!(node.frontmatter.status, TaskStatus::Todo);
        // The parser preserves the body as-is (except trimming start)
        assert_eq!(node.body, "# Body\r\nThis is the body.\r\n");
    }
}
