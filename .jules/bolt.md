## 2024-05-23 - [Zero-Allocation JSONL Scanning]
**Learning:** `BufRead::lines()` is convenient but allocates a new `String` for every line. For scanning large files where most lines are discarded, using `BufRead::read_line` with a reusable buffer combined with zero-copy deserialization (`&str` in struct) significantly reduces memory pressure and allocation overhead.
**Action:** Always prefer `read_line` with a reusable buffer for file scanning loops in performance-critical paths, especially when parsing small parts of the line.
