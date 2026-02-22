# Product Guide: Pebble

## Overview
Pebble is a streamlined, Markdown-native CLI task tracker built upon a "Permissive Writes, Strict Evaluation" design philosophy. It focuses on a minimal, high-performance core for developers who value absolute control over their data, native integration with standard Git workflows, and an architecture that AI coding agents can easily understand.

## Target Audience
- **Git-Centric Developers**: Users who want their task management to live alongside their code, leveraging their standard Git commit/push routines without separate tracking tools.
- **Minimalists**: Those who prefer a highly focused toolset that degrades gracefully to standard text editors.
- **AI Agents**: Autonomous agents that benefit from predictable file-system level APIs, readable formats (Markdown), and a CLI interface that accepts raw data without panicking.

## Core Goals
1. **Markdown-Native Storage**: All tasks are stored as discrete `.md` files with TOML frontmatter.
 
2. **Permissive Writes, Strict Evaluation**: The storage layer acts purely as a raw directed graph. It never fails or panics if you author a cycle or reference a missing ID; instead, validation is deferred to the read-path, which strictly evaluates structural issues as "not ready".
3. **No Hidden State**: Zero daemon processes, zero hidden `.git` worktrees, and no opaque databases.
4. **Developer-First Design**: Built via Test-Driven Development (TDD), providing a strictly defined CLI and JSON output interface.
5. **Dynamic Scoring**: To prevent starvation, tasks are dynamically sorted using a blocking count algorithm, ensuring critical bottlenecks naturally surface to the top of the `pebble next` queue.

## Constraints
- **Configuration Constraints**: Task directories are resolved strictly relative to the nearest `.pebble/` configuration folder.
- **Single Structural Edge**: Task relationships are entirely flattened into a single `deps` link. Temporality and hierarchy share the same axis.
