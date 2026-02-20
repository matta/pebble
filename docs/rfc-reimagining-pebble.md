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

## 3. Storage Format Paradigms

Since both humans and AIs interact with this system within a Git repository, the storage format is the most determinative architectural choice.

### Avenue A: The "Everything is a File" Markdown Approach
*Store each task as a discrete Markdown file inside a `.pebble/` or `docs/tasks/` directory, using YAML frontmatter for metadata.*

**Example `.pebble/tasks/proj-0kq.md`:**
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

**Pros:**
- **Ultimate Human Readability:** GitHub, GitLab, and local IDEs render these files perfectly. Humans can edit them natively without a CLI.
- **Agent Friendly:** LLMs have profound native understanding of Markdown. RAG algorithms chunk Markdown naturally.
- **Git Diffs:** Conflict resolution is trivial because files are separated. History per-task is just `git log <file>`.
**Cons:**
- **Graph Traversal:** Requires reading potentially hundreds of small files to build the dependency graph.
- **Data Integrity:** Users can easily make typos in YAML frontmatter unless validated via a pre-commit hook or CLI.

### Avenue B: The Single-File Human-Readable (TOML/YAML)
*Store the entire state in a single `.pebble.toml` or `pebble.yaml` file at the repository root.*

**Example `.pebble.toml`:**
```toml
[tasks.proj-0kq]
title = "Deploy staging environment"
status = "open"
parent = "proj-epic1"
depends_on = ["proj-1ab"]
description = """
Run the canary deploy pipeline against the `staging` cluster.
"""
```

**Pros:**
- **Single Source of Truth:** Easy to parse in one go. Extremely easy for an MCP server or thin CLI to load.
- **No Directory Clutter:** Just one file.
- **Human Editable:** TOML is highly readable and editable by humans.
**Cons:**
- **Merge Conflicts:** If multiple agents/humans work concurrently, a single file will suffer from merge conflicts much faster than discrete files.
- **Scalability:** Editing a 2000-line TOML file becomes painful for humans.

### Avenue C: The Append-Only Log (Refined JSONL)
*Keep a JSONL event stream or state dump (similar to current `golden.jsonl`), but heavily optimize the CLI/MCP layer to hide it from humans.*

**Pros:**
- **Machine Native:** JSON is the lingua franca of LLM tool calls.
- **Git Friendly Appends:** Adding a line never conflicts with another added line.
**Cons:**
- **Human Antagonistic:** Humans cannot easily read, re-order, or edit JSONL manually. They *must* use a UI or CLI. This violates the "degrade gracefully" principle if the CLI is unavailable.

## 4. Implementation Language & Tooling

If we assume a CLI or an agent tool is required to manage the data (enforce schemas, query the graph, etc.), the choice of language matters for distribution and integration.

### Option 1: Rust (The Current Path)
- **Why?** Blazing fast, type-safe, distributes as a single static binary. Excellent for a tool that runs on every `git commit` or is executed hundreds of times per minute by an agent.
- **Drawback:** Higher barrier to entry for casual contributors to tweak the logic.

### Option 2: TypeScript / Node (With `npx` or `uv`/Python equivalent)
- **Why?** The AI ecosystem is heavily skewed towards TS/Python. Building Model Context Protocol (MCP) servers locally is easiest in TypeScript.
- **Distribution:** Can be executed via `npx pebble-cli` without explicit installation.
- **Drawback:** Slower startup time than Rust (Node boot time).

### Option 3: Go (Golang)
- **Why?** The sweet spot. Fast startup time like Rust, single binary distribution, but with a simpler concurrency model and arguably faster development velocity for lightweight CLI tools and JSON manipulation.

## 5. Re-imagining the Workflow

If we adopt **Avenue A (Markdown + YAML)** alongside a **Go or Rust CLI**, the workflow transforms into a deeply collaborative human-agent experience:

1. **Creating a task:** A human just creates `new-feature.md` and writes their thoughts. The CLI/Agent detects it, generates an ID, and populates the frontmatter.
2. **Reviewing State:** The agent can run `pebble list --json` to get the graph, but the human can just read the `.pebble/` folder or view the Kanban board dynamically rendered by an MCP extension in VSCode.
3. **Closing Tasks:** The agent finishes a PR, adds `Closes proj-0kq` to the commit. A CI bot or pre-commit hook modifies the YAML frontmatter to `status: closed`.

## 6. Recommendations & Discussion Points

To retain the dual-audience goal while stripping away enterprise complexity, we should consider:

1. **Transitioning to Markdown + YAML (Avenue A)** for storage. The readability benefits for humans and agents heavily outweigh the parsing overhead, especially for small-to-medium single-repo projects.
2. **Abstracting the Graph:** The CLI's main job becomes parsing the metadata and answering structural questions: "What is blocking X?" or "What tasks are ready to work on?"
3. **Lean into MCP:** Instead of a complex CLI with dozens of flags, provide a brilliant, read-only Markdown structure, a minimal CLI for mutations, and an MCP server that serves the graph dynamically to IDEs (for humans) and Agents (for coding).

---
*Open for feedback: Which storage paradigm feels most aligned with the long-term vision of agent-human collaboration in this repository?*
