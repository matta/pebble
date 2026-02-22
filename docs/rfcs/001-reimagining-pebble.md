[STATUS: FROZEN]

# RFC: Re-imagining Pebble from Scratch

## 1. Introduction & Motivation

**High-Level Goal & Principles: "Permissive Writes, Strict Evaluation"**
Pebble fundamentally approaches task management as a pure, loosely coupled directed graph layered seamlessly over the filesystem. Influenced by Git's content-addressable DAG, the system is deeply unopinionated about how you author data. The CLI acts as a runtime engine that derives insights (like `pebble next`), rather than a strict compiler enforcing business logic. By removing artificial barriers (like preemptive cycle prevention, cascading external states, and bi-directional link synchronization), Pebble gains robust simplicity and becomes practically unbreakable.


The goal of this RFC is to re-imagine Pebble from the ground up. Inspired by the `beads` tool (https://github.com/steveyegge/beads) but now forging its own divergent path, with different goals. The core mission remains unaltered: **provide a project task tracking system that is equally useful and delightful for both human developers and AI coding agents**.

While the current implementation relies on a Rust CLI with a JSONL storage backbone, this document explores the solution space without those constraints. We aim for a "minimum useful feature set" tailored not for enormous enterprise projects or for coordinating concurrently running autonomous AI agents, but for the simpler, single-repo projects common in open-source development and indie hacking.

## 2. Key Decisions (TL;DR)
- Config lives in `.pebble/config.toml` (relative to the project root, i.e., the nearest parent containing `.pebble`).
- Project root is defined by `.pebble/` and is not required to be the Git repository root.
- Store tasks as Markdown files in `tasks-dir` (default `docs/pebble/`).
- YAML frontmatter defines metadata; the Markdown body is free-form description.
- `id` is canonical and user-editable; the CLI never changes it.
- Relationships: There is only one structural edge: `deps` (dependencies). Hierarchy (`parent/child`) and ordering (`after`) are collapsed. Epics simply depend on their subtasks.
- Status model: `todo`, `in_progress`, `done`, `canceled`. Note: `paused` has been removed. Waiting is a task (e.g., "Wait for vendor").
- Readiness is beautifully simple: A task is `ready` if and only if all of its `deps` exist and have a terminal status (`done` or `canceled`).
- Cycles naturally deadlock (they never surface as `ready`), and dangling pointers naturally block. The CLI accepts all writes; strict cyclic/pointer validation is moved out of the read/write path and into a diagnostic `pebble doctor` command.
- Omit audit fields (`owner`, `created_by`, `updated_at`, `closed_at`, `close_reason`); rely on git history. The `resolved_at` timestamp is explicitly maintained for archival purposes.
- CLI reads/writes Markdown directly; no hidden worktrees.
- `pebble next` is a convenience command that returns the highest-priority ready task.
- Default list order: topological (respecting `deps`), then blocking count, then `priority`, then `created_at`.
- Agents MAY read and edit task bodies directly (a core benefit); frontmatter mutations SHOULD use the CLI.
- `.pebble/AGENTS.md` provides agent bootstrapping instructions; `pebble init` creates it.
- One-time migration via a throw-away script from the existing JSONL.

## 3. Minimum Useful Feature Set

Based on the `golden.jsonl` data and typical single-repo development flows, the essential feature set is surprisingly small:

1. **Task Tracking:** Ability to define a task with an ID, title, description, and status.
   - States: `todo`, `in_progress`, `done`, `canceled`.
   - `canceled` means "not done and will never be done."
2. **The One True Edge (`deps`):** Both hierarchical composition (Epics) and temporal ordering (execution steps) are modeled as a single directed edge: `deps`. If an Epic is composed of 3 subtasks, it `deps` on them. If task A must finish before task B, B `deps` on A.
3. **Backlinks (`related`):** Directional linking simply leverages standard Markdown links inside the body, powering a dynamic "backlinks" experience. No symmetric enforcement.
4. **Basic Metadata:** Time-based metadata is restricted to creation (`created_at`), last modification (`modified_at`), and completion (`resolved_at`) timestamps. The `modified_at` field helps identify stalled or forgotten work (e.g., to review neglected tasks). The `resolved_at` field enables querying completed work and powers the deterministic `archive` feature without relying on volatile filesystem `mtime` or requiring expensive Git history lookups. Audit trail (owner, `updated_at`, `closed_at`, `close_reason`) is intentionally omitted and delegated to Git history. **Decision:** the schema uses `modified_at` (not `updated_at`); `updated_at` is legacy and appears only in migration mapping.

**Notes in Markdown:** Users and agents are free to include checklists in the Markdown body. We should consider future support for task ID auto-discovery in the body (e.g., `proj-123`) to enable "related to" queries or semantic linking without expanding frontmatter.


## 4. Technical Specification

### 4.1 Scope & Goals

**Decision:** Adopt **per-issue Markdown files with YAML frontmatter** stored under a visible, human-friendly directory such as `docs/pebble/`. The Markdown files are the source of truth. Any caches or indexes are strictly derived and optional.

**Rationale:** This maximizes human transparency, makes review diffs first-class in Git, and keeps agent tooling aligned with what humans see. It also eliminates single-file merge conflicts while keeping the architecture simple.

**Design Philosophy: Permissive Writes, Strict Evaluation (The CLI is Not a Compiler)**
Pebble embraces the liquid, unstructured nature of Markdown. It fundamentally views task tracking as a pure directed graph. It does not treat the user as an adversary by baking strict "Jira-esque" business logic (cycle prevention, cascading effective statuses) into the storage layer. Modeled after Git's mathematically sound core, the system writes the graph as you dictate it. If you author a cycle, neither task will ever emerge in the "ready" queue—perfectly matching real-world deadlocks. If you reference a missing dependency, the task blocks until the dependency exists or the reference is removed. We provide `pebble doctor` to diagnose these structural anomalies, but the system never panics, refuses writes, or requires complex in-memory cycle-breakers to function.

**Non-Goals:**
- Not targeting enterprise-scale analytics or cross-repo issue federation.
- **Model Context Protocol (MCP) Server:** Building an MCP server is explicitly a post-1.0/post-MVP decision. AI agents perform perfectly well interacting via a local CLI. 

**Scope & Risk Profile:** There are currently zero active Pebble repositories. This pivot carries low operational risk and requires no staged rollout. Validation is limited to internal tests plus the one-time migration script.

**Success Criteria:**
1. Git merges of concurrent edits to different issues resolve cleanly without manual intervention.
2. Agents can create/update issues without schema drift; human edits remain the source of truth.
3. Performance is acceptable for small single-repo teams without premature optimization.

### 4.2 Configuration

**Configuration Contract**
- Config lives at `.pebble/config.toml` (relative to the project root, i.e., the nearest parent containing `.pebble`).
- Supported keys:
  - `issue-prefix` (string): prefix for new IDs (default: `issue`).
  - `tasks-dir` (string): path to task files (default: `docs/pebble/`).
- Command-line flags:
  - `--dir <PATH>` (on any command) overrides `tasks-dir`.
  - `--issue-prefix <PREFIX>` (on `pebble init`) sets the initial prefix in config.

**Path Resolution & Project Root**
- The project root is defined as the directory containing the `.pebble` configuration folder.
- The CLI locates the project root by walking up from the current working directory to the nearest parent containing a `.pebble` directory.
- Example: If you run `pebble` from `repo/subproject/` and place `.pebble/` there, the project root is `repo/subproject/` even if the Git repo root is `repo/`.
- Note: The project root can be a subdirectory within a Git repository. Pebble does not assume `.git` is adjacent to `.pebble`, and any Git operations must resolve the repository root separately.
- If no `.pebble` directory is found, the CLI fails with a clear error (except for `pebble init`, which initializes the `.pebble` directory in the user's current working directory).
- `.pebble/config.toml` is always resolved relative to the project root.
- `tasks-dir` read from `.pebble/config.toml` MUST be a relative path and is always resolved relative to the project root. If an absolute path is found in the config, the CLI fails with a clear error to prevent breaking the configuration for other teammates.
- `--dir` overrides `tasks-dir`. It can be an absolute or relative path. When it is a relative path, it is strictly resolved relative to the user's current working directory (cwd).
- Precedence: `--dir` > `tasks-dir` in config > default `docs/pebble/`.

**Configuration Lifecycle**
- `pebble init` initializes the project by creating the `.pebble` directory and `.pebble/config.toml` in the current working directory if they do not exist. If run inside a larger Git repo, it creates `.pebble/` in the current directory (which may or may not be the repository root, depending on where you ran the command). It writes the initial `issue-prefix` and `tasks-dir` (from `--issue-prefix` / `--dir` if provided, otherwise defaults). If `--dir` is provided during `init` to seed the config, it MUST be a relative path; otherwise, the command fails.
- `pebble init` also creates `.pebble/AGENTS.md` containing agent bootstrapping instructions (see §4.8). On completion, it prints a message advising the user to include or reference `.pebble/AGENTS.md` from their project's root `AGENTS.md` (or equivalent agent configuration file such as `.cursorrules`, `.github/copilot-instructions.md`, etc.).
- `--dir` on runtime commands is a temporary override. It does not rewrite config outside of `pebble init`.
- Users may edit `.pebble/config.toml` directly to change `issue-prefix` or `tasks-dir`.
- The CLI enforces that `tasks-dir` in `.pebble/config.toml` is a strictly relative path. Visibility (hidden directory, gitignored path, etc.) is a user choice and not enforced by the tool.

### 4.3 Storage & File Layout

**Tooling Contract for File Layout**
- The canonical identifier is the frontmatter `id`; filenames are advisory only.
- The root directory defaults to `docs/pebble/` and is configurable; visibility (hidden directory, gitignored path, etc.) is a user choice.
- The CLI **recursively** treats every `*.md` file under the root as a task file.
- The CLI never changes `id`. Users may edit it manually, but the `id` **must** be unique across the repo.
- If multiple files share the same `id`, read commands treat this as a Schema Error (logging a warning and skipping all files with that ID), while write commands targeting the duplicated ID fail with a clear error.
- When creating a new task, the CLI derives a human-readable filename from the title and appends a numeric suffix if needed.
- Renaming or moving a file does not change the `id` and does not break references.
- If a user changes an `id` or deletes a file, references to the old `id` become dangling. They must be cleaned up manually; `pebble fix` reports them but does not rewrite dependency edges.
- `pebble check` fails on duplicate and dangling IDs (ideal for CI/pre-commit enforcement).

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
- Updating a task title never renames the file. Filenames are stable unless moved explicitly by `pebble archive`.

**Reference Resolution Rules**
- `deps` must reference existing task IDs to allow the task to become ready.
- Missing dependencies act as permanent blocks. The task will not evaluate as `ready` until the `deps` exist or are manually removed.
- Cycles are permitted at the storage level. A cycle simply means all nodes in the cycle will forever evaluate as "blocked by a dependency" and never surface as `ready`.
- `pebble add`/`update` blindly accept valid YAML. They do not fail on unresolved edges or potential cycle introduction.
- `pebble doctor` is the diagnostic tool that walks the graph and warns the user about dangling dependencies, deadlocked cycles, and other health issues.
- Backlinks (related tasks) are natively handled via Markdown body links (e.g. `[See also: proj-123](proj-123.md)`). The CLI dynamically discovers backlinks at read time (using equivalent `grep` logic) to display "Linked by" sections, requiring zero frontmatter synchronization.

**ID Generation Rules**
- IDs are generated on `pebble add` and follow `<issue-prefix>-<suffix>`.
- `issue-prefix` comes from config key `issue-prefix` (default: `issue` if unset).
- The suffix uses the alphabet `a-z0-9` (36 characters).
- The initial suffix length is computed from the current issue count to keep collision probability under 1e-12 (birthday paradox estimate).

**Archival & Organization Strategy**

Because the `id` within the YAML frontmatter is the canonical identifier, the physical file path of a task Markdown file is strictly advisory. The CLI scans the `tasks-dir` **recursively**, meaning files can be moved without breaking graph links.

- **Automated Lifecycle Archiving:** To prevent long-term repository bloat and IDE search pollution, Pebble provides a `pebble archive` command. This command scans the repository for `done` or `canceled` tasks whose `resolved_at` timestamp is older than a threshold (e.g., 30 days) and automatically moves them into an `archive/` subdirectory (e.g., `docs/pebble/archive/2026/`). Since the CLI recursively scans the base directory, these archived tasks remain part of the project history and graph but are visually moved out of active working directories. By relying on the `resolved_at` frontmatter field instead of a Git or filesystem `mtime`, this command remains deterministic, fast, and completely immune to repository resets or clones. If a filename collision occurs in the target archive directory, the CLI appends a numeric suffix (`-2`, `-3`, etc.) to avoid overwriting.

**Direct File Access Contract**

Task files are plain Markdown with YAML frontmatter in `tasks-dir`. Direct file access by agents, scripts, and humans is a **core design benefit** of the Markdown-native storage model — not a workaround.

- **Reading:** Agents and scripts MAY read task files directly. The file format (YAML frontmatter + Markdown body) is a stable contract. This is often faster and cheaper than shelling out to `pebble show`, especially when an agent already has file-reading tools available. `pebble show --path-only <id>` resolves an ID to its file path for this purpose.
- **Body editing:** Agents, scripts, and humans are encouraged to edit the Markdown body directly. The body is free-form content — checklists, notes, acceptance criteria, design sketches — and direct editing is the natural way to work with it. Checking off a checklist item, appending implementation notes, or restructuring sections are all expected direct-edit operations. The CLI also provides `--body` (replace) and `--append-body` (append) flags on `pebble update` for simpler mutations that benefit from automatic `modified_at` management.
- **Frontmatter mutations:** Agents and scripts SHOULD use `pebble add` and `pebble update` for frontmatter changes. The CLI provides ID generation, automatic timestamp management (`modified_at`, `resolved_at`), and strict schema validation. Direct frontmatter writes are permitted but bypass all of these safeguards — the author assumes full responsibility for schema correctness.
- **Recovery:** `pebble doctor` detects problems introduced by direct file edits (schema violations, broken references, cycles). `pebble check` serves as the strict enforcement mechanism for these rules in CI. `pebble fix` applies safe, non-destructive repairs (timestamps, formatting) and reports graph issues without rewriting dependency edges. This is the safety net for direct file access.
- **Implication for CLI output:** Because agents may use file paths to read tasks directly, `pebble list` and `pebble search` include the `path` field (relative to `tasks-dir`) in both human and `--json` output modes.

### 4.4 Task Schema

**Frontmatter Contract (Required vs Optional)**

Required:
- `id` (string)
- `title` (string)
- `status` (enum: `todo` | `in_progress` | `done` | `canceled`)
- `created_at` (RFC3339 string)

Optional:
- `deps` (string array; the single dependency edge)
- `tags` (string array)
- `priority` (integer)
- `modified_at` (RFC3339 string; automatically updated on modification; replaces legacy `updated_at`)
- `resolved_at` (RFC3339 string; automatically managed based on status)

Intentionally omitted:
- `updated_at`, `closed_at`, `owner`, `close_reason`, `created_by`, `type` (or `task_type`)

Computed:
- `is_ready` (boolean; true only when all `deps` exist and are terminal: `done` or `canceled`)
- `blocked_by` (list of `deps` that are missing or non-terminal, where non-terminal means `todo` or `in_progress`)
- `blocking` (list of task IDs whose `deps` include this task and are therefore blocked until it is terminal)

> **Rationale for Omitted Fields:**
> - **Audit Metadata (`owner`, `created_by`, `close_reason`):** Delegated to Git history. Adds parsing/updating friction and write contention without immediate value. (Note: The legacy `updated_at` and `closed_at` fields have been explicitly replaced by the more semantically distinct `modified_at` and `resolved_at` fields).
> - **Task Types (`type`):** Generic tags (`tags` array) are sufficient for lightweight categorization. Supporting configurable type taxonomies introduces complexity counter to Pebble's minimalist goals.

**Frontmatter Format**
- YAML frontmatter delimited by `---`.

**Reference Rust Schema**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")] // Ensures YAML matches exactly "todo", "in_progress", "done", "canceled".
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
    pub title: String,
    // Status strictly validated against the enum.
    pub status: TaskStatus,
    // Optional priority for ordering.
    pub priority: Option<u8>,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deps: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The in-memory representation.
/// This is what the CLI stores in its graph topology.
#[derive(Debug, Clone)]
pub struct TaskNode {
    pub path: PathBuf,
    pub frontmatter: TaskFrontmatter,
    /// Raw Markdown content after the frontmatter delimiter. Free-form; no structural requirements.
    pub body: String,
}
```

**Timestamp Rules**
- `created_at` is required in frontmatter and must be RFC3339.
- The CLI sets `created_at` on `add` to the current time in UTC.
- `modified_at` is optional and must be RFC3339 if present.
- The CLI automatically sets/updates `modified_at` to the current time in UTC during any `pebble update` operation.
- `resolved_at` is optional and must be RFC3339 if present.
- The CLI sets `resolved_at` to the current UTC time when updating a task's status to `done` or `canceled` (if not already set).
- The CLI removes `resolved_at` when updating a task's status from `done` or `canceled` to any other status.
- Users may edit timestamps manually, but `pebble doctor` fails on invalid format.
- If `created_at` is missing, `pebble fix` sets it to the current time in UTC.

**Body Contract**
- The Markdown body is free-form. There are no structural requirements (no mandatory H1, no required sections).
- The body may be empty.

### 4.5 Graph Semantics

**The One True Edge (`deps`)**
- All relationships are collapsed into a single directed edge: `deps` (dependencies).
- If Task E is an Epic requiring tasks X and Y, Task E simply lists `deps: [X, Y]`.
- If Task B must wait for Task A, Task B lists `deps: [A]`.
- This removes the need for cascading effective statuses or complex hierarchy traversal. An epic isn't "done" until its deps are done, exactly like a sequential step.

**Status & Readiness**
- Task statuses are absolute, local enum values (`todo`, `in_progress`, `done`, `canceled`). There is no `paused` state. "Waiting" is modeled by creating a task (e.g. "Wait on Legal") and depending on it.
- **Ready (computed boolean):** A task is `ready` if:
  1. Its status is `todo` or `in_progress`
  2. EVERY task listed in its `deps` array exists AND has a status of `done` or `canceled`.
- Missing/Dangling IDs in `deps` cause the task to be strictly NOT ready.
- Cycles cause all tasks in the cycle to be strictly NOT ready.

**Dynamic Scoring (Starvation Prevention)**
- Starvation prevention is a runtime sorting concern, not a data-layer rule. There is no computed or inherited priority — the `priority` field in YAML is strictly local.
- `pebble next` sorts the ready frontier using the sort key tuple: **`(len(blocking) DESC, priority ASC, created_at ASC)`**. Blocking count is the primary key; local priority breaks ties; creation time is the final tiebreaker. A task blocking 5 others always ranks above a task blocking 0, regardless of local priority. This means a small task that is the sole blocker of a massive Epic naturally bubbles to the top of the queue.
- This replaces the old transitive `effective_priority` inheritance model. Priority information is never written back to YAML; it exists only as a runtime sort key.

### 4.6 CLI

**CLI Behavior Changes**
- **Unchanged:** `list`, `show`, `search` remain the primary read commands.
- **Change in storage location:** `add`/`update`/`show`/`list` read and write Markdown files under `tasks-dir` (default `docs/pebble/`).
- **New read behavior:** `list` and `search` scan Markdown files directly; any caches are optional and non-canonical.
- **No hidden worktree dependency:** The CLI no longer requires a sync worktree for reads/writes under this model, which dramatically reduces git worktree complexity.

**CLI Command Surface (Authoritative)**

This RFC supersedes the earlier `003-cli-contract.md` snapshot in this directory; the live `docs/cli-contract.md` is the authoritative interface specification and will be kept in sync during implementation.

**Global options**
- `--json`: Universal structured output flag. Also accepted at the sub-command level with the same effect. Intended usage: `pebble --json <command> <args>` or `pebble <command> <args> --json`.
- `--dir <PATH>`: Override the default tasks directory (default: `docs/pebble/`). Users can pass `--dir` on any command to point at a non-default task root. If a relative path is provided, it is strictly resolved relative to the user's current working directory.
- `--help-json`: Emit a machine-readable schema of commands, flags, and output shapes to stdout, then exit.

**Repository management**
- `pebble init`: Bootstraps the environment, creates the tasks directory, creates `.pebble/AGENTS.md` (see §4.8), and prints a message advising the user to include it in their project's agent configuration.

**Query commands**
- `pebble list` (alias: `ls`): Parses the directory and builds the DAG. By default, tasks with an `status` of `done` or `canceled` are implicitly filtered out. Filters: `--status`, `--tag`, `--dep`, `--priority`, `--is-ready` (computed; shows only tasks whose `deps` are all `done` or `canceled` and whose `status` is actionable), `--all` (bypass default omission to include `done` and `canceled` tasks). Ordering: `--sort <field>` (see "Default Sort Order" below). Pagination: `--limit <N>` returns only the first N results after sorting.
- `pebble next`: Returns the single highest-scoring ready task using dynamic scoring (see §4.5 "Dynamic Scoring"). Equivalent to `pebble list --is-ready --limit 1` with the dynamic score as the primary sort key. Accepts `--json`. This is the canonical "what should I work on?" entry point for agents and humans alike.
- `pebble show <id>`: Prints the full details, tree-context, and Markdown body of a specific task. `--path-only` prints only the file path (relative to `tasks-dir`) and nothing else — useful for agents and scripts that want to read the file directly.
- `pebble search <query>`: Full-text search across titles and Markdown bodies.
- `pebble config get <key>`: Reads a configuration value. Supported keys: `issue-prefix`, `tasks-dir`. Also serves as a way for users and agents to discover the resolved config file location and effective values.

**MVP search semantics**
- `pebble search` performs a plain substring match against the task `title` (frontmatter) and the raw Markdown `body` (frontmatter excluded).
- Matching is case-insensitive. No regex, stemming, or tokenization in MVP.
- Results are returned in the default list order.

**MVP filter semantics**
- `--status <status>` matches tasks whose `status` equals `<status>`. The flag is repeatable; multiple values are OR'ed. Explicitly requesting `--status done` or `--status canceled` guarantees those tasks are included, overriding the default omission.
- `--priority <N>` matches tasks whose `priority` equals `N`. The flag is repeatable; multiple values are OR'ed. Tasks with no `priority` never match `--priority`.
- `--tag <tag>` matches tasks whose `tags` array contains `<tag>`. The flag is repeatable; multiple values are AND'ed (a task must possess all provided tags to match).
- `--dep <id>` matches tasks whose `deps` array contains `<id>`. The flag is repeatable; multiple values are OR'ed (a task matches if it depends on any of the provided IDs).
- `--all` disables the default omission, ensuring `done` and `canceled` tasks are evaluated alongside actionable ones.

**Default Sort Order**

The default sort order for `pebble list` (and by extension `pebble next`) is deterministic and dependency-aware:

1. **Topological order** (respecting `deps`): if task B has `deps: [A]`, then A appears before B regardless of priority. Among tasks at the same topological level (no dependency relationship between them), the remaining tiebreakers apply.
2. **Blocking count** descending (`len(blocking)`). Tasks blocking a larger number of downstream tasks appear first. This creates the dynamic scoring that surfaces critical bottlenecks.
3. **Priority** ascending (lower number = higher priority), using `priority`. Tasks with no `priority` sort after all prioritized tasks.
4. **`created_at`** ascending (oldest first) as the final tiebreaker.

The `--sort <field>` flag overrides this default. Supported fields: `priority` (which sorts by `priority`), `blocking`, `created_at`, `modified_at`, `status` (which sorts by `status`), `title`. When `--sort` is specified, topological ordering is NOT applied — the results are sorted purely by the requested field. `--sort` defaults to ascending; prefix with `-` for descending (e.g., `--sort -created_at`). When sorting by `status`, the order is: `todo`, `in_progress`, `done`, `canceled`.

Note: when `--is-ready` is active, all returned tasks are at the dependency frontier (their prerequisites are all `done` or `canceled`), so the topological component of the default sort has no effect and the order is effectively blocking count → priority → created_at.

**Future direction for retrieval**
- Keep simple flags for common cases, and add a small, explicit query language only if needed. A future `--filter <expr>` (or a dedicated `pebble query`) can provide compound conditions and ranges for both humans and agents without re-inventing SQL.

**Mutation commands**
- `pebble add <title>`: Generates the boilerplate `.md` file. By default, `status` is initialized to `todo`. Options: `--status <status>`, `--priority <N>`, `--body <text>`, `--dep <id>`, `--tag <tag>`. The `--body` text becomes the Markdown body of the file.
- `pebble update <id>`: Safely modifies the frontmatter, title, and/or body. Options: `--title <text>`, `--status <status>`, `--priority <N>`, `--clear-priority`, `--body <text>`, `--append-body <text>`, `--add-tag <tag>`, `--remove-tag <tag>`, `--add-dep <id>`, `--remove-dep <id>`. To unset optional singular fields, use `--clear-priority`. `--body` replaces the entire Markdown body; `--append-body` appends text to the existing body (separated by a blank line). If the existing body is empty, `--append-body` writes the text without a leading blank line. Both are provided as a convenience for simple mutations — for complex body editing (restructuring sections, checking off checklist items, etc.), direct file editing is the expected workflow (see §4.3 Direct File Access Contract). When modifying the task, the CLI automatically sets the `modified_at` timestamp. When setting `--status done` or `--status canceled`, the CLI automatically sets `resolved_at`. Updating a title never renames the file.
- `pebble archive`: Automatically moves tasks with a status of `done` or `canceled` where `resolved_at` is older than a threshold (e.g., `> 30 days`) into an `archive/` subdirectory to reduce IDE clutter. If a filename collision occurs, the CLI appends a numeric suffix to the archived filename.
- Agents and users are encouraged to edit Markdown bodies directly — this is a core benefit of the Markdown-native model. The CLI also provides `--body` and `--append-body` on `pebble update` for simple cases.

**Validation & Diagnostics**
- `pebble doctor`: A non-fatal diagnostic tool that examines the health of the task graph. It reports ID collisions, missing dependencies (dangling pointers), and cyclic deadlocks. It exits with a `0` status code; it serves purely as user advisory.
- `pebble check`: The strict, fatal counterpart to `doctor`. It runs the exact same graph and schema validations, but exits with a non-zero status code if any anomalies (cycles, dangling pointers, malformed frontmatter) are found. Ideal for CI pipelines and Git `pre-commit` hooks.
- `pebble fix`: Applies safe, deterministic formatting repairs that do not change dependency meaning (e.g., inserting missing `created_at`, sorting YAML keys, normalizing whitespace). Graph issues are reported but dependency edges are not rewritten.

**Read/Write Policy**
- **Read-only:** `list`, `next`, `show`, `search`, `config get`, `doctor`, and `check` never modify files.
- **Write commands:** `add`, `update`, `fix`, and `archive` are the only commands that mutate task files or their locations.

**Strictness & Failure Modes**
- Validation is relaxed across the board. The CLI is not a schema compiler.
- **Read Commands:** Full recursive scans build the graph. If a cycle or dangling pointer exists, it is naturally reflected in `is_ready: false` states. Only unparseable YAML or duplicate IDs cause a file to be skipped (with a stderr warning).
- **Write Commands:** Graph modifications (`add`, `update`) write directly without checking for loops. If you author a loop, `pebble doctor` will catch it asynchronously. This guarantees developers can always write to their files.
- **Structured Errors:** Full graph anomalies are available via `pebble doctor --json` or `pebble check --json`.

- If full scans become a bottleneck, add a cached index as a strictly derived (non-canonical) optimization. The MVP assumes full scans are acceptable for single-repo scale.

**JSON Output Contract**

All `--json` output is a single JSON value printed to stdout per invocation.
On failure, no JSON is emitted; `stdout` is empty, `stderr` contains a human-readable error message, and the exit code is non-zero.

- **Query commands** (`list`, `search`): `{"tasks": [<TaskObject>, ...]}`.
- **`next --json`**: A single unwrapped `TaskObject`, or `null` if no ready tasks exist.
- **`show --json`**: A single unwrapped `TaskObject`. With `--path-only`, emits `{"path": "..."}` instead.
- **`show --path-only`** (without `--json`): Prints just the file path as a bare string to stdout (no JSON wrapping, no trailing newline decoration).
- **Mutation commands** (`add`, `update`): Echo back the full `TaskObject` after the write.
- **`check --json` / `doctor --json`**: `{"ok": bool, "errors": [{"file": "...", "line": N|null, "message": "...", "code": "<string>"?}]}`.
- **`archive --json`**: `{"archived": [{"id": "...", "moved_to": "..."}]}`.
- **`config get --json`**: `{"key": "<key>", "value": "<value>"}`.

A `TaskObject` includes:
- All stored frontmatter fields (`id`, `title`, `status`, `priority`, `created_at`, `modified_at`, `resolved_at`, `deps`, `tags`).
- Computed fields: `is_ready` (boolean), `blocked_by` (array of missing or open deps), `blocking` (array of tasks that are blocked by this task).
- `body`: the raw Markdown content after the frontmatter delimiter, verbatim.
- `path`: the file path relative to `tasks-dir`.

**Command Deprecations & Removals**
- `pebble sync` is removed under the Markdown-native model because no worktree sync exists; task files are normal repo content.
- `sync-branch` configuration is removed.
- `pebble import` is removed. The one-time JSONL → Markdown migration is handled by a throw-away script (e.g., Python) that directly writes `.md` files to disk, bypassing the CLI entirely. By writing directly to disk, it preserves legacy IDs (maintaining external links), inherently supports single-pass generation, and perfectly leverages the "Forgiving Reads" philosophy to ride over any dependency cycles inherent in the legacy data (which developers can safely clean up later via manual edits, with `pebble doctor` validating the result).
- `pebble init` only creates the tasks directory and config; it no longer creates a worktree.

### 4.7 Migration

There is exactly one existing JSONL database to migrate. A one-time throw-away script transforms the current JSONL into Markdown files under `docs/pebble/`. The Markdown schema intentionally drops audit fields (`owner`, `created_by`, `close_reason`) and per-edge audit metadata; Git history remains the fallback. The exhaustive field-by-field mapping is in Appendix C.

### 4.8 Agent Bootstrapping & Discoverability

AI coding agents (Amp, Claude Code, Gemini CLI, Copilot, Cursor, etc.) discover project conventions by reading well-known configuration files at the project root (`AGENTS.md`, `.cursorrules`, `.github/copilot-instructions.md`, etc.). Without an entry point in one of these files, an agent will never know pebble exists — rendering the "equally useful for AI coding agents" goal moot.

**`.pebble/AGENTS.md`**

`pebble init` generates `.pebble/AGENTS.md` with the following content (adapting `issue-prefix` and `tasks-dir` from the resolved config):

```markdown
# Pebble Task Tracker

This project uses [pebble](https://github.com/matta/pebble) for task tracking.
Tasks are stored as Markdown files in `docs/pebble/`.

## Quick Reference

All commands support `--json` for structured output. Prefer `--json` when
parsing results programmatically.

- **Full CLI reference:** `pebble --help-json` (machine-readable schema of all commands, flags, and output shapes)
- **Help for a specific command:** `pebble <command> --help`

- **What should I work on?** `pebble next --json`
- **List all ready tasks:** `pebble list --is-ready --json`
- **View a task:** `pebble show <id> --json` (or read the file directly)
- **Get file path for a task:** `pebble show --path-only <id>`
- **Create a task:** `pebble add "title" --body "description" --json`
- **Update a task:** `pebble update <id> --status in_progress --json`
- **Append notes:** `pebble update <id> --append-body "implementation notes..." --json`
- **Validate the database:** `pebble doctor --json`

## Workflow

1. Run `pebble next` to find the highest-priority actionable task.
2. Run `pebble update <id> --status in_progress` to claim it.
3. Do the work.
4. Run `pebble update <id> --status done` when finished.

## Direct File Access

Task files are plain Markdown with YAML frontmatter in `docs/pebble/`.
You can read and edit them directly — this is a core feature of the
system, not a workaround. Use `pebble show --path-only <id>` to resolve
an ID to a file path.

Direct file editing is especially useful for body changes: checking off
checklist items, appending notes, or restructuring sections. Direct body
edits do not automatically update `modified_at`; use `pebble update
--append-body` or `--body` if you want `modified_at` refreshed. For
frontmatter changes (status, priority, tags, dependencies), prefer the
CLI — it handles timestamps, validation, and cross-references
automatically.
```

This file is designed to be included by reference from the project's root agent configuration. For example, in `AGENTS.md`:

```markdown
Read @.pebble/AGENTS.md
```

Or for agents that don't support transclusion, the user can copy the content directly. `pebble init` prints this guidance on completion.

**Design Constraints:**
- The generated file must be concise (agents have limited context windows).
- It must include the canonical "what to work on next" workflow as the first entry.
- It must reference the actual configured `tasks-dir`, not a hardcoded default.
- `pebble init` never overwrites an existing `.pebble/AGENTS.md`. If the file exists, it prints a warning and skips.

### 4.9 Risks & Mitigations

1. **Filename collisions / human-editable names**
   - *Risk:* Two issues could map to the same filename, or renames could break links.
   - *Mitigation:* Filenames are advisory only; `id` is canonical. On write, the CLI ensures uniqueness by suffixing `-2`, `-3`, etc. On read, `id` is authoritative.
2. **Schema drift from manual edits**
   - *Risk:* Users edit frontmatter by hand and introduce invalid fields or types.
  - *Mitigation:* CLI validates frontmatter strictly and reports precise errors (line/field). `pebble doctor` is read-only; `pebble fix` performs safe, non-destructive repairs.
3. **Query performance (deferred)**
   - *Risk:* Large repos may need faster list/search than raw file scans provide.
   - *Mitigation:* Defer optimization until user reports demand. Architectural options include lazy caching, background file watchers, incremental indexing, and derived query indices (JSONL or SQLite) that are strictly non-canonical.

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
- **Temporal Consistency:** If you `git checkout` a release from 6 months ago, you see exactly what tasks were pending, in-progress, or completed *at that exact moment in time*. The state of the project planner perfectly matches the state of the codebase.
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
title: Deploy staging environment
status: todo
created_at: 2026-01-15T10:30:00Z
---
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
- **Cons:** Because directory paths are not indexed or surfaced by `pebble search` or `pebble list`, this organization becomes a "shadow taxonomy." It is completely invisible to the CLI's queries, meaning users cannot rely on it for actual task retrieval. Pebble enforces a flat semantic structure using `tags` and graph edges (`deps`), using directory structure and paths purely for automated lifecycle `archive` sorting.

## Appendix B: Migration Field Mapping

This is the authoritative and exhaustive mapping used by the one-time migration script. Every field present in the JSONL schema is listed below with its disposition. Any JSONL field not listed here is a migration error — the script must fail rather than silently drop data.

**Compatibility & Data Loss**
- The Markdown schema intentionally drops: `owner`, `created_by`, `updated_at`, `closed_at`, and `close_reason`.
- The migration script will **not** preserve per-edge audit metadata (`dependencies[].created_at`, `dependencies[].created_by`). It preserves the logical dependency graph by collapsing structural edges into `deps` and converting `relates-to` into Markdown backlinks.

> **Rationale:** There is no concrete use case for these audit fields today; Git history remains the fallback for audit-style questions.

**Field mapping (exhaustive)**
- `id` → preserved exactly as-is. The migration script bypasses the CLI and writes the `.md` files directly, maintaining external links and avoiding the need to translate cross-references during generation.
- `title` → frontmatter `title`.
- `description` → Markdown body (verbatim).
- `status` → see status mapping below.
- `priority` → frontmatter `priority` (preserved as integer).
- `issue_type` → `tags` entry with the same string (preserves information without a formal type system).
- `created_at` → frontmatter `created_at` (required; missing value is a migration error).
- `updated_at` → frontmatter `modified_at` (if present).
- `closed_at` → frontmatter `resolved_at` when status maps to `done` or `canceled` (if present).
- `owner` → dropped (audit via Git history).
- `created_by` → dropped (audit via Git history).
- `close_reason` → used only for status mapping; otherwise dropped.
- `labels` → merged into `tags` (deduplicated with any tag derived from `issue_type`).
- `acceptance_criteria` → appended to body under an `## Acceptance Criteria` heading (if non-empty).
- `notes` → each note appended to body under a `## Notes` heading (if non-empty).
- `comments` → each comment appended to body under a `## Comments` heading (if non-empty).
- `defer_until` → preserved by appending the deferred date as a standard note in the Markdown body under the `## Notes` heading.
- `original_type` → dropped (internal bookkeeping from beads type migrations; no semantic value).
- `deleted_at`, `deleted_by`, `delete_reason` → used only for `tombstone` status mapping (see below); otherwise dropped.
- `dependencies` → see dependency type mapping below.

**Status mapping**
- `open` → `todo`.
- `in_progress` → `in_progress`.
- `deferred` → `todo`.
- `closed` → `done` unless `close_reason` indicates cancellation (`canceled` / `cancelled`, case-insensitive) in which case → `canceled`.
- `tombstone` → `canceled` (with `resolved_at` set from `deleted_at` if present, falling back to `updated_at`).
- Any other status value is a migration error (explicitly surfaced).

**Dependency type mapping**
- All structural dependencies (`parent-child`, `blocks`, `depends-on`) are collapsed into the `deps` array. (e.g. if A blocks B, B lists A in `deps`. If Epic E has child C, Epic E lists C in `deps`).
- `relates-to` → converted into Markdown backlinks appended to the `description` body as `[Related: <id>](path)`.
- Edge audit metadata is dropped.

**Timestamp mapping**
- `modified_at` is set from `updated_at` if present; otherwise omitted.
- `resolved_at` is set from `closed_at` if present. If `closed_at` is missing but status maps to `done` or `canceled`, use `updated_at` when available; otherwise omit. For `tombstone` → `canceled`, use `deleted_at` if present, falling back to `updated_at`.

**Body assembly**
- Each task file starts with YAML frontmatter, followed by the `description` as the Markdown body (if non-empty).
- If `acceptance_criteria`, `notes`, or `comments` are non-empty, they are appended after the description as separate H2 sections.

**Command mapping summary**
- `pebble sync` and `sync-branch` config are removed (see "Command Deprecations & Removals").
- `pebble list/show/search/add/update` remain, but operate on Markdown files in `tasks-dir`.

## Appendix C: Iterative Refinement

DO NOT REMOVE THIS SECTION

You are a principle staff engineer considering docs/rfc-reimagining-pebble.md for approval. Are there any concrete, blocking, issues that prevent you from approving the document and allowing implementation planning to begin? If so, please list them.
