## 2024-03-11 - [Refactored duplicate check for canonical task nodes]
**Duplication:** Checking whether a task file's disk contents match its canonical format was duplicated across `commands_fix.rs` and `commands_diagnostics.rs`. Both places manually read from the file system, got the canonical string representation, and string-compared them.
**Learning:** The check involved knowing both about the disk state and the task's formatting rules, causing business logic to leak into command runners instead of being encapsulated in the core data model.
**Abstraction:** Created `is_canonical(&self) -> Result<bool>` on `TaskNode` in `models.rs`, encapsulating both the disk read and the comparison within the model.
