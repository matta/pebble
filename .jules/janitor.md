## 2024-03-16 - [Centralize Canonical Check in TaskNode]
**Duplication:** The logic to check if a task file's disk content matches its canonical format (reading from disk, generating canonical content, and comparing) was duplicated across `commands_diagnostics.rs` and `commands_fix.rs`.
**Learning:** This reveals a missing domain concept representing "is this task in its canonical state?", which belongs on the `TaskNode` model itself rather than scattered in CLI command logic.
**Abstraction:** Introduced `is_canonical(&self) -> Result<bool>` on `TaskNode` to encapsulate this logic cleanly.
