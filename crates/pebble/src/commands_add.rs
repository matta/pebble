use crate::commands::{RunContext, TaskObject, validate_task_references};
use crate::graph::TaskGraph;
use crate::models::{Priority, TaskFrontmatter, TaskNode, TaskStatus};
use crate::task_io::current_toml_time;
use color_eyre::eyre::{Result, eyre};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const ID_ALPHABET: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const MAX_SLUG_LEN: usize = 80;

fn required_random_id_length(n: usize) -> usize {
    if n == 0 {
        return 8;
    }
    let n_f: f64 = n as f64;
    let target_prob: f64 = 1e-12;
    let alphabet_size: f64 = 36.0;
    let l: f64 = ((n_f * n_f) / (2.0 * target_prob)).ln() / alphabet_size.ln();
    l.ceil().max(8.0) as usize
}

fn unique_task_path(tasks_dir: &Path, title: &str) -> PathBuf {
    let base_slug = slugify(title);
    let mut filename = format!("{}.md", base_slug);
    let mut filepath = tasks_dir.join(&filename);
    let mut counter = 2;

    // TODO(pebble: docs/pebble/toctou-race-in-slug-collision-loop.md): Use create_new + retry to make filename selection atomic.
    while filepath.exists() {
        filename = format!("{}-{}.md", base_slug, counter);
        filepath = tasks_dir.join(&filename);
        counter += 1;
    }
    filepath
}

fn apply_reverse_blocks(
    graph: &mut TaskGraph,
    block_targets: Vec<String>,
    new_id: &str,
) -> Result<()> {
    for target_id in block_targets {
        let mut target_node = graph
            .nodes
            .get(&target_id)
            .cloned()
            .ok_or_else(|| eyre!("Task '{}' not found for --blocks", target_id))?;

        if target_node
            .frontmatter
            .needs
            .iter()
            .any(|need| need == new_id)
        {
            continue;
        }

        target_node.frontmatter.needs.push(new_id.to_string());
        target_node.frontmatter.modified_at = Some(current_toml_time()?);
        target_node.write_to_disk()?;
        graph.nodes.insert(target_id, target_node);
    }
    Ok(())
}

pub fn slugify(s: &str) -> String {
    let mut slug = String::with_capacity(s.len());
    let mut last_was_dash = false;

    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if c == '_' {
            slug.push('_');
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.truncate(slug.trim_end_matches('-').len());
    if slug.len() > MAX_SLUG_LEN {
        slug.truncate(MAX_SLUG_LEN);
        slug.truncate(slug.trim_end_matches('-').len());
    }
    if slug.is_empty() {
        "task".to_string()
    } else {
        slug
    }
}

pub struct RunAddInput {
    pub title: String,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub body: Option<String>,
    pub needs: Vec<String>,
    pub tags: Vec<String>,
    pub blocks: Vec<String>,
}

pub fn run_add(ctx: &RunContext, input: RunAddInput) -> Result<()> {
    let RunAddInput {
        title,
        status,
        priority,
        body,
        needs,
        tags,
        blocks,
    } = input;
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let deduped_blocks = validate_task_references(&graph, blocks, None, "--blocks")?;

    let random_length = required_random_id_length(graph.nodes.len());
    let id_str = nanoid::nanoid!(random_length, ID_ALPHABET);
    let new_id = format!("{}-{}", ctx.config.issue_prefix, id_str);
    let status = status.unwrap_or(TaskStatus::Todo);
    let created_at = current_toml_time()?;
    let resolved_at = if status.is_closed() {
        Some(created_at)
    } else {
        None
    };

    let frontmatter = TaskFrontmatter {
        id: new_id.clone(),
        title: title.clone(),
        status,
        priority,
        created_at: Some(created_at),
        modified_at: None,
        resolved_at,
        needs,
        tags,
        extra: HashMap::new(),
    };

    fs::create_dir_all(&ctx.tasks_dir)?;
    let node = TaskNode {
        path: unique_task_path(&ctx.tasks_dir, &title),
        frontmatter,
        body: body.unwrap_or_default(),
    };
    node.write_to_disk()?;

    apply_reverse_blocks(&mut graph, deduped_blocks, &new_id)?;
    graph
        .nodes
        .insert(node.frontmatter.id.clone(), node.clone());

    let current_dir = &ctx.current_dir;
    let path_for_output = node.path.strip_prefix(current_dir).unwrap_or(&node.path);
    let display_path = path_for_output.display().to_string();

    if ctx.json {
        let updated_graph = TaskGraph::new(graph.nodes);
        let mut obj = TaskObject::from_node(&node, &updated_graph, &ctx.tasks_dir);
        obj.path = display_path;
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        eprintln!("Created task {} at {}", new_id, display_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::slugify;

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
}
