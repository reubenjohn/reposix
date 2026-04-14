# Phase 24 CONTEXT — Confluence whiteboards, live docs, attachments, and folders (OP-9b)

> Status: scoped in session 5, 2026-04-14.
> Author: planning agent, session 6 prep.
> Follows Phase 23 (OP-9a — comments). User pain order: whiteboards first, then attachments, then live docs, then folders.

## Phase identity

**Name:** Confluence whiteboards, live docs, attachments, and folders (OP-9b).

**Scope tag:** v0.7.0 (read-only; no write path for any of these content types in this phase).

**Addresses:** OP-9 (whiteboards + live docs + attachments + folders sub-items) from HANDOFF.md.

**Depends on:** Phase 23 (OP-9a — comments) establishes the multi-content-type dispatch pattern this phase extends.

## Goal (one paragraph)

Confluence Cloud exposes several content types beyond pages that are invisible in the current FUSE mount: whiteboards (current-state architecture diagrams), live docs (real-time collab format), attachments (binary files on pages), and folders (new Confluence org concept). This phase surfaces all four as POSIX paths — `whiteboards/<id>.json`, `livedocs/<id>.md`, `pages/<id>.attachments/<filename>` (binary passthrough), and a `folders/` tree — giving agents grep-based access to content that today requires navigating the Atlassian web UI or parsing JSON REST responses. Whiteboards are prioritized first because most Atlassian-using agents need architecture diagrams more urgently than any other non-page content type.

## Source design context

From HANDOFF.md §OP-9 (whiteboards + live docs + attachments + folders bullets, verbatim):

- **Whiteboards.** `GET /wiki/api/v2/whiteboards` returns board metadata; the body is a custom JSON graph format. Expose as `whiteboards/<id>.json` initially (raw), later as `whiteboards/<id>.svg` once we render the graph. Most Atlassian-using agents need this more than pages; whiteboards are where the current-state architecture lives.
- **Live docs.** Confluence's newer real-time collab doc format. v2 API coverage is partial; some endpoints live under `/wiki/api/v2/custom-content/` with a type discriminator. Expose as `livedocs/<id>.md` using the same storage-format path as pages, with a "last-synced-at" frontmatter field since live docs are by nature a moving target.
- **Attachments.** `GET /wiki/api/v2/pages/{id}/attachments`. Expose as `pages/<id>.attachments/<filename>` — binary passthrough. `grep -l "passw" pages/**/attachments/*` becomes a real security-audit tool.
- **Folders** (Confluence's new-ish org concept, distinct from page parents). These already render via page hierarchy if we do OP-1, but there's a dedicated `/folders` endpoint the user may want as a separate tree.

From HANDOFF.md §OP-9 (ordering note):

> Order by user pain: whiteboards first (most underserved), then comments (agent workflow multiplier), then attachments (security-audit use-case), then live docs (UI churn risk), then folders + multi-space (polish).

## Design questions

1. **Whiteboard body format.** The raw JSON graph format from `GET /wiki/api/v2/whiteboards/{id}` is not human-readable. For v0.7.0 ship as `.json` (raw passthrough). An SVG render is a future phase. Define: does the `.json` file include the full board body or only metadata? Board body may be very large.
2. **Live docs API stability.** The `/wiki/api/v2/custom-content/` endpoint with type discriminator is partial in v2. Verify which fields are stable before committing to a frontmatter schema. Add a `last-synced-at` frontmatter field (ISO 8601 UTC) since live docs change under the agent between reads.
3. **Attachment binary passthrough.** Attachments can be large binaries (PDFs, images). FUSE `read` must stream them without loading the full binary into the FUSE process heap. Use `reqwest` streaming response + chunked FUSE `read` callbacks.
4. **Folders vs page hierarchy.** If Phase 13 (parentId tree) already renders the folder-equivalent structure under `tree/`, the `/folders` endpoint may be redundant. Audit the Confluence folders API to see whether it exposes a distinct organizational concept before implementing a separate `folders/` FUSE tree.
5. **Inode allocation at scale.** For a space with 500 pages × N attachments each, inode count grows fast. Define the inode allocation strategy for `.attachments/` dirs and their file entries before implementation.

## Implementation order within this phase

Per user pain ordering (whiteboards most underserved → folders least urgent):

1. `whiteboards/<id>.json` — raw whiteboard metadata + body.
2. `pages/<id>.attachments/<filename>` — binary passthrough (security-audit use-case).
3. `livedocs/<id>.md` — live doc body with `last-synced-at` frontmatter.
4. `folders/` tree — only if distinct from the existing `tree/` parentId hierarchy.

## Canonical refs

- `crates/reposix-confluence/src/lib.rs` — Confluence adapter; add `list_whiteboards()`, `list_attachments(page_id)`, `list_live_docs()` methods.
- `crates/reposix-fuse/src/fs.rs` — add dispatch for new top-level dirs (`whiteboards/`, `livedocs/`) and per-page subdirs (`.attachments/`).
- Confluence v2 API: `GET /wiki/api/v2/whiteboards`, `GET /wiki/api/v2/pages/{id}/attachments`, `GET /wiki/api/v2/custom-content/` (live docs), `/folders`.
- `.planning/phases/23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme/CONTEXT.md` — sibling phase; same multi-content-type dispatch pattern.
- `HANDOFF.md §OP-9` — original design capture.
