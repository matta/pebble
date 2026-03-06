use crate::commands::{
    RunContext, TaskObject, read_stdin_if_dash, update_reverse_dependencies,
    validate_task_references,
};
use crate::graph::TaskGraph;
use crate::models::{Priority, TaskFrontmatter, TaskNode, TaskStatus};
use crate::task_io::current_task_time;
use color_eyre::eyre::{Result, eyre};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

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

pub fn slugify(s: &str) -> String {
    let transliterated = deunicode::deunicode(s);
    let mut slug = String::with_capacity(transliterated.len());
    let mut last_was_dash = false;

    for c in transliterated.chars() {
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

fn create_node_with_unique_filename(
    tasks_dir: &Path,
    title: &str,
    frontmatter: TaskFrontmatter,
    body: Option<String>,
) -> Result<TaskNode> {
    let base_slug = slugify(title);
    let mut filename = format!("{}.md", base_slug);
    let mut filepath = tasks_dir.join(&filename);
    let mut counter = 2;

    let mut node = TaskNode {
        path: filepath.clone(),
        frontmatter,
        body: body.unwrap_or_default(),
    };

    loop {
        if counter > 1000 {
            return Err(eyre!(
                "Failed to find a unique filename for task after 1000 attempts: {}",
                title
            ));
        }

        match node.create_new_to_disk() {
            Ok(_) => break,
            Err(e)
                if e.downcast_ref::<Error>()
                    .map(|io| io.kind() == ErrorKind::AlreadyExists)
                    .unwrap_or(false) =>
            {
                filename = format!("{}-{}.md", base_slug, counter);
                filepath = tasks_dir.join(&filename);
                node.path = filepath.clone();
                counter += 1;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(node)
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
    let body = read_stdin_if_dash(body)?;
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let deduped_blocks = validate_task_references(&graph, blocks, None, "--blocks")?;

    let random_length = required_random_id_length(graph.nodes.len());
    let id_str = nanoid::nanoid!(random_length, ID_ALPHABET);
    let new_id = format!("{}-{}", ctx.config.issue_prefix, id_str);
    let status = status.unwrap_or(TaskStatus::todo());
    let created_at = current_task_time();
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
        extra: BTreeMap::new(),
    };

    fs::create_dir_all(&ctx.tasks_dir)?;

    let node = create_node_with_unique_filename(&ctx.tasks_dir, &title, frontmatter, body)?;

    update_reverse_dependencies(&mut graph, &new_id, deduped_blocks, Vec::<String>::new())?;
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

    #[test]
    fn test_slugify_transliterates_non_ascii() {
        assert_eq!(slugify("café"), "cafe");
        assert_eq!(slugify("über cool"), "uber-cool");
        assert_eq!(slugify("naïve approach"), "naive-approach");
        assert_eq!(slugify("résumé"), "resume");
        assert_eq!(slugify("Æneid"), "aeneid");
        assert_eq!(slugify("日本語テスト"), "ri-ben-yu-tesuto");
    }

    #[test]
    fn test_slugify_strips_newlines_from_transliteration() {
        // U+2028 LINE SEPARATOR transliterates to "\n"
        assert_eq!(slugify("before\u{2028}after"), "before-after");
        // U+2029 PARAGRAPH SEPARATOR transliterates to "\n\n"
        assert_eq!(slugify("before\u{2029}after"), "before-after");
        // Multiple separators should not produce consecutive dashes
        assert_eq!(slugify("a\u{2028}\u{2029}b"), "a-b");
    }

    #[test]
    fn test_slugify_handles_unknown_characters() {
        // U+0378 is unknown to deunicode, producing "[?]"
        assert_eq!(slugify("test\u{0378}value"), "test-value");
        // Only unknown characters should fall back to "task"
        assert_eq!(slugify("\u{0378}\u{0379}"), "task");
    }

    #[test]
    fn test_slugify_handles_empty_transliterations() {
        // Control characters pass through deunicode as-is; slugify treats
        // them as non-alphanumeric separators.
        assert_eq!(slugify("\u{0001}hello"), "hello");
        assert_eq!(slugify("hello\u{0002}world"), "hello-world");
        // BOM (U+FEFF) and soft hyphen (U+00AD) transliterate to empty
        assert_eq!(slugify("\u{FEFF}hello"), "hello");
        assert_eq!(slugify("soft\u{00AD}hyphen"), "softhyphen");
    }

    #[test]
    fn test_slugify_handles_multi_char_transliterations() {
        // "北" transliterates to "Bei " (with trailing space)
        assert_eq!(slugify("北"), "bei");
        // "Ж" transliterates to "Zh"
        assert_eq!(slugify("Ж"), "zh");
        // Mixed multi-char transliterations
        assert_eq!(slugify("Жизнь"), "zhizn");
    }

    #[test]
    fn test_slugify_never_produces_unsafe_filenames() {
        let edge_cases = [
            "\u{2028}\u{2029}", // only newline producers
            "\u{0378}\u{0379}", // only unknowns
            "\u{0001}\u{0002}", // only empty transliterations
            "\u{200B}\u{FEFF}", // zero-width space + BOM
        ];
        for input in &edge_cases {
            let slug = slugify(input);
            assert!(!slug.is_empty(), "slug must never be empty for {input:?}");
            assert!(
                !slug.contains('\n'),
                "slug must never contain newline for {input:?}"
            );
            assert!(
                !slug.contains('/'),
                "slug must never contain slash for {input:?}"
            );
            assert!(
                !slug.contains('\0'),
                "slug must never contain null for {input:?}"
            );
            assert!(
                slug.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                "slug contains unexpected char for {input:?}: {slug:?}"
            );
        }
    }
}
