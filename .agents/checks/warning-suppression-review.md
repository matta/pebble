---
name: Warning Suppression Review
description: Require review and justification for new warning suppressions
---
New Rust warning suppressions (e.g., `#[allow(...)]`, `#![allow(...)]`) must be flagged in code review.
Prefer to include a short justification comment or a trailing TODO explaining why the suppression is needed.
