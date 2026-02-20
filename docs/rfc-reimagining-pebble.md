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
*Store each task as a discrete Markdown file inside a visible `.pebble/` directory, using YAML frontmatter for metadata. These files are committed to standard Git.*

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
If we want the benefits of Git tooling (PR reviews, history, blame) and the benefits of AI Agents natively understanding the context files, **the files themselves must be committed to the main branch as plain text (Avenue A).**

## 7. Recommendations & Discussion Points

To retain the dual-audience goal while stripping away enterprise complexity, we should consider:

1. **Embrace In-Band Synchronization:** Accept that tracking tasks with code is a feature. If you need an out-of-band bug filed, branch from `main`, add the Markdown file, and merge it. Enjoy the temporal consistency of checking out old Git refs and seeing the exact state of the project map.
2. **Commit Markdown Natively:** Use Avenue A (Markdown + YAML Frontmatter) committed directly to the main branch. This provides instant, out-of-the-box UI on GitHub and native semantic understanding for Agents.
3. **The CLI as a Cache/Accelerator:** The CLI's job isn't to hide the storage; it is to quickly parse the hundreds of Markdown files, build the dependency DAG, and answer questions like "What tasks are blocking X?" or serve that graph locally via MCP.

---
*Open for feedback: Does fully committing to Markdown files in the main branch (Option A) create too much directory clutter, or is the benefit of native GitHub PR capabilities worth the noise?*
