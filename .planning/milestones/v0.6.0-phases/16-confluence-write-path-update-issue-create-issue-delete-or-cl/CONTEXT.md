# Phase 16 CONTEXT — Confluence Write Path

> Status: queued (session 6, 2026-04-14). Not yet planned or executed.
> Follows Phase 15 (v0.5.0 shipped). Milestone: v0.6.0.

## Phase identity

**Name:** Confluence write path — `update_issue`, `create_issue`, `delete_or_close` on `ConfluenceBackend` plus `atlas_doc_format` ↔ Markdown round-trip.

**Scope tag:** v0.6.0 (feature scope — first write-capable Confluence backend).

**Addresses:** HANDOFF.md "Known open gaps" — `create_issue` / `update_issue` / `delete_or_close` on `ConfluenceReadOnlyBackend` returns `NotSupported`. Phase 14 unblocked this: the FUSE write path and git-remote helper now route through `IssueBackend` trait, so implementing the three write methods on `ConfluenceBackend` is all that's needed.

## Goal (one paragraph)

Implement the Confluence write path so that agents can create, update, and delete Confluence pages via the FUSE mount or `git push`. This requires implementing `IssueBackend::create_issue`, `update_issue`, and `delete_or_close` on `ConfluenceBackend` against the Confluence Cloud REST v2 API (`POST /wiki/api/v2/pages`, `PUT /wiki/api/v2/pages/{id}`, `DELETE /wiki/api/v2/pages/{id}`). Also requires an `atlas_doc_format` ↔ Markdown converter so that Markdown bodies in `.md` files round-trip through Confluence's ADF storage format without data loss for headings, paragraphs, code blocks, and lists.

## Source design context (migrated from HANDOFF.md)

### From session-5 §Cluster A

> **Cluster A — Confluence writes.** Not started. Phase 14 unblocked the FUSE write path; `ConfluenceBackend` now just needs `create_issue`/`update_issue`/`delete_or_close` + an `atlas_doc_format` ↔ Markdown converter. Highest user-visible ROI left. Realistically multi-session; session-6 could scope-tight to just `update_issue` (the most common op) + a minimal storage-format↔markdown renderer. Ships v0.6.0.

### From CHANGELOG [v0.4.1] §Changed (trait context)

Phase 14 established the `IssueBackend` trait routing:
- FUSE writes: `crates/reposix-fuse/src/fs.rs` → `IssueBackend::update_issue` / `create_issue` / `delete_or_close`
- git-remote helper: `crates/reposix-remote/src/main.rs` → same trait methods
- Deleted: `crates/reposix-fuse/src/fetch.rs`, `crates/reposix-fuse/tests/write.rs`, `crates/reposix-remote/src/client.rs` (~1,068 LoC)

`ConfluenceReadOnlyBackend::update_issue` currently returns `Err(Error::NotSupported)` — this phase changes it to a real implementation.

## Design questions to resolve (run /gsd-discuss-phase 16 before planning)

1. **Scope of write methods:** Implement all three (`create`, `update`, `delete`) or just `update` (the most common FUSE operation)?
2. **`atlas_doc_format` round-trip fidelity:** What Markdown constructs must survive? Minimum: headings, paragraphs, code blocks, inline code. Tables and footnotes are nice-to-have.
3. **Conflict detection:** Confluence uses `version.number` for optimistic locking. The FUSE write path should read the current version and pass it in `PUT /wiki/api/v2/pages/{id}` — otherwise concurrent edits silently stomp. Does this mirror the sim's `If-Match` ETag pattern?
4. **Rate-limit handling:** Confluence REST v2 has a 429 rate-limit. The write path should honor the `Retry-After` header.
5. **Space key vs page ID:** `create_issue` needs to know the parent space key AND optionally the parent page ID. How does this flow through the `IssueBackend` trait signature?

## Locked decisions (from Phase 14)

- **LD-16-01:** Write path routes through `IssueBackend` trait, not direct REST calls.
- **LD-16-02:** `Tainted<T>` wraps all bytes from Confluence responses; `sanitize()` strips server-controlled frontmatter fields (`id`, `created_at`, `version`, `updated_at`) on the inbound path.
- **LD-16-03:** Every write call gets an audit log row (sim WAL pattern).

## Non-goals / scope boundaries

- Do NOT implement Confluence comments write path (that's Phase 23 OP-9a, read-only).
- Do NOT implement attachment upload.
- Do NOT implement the subprocess/JSON-RPC connector ABI (Phase 12, user-gated).

## Canonical refs

- `crates/reposix-confluence/src/lib.rs` — `ConfluenceReadOnlyBackend`; write stubs return `Err(Error::NotSupported)`.
- `crates/reposix-core/src/lib.rs` — `IssueBackend` trait with `create_issue`, `update_issue`, `delete_or_close` signatures.
- `crates/reposix-fuse/src/fs.rs` — FUSE write dispatch through `IssueBackend`.
- `docs/decisions/002-confluence-page-mapping.md` — ADR-002 field mapping (ADF Option A).
- HANDOFF.md §"Known open gaps" (now trimmed; content migrated here).
