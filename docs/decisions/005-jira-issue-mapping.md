# ADR-005: JIRA Issue Mapping

**Date:** 2026-04-16
**Status:** Accepted
**Phase:** 28 (reposix-jira read-only adapter)

## Context

The `reposix-jira` crate maps JIRA Cloud REST v3 response objects to the canonical
`reposix_core::Issue` type. Several JIRA concepts don't map cleanly to the Issue schema —
this ADR records the decisions made for each.

## Decisions

### 1. ID vs Key

JIRA issues have two identifiers: a numeric `id` (e.g. `10001`) and a human-readable
`key` (e.g. `PROJ-42`). The numeric `id` is used as the canonical `IssueId` because:
- It is stable (keys can be renamed by admins)
- It is numeric (matches `IssueId(u64)`)
- It can be used directly in REST API paths (`GET /rest/api/3/issue/10001`)

The `key` is preserved in `Issue.extensions["jira_key"]` as a string value for consumers
that need the human-readable form.

**Consequence:** FUSE filenames use the numeric ID (e.g. `10001.md`), not the key.
Phase 29+ may add a symlink from `PROJ-42.md → 10001.md`.

### 2. Status + Resolution Mapping

JIRA uses a two-level status model: `statusCategory.key` (coarse, stable API) and
`status.name` (display name, tenant-customizable). The canonical `IssueStatus` mapping is:

| `statusCategory.key` | `status.name` | `resolution.name` | → `IssueStatus` |
|---------------------|---------------|-------------------|-----------------|
| `"new"` | any | any | `Open` |
| `"indeterminate"` | does not contain "review" | any | `InProgress` |
| `"indeterminate"` | contains "review" | any | `InReview` |
| `"done"` | any | does not match WontFix | `Done` |
| `"done"` | any | contains "won't"/"wont"/"not a bug"/"duplicate"/"cannot reproduce" | `WontFix` |
| unknown | any | any | `Open` (safe fallback) |

Resolution override takes priority over `statusCategory` for WontFix because JIRA can
mark an issue "Done" with resolution "Won't Fix" — the resolution is more informative.
The `status.name` for InReview uses a substring match (case-insensitive) to handle
tenant-customized names like "Code Review", "In Review", "Peer Review".

### 3. Version Synthesis

JIRA Cloud REST v3 does not expose an ETag or sequential version counter on issue lists.
`Issue.version` is synthesized from `fields.updated` as Unix milliseconds (`u64`):

```rust
version = fields.updated.timestamp_millis() as u64
```

This is a monotonic proxy — two issues updated at the same millisecond get the same
"version", but in practice JIRA timestamps have millisecond resolution and the primary
use of `version` (ETag-based write conflict detection) is disabled for JIRA Phase 28
(`StrongVersioning: false`). Phase 29 will handle concurrency via JIRA's Transitions API
rather than optimistic locking.

### 4. ADF Description Stripping

JIRA `fields.description` is an ADF (Atlassian Document Format) JSON document, or `null`.
The Phase 28 adapter extracts **plain text** only (no Markdown conversion) by walking the
content tree recursively:
- `text` nodes: emit `text` field value
- `hardBreak` nodes: emit `\n`
- `paragraph`, `doc` nodes: recurse into `content[]` children, append `\n`
- `codeBlock` nodes: recurse into `content[]` children, append `\n`
- Unknown/future node types: recurse into `content[]` children silently

`null` description produces an empty string. This is intentionally simple — the
ADF→Markdown conversion used by `reposix-confluence` is not reused here because JIRA
issue bodies are most commonly read by agents that want plain text, not Markdown.
Phase 29+ may add a `--format markdown` flag.

### 5. Attachments and Comments

JIRA attachments (on `fields.attachment[]`) and comments (on `fields.comment.comments[]`)
are deliberately excluded from Phase 28. Rationale:
- The 12 required wiremock tests already validate the core read path.
- Attachments require a separate download request per file.
- Comments add a second REST endpoint (`GET /rest/api/3/issue/{id}/comment`).
- Parity with Phase 23 (Confluence comments) and Phase 24 (Confluence attachments)
  is the right sequencing — JIRA attachments/comments will land in a future phase.

## Consequences

- FUSE consumers see numeric IDs in filenames; `jira_key` in frontmatter `extensions`.
- `StrongVersioning: false` — JIRA writes in Phase 29 will not use ETag conflict detection.
- Plain-text bodies are suitable for LLM agent consumption (Phase 28's primary use case).
- Comment and attachment gaps are tracked in the backlog.
