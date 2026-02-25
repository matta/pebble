---
name: Rust Expect Message
description: Ensure expect messages are clear and descriptive, and are phrased as error messages for failed preconditions, usually with "should" or "must".
---
If you’re having trouble remembering how to phrase expect-as-precondition style error messages remember to focus on the word “should” as in “env variable should be set by blah” or “the given binary should be available and executable by the current user”.

# Good

```rust
let path = std::env::var("IMPORTANT_PATH")
    .expect("env variable `IMPORTANT_PATH` should be set");
```

# Bad

```rust
let path = std::env::var("IMPORTANT_PATH")
    .expect("env variable `IMPORTANT_PATH` is not set");
```
