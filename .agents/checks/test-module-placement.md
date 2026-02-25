---
name: Test Module Placement
description: Tests must live in the module that owns the code under test
---
### Test Module Placement

Unit tests must reside in the same module that defines the code they exercise.

#### Rule
A `#[cfg(test)] mod tests` block (or a `#[path]`-linked test file) must only test items defined in its parent module. If a test imports a function from a *different* module, that test belongs in the other module instead.

#### Why
Misplaced tests make it look like the host module has coverage when it doesn't,
and they obscure what the tested module actually guarantees.

#### What to check
- Every `use super::…` or `use crate::…` import inside a test module should resolve to the module the test block lives in.
- Cross-module test imports are a sign the test needs to move.
