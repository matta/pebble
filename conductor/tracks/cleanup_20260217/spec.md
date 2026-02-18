# Specification: Renaming and Cleanup

## Goal
Complete the transition from `beads` to `pebble` by renaming project-specific directories, branches, and code references. This track focuses on establishing the project's new identity while maintaining compatibility with legacy "bead" terminology where it refers to external compatibility or specific legacy configurations.

## User Persona
- **Developer**: A user of the Pebble CLI who expects consistent naming and a clear distinction between the current tool (Pebble) and the legacy tool (Beads).

## Core Requirements
1. **Directory Renaming**: The project-local configuration directory must be renamed from `.beads` to `.pebble`.
2. **Branch Management**: The synchronization branch must be updated from `beads-sync` to `pebble-sync`.
3. **Worktree Management**: The internal Git worktree path must be updated from `.git/beads-worktrees` to `.git/pebble-worktrees`.
4. **Tooling & Xtask**: The `check-beads` xtask must remain named `check-beads` as it verifies the absence of forbidden words like "bead" or "beads".
5. **Code & Documentation**: Update internal string references, error messages, and documentation (e.g., `AGENTS.md`) to reflect the new project name.

## Constraints & Legacy Preservation
- **Preserve `.bead-whitelist` and `check-beads`**: These must NOT be renamed, as they represent the mechanism for enforcing minimal mentions of the legacy tool.
- **Selective Renaming**: Do not blindly rename "bead" or "beads" when it refers specifically to the original tool or its legacy components.
- **TDD Mandatory**: All changes must be driven by failing tests first.

## Acceptance Criteria
- [ ] Running `pebble config get sync.branch` returns `pebble-sync` (after update).
- [ ] Pebble successfully locates and syncs data from `.git/pebble-worktrees`.
- [ ] Xtask `check-pebble` runs and correctly references the existing `.bead-whitelist`.
- [ ] Documentation and CLI output consistently use the name "Pebble".
- [ ] Test coverage for affected modules remains above 80%.
