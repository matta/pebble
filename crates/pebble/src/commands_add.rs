use crate::commands::{RunContext, TaskObject};
use crate::graph::TaskGraph;
use crate::models::{TaskFrontmatter, TaskNode, TaskStatus};
use crate::task_io::{current_toml_time, write_task_file};
use color_eyre::eyre::{Result, eyre};
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

fn dedupe_and_validate_blocks(graph: &TaskGraph, blocks: Vec<String>) -> Result<Vec<String>> {
    let mut deduped_blocks = Vec::new();
    let mut seen_blocks = std::collections::HashSet::new();
    for target_id in blocks {
        if !seen_blocks.insert(target_id.clone()) {
            continue;
        }
        if graph.is_duplicate_id(&target_id) {
            return Err(eyre!(
                "Duplicate task ID '{}' found in multiple files; cannot safely target this ID.",
                target_id
            ));
        }
        if !graph.nodes.contains_key(&target_id) {
            return Err(eyre!("Task '{}' not found for --blocks", target_id));
        }
        deduped_blocks.push(target_id);
    }
    Ok(deduped_blocks)
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
        write_task_file(
            &target_node.path,
            &target_node.frontmatter,
            &target_node.body,
        )?;
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
    pub priority: Option<u8>,
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
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)
        .unwrap_or_else(|_| TaskGraph::new(Default::default()));
    let deduped_blocks = dedupe_and_validate_blocks(&graph, blocks)?;

    let random_length = required_random_id_length(graph.nodes.len());
    let id_str = nanoid::nanoid!(random_length, ID_ALPHABET);
    let new_id = format!("{}-{}", ctx.config.issue_prefix, id_str);
    let fm = TaskFrontmatter {
        id: new_id.clone(),
        title: title.clone(),
        status: status.unwrap_or(TaskStatus::Todo),
        priority,
        created_at: current_toml_time()?,
        modified_at: None,
        resolved_at: None,
        needs,
        tags,
    };

    std::fs::create_dir_all(&ctx.tasks_dir)?;
    let node = TaskNode {
        path: unique_task_path(&ctx.tasks_dir, &title),
        frontmatter: fm,
        body: body.unwrap_or_default(),
    };
    write_task_file(&node.path, &node.frontmatter, &node.body)?;

    apply_reverse_blocks(&mut graph, deduped_blocks, &new_id)?;
    graph
        .nodes
        .insert(node.frontmatter.id.clone(), node.clone());

    if ctx.json {
        let obj = TaskObject::from_node(&node, &graph, &ctx.tasks_dir);
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        eprintln!("Created task {} at {}", new_id, node.path.display());
    }

    Ok(())
}
