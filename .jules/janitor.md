## 2025-03-10 - Extracted duplicate canonical content check into is_canonical
**Duplication:** The logic to read a task file from disk and compare it against its canonical `get_content_for_disk` representation was duplicated in `commands_diagnostics.rs` and `commands_fix.rs`.
**Learning:** The duplication occurred because `TaskNode` provided a way to generate the canonical disk payload, but lacked a higher-level abstraction to verify if the physical file matched this state.
**Abstraction:** Introduced the `is_canonical(&self) -> Result<bool>` method on `TaskNode` in `models.rs`.