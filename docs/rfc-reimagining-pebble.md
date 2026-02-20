# RFC: Re-imagining Pebble from Scratch

## 1. Introduction & Motivation

The goal of this RFC is to step back and re-imagine Pebble from the ground up. Inspired by the legacy `bd` tool but now forging its own divergent path, the core mission remains unaltered: **provide a project task tracking system that is equally useful and delightful for both human developers and autonomous AI coding agents**.

While the current implementation relies on a Rust CLI with a JSONL storage backbone, this document explores the solution space without those constraints. We aim for a "minimum useful feature set" tailored not for enormous enterprise projects or for coordinating concurrently running autonomous AI agents, but for the simpler, single-repo projects common in open-source development and indie hacking.

## 2. Key Decisions (TL;DR)
- Config lives in `.pebble/config.toml` (relative to the repository root).
- Store tasks as Markdown files in a visible repo directory (default `docs/pebble/`).
- YAML frontmatter defines metadata; the Markdown body is the description.
- `id` is canonical and user-editable; the CLI never changes it.
- Relationships: store `after` (prerequisites), compute `before` as inverse.
- Status model: `todo`, `in_progress`, `paused`, `done`, `canceled`.
- Readiness is computed: a task is `ready` when prerequisites are satisfied and the status is actionable (not `paused`, `done`, or `canceled`).
- Omit audit fields (`owner`, `created_by`, `updated_at`, `closed_at`, `close_reason`); rely on git history. The `resolved_at` timestamp is explicitly maintained for archival purposes.
- CLI reads/writes Markdown directly; no hidden worktrees.
- One-time migration via a throw-away script from the existing JSONL.

## 3. Minimum Useful Feature Set

Based on the `golden.jsonl` data and typical single-repo development flows, the essential feature set is surprisingly small:

1. **Task Tracking:** Ability to define a task with an ID, title, description, and status.
   - States: `todo`, `in_progress`, `paused`, `done`, `canceled` (core states align with GitHub Projects; `paused` is Pebble-specific).
   - `canceled` means "not done and will never be done."
2. **Hierarchy & Composition:** Epics and subtasks. A task can be heavily composed of smaller tasks (`parent/child`).
3. **Ordering & Dependencies:** Execution ordering. Knowing what to do *next* is critical for agents. We need `before` / `after` relationships; **readiness** is computed from whether all `after` prerequisites are `done` and the task is not `paused`.
4. **Basic Metadata:** Time-based metadata is restricted to creation (`created_at`), last modification (`modified_at`), and completion (`resolved_at`) timestamps. The `modified_at` field helps identify stalled or forgotten work (e.g., to review neglected tasks). The `resolved_at` field enables querying completed work and powers the deterministic `archive` feature without relying on volatile filesystem `mtime` or requiring expensive Git history lookups. Audit trail (owner, updated_at, closed_at, close_reason) is intentionally omitted and delegated to Git history.

**Notes in Markdown:** Users and agents are free to include checklists in the Markdown body. We should consider future support for task ID auto-discovery in the body (e.g., `proj-123`) to enable "related to" queries or semantic linking without expanding frontmatter.


## 4. Technical Specification

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
- `pebble list` (alias: `ls`): Parses the directory and builds the DAG. Filters: `--status`, `--tag`, `--parent`, `--is-ready` (computed; shows only tasks whose prerequisites are `done` and whose status is actionable).
- `pebble show <id>`: Prints the full details, tree-context, and Markdown body of a specific task.
- `pebble search <query>`: Full-text search across titles and Markdown bodies.
- `pebble config get <key>`: Reads a configuration value. Supported keys: `issue-prefix`, `tasks-dir`. Also serves as a way for users and agents to discover the resolved config file location and effective values.

**Mutation commands**
- `pebble add <title>`: Generates the boilerplate `.md` file. By default, `status` is initialized to `todo`. Options: `--status <status>`, `--body <text>`, `--parent <id>`, `--tag <tag>`, `--after <id>`, `--before <id>`. The `--body` text is inserted after the `# <title>` heading, separated by a blank line.
- `pebble update <id>`: Safely modifies the frontmatter. Options: `--status <status>`, `--parent <id>`, `--add-tag <tag>`, `--remove-tag <tag>`, `--add-after <id>`, `--remove-after <id>`, `--add-before <id>`, `--remove-before <id>`. `--before` / `--add-before` / `--remove-before` are syntactic sugar; they update the referenced task(s)' `after` lists to include or remove the current task's `id`. No `before` field is stored in frontmatter. When modifying the frontmatter, the CLI automatically sets the `modified_at` timestamp. When setting `--status done` or `--status canceled`, the CLI automatically sets `resolved_at`.
- `pebble archive`: Automatically moves tasks with a status of `done` or `canceled` where `resolved_at` is older than a threshold (e.g., `> 30 days`) into an `archive/` subdirectory to reduce IDE clutter.
- Users can edit Markdown bodies directly; no dedicated `edit` command is required.

**Validation**
- `pebble check`: A strict linter that evaluates the `.md` database. Checks: ID collisions, broken `after` links, circular dependencies, schema adherence, and state consistency (e.g., flagging a `done` parent that still has non-`done` children).
- `pebble fix`: Applies safe, deterministic repairs (e.g., inserting missing `created_at`, sorting YAML keys, normalizing whitespace).

**Read/Write Policy**
- **Read-only:** `list`, `show`, `search`, `config get`, and `check` never modify files.
- **Write commands:** `add`, `update`, `fix`, and `archive` are the only commands that mutate task files or their locations.

**Command Deprecations & Removals**
- `pebble sync` is removed under the Markdown-native model because no worktree sync exists; task files are normal repo content.
- `sync-branch` configuration is removed.
- `pebble import` is removed. The one-time JSONL → Markdown migration is handled by a throw-away script (e.g., Python) that invokes the new `pebble` CLI.
- `pebble init` only creates the tasks directory and config; it no longer creates a worktree.

**Frontmatter Contract (Required vs Optional)**
Required:
- `id` (string)
- `status` (enum: `todo` | `in_progress` | `paused` | `done` | `canceled`)
- `created_at` (RFC3339 string)

Optional:
- `parent` (string)
- `after` (string array; multiple prerequisites allowed)
- `tags` (string array)
- `priority` (integer)
- `modified_at` (RFC3339 string; automatically updated on modification)
- `resolved_at` (RFC3339 string; automatically managed based on status)

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
#[serde(rename_all = "snake_case")] // Ensures YAML matches exactly "todo", "in_progress", "paused", "done", "canceled".
pub enum TaskStatus {
    Todo,
    InProgress,
    Paused,
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
    pub modified_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
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
    /// Extracted from the first H1 heading in `body`. Computed, never stored.
    pub title: String,
    /// Raw Markdown content after the frontmatter delimiter, including the H1.
    pub body: String,
}
```
**Rationale for Omitted Fields:** 
- **Audit Metadata (`owner`, `created_by`, `close_reason`):** Delegated to Git history. Adds parsing/updating friction and write contention without immediate value. (Note: The legacy `updated_at` and `closed_at` fields have been explicitly replaced by the more semantically distinct `modified_at` and `resolved_at` fields).
- **Task Types (`type`):** Generic tags (`tags` array) are sufficient for lightweight categorization. Supporting configurable type taxonomies introduces complexity counter to Pebble's minimalist goals.

**Timestamp Rules**
- `created_at` is required in frontmatter and must be RFC3339.
- The CLI sets `created_at` on `add` to the current time in UTC.
- `modified_at` is optional and must be RFC3339 if present.
- The CLI automatically sets/updates `modified_at` to the current time in UTC during any `pebble update` operation.
- `resolved_at` is optional and must be RFC3339 if present.
- The CLI sets `resolved_at` to the current UTC time when updating a task's status to `done` or `canceled` (if not already set).
- The CLI removes `resolved_at` when updating a task's status from `done` or `canceled` to any other status.
- Users may edit timestamps manually, but `pebble check` fails on invalid format.
- If `created_at` is missing, `pebble fix` sets it to the current time in UTC.

**Title & Body Contract**
- The Markdown body must start with a single H1 title.
- `title` is not stored in frontmatter; the H1 is the sole title source.

**Ordering Semantics (after/before)**
- `after` is the stored field and represents prerequisites.
- `before` is computed as the inverse of `after`.
- Cycles are invalid and rejected by `pebble check`.

**Ready and Paused: Two Independent Concepts**

A task has two independent concepts:

1. **Ready (computed):** A task is `ready` when all `after` prerequisites are `done` and its status is actionable (`todo` or `in_progress`). Tasks with status `paused`, `done`, or `canceled` are never `ready`, even if dependencies are satisfied.
2. **Paused (explicit):** The user sets `status: paused` to represent an external hold not captured in the graph (e.g., waiting on a vendor, approval, or shipment). This is manual and only clears when the user changes the status.

The `--is-ready` filter on `list` matches tasks that are `ready`. In `--json` output, `TaskObject` includes a computed boolean `is_ready` so agents can filter without re-deriving readiness.

**Rationale for `paused` as a stored status:** Without it, the only way to represent an external hold is to create a dummy prerequisite task (e.g., "Wait for Apple review") — a non-actionable task that pollutes the tracker purely to manipulate graph state. `status: paused` avoids this antipattern. The staleness risk (user forgets to un-pause after the external condition resolves) is a user discipline problem common to every task tracker; `pebble list --status paused` serves as the periodic review queue.

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

**JSON Output Contract**

All `--json` output is a single JSON object printed to stdout per invocation.

- **Query commands** (`list`, `search`): `{"tasks": [<TaskObject>, ...]}`.
- **`show --json`**: A single unwrapped `TaskObject`.
- **Mutation commands** (`add`, `update`): Echo back the full `TaskObject` after the write.
- **`check --json`**: `{"ok": bool, "errors": [{"file": "...", "line": N, "message": "..."}]}`.
- **`archive --json`**: `{"archived": [{"id": "...", "moved_to": "..."}]}`.
- **`config get --json`**: `{"key": "<key>", "value": "<value>"}`.

A `TaskObject` includes:
- All stored frontmatter fields (`id`, `status`, `priority`, `parent`, `created_at`, `modified_at`, `resolved_at`, `after`, `tags`).
- Computed fields: `title` (extracted from the H1 heading), `before` (inverse of `after` across the repo), `is_ready` (boolean; true if all prerequisites are `done` and status is `todo` or `in_progress`).
- `body`: the raw Markdown content after the frontmatter delimiter, verbatim (including the H1 heading).
- `path`: the file path relative to `tasks-dir`.

Rationale: `title` and `before` are computed convenience fields, analogous to each other—derived from the file on read, never stored. `body` is a faithful reproduction of the file content; the CLI does not strip or transform it. This means `title` appears twice in JSON output (once as a top-level key, once inside `body` as the H1). This minor redundancy is an acceptable tradeoff: agents get a structured `title` for filtering and display without parsing Markdown, while `body` remains a lossless round-trip representation of the file.

**Success Criteria:**
1. Git merges of concurrent edits to different issues resolve cleanly without manual intervention.
2. Agents can create/update issues without schema drift; human edits remain the source of truth.
3. Performance is acceptable for small single-repo teams without premature optimization.

**Archival & Organization Strategy:**
Because the `id` within the YAML frontmatter is the canonical identifier, the physical file path of a task Markdown file is strictly advisory. The CLI scans the `tasks-dir` **recursively**, meaning files can be moved without breaking graph links.

- **Automated Lifecycle Archiving:** To prevent long-term repository bloat and IDE search pollution, Pebble provides a `pebble archive` command. This command scans the repository for `done` or `canceled` tasks whose `resolved_at` timestamp is older than a threshold (e.g., 30 days) and automatically moves them into an `archive/` subdirectory (e.g., `docs/pebble/archive/2026/`). Since the CLI recursively scans the base directory, these archived tasks remain part of the project history and graph but are visually moved out of active working directories. By relying on the `resolved_at` frontmatter field instead of a Git or filesystem `mtime`, this command remains deterministic, fast, and completely immune to repository resets or clones.

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
- The CLI **recursively** treats every `*.md` file under the root as a task file.
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
- Config lives at `.pebble/config.toml` (relative to the repository root).
- Supported keys:
  - `issue-prefix` (string): prefix for new IDs (default: `issue`).
  - `tasks-dir` (string): path to task files (default: `docs/pebble/`).
- Command-line flags:
  - `--dir <PATH>` (on any command) overrides `tasks-dir`.
  - `--issue-prefix <PREFIX>` (on `pebble init`) sets the initial prefix in config.

**Path Resolution & Repo Root**
- The CLI locates the repository root by walking up from the current working directory to the nearest parent containing `.git`.
- If no `.git` directory is found, the CLI fails with a clear error.
- `.pebble/config.toml` is always resolved relative to the repository root.
- `tasks-dir` is resolved relative to the repository root when it is a relative path.
- `--dir` overrides `tasks-dir` and is resolved relative to the repository root when it is a relative path.
- Precedence: `--dir` > `tasks-dir` in config > default `docs/pebble/`.

**Configuration Lifecycle**
- `pebble init` creates `.pebble/config.toml` if it does not exist and writes the initial `issue-prefix` and `tasks-dir` (from `--issue-prefix` / `--dir` if provided, otherwise defaults).
- `--dir` is a runtime override. It does not rewrite config outside of `pebble init`.
- Users may edit `.pebble/config.toml` directly to change `issue-prefix` or `tasks-dir`.
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
- **Temporal Consistency:** If you `git checkout` a release from 6 months ago, you see exactly what tasks were pending, paused, or completed *at that exact moment in time*. The state of the project planner perfectly matches the state of the codebase.
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

##### Semantic Subdirectories (Visual Organization)
*Manually organizing active tasks into nested folders (e.g., `docs/pebble/frontend/` or `docs/pebble/epics/epic-1/`) just to group them visually.*

- **Pros:** Makes reading the raw file tree theoretically easier for humans.
- **Cons:** Because directory paths are not indexed or surfaced by `pebble search` or `pebble list`, this organization becomes a "shadow taxonomy." It is completely invisible to the CLI's queries, meaning users cannot rely on it for actual task retrieval. Pebble enforces a flat semantic structure using `tags` and graph edges (`parent`/`after`), reserving the recursive directory scan feature purely for automated lifecycle `archive` sorting.

## Appendix B: Iterative Refinement

DO NOT REMOVE THIS SECTION

Use the following as an agent prompt for iterative refinement:

You are a principle staff engineer that is in favor of the rfc-reimagining-pebble.md ideas, and are helping me whip the document into shape such that you'd be persuaded to approve it. Choose a concrete improvement to make to the document, propose it to me for implementation.

Consider: removing unecessary complexity; editorial issues like section order, presentation language;  content improvements; missing gaps in the proposal; failures to consider every detail of the current pebble schema or command set; anything else you can think of.

Pick the most important improvement you can think of, and propose it to me for implementation.
