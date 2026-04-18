---
status: partial
phase: 23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme
source: [23-VERIFICATION.md]
started: 2026-04-16T00:00:00Z
updated: 2026-04-16T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Live FUSE Mount `.comments/` End-to-End
expected: Run a live FUSE mount against a Confluence tenant. `ls pages/<id>.comments/` lists comment files named `<cid>.md`. `cat pages/<id>.comments/<cid>.md` returns YAML-frontmatter Markdown with `kind:`, `resolved:`, `author:`, `created_at:` fields followed by the comment body. `ls pages/` does NOT include any `.comments` entries (DoS amplifier prevention — Pitfall 2).
result: [pending]

### 2. ANSI Escape Sequences in Space Names (accepted risk T-23-02-03)
expected: `reposix spaces --backend confluence` against a tenant with an ANSI escape sequence in a space name prints the raw bytes without crashing. Accepted risk — noted in threat model; no sanitization in v0.7.0.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
