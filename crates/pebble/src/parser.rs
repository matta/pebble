use crate::models::{FRONTMATTER_DELIMITER, Priority, TaskFrontmatter, TaskNode, TaskStatus};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct YamlTaskFrontmatter {
    id: String,
    title: String,
    status: TaskStatus,
    priority: Option<Priority>,
    created_at: Option<String>,
    modified_at: Option<String>,
    resolved_at: Option<String>,
    #[serde(default)]
    needs: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

fn parse_optional_datetime(value: Option<String>, field: &str) -> Result<Option<DateTime<Utc>>> {
    value
        .map(|raw| {
            DateTime::parse_from_rfc3339(&raw)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|err| eyre!("Invalid '{}' datetime '{}': {}", field, raw, err))
        })
        .transpose()
}

fn extract_body_after_frontmatter(content: &str, end_idx: usize) -> &str {
    let mut offset = 0usize;

    for (i, segment) in content.split_inclusive('\n').enumerate() {
        offset += segment.len();
        if i == end_idx {
            return &content[offset..];
        }
    }

    ""
}

/// Parses a Markdown file with YAML frontmatter into a [`TaskNode`].
///
/// The file must start with a YAML frontmatter block delimited by `---` on the first
/// line and another `---` on a subsequent line. The content after the second delimiter
/// is treated as the task body.
///
/// # Errors
///
/// Returns an error if:
/// * The file does not start with `---`.
/// * The closing `---` delimiter is missing.
/// * The YAML content cannot be parsed into [`TaskFrontmatter`].
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use pebble::parser::parse_task_file;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let content = r#"---
/// id: issue-1
/// title: Fix bug
/// status: todo
/// created_at: 2023-01-01T00:00:00Z
/// ---
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
    if lines.is_empty() || lines[0].trim() != FRONTMATTER_DELIMITER {
        return Err(eyre!(
            "Missing or invalid YAML frontmatter: file must start with '{FRONTMATTER_DELIMITER}'"
        ));
    }

    // Find the end of the frontmatter.
    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == FRONTMATTER_DELIMITER {
            end_idx = Some(i);
            break;
        }
    }

    let end_idx = end_idx
        .ok_or_else(|| eyre!("Missing closing '{FRONTMATTER_DELIMITER}' for YAML frontmatter"))?;

    // Extract frontmatter string.
    let yaml_str = lines[1..end_idx].join("\n");
    let raw_frontmatter: YamlTaskFrontmatter = serde_saphyr::from_str(&yaml_str)
        .map_err(|e| eyre!("Failed to parse YAML frontmatter: {}", e))?;

    let frontmatter = TaskFrontmatter {
        id: raw_frontmatter.id,
        title: raw_frontmatter.title,
        status: raw_frontmatter.status,
        priority: raw_frontmatter.priority,
        created_at: parse_optional_datetime(raw_frontmatter.created_at, "created_at")?,
        modified_at: parse_optional_datetime(raw_frontmatter.modified_at, "modified_at")?,
        resolved_at: parse_optional_datetime(raw_frontmatter.resolved_at, "resolved_at")?,
        needs: raw_frontmatter.needs,
        tags: raw_frontmatter.tags,
        extra: raw_frontmatter.extra,
    };

    let body = extract_body_after_frontmatter(content, end_idx);

    Ok(TaskNode {
        path: path.to_path_buf(),
        frontmatter,
        body: body.to_string(),
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
created_at: "2026-02-21T17:00:00Z"
---

# Body
This is the body.
"#;
        let node = parse_task_file(Path::new("issue-1.md"), content)
            .expect("Should parse valid task file");
        assert_eq!(node.frontmatter.id, "issue-1");
        assert_eq!(node.frontmatter.title, "Test");
        assert_eq!(node.frontmatter.status, TaskStatus::todo());
        assert_eq!(node.body, "\n# Body\nThis is the body.\n");
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# Just a markdown file";
        let err = parse_task_file(Path::new("file.md"), content)
            .expect_err("Should fail when frontmatter is missing");
        assert!(
            err.to_string()
                .contains(&format!("must start with '{FRONTMATTER_DELIMITER}'"))
        );
    }

    #[test]
    fn test_parse_unclosed_yaml_frontmatter() {
        let content = r#"---
id: issue-1
title: Test
status: todo
created_at: "2026-02-21T17:00:00Z"
"#;
        let err = parse_task_file(Path::new("file.md"), content)
            .expect_err("Should fail when frontmatter is unclosed");
        assert!(
            err.to_string()
                .contains(&format!("Missing closing '{FRONTMATTER_DELIMITER}'"))
        );
    }

    #[test]
    fn test_parse_invalid_yaml_frontmatter() {
        let content = r#"---
id: issue-1
title: Test
status: invalid_status
created_at: "2026-02-21T17:00:00Z"
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
        assert!(
            err.to_string()
                .contains(&format!("must start with '{FRONTMATTER_DELIMITER}'"))
        );
    }

    #[test]
    fn test_parse_yaml_delimiters_with_trailing_spaces() {
        let content = "---   \n\
id: issue-1\n\
title: Test\n\
status: todo\n\
created_at: \"2026-02-21T17:00:00Z\"\n\
---   \n\
\n\
# Body\n\
This is the body.\n";

        let node = parse_task_file(Path::new("issue-1.md"), content)
            .expect("Should parse valid YAML with trailing spaces on both delimiters");
        assert_eq!(node.frontmatter.id, "issue-1");
        assert_eq!(node.body, "\n# Body\nThis is the body.\n");
    }
}
