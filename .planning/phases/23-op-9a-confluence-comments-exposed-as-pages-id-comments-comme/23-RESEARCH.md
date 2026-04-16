# Phase 23: OP-9a — Confluence Comments as `.comments/` FUSE Subdirectory - Research

**Researched:** 2026-04-16
**Domain:** Confluence REST v2 comments API + FUSE multi-level directory synthesis
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Read-only in this phase; no comment write path.
- Threaded replies: **flatten** all comments into the same `.comments/` dir — no subdirectory per thread. `parent_comment_id` exposed in frontmatter for agent traversal.
- Resolved comments: **include by default** with `resolved: true` in frontmatter.
- `reposix spaces --backend confluence` subcommand: read-only listing. Output: table of space key + name + URL.
- Pagination: use same cursor-based pattern as `list_issues`; apply 500-cap + WARN from OP-7 (Phase 21 HARD-02).

### Claude's Discretion
- Whether to add `CommentSnapshot` as a new in-memory struct (parallel to `LabelSnapshot`) vs. embedding comment state directly on `ReposixFs`.
- Inode allocation strategy for `.comments/` dirs: sequential offset (like `LABELS_DIR_INO_BASE`) vs. hash-based.
- Whether `list_comments` is a new method only on `ConfluenceBackend` (not on `IssueBackend` trait) — CONTEXT implies this; it is not a generic concern.

### Deferred Ideas (OUT OF SCOPE)
- `--include-resolved` / `--exclude-resolved` flags — always include resolved.
- Nested subdirectory per thread — flat only.
- `--project all` or multi-space mount.
- Write path for comments.
- `mount/recent/`, `mount/pulls/`, offline mode.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CONF-01 | `cat mount/pages/<id>.comments/<comment-id>.md` returns comment body in Markdown frontmatter format | Confluence v2 `GET /wiki/api/v2/pages/{id}/inline-comments` + `footer-comments` return comment body in `body.atlas_doc_format` or `body.storage`; translate via existing `adf_to_markdown` path |
| CONF-02 | `ls mount/pages/<id>.comments/` lists all inline + footer comments for that page | FUSE `readdir` dispatch on a new `InodeKind::CommentsDir` parent inode that is synthesized per page-inode |
| CONF-03 | Comments are read-only (no write path in this phase) | No FUSE `write`/`create`/`unlink` dispatch needed for comment inodes |
</phase_requirements>

---

## Summary

Phase 23 adds two capabilities: (1) synthesize a `.comments/` pseudo-subdirectory under every page in the Confluence FUSE mount, populated from the Confluence v2 inline-comments and footer-comments endpoints; and (2) add a `reposix spaces --backend confluence` CLI subcommand that lists all readable Confluence spaces.

The Confluence REST v2 comments API is well-shaped for this use: both `GET /wiki/api/v2/pages/{id}/inline-comments` and `GET /wiki/api/v2/pages/{id}/footer-comments` return `results[]` with `id`, `version.authorId`, `version.createdAt`, `status`, `parentCommentId` (inline only), `resolutionStatus` (inline only), and a `body` field accepting `?body-format=atlas_doc_format`. Pagination follows the same `_links.next` cursor pattern as `list_issues` — the existing `parse_next_cursor` helper reuses directly.

For FUSE inode allocation the dominant precedent is the Phase 19 labels overlay: sequential inode offsets from a declared base constant, disjoint from all existing ranges. A `CommentsSnapshot` struct (parallel to `LabelSnapshot`) holds per-page comment lists keyed by page-inode. The FUSE `lookup`/`readdir`/`getattr`/`read` callbacks gain two new `InodeKind` variants: `CommentsDir` (one per page that has a `.comments/` entry) and `CommentFile` (one per comment). The FUSE state for comments is **lazily fetched per-page** on first access rather than bulk-fetched across all pages at `readdir` time — this is a critical distinction from labels, where label data already rides on the page list response. Comment bodies are a separate network round-trip per page, so loading them all at mount-open time would be O(n_pages × 2 HTTP requests) and would violate SG-07 budget.

For the `reposix spaces` subcommand, `GET /wiki/api/v2/spaces` is already partially wired: `ConfluenceBackend` uses `GET /wiki/api/v2/spaces?keys=KEY` in `resolve_space_id`. A new `list_spaces()` method issues the same endpoint without a key filter; the result is rendered as a table of key + name + URL.

**Primary recommendation:** Add `CommentsSnapshot` as a lazy per-page cache in `ReposixFs`; new inode ranges above `LABELS_SYMLINK_INO_BASE`; and a new `list_spaces()` method on `ConfluenceBackend` plus a `Spaces` dispatch arm in `reposix-cli`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Fetch comments from Confluence API | API adapter (`reposix-confluence`) | — | Comments are a backend concern; FUSE calls a trait-like method |
| Store per-page comment list in memory | FUSE daemon (`reposix-fuse`) | — | Same model as `LabelSnapshot`; FUSE owns the in-process cache |
| Render comment file as Markdown frontmatter | FUSE daemon (pure fn) | — | Body conversion uses existing `adf_to_markdown`; rendering is a pure function |
| Synthesize `.comments/` FUSE directories | FUSE daemon | — | FUSE callback dispatch; no backend involvement |
| `reposix spaces` CLI subcommand | CLI (`reposix-cli`) | API adapter | Dispatch arm + a new `list_spaces()` on `ConfluenceBackend` |

---

## Standard Stack

### Core (all already in `Cargo.toml` workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `wiremock` (dev) | 0.6 [VERIFIED: grep Cargo.toml] | Mock HTTP server for unit tests | Already used for all Confluence tests in this crate |
| `reqwest` | workspace | HTTP client | Sealed via `reposix_core::http::client()` — SG-01 |
| `serde_json` | workspace | JSON deserialization of API response | Already used for all Confluence JSON parsing |
| `chrono` | workspace | `DateTime<Utc>` for `created_at` fields | Already used in `ConfPage` / `ConfVersion` |
| `async-trait` | workspace | `IssueBackend` trait impls | No change needed |

No new dependencies required. `adf_to_markdown` for comment body conversion is already in `crates/reposix-confluence/src/adf.rs`.

**Version verification:** All packages are workspace-inherited; no new packages to add. [VERIFIED: crates/reposix-confluence/Cargo.toml]

---

## Architecture Patterns

### System Architecture Diagram

```
reposix-cli (Spaces subcommand)
    │
    └──► ConfluenceBackend::list_spaces()
              │
              └──► GET /wiki/api/v2/spaces        (paginated, same _links.next cursor)
                   ← [{id, key, name, type, _links.webui}]

FUSE mount (pages/<padded-id>.comments/)
    │
    ├── lookup(pages/<id>/.comments)
    │       └──► CommentsSnapshot::ensure_loaded(page_id)
    │                 └──► ConfluenceBackend::list_comments(page_id)
    │                           ├── GET /wiki/api/v2/pages/{id}/inline-comments
    │                           └── GET /wiki/api/v2/pages/{id}/footer-comments
    │                               ← merged, deduplicated Vec<ConfComment>
    │
    ├── readdir(CommentsDir inode)
    │       └──► CommentsSnapshot → Vec<(comment_ino, comment_id, filename)>
    │
    └── read(CommentFile inode)
             └──► CommentsSnapshot → render_comment_file(comment) → Vec<u8>
```

### Recommended Project Structure

No new crates. New/modified files:

```
crates/reposix-confluence/src/
├── lib.rs                   # add list_comments() + list_spaces() methods + new serde structs
crates/reposix-fuse/src/
├── inode.rs                 # add COMMENTS_DIR_INO_BASE + COMMENTS_FILE_INO_BASE constants
├── comments.rs              # NEW: CommentsSnapshot, render_comment_file()
├── fs.rs                    # new InodeKind variants + dispatch arms
crates/reposix-cli/src/
├── main.rs                  # add Spaces subcommand + dispatch arm
├── spaces.rs                # NEW: spaces::run() — table rendering
```

### Pattern 1: Lazy per-page comment fetching

**What:** Comments are NOT fetched at `readdir(Bucket)` time (that would be O(n_pages) requests). They are fetched on first `readdir(CommentsDir)` or `lookup(CommentsDir, name)` for a specific page.

**When to use:** Any synthesized subdirectory whose contents require an extra backend round-trip per parent entity. Contrast with labels (which are free — labels already come in the `list_issues` payload).

**Implementation anchor:**

```rust
// In CommentsSnapshot (new comments.rs)
//
// Keyed by the real-file inode of the parent page (same inode InodeRegistry assigns).
// DashMap so FUSE callbacks can write without holding a write lock on the whole snapshot.
struct CommentsSnapshot {
    // page_ino → (fetched: bool, comments: Vec<CommentEntry>)
    by_page: DashMap<u64, PageComments>,
    // comment_ino → (page_ino, comment index within that page's Vec)
    ino_to_comment: DashMap<u64, (u64, usize)>,
    // Monotonic inode allocator for CommentFile inodes.
    next_comment_ino: AtomicU64,
    // Monotonic inode allocator for CommentsDir inodes.
    next_dir_ino: AtomicU64,
    // page_ino → CommentsDir inode (lazily allocated)
    page_to_dir_ino: DashMap<u64, u64>,
}
```

**Source:** [ASSUMED] — design derived from Phase 19 LabelSnapshot pattern in `crates/reposix-fuse/src/labels.rs`. No existing `CommentsSnapshot` in codebase.

### Pattern 2: `.comments/` directory naming as a filesystem-safe suffix

**What:** The `.comments/` directory sits INSIDE the `pages/` bucket (i.e. at `pages/<padded-id>.comments/`), not alongside it at the mount root. This means its parent inode is a *per-page* directory inode — a new concept the codebase has not needed before.

**Key design decision:** In the existing FUSE layout, `InodeKind::Bucket` IS the only parent of real-file inodes. For `.comments/`, the parent is a `CommentsDir` inode and its "grandparent" is the bucket (`pages/`). The FUSE `lookup` call sequence is:

```
lookup(Bucket, "00000098765.comments") → CommentsDir inode
lookup(CommentsDir, "<comment-id>.md")  → CommentFile inode
```

This requires:
1. The bucket `lookup` arm recognizes the `.comments` suffix pattern.
2. A new `InodeKind::CommentsDir` variant dispatched in all callbacks.
3. A new `InodeKind::CommentFile` variant.

**IMPORTANT:** The `.comments` suffix is unique enough that `validate_issue_filename` (which rejects anything not matching `\d{11}.md`) will correctly reject `.comments`-suffixed names — so the ordering in `lookup(Bucket, name)` must check `.comments` suffix BEFORE calling `validate_issue_filename`.

**Source:** [VERIFIED: crates/reposix-fuse/src/fs.rs lines 1254–1276 — Bucket lookup arm]

### Pattern 3: Inode range allocation (Phase 19 precedent)

**What:** Sequential ranges declared in `inode.rs`, disjoint from all existing ranges. The compile-time assertion test in `inode.rs` must be extended.

**Proposed ranges** (must not overlap existing ranges — current max is `LABELS_SYMLINK_INO_BASE = 0x14_0000_0000`):

```
COMMENTS_DIR_INO_BASE   = 0x18_0000_0000  (one per page, O(n_pages) — max ~500 for cap)
COMMENTS_FILE_INO_BASE  = 0x1C_0000_0000  (one per comment — max ~500*50=25000 practical)
```

Both are above the existing `LABELS_SYMLINK_INO_BASE = 0x14_0000_0000`.

**Source:** [VERIFIED: crates/reposix-fuse/src/inode.rs — full range table]

### Pattern 4: Comment file frontmatter + body format

**What:** Each comment file is a Markdown file with YAML frontmatter, consistent with page files.

**Proposed shape:**

```markdown
---
id: "123456"
page_id: "98765"
author: "5b10ac8d82e05b22cc7d4ef5"
created_at: 2026-01-15T10:30:00Z
updated_at: 2026-01-15T10:30:00Z
resolved: false
parent_comment_id: null
kind: inline
---

Comment body in Markdown here.
```

Fields:
- `id` — Confluence comment id (numeric string)
- `page_id` — parent page id (for cross-reference)
- `author` — `version.authorId` (Atlassian accountId)
- `created_at` / `updated_at` — `version.createdAt` (inline comments have one version; use for both)
- `resolved` — `resolutionStatus == "open"` → `false`; `"closed"` or absent (footer) → `false` unless explicitly closed
- `parent_comment_id` — `parentCommentId` field (inline only), null for footer comments and top-level inline
- `kind` — `"inline"` or `"footer"` (distinguishes source endpoint)

**Source:** [CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-comment/] for field names

### Pattern 5: `list_comments(page_id)` method on `ConfluenceBackend`

**What:** A new async method (not on `IssueBackend` trait — it's Confluence-specific) that fetches and merges inline + footer comments for a page.

```rust
// In ConfluenceBackend impl (NOT on IssueBackend trait)
pub async fn list_comments(&self, page_id: u64) -> Result<Vec<ConfComment>>
```

Implementation:
1. GET `{base}/wiki/api/v2/pages/{page_id}/inline-comments?limit=100&body-format=atlas_doc_format`
2. GET `{base}/wiki/api/v2/pages/{page_id}/footer-comments?limit=100&body-format=atlas_doc_format`
3. Both paginate via `_links.next` using existing `parse_next_cursor`.
4. Apply same 500-item cap + `tracing::warn!` as `list_issues` (HARD-02 compliance).
5. Merge and return — caller can distinguish inline vs footer by `kind` field in `ConfComment`.

**Source:** [CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-comment/] for endpoint paths

### Pattern 6: `reposix spaces` CLI subcommand

**What:** New `Cmd::Spaces` arm in `reposix-cli/src/main.rs` + new `spaces.rs` module. Calls `ConfluenceBackend::list_spaces()`.

`list_spaces()` hits `GET /wiki/api/v2/spaces?limit=250` (max Confluence allows) with the same `standard_headers()` and `_links.next` cursor pagination. Returns `Vec<ConfSpaceSummary>` (new struct: `id`, `key`, `name`, `webui_link`).

CLI output: pretty-printed table:
```
KEY         NAME                  URL
REPOSIX     Reposix Project       https://reuben-john.atlassian.net/wiki/spaces/REPOSIX
TEAM        Team Space            https://reuben-john.atlassian.net/wiki/spaces/TEAM
```

**Source:** [CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-space/]

### Anti-Patterns to Avoid

- **Bulk-fetching comments for all pages at `readdir(Bucket)` time:** This turns every `ls pages/` into O(n_pages × 2) HTTP calls. Comments must be lazy-loaded per-page.
- **Adding `list_comments` to the `IssueBackend` trait:** Comments are Confluence-specific. The FUSE dispatch must downcast or use a separate `ConfluenceExtBackend` trait. Simplest approach: `ReposixFs` stores the `backend: Arc<dyn IssueBackend>` AND a separate `comment_fetcher: Option<Arc<ConfluenceBackend>>`. When `comment_fetcher.is_some()`, the `.comments/` dirs are emitted; when `None` (sim/github), they are silently absent. This avoids trait churn and is consistent with CONTEXT.md's read-only scope decision.
- **Storing comment body text (even hashed) in the audit log:** Comments are user content — same T-16-C-04 rule applies. If an audit row is written for a `list_comments` call, store only the page_id and comment count, never body content.
- **Using `.comments` as a child directory under `pages/` folder:** `.comments` is a NAME inside the `pages/` bucket, not a separate top-level directory. `readdir(Bucket)` must NOT emit `.comments` entries — those only appear as the child of a specific page-inode lookup.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Body conversion for comment body | Custom ADF parser | Existing `adf::adf_to_markdown` in `crates/reposix-confluence/src/adf.rs` | Already handles recursive ADF doc traversal; unknown nodes fallback gracefully |
| Pagination cursor extraction | Custom `_links` parser | `parse_next_cursor(&body_json: &serde_json::Value)` in `lib.rs` | Already handles relative vs absolute URL case, tested |
| Rate limit backoff | Custom retry loop | `ingest_rate_limit` + `await_rate_limit_gate` on `ConfluenceBackend` | Phase 11 implementation already handles 429 + `x-ratelimit-remaining` |
| Inode allocation | Custom hash scheme | Sequential base + offset (same as `LABELS_DIR_INO_BASE` pattern) | Deterministic, collision-free, already validated by `fixed_inodes_are_disjoint_from_dynamic_ranges` test |
| 500-item truncation | New logic | Existing pattern from `list_issues_impl`: loop with counter, `tracing::warn!` at cap | HARD-02 closure — reuse exact same loop shape |

**Key insight:** The entire HTTP client, auth, rate-limit, pagination, and ADF body conversion machinery is already built. The new code is mostly: (a) two new API call paths using existing helpers, (b) new inode range constants, (c) new `CommentsSnapshot` struct mirroring `LabelSnapshot`, and (d) FUSE dispatch arms following the label overlay pattern exactly.

---

## Common Pitfalls

### Pitfall 1: `.comments` lookup in Bucket arm with wrong ordering
**What goes wrong:** `validate_issue_filename("00000098765.comments")` returns `Err` (no `.md` extension). If the Bucket `lookup` arm calls `validate_issue_filename` BEFORE checking for the `.comments` suffix, the lookup returns `ENOENT` and the `.comments` directory never materializes.
**Why it happens:** The existing Bucket lookup arm checks `BUCKET_INDEX_FILENAME` first, then calls `validate_issue_filename` on anything else and immediately returns `ENOENT` on error.
**How to avoid:** In `lookup(Bucket, name)`, add a `.ends_with(".comments")` check BEFORE the `validate_issue_filename` call. Parse the numeric prefix to get the page ID, then return the `CommentsDir` inode.
**Warning signs:** `ls pages/00000098765.comments/` returns `ls: cannot access '...': No such file or directory` even though the page exists.

### Pitfall 2: `.comments/` emitted in `readdir(Bucket)` vs. only via `lookup`
**What goes wrong:** If `readdir(Bucket)` emits `<padded-id>.comments` entries for every page, `ls pages/` will show thousands of extra entries, and any tool doing a recursive `ls` or `grep -r` will trigger comment fetches for every page.
**Why it happens:** It feels natural to emit all children of the bucket in `readdir`.
**How to avoid:** Do NOT emit `.comments` entries from `readdir(Bucket)`. The `.comments/` directory is only visible via explicit `lookup`. Agents will do `ls pages/00000098765.comments/` or `cat pages/00000098765.comments/*.md` — both start with `lookup`, not `readdir(Bucket)`. CONF-02 says "lists all inline + footer comments for that page" not "lists all comment dirs for all pages."
**Warning signs:** `ls pages/` output doubles in length with `.comments` entries.

### Pitfall 3: Forgetting `resolved: true` for resolved inline comments
**What goes wrong:** `resolutionStatus` is only present on inline-comment responses, not footer-comment responses. Treating absence as `resolved: false` is wrong for footer comments (footer comments have no resolution concept) but treating it as `resolved: true` would also be wrong.
**Why it happens:** Mapping two slightly different schemas into one frontmatter format.
**How to avoid:** `resolved: false` for footer comments (no resolution concept); for inline comments, `resolved: resolutionStatus != "open"`. Use `kind: inline` or `kind: footer` frontmatter field to let agents distinguish.
**Warning signs:** All footer comments appear `resolved: false` in the frontmatter (correct) but tests fail if the field is absent.

### Pitfall 4: Comment inode collisions across pages
**What goes wrong:** If `COMMENTS_FILE_INO_BASE` is too close to `COMMENTS_DIR_INO_BASE`, and there are many comments, inode ranges can collide when the filesystem has hundreds of pages each with dozens of comments.
**Why it happens:** Under-sized inode range for comment dirs.
**How to avoid:** Allocate `COMMENTS_DIR_INO_BASE = 0x18_0000_0000` with a 64-bit space of `0x4_0000_0000` (4 billion slots) for dirs — vastly more than 500-page cap. `COMMENTS_FILE_INO_BASE = 0x1C_0000_0000` for files. Extend the `fixed_inodes_are_disjoint_from_dynamic_ranges` test in `inode.rs` with new assertions.
**Warning signs:** `getattr` on a comment file returns data for a `CommentsDir` (or vice versa).

### Pitfall 5: SG-01 — comment body text in log messages
**What goes wrong:** `tracing::warn!(body = %comment.body, ...)` or similar leaks comment body into logs. Comment bodies are tainted content.
**Why it happens:** Debug logging convenience.
**How to avoid:** Only log `page_id`, `comment_id`, `kind` in tracing spans. Never log `body` or `title` of comments. Follow the T-16-C-04 pattern from Phase 16 audit logging.
**Warning signs:** Log output contains comment text.

---

## Code Examples

### Confluence v2 inline-comment response shape (verified)

```json
{
  "results": [
    {
      "id": "123456",
      "status": "current",
      "title": "Re: some section",
      "pageId": "98765",
      "parentCommentId": null,
      "version": {
        "createdAt": "2026-01-15T10:30:00.000Z",
        "number": 1,
        "authorId": "5b10ac8d82e05b22cc7d4ef5"
      },
      "body": {
        "atlas_doc_format": { "value": { "type": "doc", "version": 1, "content": [...] } }
      },
      "resolutionStatus": "open",
      "properties": {
        "inlineMarkerRef": "abc123",
        "inlineOriginalSelection": "some text"
      },
      "_links": { "webui": "/wiki/..." }
    }
  ],
  "_links": { "next": "/wiki/api/v2/pages/98765/inline-comments?cursor=ABC&limit=100" }
}
```
[CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-comment/]

### Footer-comment response shape (verified)

```json
{
  "results": [
    {
      "id": "789012",
      "status": "current",
      "pageId": "98765",
      "version": {
        "createdAt": "2026-02-01T09:00:00.000Z",
        "number": 1,
        "authorId": "5b10ac8d82e05b22cc7d4ef5"
      },
      "body": {
        "atlas_doc_format": { "value": { "type": "doc", "version": 1, "content": [...] } }
      },
      "_links": { "webui": "/wiki/..." }
    }
  ],
  "_links": {}
}
```
Note: no `parentCommentId`, no `resolutionStatus` in footer comments. [CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-comment/]

### Spaces response shape (verified)

```json
{
  "results": [
    {
      "id": "360450",
      "key": "REPOSIX",
      "name": "Reposix Project",
      "type": "global",
      "status": "current",
      "_links": { "webui": "/wiki/spaces/REPOSIX" }
    }
  ],
  "_links": { "next": "/wiki/api/v2/spaces?cursor=XYZ&limit=250" }
}
```
[CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-space/]

### Existing reusable helpers

```rust
// Source: crates/reposix-confluence/src/lib.rs — parse_next_cursor
// Reuse directly for comments pagination — no change needed.
let next_cursor = parse_next_cursor(&body_json);

// Source: crates/reposix-confluence/src/lib.rs — standard_headers
// Reuse for all comment and spaces requests.
let header_owned = self.standard_headers();

// Source: crates/reposix-confluence/src/adf.rs — adf_to_markdown
// Reuse for comment body conversion.
let body_md = adf::adf_to_markdown(&adf_value).unwrap_or_default();

// Source: crates/reposix-fuse/src/labels.rs (LABELS_DIR_INO_BASE pattern)
// Comment dir inode: COMMENTS_DIR_INO_BASE + sequential_counter
let dir_ino = COMMENTS_DIR_INO_BASE + dir_offset;
```

### Bucket lookup arm — adding `.comments` check

```rust
// Source: crates/reposix-fuse/src/fs.rs — lookup, InodeKind::Bucket arm
// This is where the new .comments check goes:
InodeKind::Bucket => {
    if name_str == BUCKET_INDEX_FILENAME { /* existing */ }
    // NEW: check .comments suffix BEFORE validate_issue_filename
    if let Some(page_id_str) = name_str.strip_suffix(".comments") {
        if let Ok(padded_id) = parse_padded_page_id(page_id_str) {
            // return or allocate CommentsDir inode for this page
        }
    }
    if validate_issue_filename(name_str).is_err() { /* existing */ }
    // ...
}
```

### Wiremock test boilerplate for comments (new tests)

```rust
// Pattern mirrors existing Confluence wiremock tests in lib.rs #[cfg(test)]
// Source: crates/reposix-confluence/src/lib.rs — list_resolves_space_key_and_fetches_pages

#[tokio::test]
async fn list_comments_returns_inline_and_footer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/98765/inline-comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                { "id": "111", "pageId": "98765", "parentCommentId": null,
                  "resolutionStatus": "open",
                  "version": {"createdAt": "2026-01-15T10:30:00Z", "number": 1, "authorId": "abc"},
                  "body": {"atlas_doc_format": {"value": {"type":"doc","version":1,"content":[]}}},
                  "_links": {} }
            ],
            "_links": {}
        })))
        .mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/98765/footer-comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                { "id": "222", "pageId": "98765",
                  "version": {"createdAt": "2026-02-01T09:00:00Z", "number": 1, "authorId": "def"},
                  "body": {"atlas_doc_format": {"value": {"type":"doc","version":1,"content":[]}}},
                  "_links": {} }
            ],
            "_links": {}
        })))
        .mount(&server).await;

    let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
    let comments = backend.list_comments(98765).await.expect("comments");
    assert_eq!(comments.len(), 2);
    // ...
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flat `<id>.md` at mount root | Per-backend bucket (`pages/`) + tree overlay | Phase 13 | Comments sit inside `pages/`, not at root |
| No multi-level synth dirs below bucket | Labels overlay (`labels/<label>/`) adds one level | Phase 19 | `.comments/` follows same pattern, one level deeper inside bucket |
| No Confluence comments in FUSE | Phase 23 target state | This phase | New `.comments/` synthesized dirs per page |

**Deprecated/outdated:**
- None relevant to this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `readdir(Bucket)` should NOT emit `.comments` entries (only accessible via explicit `lookup`) | Architecture Patterns §2 | If wrong, `ls pages/` output explodes; agent `grep -r pages/` triggers all comment fetches |
| A2 | Footer comments have no `resolutionStatus` field | Code Examples | If wrong, comment files may show `resolved: true` for non-resolved footer comments |
| A3 | `version.authorId` on comments is the Atlassian accountId string | Standard Stack / Code Examples | If wrong, `author` frontmatter field shows non-meaningful ID |
| A4 | `GET /wiki/api/v2/spaces` paginates via `_links.next` (same as pages) | Architecture Patterns §6 | If wrong, `reposix spaces` silently returns only first page of spaces |

---

## Open Questions

1. **`comment_fetcher` coupling in `ReposixFs`**
   - What we know: `ReposixFs.backend` is `Arc<dyn IssueBackend>`, and `list_comments` is NOT on `IssueBackend`. Phase 19 had no such problem because label data rides on `list_issues`.
   - What's unclear: The cleanest way to give the FUSE callbacks access to `list_comments` without adding it to the trait. Options: (a) Store `Option<Arc<ConfluenceBackend>>` on `ReposixFs` alongside `backend`; (b) add a `BackendFeature::Comments` feature flag and a `list_comments_for_page` trait method that returns `Err` by default; (c) a new `CommentBackend` sub-trait.
   - Recommendation: Option (a) — `Option<Arc<ConfluenceBackend>>` stored on `ReposixFs`, set during `reposix mount --backend confluence`. Simplest, no trait churn. Planner should decide which option fits the overall architecture better.

2. **`.comments` dir not surfaced in `readdir(Bucket)` — discoverability**
   - What we know: Agents doing `ls pages/` will not see `.comments` dirs, but `cat pages/0001.comments/*.md` (via `lookup`) will work.
   - What's unclear: Whether CONF-02 (`ls mount/pages/<id>.comments/`) requires that a plain `ls pages/` also show the `.comments` entry.
   - Recommendation: Planner should confirm: if `ls pages/<id>.comments/` works, CONF-02 is met regardless of whether `ls pages/` shows `.comments` entries. The CONTEXT.md design pattern says "synthesized subdirectory" which suggests explicit path access, not that every `ls pages/` must emit them.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | All | ✓ | 1.94.1 [VERIFIED: cargo --version] | — |
| `wiremock` crate | Test mocks | ✓ | 0.6 [VERIFIED: Cargo.toml] | — |
| Confluence REST v2 API | Integration/live tests | live only via `#[ignore]` | — | wiremock mocks cover all CI paths |

No missing dependencies. All Rust compilation dependencies are already workspace members.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (built-in) + `tokio::test` for async |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p reposix-confluence -p reposix-fuse -p reposix-cli` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONF-01 | `list_comments(page_id)` fetches inline + footer, converts ADF to Markdown | unit (wiremock) | `cargo test -p reposix-confluence list_comments` | ❌ Wave 0 |
| CONF-01 | render_comment_file produces valid YAML frontmatter + body | unit (pure) | `cargo test -p reposix-fuse render_comment_file` | ❌ Wave 0 |
| CONF-02 | `lookup(Bucket, "<id>.comments")` returns CommentsDir inode | unit (in-process) | `cargo test -p reposix-fuse comments_dir_lookup` | ❌ Wave 0 |
| CONF-02 | `readdir(CommentsDir)` returns comment filenames | unit (in-process) | `cargo test -p reposix-fuse comments_dir_readdir` | ❌ Wave 0 |
| CONF-02 | `list_comments` paginates via `_links.next` | unit (wiremock) | `cargo test -p reposix-confluence list_comments_paginates` | ❌ Wave 0 |
| CONF-03 | `write`/`create` on CommentFile returns `EROFS`/`EACCES` | unit (in-process) | `cargo test -p reposix-fuse comment_write_is_rejected` | ❌ Wave 0 |
| (spaces) | `list_spaces()` returns key + name + URL | unit (wiremock) | `cargo test -p reposix-confluence list_spaces` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-confluence -p reposix-fuse -p reposix-cli`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite + `cargo clippy --workspace --all-targets -- -D warnings` green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/reposix-confluence/src/lib.rs` — `ConfComment` struct, `list_comments()`, `list_spaces()` (5+ wiremock tests)
- [ ] `crates/reposix-fuse/src/comments.rs` — `CommentsSnapshot`, `render_comment_file()` pure fn
- [ ] `crates/reposix-fuse/src/inode.rs` — `COMMENTS_DIR_INO_BASE`, `COMMENTS_FILE_INO_BASE` + disjoint assertions
- [ ] `crates/reposix-fuse/src/fs.rs` — `InodeKind::CommentsDir`, `InodeKind::CommentFile` + dispatch arms
- [ ] `crates/reposix-cli/src/spaces.rs` — `spaces::run()` table rendering
- [ ] `crates/reposix-cli/src/main.rs` — `Cmd::Spaces` arm

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Auth already on ConfluenceBackend (basic-auth header) |
| V3 Session Management | no | Stateless HTTP |
| V4 Access Control | yes | Comment bodies are tainted; read-only FUSE (`EROFS` for writes) |
| V5 Input Validation | yes | Comment IDs and page IDs validated to numeric before URL construction (WR-01 pattern from `resolve_space_id`) |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns for Confluence Comments

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SSRF via attacker-controlled `_links.next` absolute URL in comment pagination response | Spoofing | `parse_next_cursor` relative-prepend pattern already defeats this for pages; apply same for comments |
| Comment body contains `---\n` YAML fence injection into frontmatter | Tampering | Render body AFTER closing frontmatter fence `---\n\n`; body is not parsed as YAML |
| Comment author id (`version.authorId`) leaks into error logs | Info Disclosure | Cap to numeric-only validation; never log author tokens |
| `parentCommentId` path traversal via non-numeric value | Tampering | Validate parentCommentId is numeric before embedding in frontmatter (same WR-02 pattern from `resolve_space_id`) |

---

## Sources

### Primary (HIGH confidence)
- `crates/reposix-confluence/src/lib.rs` — full ConfluenceBackend HTTP pattern, pagination, rate-limit, auth headers, `parse_next_cursor`, `resolve_space_id`
- `crates/reposix-fuse/src/fs.rs` — full FUSE dispatch, `InodeKind` classify, all callback arms
- `crates/reposix-fuse/src/inode.rs` — complete inode range layout with compile-time assertions
- `crates/reposix-fuse/src/labels.rs` — `LabelSnapshot` precedent (Phase 19 direct analog)

### Secondary (MEDIUM confidence)
- [CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-comment/] — inline-comment and footer-comment response shapes, `parentCommentId`, `resolutionStatus`, `body-format` query param
- [CITED: https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-space/] — GET /wiki/api/v2/spaces response shape, `key`, `name`, `_links.webui`

### Tertiary (LOW confidence — needs live verification)
- Pagination limit parameter for `/inline-comments` and `/footer-comments` — assumed same `?limit=100` semantics as pages endpoint. [LOW]
- `version.authorId` on comment = Atlassian accountId string (not display name) — assumed from field naming. [LOW]

---

## Metadata

**Confidence breakdown:**
- Confluence API response shape: MEDIUM — confirmed from official docs, `parentCommentId` / `resolutionStatus` presence verified
- FUSE inode allocation pattern: HIGH — reading directly from shipped Phase 19 code
- FUSE dispatch wiring pattern: HIGH — reading directly from shipped Phase 19/fs.rs code
- `reposix spaces` API shape: MEDIUM — confirmed from official docs
- Test patterns: HIGH — reading directly from existing wiremock tests in lib.rs

**Research date:** 2026-04-16
**Valid until:** 2026-05-16 (Confluence v2 API stable; internal patterns only change with code changes)
