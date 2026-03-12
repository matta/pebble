## 2026-03-12 - Centralize TaskNode Canonicalization Logic

**Duplication:** Found identical `fs::read_to_string` and `get_content_for_disk()` comparison logic in both `commands_diagnostics.rs` (to report uncanonical files) and `commands_fix.rs` (to rewrite them).
**Learning:** Checking if a file is canonically formatted requires reading the disk and re-serializing the node, which is a core domain behavior of the model, not a command-level concern.
**Abstraction:** Introduced `is_canonical(&self) -> Result<bool>` directly onto `TaskNode` in `models.rs` to provide a single, consistent way to verify formatting adherence across all commands.