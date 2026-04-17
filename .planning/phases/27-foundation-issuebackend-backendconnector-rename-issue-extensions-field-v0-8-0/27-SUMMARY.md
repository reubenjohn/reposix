---
phase: 27
title: "Foundation — IssueBackend → BackendConnector rename + Issue.extensions field (v0.8.0)"
status: complete
completed_at: "2026-04-16"
plans: [27-01, 27-02, 27-03]
---

# Phase 27 Summary — BackendConnector rename + Issue.extensions (v0.8.0)

## Phase outcome

Phase 27 shipped all three plans. The workspace is at v0.8.0 with:
1. `IssueBackend` renamed to `BackendConnector` across the entire codebase (RENAME-01).
2. `Issue.extensions: BTreeMap<String, serde_yaml::Value>` added with full round-trip support (EXT-01).
3. ADR-004 documenting the rename rationale.
4. CHANGELOG promoted to `[v0.8.0]`.

## Plans shipped

| Plan | Title | Key output |
|------|-------|------------|
| 27-01 | Rename IssueBackend → BackendConnector in reposix-core | `crates/reposix-core/src/backend.rs` trait renamed; all in-crate impls updated |
| 27-02 | Update all external impls and call-sites | All 6 consumer crates updated; workspace tests green |
| 27-03 | Issue.extensions + ADR-004 + v0.8.0 + CHANGELOG | extensions field, ADR-004, version bump, CHANGELOG |

## Requirements closed

- **RENAME-01**: `BackendConnector` trait name throughout; zero `IssueBackend` references in source
- **EXT-01**: `Issue.extensions` field with serde default/skip_serializing_if + frontmatter roundtrip + 3 unit tests

## Test counts

All workspace test results ok after Phase 27. The pre-existing reqwest
`E0463` failure in 3 doc-test binaries is unrelated (confirmed by baseline).

## Next phase

Phase 28 — JIRA Cloud read-only adapter (`reposix-jira`), which uses
`Issue.extensions` to store `jira_key`, `issue_type`, `priority`, etc.
