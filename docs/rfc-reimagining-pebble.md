# RFC: Re-imagining Pebble from Scratch

## 1. Introduction & Motivation

The goal of this RFC is to step back and re-imagine Pebble from the ground up. Inspired by the legacy `bd` tool but now forging its own divergent path, the core mission remains unaltered: **provide a project task tracking system that is equally useful and delightful for both human developers and autonomous AI coding agents**.

While the current implementation relies on a Rust CLI with a JSONL storage backbone, this document explores the solution space without those constraints. We aim for a "minimum useful feature set" tailored not for enormous enterprise projects or for coordinating concurrently running autonomous AI agents, but for the simpler, single-repo projects common in open-source development and indie hacking.

## Key Decisions (TL;DR)
- Store tasks as Markdown files in a visible repo directory (default `docs/pebble/`).
- YAML frontmatter defines metadata; the Markdown body is the description.
- `id` is canonical and user-editable; the CLI never changes it.
- Relationships: store `after` (prerequisites), compute `before` as inverse.
- Status model: `todo`, `in_progress`, `done`, `canceled`.
- Omit audit fields (`owner`, `created_by`, `updated_at`, `closed_at`, `close_reason`); rely on git history.
- CLI reads/writes Markdown directly; no hidden worktrees.
- One-time migration via a throw-away script from the existing JSONL.

## 2. Minimum Useful Feature Set

Based on the `golden.jsonl` data and typical single-repo development flows, the essential feature set is surprisingly small:

1. **Task Tracking:** Ability to define a task with an ID, title, description, and status.
   - States: `todo`, `in_progress`, `done`, `canceled` (aligned with GitHub Projects terminology).
   - `canceled` means "not done and will never be done."
2. **Hierarchy & Composition:** Epics and subtasks. A task can be heavily composed of smaller tasks (`parent/child`).
3. **Ordering & Dependencies:** Execution ordering. Knowing what to do *next* is critical for agents. We need `before` / `after` relationships; "blocked" is derived from unmet `after` prerequisites.
4. **Basic Metadata:** Creation timestamp (`created_at`) only. Audit trail (owner, closed_at, close_reason) is intentionally omitted and delegated to Git history.

**Notes in Markdown:** Users and agents are free to include checklists in the Markdown body. We should consider future support for task ID auto-discovery in the body (e.g., `proj-123`) to enable "related to" queries or semantic linking without expanding frontmatter.


## 3. Technical Specification

**Decision:** Adopt **per-issue Markdown files with YAML frontmatter** stored under a visible, human-friendly directory such as `docs/pebble/`. The Markdown files are the source of truth. Any caches or indexes are strictly derived and optional.

**Rationale:** This maximizes human transparency, makes review diffs first-class in Git, and keeps agent tooling aligned with what humans see. It also eliminates single-file merge conflicts while keeping the architecture simple.

**Non-Goals:**
- Not targeting enterprise-scale analytics or cross-repo issue federation.
- **Model Context Protocol (MCP) Server:** Building an MCP server is explicitly a post-1.0/post-MVP decision. AI agents perform perfectly well interacting via a local CLI. 

**Scope & Risk Profile:** There are currently zero active Pebble repositories. This pivot carries low operational risk and requires no staged rollout. Validation is limited to internal tests plus the one-time migration script.

**CLI Behavior Changes**
- **Unchanged:** `list`, `show`, `search` remain the primary read commands.
- **Change in storage location:** `add`/`update`/`show`/`list` read and write Markdown files under the visible directory (default `docs/pebble/`).
- **New read behavior:** `list` and `search` scan Markdown files directly; any caches are optional and non-canonical.
- **No hidden worktree dependency:** The CLI no longer requires a sync worktree for reads/writes under this model, which dramatically reduces git worktree complexity.

**CLI Command Surface (Authoritative)**
**Global options**
- `--json`: Universal structured output flag. Also accepted at the sub-command level with the same effect. Intended usage: `pebble --json <command> <args>` or `pebble <command> <args> --json`.
- `--dir <PATH>`: Override the default tasks directory (default: `docs/pebble/`). Users can pass `--dir` on any command to point at a non-default task root.

**Repository management**
- `pebble init`: Bootstraps the environment and creates the tasks directory.

**Query commands**
- `pebble list` (alias: `ls`): Parses the directory and builds the DAG. Filters: `--status`, `--tag`, `--parent`, `--is-blocked` (computed from `after`, shows only tasks where dependencies are not `done`).
- `pebble show <id>`: Prints the full details, tree-context, and Markdown body of a specific task.
- `pebble search <query>`: Full-text search across titles and Markdown bodies.

**Mutation commands**
- `pebble add <title>`: Generates the boilerplate `.md` file. Options: `--parent <id>`, `--tag <tag>`, `--after <id>`, `--before <id>`.
- `pebble update <id>`: Safely modifies the frontmatter. Options: `--status <status>`, `--parent <id>`, `--add-tag <tag>`, `--remove-tag <tag>`, `--add-after <id>`, `--remove-after <id>`, `--add-before <id>`, `--remove-before <id>`. `--before` / `--add-before` / `--remove-before` are syntactic sugar; they update the referenced task(s)' `after` lists to include or remove the current task's `id`. No `before` field is stored in frontmatter.
- Users can edit Markdown bodies directly; no dedicated `edit` command is required.

**Validation**
- `pebble check`: A strict linter that evaluates the `.md` database. Checks: ID collisions, broken `after` links, circular dependencies, schema adherence, and state consistency (e.g., flagging a `done` parent that still has non-`done` children).
- `pebble fix`: Applies safe, deterministic repairs (e.g., inserting missing `created_at`, sorting YAML keys, normalizing whitespace).

**Read/Write Policy**
- **Read-only:** `list`, `show`, `search`, and `check` never modify files.
- **Write commands:** `add`, `update`, and `fix` are the only commands that mutate task files.

**Command Deprecations & Removals**
- `pebble sync` is removed under the Markdown-native model because no worktree sync exists; task files are normal repo content.
- `sync-branch` configuration is removed.
- `pebble import` is removed. The one-time JSONL → Markdown migration is handled by a throw-away script (e.g., Python) that invokes the new `pebble` CLI.
- `pebble init` only creates the tasks directory and config; it no longer creates a worktree.

**Frontmatter Contract (Required vs Optional)**
Required:
- `id` (string)
- `status` (enum: `todo` | `in_progress` | `done` | `canceled`)
- `created_at` (RFC3339 string)

Optional:
- `parent` (string)
- `after` (string array; multiple prerequisites allowed)
- `tags` (string array)
- `priority` (integer)

Intentionally omitted:
- `updated_at`, `closed_at`, `owner`, `close_reason`, `created_by`, `type` (or `task_type`)

Computed:
- `before` (derived as the inverse of `after` across the repo; set/list of IDs)

**Frontmatter Format**
- YAML frontmatter delimited by `---`.

**Reference Rust Schema**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")] // Ensures YAML matches exactly "todo", "in_progress", "done", "canceled".
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Canceled,
}

/// Represents the exact structure of the YAML front matter.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskFrontmatter {
    pub id: String,
    // Status strictly validated against the enum.
    pub status: TaskStatus,
    // Optional priority for ordering.
    pub priority: Option<u8>,
    pub parent: Option<String>,
    pub created_at: DateTime<Utc>,
    // Graph edges: empty arrays default nicely.
    #[serde(default)]
    pub after: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The in-memory representation.
/// This is what the CLI stores in its graph topology.
#[derive(Debug, Clone)]
pub struct TaskNode {
    pub path: PathBuf,
    pub frontmatter: TaskFrontmatter,
    // Note: `description` is gone. This body captures the rest of the file.
    pub body: String,
}
```
**Rationale for Omitted Fields:** 
- **Audit Metadata (`owner`, `updated_at`, etc):** Delegated to Git history. Adds parsing/updating friction and write contention without immediate value.
- **Task Types (`type`):** Generic tags (`tags` array) are sufficient for lightweight categorization. Supporting configurable type taxonomies introduces complexity counter to Pebble's minimalist goals.

**Timestamp Rules**
- `created_at` is required in frontmatter and must be RFC3339.
- The CLI sets `created_at` on `add` to the current time in UTC.
- Users may edit `created_at` manually, but `pebble check` fails on invalid format.
- If `created_at` is missing, `pebble fix` sets it to the current time in UTC.

**Title & Body Contract**
- The Markdown body must start with a single H1 title.
- `title` is not stored in frontmatter; the H1 is the sole title source.

**Ordering Semantics (after/before)**
- `after` is the stored field and represents prerequisites.
- `before` is computed as the inverse of `after`.
- A task is **blocked** if any item in its `after` list is not `done`. (This includes `canceled` prerequisites.) The computed `before` list is the inverse and is not used to determine whether the task is blocked.
- Cycles are invalid and rejected by `pebble check`.

Example:
```yaml
# A.md
id: A
after: []

# B.md
id: B
after: [A]

# C.md
id: C
after: [B]
```

Computed:
- `before(A) = [B]`
- `before(B) = [C]`
- `before(C) = []`

**Success Criteria:**
1. Git merges of concurrent edits to different issues resolve cleanly without manual intervention.
2. Agents can create/update issues without schema drift; human edits remain the source of truth.
3. Performance is acceptable for small single-repo teams without premature optimization.

**Migration Plan:**
1. There is exactly one existing database to migrate.
2. Use a one-time throw-away script to transform the current JSONL into Markdown files under `docs/pebble/`.
3. Validate the result manually and discard the script after migration.

**Compatibility & Data Loss**
- The Markdown schema intentionally drops: `owner`, `created_by`, `updated_at`, `closed_at`, and `close_reason`.
- The migration script will **not** preserve per-edge audit metadata (`dependencies[].created_at`, `dependencies[].created_by`, `dependencies[].type`). It will preserve the logical graph as `parent` and `after` (with `before` computed).
- Rationale: there is no concrete use case for these audit fields today; Git history remains the fallback for audit-style questions.

**Risks & Mitigations:**
1. **Filename collisions / human-editable names**
   - *Risk:* Two issues could map to the same filename, or renames could break links.
   - *Mitigation:* Filenames are advisory only; `id` is canonical. On write, the CLI ensures uniqueness by suffixing `-2`, `-3`, etc. On read, `id` is authoritative.
2. **Schema drift from manual edits**
   - *Risk:* Users edit frontmatter by hand and introduce invalid fields or types.
   - *Mitigation:* CLI validates frontmatter strictly and reports precise errors (line/field). `pebble check` is read-only; `pebble fix` performs safe repairs.
3. **Query performance (deferred)**
   - *Risk:* Large repos may need faster list/search than raw file scans provide.
   - *Mitigation:* Defer optimization until user reports demand. Architectural options include lazy caching, background file watchers, incremental indexing, and derived query indices (JSONL or SQLite) that are strictly non-canonical.

**Tooling Contract for File Layout**
- The canonical identifier is the frontmatter `id`; filenames are advisory only.
- The root directory defaults to `docs/pebble/` and is configurable; visibility (hidden directory, gitignored path, etc.) is a user choice.
- The CLI treats every `*.md` file under the root as a task file.
- The CLI never changes `id`. Users may edit it manually, but the `id` **must** be unique across the repo.
- If two files share the same `id`, the CLI fails with a clear error and no writes.
- When creating a new task, the CLI derives a human-readable filename from the title and appends a numeric suffix if needed.
- Renaming or moving a file does not change the `id` and does not break references.
- If a user changes an `id`, they must update all references (`parent`, `after`) for consistency.
- `pebble check` fails on duplicates or dangling references.

**Filename Normalization Rules**
- CLI generates filenames from the **task title provided at creation time** using a deterministic slug:
  - lowercase
  - ASCII only (strip/replace non-ASCII)
  - whitespace → `-`
  - remove punctuation
  - collapse repeated `-`
  - trim leading/trailing `-`
- If the result is empty, use `task` and append a numeric suffix.
- If the filename already exists, append `-2`, `-3`, etc.

**Reference Resolution Rules**
- `parent` and `after` must reference existing task IDs.
- `pebble check` fails if any reference is missing.
- `pebble add`/`update` fail fast when given non-existent IDs (default: strict).
- `list`/`show` should surface missing references in output (human) and include them explicitly in `--json`.

**ID Generation Rules (Current Implementation)**
- IDs are generated on `pebble add` and follow `<issue-prefix>-<suffix>`.
- `issue-prefix` comes from config key `issue-prefix` (default: `issue` if unset).
- The suffix uses the alphabet `a-z0-9` (36 characters).
- The initial suffix length is computed from the current issue count to keep collision probability under 1e-12 (birthday paradox estimate).

**Configuration Contract**
- Config lives at `pebble.toml`.
- Supported keys:
  - `issue-prefix` (string): prefix for new IDs (default: `issue`).
  - `tasks-dir` (string): path to task files (default: `docs/pebble/`).
- Command-line flags:
  - `--dir <PATH>` (on any command) overrides `tasks-dir`.
  - `--issue-prefix <PREFIX>` (on `pebble init`) sets the initial prefix in config.

**Configuration Lifecycle**
- `pebble init` creates `pebble.toml` if it does not exist and writes the initial `issue-prefix` and `tasks-dir` (from `--issue-prefix` / `--dir` if provided, otherwise defaults).
- `--dir` is a runtime override. It does not rewrite config outside of `pebble init`.
- Users may edit `pebble.toml` directly to change `issue-prefix` or `tasks-dir`.
- The CLI accepts any relative or absolute path for `tasks-dir`. Visibility (hidden directory, gitignored path, etc.) is a user choice and not enforced by the tool.

## Appendix A: Alternatives Considered

##### The "State Synchronization" Problem

Before discussing specific file formats, we must address the fundamental friction—or feature—of co-locating a task database with application code inside a version control system like Git.

##### The Problem: "The Bug Database Friction"
If task state is tracked in the main branch (e.g., inside `docs/pebble/`), it feels like it creates a workflow bottleneck:
- **Tangent Discoveries:** A developer working on `feature-A` discovers a bug related to `feature-B`. If they create the task locally and commit it, it's not visible project-wide until `feature-A` is merged.
- **Merge Conflicts on State:** Changing the status of an ongoing epic on multiple feature branches simultaneously can lead to merge conflicts simply trying to track *what* is being done.

*This explains why most industry-standard bug trackers (Jira, Linear, GitHub Issues) exist completely "out-of-band" (hosted externally) rather than in the repository itself.*

##### The Counter-Argument: "In-Band Storage is a Feature, Not a Bug"
Alternatively, tracking task states directly with the code is a massive benefit that out-of-band trackers lack:
- **Temporal Consistency:** If you `git checkout` a release from 6 months ago, you see exactly what tasks were pending, blocked, or completed *at that exact moment in time*. The state of the project planner perfectly matches the state of the codebase.
- **The Solution to Tangent Discoveries:** If a user needs to file an issue separate from their current development track, the solution isn't to build a complex global sync mechanism—they simply branch off `main`, commit the new task, and merge it quickly. It forces good hygiene.

##### Paradigm 1: The Out-of-Band Service (The External API)
*Move state out of the repository entirely, relying on a lightweight backend service.*
- **Pros:** Eliminates Git friction. Task status is instantly globally visible.
- **Cons:** Violates the "local first, offline capable, single-repo" ethos. Introduces infrastructure overhead. Destroys temporal consistency with the codebase.

##### Paradigm 2: The In-Band Hidden Worktree (The Current Pebble Approach)
*Store the data in the repository, but utilize a "hidden" Git branch (e.g., `pebble-data`) mounted via a Git worktree inside a `.git/` subdirectory.*
- **Pros:** Maintains local-first offline capability. Tasks are instantly synced across feature branches because the worktree operates independently.
- **Cons:** Extremely high complexity. `git worktree` commands are brittle to setup, difficult for agents to intuitively reason about, and create edge cases around `git push/pull`.

##### Paradigm 3: The SQLite / Local Database Approach
*Store state in an un-tracked local `.pebble.sqlite` database file. Sync via a secondary mechanism.*
- **Pros:** Immediate reads/writes. No git branch interference. Easy to query using SQL.
- **Cons:** Merging binary SQLite dumps across branches is incredibly difficult. Natively unreadable by humans without dedicated tooling.

##### Storage Format Considerations

Assuming we embrace the "In-Band Storage is a Feature" argument (abandoning the hidden worktree and just committing tasks to the main branch), what format should the data take?

##### Avenue A: The "Everything is a File" Markdown Approach
*Store each task as a discrete Markdown file inside a visible directory in the source tree (e.g., `docs/pebble/` or `docs/tasks/`), using YAML frontmatter for metadata. These files are committed to standard Git.*

**Example `docs/pebble/deploy-staging-environment.md`:**
```markdown
---
id: proj-0kq
status: todo
parent: proj-epic1
created_at: 2026-01-15T10:30:00Z
---
# Deploy staging environment

Run the canary deploy pipeline against the `staging` cluster.
```

**Filename rules:** Filenames are human-readable and do **not** need to embed the `id`. The `id` lives in frontmatter; the filename is purely for human navigation.

**Pros:**
- **Ultimate Human Readability:** GitHub, GitLab, and local IDEs render these files perfectly natively.
- **The Ultimate Code Review:** Because they are just text files in the main branch, changes to tasks show up in GitHub Pull Requests natively. You can comment on a task definition change just like a code change.
- **Agent Friendly:** LLMs have profound native understanding of Markdown.
- **Git Diffs:** Conflict resolution is trivial because files are separated.

**Cons:**
- **Graph Traversal:** Requires reading potentially hundreds of small files to build the dependency graph.

##### Avenue B: The Append-Only Log (Refined JSONL)
*Keep a JSONL event stream or state dump (similar to current `golden.jsonl`), heavily optimizing the CLI/MCP layer to hydrate the state.*

**Pros:**
- **Machine Native:** JSON is the lingua franca of LLMs.
- **Git Friendly Appends:** Adding a line rarely conflicts with another added line.
**Cons:**
- **Human Antagonistic:** Humans cannot read or edit JSONL manually. This violates the "degrade gracefully" principle if the CLI/UI is unavailable. Pull Request diffs for a JSONL state change are extremely difficult for human reviewers to parse.

## Appendix B: Iterative Refinement

DO NOT REMOVE THIS SECTION

Use the following as an agent prompt for iterative refinement:

You are a principle staff engineer that is in favor of the rfc-reimagining-pebble.md ideas, and are helping me whip the document into shape such that you'd be persuaded to approve it. Choose a concrete improvement to make to the document, propose it to me for implementation.

Consider: removing unecessary complexity; editorial issues like section order, presentation language;  content improvements; missing gaps in the proposal; failures to consider every detail of the current pebble schema or command set; anything else you can think of.

Pick the most important improvement you can think of, and propose it to me for implementation.

---
*Open for feedback: Does fully committing to Markdown files in the main branch (Option A) create too much directory clutter, or is the benefit of native GitHub PR capabilities worth the noise?*
