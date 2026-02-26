# Decisions

This file documents the architectural decisions that guide the Pebble project.

## Storage Strategy

**Decision:** Pebble uses **per-issue Markdown files** with YAML frontmatter as the single source of truth for task storage.
- **Location:** Tasks are stored in a configurable directory (default: `docs/pebble/`).
- **File Format:** Markdown files (`.md`) with YAML frontmatter delimited by `---`.
- **Invariants:** The `id` field in the frontmatter is the canonical identifier. Filenames are advisory and derived from the task title.
- **Git Integration:** Changes are committed directly to the repository. There is no hidden database or worktree.

## Merge Strategy

**Decision:** Concurrent edits are handled via **Git merges** at the file level.
- **Conflict Resolution:** Since tasks are stored in separate files, merge conflicts are rare. When they occur (e.g., two people edit the same task simultaneously), standard Git merge tools are used to resolve them.
- **Deterministic Merge Work:** Future work on deterministic merging (e.g., custom merge drivers) will build upon this file-based structure.

## MVP Command Surface

**Decision:** The Minimum Viable Product (MVP) command surface for daily use includes:
- **`list`**: List tasks with filters (`--status`, `--tag`, `--priority`, `--need`, `--is-ready`).
- **`next`**: Show the next task to work on based on dynamic scoring.
- **`search`**: Search tasks by title and body content.
- **`add`**: Create new tasks.
- **`update`**: Update existing tasks (status, priority, body, dependencies).
- **`archive`**: Archive completed tasks.
- **`check`**: Validate the task graph for consistency (cycles, dangling pointers).
- **`config`**: Manage configuration.

## CLI Contract

**Decision:** The CLI adheres to a strict contract for machine readability:
- **`--json`**: Supported as a global flag across all commands to output structured JSON.
- **`--help-json`**: Supported as a global flag to output the CLI schema in JSON format.
- **Stdout/Stderr Separation**: All command output goes to `stdout`, while logs and errors go to `stderr`.
