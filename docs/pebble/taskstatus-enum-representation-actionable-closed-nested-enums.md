---
id: "pebl-MqmhmZ"
title: "TaskStatus enum representation: actionable/closed + nested enums"
status: "todo"
created_at: "2026-02-22T23:43:31.512707+00:00"
needs: []
tags: ["design"]
---
## Context
While implementing blocking/readiness logic we found “terminal/non-terminal” wording confusing. The match expressions also felt scattered. We introduced helper methods (`is_actionable`, `is_closed`) but the model could go further to make invalid states unrepresentable.

## Goals
- Replace “terminal/non-terminal” terminology with clearer language: **actionable** vs **closed**.
- Make it harder to construct invalid status groupings.
- Preserve the TOML/JSON wire format (`"todo"`, `"in_progress"`, `"done"`, `"canceled"`) for compatibility.

## Proposed Representation
Introduce nested enums:

```rust
pub enum TaskStatus {
    Live(LiveStatus),
    Closed(ClosedStatus),
}

pub enum LiveStatus {
    Todo,
    InProgress,
}

pub enum ClosedStatus {
    Done,
    Canceled,
}
```

### Convenience Methods
- `TaskStatus::is_actionable()` => `matches!(self, TaskStatus::Live(_))`
- `TaskStatus::is_closed()` => `matches!(self, TaskStatus::Closed(_))`
- `TaskStatus::as_live()` / `as_closed()` helpers for downstream logic.

## Serialization / Deserialization
Maintain the existing string values for TOML/JSON:
- `"todo"` -> `TaskStatus::Live(LiveStatus::Todo)`
- `"in_progress"` -> `TaskStatus::Live(LiveStatus::InProgress)`
- `"done"` -> `TaskStatus::Closed(ClosedStatus::Done)`
- `"canceled"` -> `TaskStatus::Closed(ClosedStatus::Canceled)`

Implementation options:
- Custom `Serialize` / `Deserialize` on `TaskStatus` to map the string values.
- Keep `LiveStatus` / `ClosedStatus` as internal-only, or expose for clarity.

## CLI Parsing
- Map string values to the nested representation via a dedicated parser (or `FromStr`).
- Keep error messages identical to today’s behavior.

## Code Touchpoints
- `models.rs`: redefine `TaskStatus` and add nested enums + helper methods.
- `graph.rs`: replace actionability/closed checks with helpers or pattern matches.
- `commands.rs`: same for `blocking` filtering.
- `tests`: update serialization/deserialization tests and add tests for helpers.

## Compatibility Notes
- No changes to on-disk format or CLI interface.
- Only internal representation changes.
- Specs in `docs/schema.md` should remain aligned with the wire format string values.

## Open Questions
- Should `LiveStatus` / `ClosedStatus` be public or private to the crate?
- Do we want additional helpers (`is_todo`, `is_in_progress`, etc.)?
- Should `TaskStatus` expose a `kind()` for logging/debugging?