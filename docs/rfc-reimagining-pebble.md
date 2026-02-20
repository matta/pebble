# RFC: Re-imagining Pebble from Scratch

## 1. Introduction & Motivation

The goal of this RFC is to step back and re-imagine Pebble from the ground up. Inspired by the legacy `bd` tool but now forging its own divergent path, the core mission remains unaltered: **provide a project task tracking system that is equally useful and delightful for both human developers and autonomous AI coding agents**.

While the current implementation relies on a Rust CLI with a JSONL storage backbone, this document explores the solution space without those constraints. We aim for a "minimum useful feature set" tailored not for enormous enterprise projects, but for the simpler, single-repo projects common in open-source development and indie hacking.

## 2. Minimum Useful Feature Set

Based on the `golden.jsonl` data and typical single-repo development flows, the essential feature set is surprisingly small:

1. **Task Tracking:** Ability to define a task with an ID, title, description, and status.
   - States: `open`, `in_progress`, `closed` (and potentially `tombstone` for deleted tasks).
2. **Hierarchy & Composition:** Epics and Sub-tasks. A task can be heavily composed of smaller tasks (`parent-child`).
3. **Ordering & Dependencies:** Execution ordering. Knowing what to do *next* is critical for agents. We need `blocks` / `depends-on` relationships.
4. **Basic Metadata:** Creation timestamps, resolution reasons, and perhaps basic assignment/ownership (useful when humans and agents collaborate).

## 3. The "State Synchronization" Problem

Before discussing specific file formats, we must address the fundamental friction—or feature—of co-locating a task database with application code inside a version control system like Git.

### The Problem: "The Bug Database Friction"
If task state is tracked in the main branch (e.g., inside a `.pebble/` folder), it feels like it creates a workflow bottleneck:
- **Tangent Discoveries:** A developer working on `feature-A` discovers a bug related to `feature-B`. If they create the task locally and commit it, it's not visible project-wide until `feature-A` is merged.
- **Merge Conflicts on State:** Changing the status of an ongoing epic on multiple feature branches simultaneously can lead to merge conflicts simply trying to track *what* is being done.

*This explains why most industry-standard bug trackers (Jira, Linear, GitHub Issues) exist completely "out-of-band" (hosted externally) rather than in the repository itself.*

### The Counter-Argument: "In-Band Storage is a Feature, Not a Bug"
Alternatively, tracking task states directly with the code is a massive benefit that out-of-band trackers lack:
- **Temporal Consistency:** If you `git checkout` a release from 6 months ago, you see exactly what tasks were pending, blocked, or completed *at that exact moment in time*. The state of the project planner perfectly matches the state of the codebase.
- **The Solution to Tangent Discoveries:** If a user needs to file an issue separate from their current development track, the solution isn't to build a complex global sync mechanism—they simply branch off `main`, commit the new task, and merge it quickly. It forces good hygiene.

### Paradigm 1: The Out-of-Band Service (The External API)
*Move state out of the repository entirely, relying on a lightweight backend service.*
- **Pros:** Eliminates Git friction. Task status is instantly globally visible.
- **Cons:** Violates the "local first, offline capable, single-repo" ethos. Introduces infrastructure overhead. Destroys temporal consistency with the codebase.

### Paradigm 2: The In-Band Hidden Worktree (The Current Pebble Approach)
*Store the data in the repository, but utilize a "hidden" Git branch (e.g., `pebble-data`) mounted via a Git worktree inside a `.git/` subdirectory.*
- **Pros:** Maintains local-first offline capability. Tasks are instantly synced across feature branches because the worktree operates independently.
- **Cons:** Extremely high complexity. `git worktree` commands are brittle to setup, difficult for agents to intuitively reason about, and create edge cases around `git push/pull`.

### Paradigm 3: The SQLite / Local Database Approach
*Store state in an un-tracked local `.pebble.sqlite` database file. Sync via a secondary mechanism.*
- **Pros:** Immediate reads/writes. No git branch interference. Easy to query using SQL.
- **Cons:** Merging binary SQLite dumps across branches is incredibly difficult. Natively unreadable by humans without dedicated tooling.

## 4. Storage Format Considerations

Assuming we embrace the "In-Band Storage is a Feature" argument (abandoning the hidden worktree and just committing tasks to the main branch), what format should the data take?

### Avenue A: The "Everything is a File" Markdown Approach
*Store each task as a discrete Markdown file inside a visible directory in the source tree (e.g., `docs/pebble/` or `docs/tasks/`), using YAML frontmatter for metadata. These files are committed to standard Git.*

**Example `docs/pebble/deploy-staging-environment.md`:**
```markdown
---
id: proj-0kq
status: open
parent: proj-epic1
depends_on: [proj-1ab]
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

### Avenue B: The Append-Only Log (Refined JSONL)
*Keep a JSONL event stream or state dump (similar to current `golden.jsonl`), heavily optimizing the CLI/MCP layer to hydrate the state.*

**Pros:**
- **Machine Native:** JSON is the lingua franca of LLMs.
- **Git Friendly Appends:** Adding a line rarely conflicts with another added line.
**Cons:**
- **Human Antagonistic:** Humans cannot read or edit JSONL manually. This violates the "degrade gracefully" principle if the CLI/UI is unavailable. Pull Request diffs for a JSONL state change are extremely difficult for human reviewers to parse.

## 5. Implementation Language & Tooling

If we assume a CLI or an agent tool is required to enforce schemas, the choice of language matters for distribution and integration.

### Option 1: Rust (The Current Path)
- **Why?** Blazing fast, type-safe, distributes as a single static binary. Excellent for a tool that runs on every `git commit`.

### Option 2: TypeScript / Node
- **Why?** The AI ecosystem is heavily skewed towards TS/Python. Building Model Context Protocol (MCP) servers locally is easiest in TypeScript. Can be executed via `npx pebble-cli`.

### Option 3: Go (Golang)
- **Why?** Fast startup time like Rust, single binary distribution, but with a simpler concurrency model and arguably faster development velocity.

## 6. Re-imagining the Workflow & The Flawed Hybrid Paradigm

### A Flawed Idea: The "Read-Model / Write-Model Projection"
One brainstorming idea to resolve format friction was to separate presentation from storage:
1. Store actual task data in a hidden SQLite or JSONL Worktree (Write-Model).
2. Generate a `.pebble/` folder full of `.gitignore`'d Markdown files purely for the human UI (Read-Model).

**Why this fails:**
- **Agent Blindness:** Agent frameworks (Cursor, Copilot, Aider, etc.) are explicitly hard-coded to ignore `.gitignore` files. Generating ignored Markdown files means agents will completely ignore the UI you just built for them.
- **Loss of Code Review:** Version control diffs and review tooling (like GitHub PR reviews) require the Markdown files to be committed. If they are `.gitignore`'d, you cannot comment on a task description change in a Pull Request.

### The Honest Conclusion
If we want the benefits of Git tooling (PR reviews, history, blame) and the benefits of AI Agents natively understanding the context files, **the files themselves must be committed to the main branch as plain text (Avenue A), and they must live in a visible directory in the source tree (not a hidden folder).**

## 7. Decision & Recommendation

**Decision:** Adopt **per-issue Markdown files with YAML frontmatter** stored under a visible, human-friendly directory such as `docs/pebble/`. The Markdown files are the source of truth. A derived `issues.jsonl` index may be generated/maintained by the CLI for fast list/search, but it is not the canonical store.

**Rationale:** This maximizes human transparency, makes review diffs first-class in Git, and keeps agent tooling aligned with what humans see. It also eliminates single-file merge conflicts while preserving fast query paths.

**Non-Goals:** Not targeting enterprise-scale analytics or cross-repo issue federation.

**Success Criteria:**
1. Add/update/list/show remain under 200ms for 1k issues on a typical developer machine.
2. Git merges of concurrent edits to different issues resolve cleanly without manual intervention.
3. Agents can create/update issues without schema drift; human edits remain the source of truth.

**Migration Plan:**
1. `pebble import` reads legacy `issues.jsonl` and writes per-issue Markdown to `docs/pebble/`.
2. `pebble export` regenerates `issues.jsonl` for compatibility/test fixtures.
3. Provide a one-time `pebble migrate` command guarded by `--yes`.

## 8. Recommendations & Discussion Points

To retain the dual-audience goal while stripping away enterprise complexity, we should consider:

1. **Embrace In-Band Synchronization:** Accept that tracking tasks with code is a feature. If you need an out-of-band bug filed, branch from `main`, add the Markdown file, and merge it. Enjoy the temporal consistency of checking out old Git refs and seeing the exact state of the project map.
2. **Commit Markdown Natively:** Use Avenue A (Markdown + YAML Frontmatter) committed directly to the main branch in a visible directory like `docs/pebble/`. This provides instant, out-of-the-box UI on GitHub and native semantic understanding for Agents.
3. **The CLI as a Cache/Accelerator:** The CLI's job isn't to hide the storage; it is to quickly parse the hundreds of Markdown files, build the dependency DAG, and answer questions like "What tasks are blocking X?" or serve that graph locally via MCP.

## 9. Detailed Design Explorations & Open Decisions

To fully realize the "Markdown Native" Avenue A paradigm, several technical details must be debated and finalized:

- [x] **Frontmatter Format: YAML vs TOML**
  While TOML is preferred for configuration in the Rust ecosystem, **YAML is the definitive recommendation here because of network effects.**
  - **The YAML Network Effect:** YAML frontmatter bounded by `---` is recognized natively by GitHub, Obsidian, Hugo, Prettier, language servers, and nearly all IDEs. Choosing TOML (often bounded by `+++`) breaks this ecosystem interoperability, defeating a major benefit of Avenue A.
  - **Mitigating YAML's Flaws in Rust:** YAML is infamous for type-inference quirks (the "Norway problem" where `no` becomes a boolean). However, in a Rust context using `serde_yml` (the community fork of `serde_yaml`), we can mitigate this entirely by defining a strictly typed `struct TaskFrontmatter`. If a user types `status: no` and the struct expects an enum, `serde_yml` will throw a clear validation error. We could also explore a "StrictYAML" parsing crate to intentionally reject complex, ambiguous YAML features.

- [x] **Frontmatter Schema Design: Resolving Discrepancies**
  Comparing the current Pebble `golden.jsonl` schema against the minimalist Rust schema proposed, there are several key discrepancies. Handling these one by one reveals the philosophical shifts of moving to a Markdown-native approach:

  **1. The `description` field (In Pebble, missing from Snippet)**
  - *Current:* Pebble stores the prose of the task inside a JSON string field (`description`).
  - *Decision:* **Drop from Frontmatter.** The entire justification for Avenue A is that the Markdown body *is* the description. The frontmatter only handles metadata; the prose lives natively below the `---` delimiters.

  **2. Graph Edges (`parent` and `depends_on` vs complex `dependencies`)**
  - *Current:* Pebble uses a complex `dependencies` array of objects (e.g., `[{"issue_id": "A", "depends_on_id": "B", "type": "parent-child", "created_at": "..."}]`) to track every edge and who created it.
  - *Snippet:* Simplifies this to `parent: Option<String>` and `depends_on: Vec<String>`.
  - *Decision:* **Adopt the Snippet.** Tracking the `created_at` of an edge link is overkill for local-first single-repo development. Explicitly defining `parent` as a scalar makes tree traversal much faster and easier for humans to read and edit.

  **3. `tags` (In Snippet, missing from Pebble)**
  - *Current:* Pebble doesn't have a first-class `tags` string array in the examined golden schema.
  - *Decision:* **Keep.** A `tags` array is highly idiomatic in Markdown frontmatter (Obsidian, Jekyll) and provides lightweight categorization without needing rigorous epic/parent linkage.

  **4. `priority` (In Pebble, missing from Snippet)**
  - *Current:* Pebble relies on an explicit integer `priority` (e.g., 0, 1, 2) which dictates sorting.
  - *Decision:* **Requires Debate.** Does minimal single-repo development need explicit numerical priorities (P0, P1, P2), or is graph topology (what is blocking what) alongside `status` sufficient? For now, we should probably add `priority: Option<u8>` to the schema to maintain feature parity.

  **5. Audit Trail (`owner`, `created_by`, `closed_at`, `close_reason`)**
  - *Current:* Pebble tracks exhaustive audit metadata for every task closure.
  - *Snippet:* Drops most of this, reducing to just `created_at` and `updated_at`.
  - *Decision:* **Delegate to Git.** In a repository-backed bug tracker, `git blame` natively tracks who created a task, who closed it, and when. Storing `owner` or `close_reason` in the file duplicates version control features. We should keep `created_at` for simple UI sorting, but drop `created_by`, `updated_at`, `closed_at`, and `owner` in favor of trusting `git log`.

  **Proposed Final Rust Schema:**
  ```rust
  use chrono::{DateTime, Utc};
  use serde::{Deserialize, Serialize};
  use std::path::PathBuf;

  #[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
  #[serde(rename_all = "lowercase")] // Ensures YAML matches exactly "todo", "done" etc.
  pub enum TaskStatus {
      Todo,
      Active, // AKA in_progress
      Blocked,
      Done,
  }

  /// Represents the exact structure of the YAML front matter.
  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct TaskFrontmatter {
      pub id: String,
      // Status strictly validated against the enum
      pub status: TaskStatus,
      
      // Kept for parity, but kept optional if not strictly needed
      pub priority: Option<u8>,
      
      pub parent: Option<String>,
      pub created_at: DateTime<Utc>,
      pub updated_at: Option<DateTime<Utc>>,
      
      // Graph edges: empty arrays default nicely.
      #[serde(default)]
      pub depends_on: Vec<String>,
      
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

- [x] **CLI Command Surface: Reconciling with `trk` Concepts**
  If the CLI is an accelerator rather than a database dictating access, its job is to make querying the DAG and bulk-mutating files effortless for humans and agents. Drawing inspiration from the `trk` CLI model, here is the reconciled, optimal command surface:

  - **Global Options:**
    - `--format <text|json|tree>`: Universal structured output flag (agents use `json`). Replaces command-specific `--json` flags.
    - `--dir <PATH>`: Override the default tasks directory (e.g., `.pebble/` or `docs/tasks/`).

  - **Repository Management:**
    - `pebble init`: Bootstraps the environment, creates the tasks directory, and generates a `pebble.toml` configuration.

  - **Query Commands:**
    - `pebble list` (alias: `ls`): Parses the directory and builds the DAG.
      - Filters: `--status`, `--tag`, `--parent`, `--is-blocked` (shows only tasks where dependencies are not `done`).
    - `pebble show <id>`: Prints the full details, tree-context, and Markdown body of a specific task.

  - **Mutation Commands** (These modify the Markdown files directly):
    - `pebble new <title>` (alias: `add`, `create`): Generates the boilerplate `.md` file. 
      - Options: `--parent <id>`, `--tag <tag>`, `--depends-on <id>`, `--no-edit` (crucial for agents to skip `$EDITOR`).
    - `pebble update <id>`: Safely modifies the frontmatter and bumps the `updated_at` timestamp.
      - Options: `--status <status>`, `--parent <id>`, `--add-tag <tag>`, `--remove-tag <tag>`, `--add-depends-on <id>`, `--remove-depends-on <id>`. (Adheres to the CLI contract for incremental list mutations).
    - `pebble append <id> --message <text>`: Safely appends raw Markdown to the body without risking frontmatter parsing corruption. Excellent for agents adding notes.
    - `pebble edit <id>`: Locates the file and opens it natively in `$EDITOR`.

  - **Validation:**
    - `pebble check`: A strict linter that evaluates the `.md` database.
      - Checks: ID collisions, broken `depends_on` links, circular dependencies, schema adherence, and state consistency (e.g., flagging a `done` parent that still has `active` children).
      - Options: `--fix` to automatically rectify safe, deterministic errors (e.g., sorting TOML/YAML keys, injecting missing timestamps).

---
*Open for feedback: Does fully committing to Markdown files in the main branch (Option A) create too much directory clutter, or is the benefit of native GitHub PR capabilities worth the noise?*
