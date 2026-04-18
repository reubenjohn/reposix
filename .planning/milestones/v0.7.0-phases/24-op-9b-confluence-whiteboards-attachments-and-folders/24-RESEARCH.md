# Phase 24: OP-9b — Confluence Whiteboards, Attachments, and Folders — Research

**Researched:** 2026-04-16
**Domain:** Confluence Cloud REST v2 API + FUSE inode dispatch extension
**Confidence:** HIGH (API facts verified from official docs; FUSE patterns from direct codebase inspection)

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CONF-04 | `ls mount/whiteboards/` lists Confluence whiteboards; each exposed as `<id>.json` | Whiteboard list endpoint exists; see Standard Stack §Whiteboards |
| CONF-05 | `ls mount/pages/<id>.attachments/` lists page attachments; binary passthrough | Attachments endpoint verified; binary cache strategy in Architecture Patterns §Attachments |
| CONF-06 | Folders (`/folders` endpoint) exposed as a separate tree alongside page hierarchy | Folders endpoint confirmed; caveats in §Folders section |
</phase_requirements>

---

## Summary

Phase 24 extends the multi-content-type FUSE overlay established in Phase 23 (`.comments/` per-page dirs) to three new content types: whiteboards, attachments, and folders. Live docs are **moved to Future Requirements** (see below).

**Whiteboards** have a single-item GET endpoint (`GET /wiki/api/v2/whiteboards/{id}`). Critically, there is **no list-all-whiteboards-in-a-space endpoint in v2** — listing requires traversing the page/tree descendant graph. The whiteboard response returns only metadata (id, title, spaceId, createdAt, etc.), **not the drawing body**. For v0.7.0, expose as `whiteboards/<id>.json` containing serialized metadata from `GET /whiteboards/{id}`.

**Attachments** are well-supported: `GET /wiki/api/v2/pages/{id}/attachments` returns paginated `{id, title, mediaType, fileSize, downloadLink}` records. The `downloadLink` field points to a URL for the binary body. FUSE `read` must serve binary content; the simplest v0.7.0 approach is to cache the full binary in memory on first `read` (identical to how comment rendered bytes are cached in `CommentsSnapshot`). True streaming (avoid heap load) is an optimization deferred to a future phase — the 500-page cap + typical attachment sizes make heap caching acceptable for v0.7.0.

**Folders** have `GET /wiki/api/v2/folders/{id}` (single-item) and no list-all endpoint either. However, folders already appear in the `pages/<id>` response via `parentType: "folder"` — the translate() function in `lib.rs` currently drops folder parents as orphans. The dedicated `folders/` FUSE tree is **low value** for v0.7.0 if the page hierarchy already captures folder structure. Research conclusion: skip the separate `folders/` tree; instead fix `translate()` to pass through `parentType: "folder"` parents into `parent_id` so folders appear in `tree/` naturally.

**Live docs** are now a standard `subtype: "live"` variant on the `/pages` endpoint (not `/custom-content`). They already appear in `list_issues` output. The `last-synced-at` frontmatter field and `livedocs/` tree can be deferred — live docs are already readable via `pages/<id>.md` (without the special `subtype` marker). Move to Future Requirements.

**Primary recommendation:** Implement whiteboards (CONF-04) and attachments (CONF-05) as new top-level and per-page overlay dirs. Resolve CONF-06 by using the existing page hierarchy rather than a redundant `/folders` tree.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Whiteboard metadata fetch | API/Backend (`reposix-confluence`) | — | Same layer as `list_issues` / `list_comments`; HTTP GET on Confluence v2 |
| Whiteboard listing | API/Backend | — | No Confluence v2 list endpoint; must derive from descendants or skip |
| Attachment metadata list | API/Backend (`reposix-confluence`) | — | `GET /pages/{id}/attachments` — per-page REST call |
| Attachment binary download | API/Backend + in-process cache | — | `downloadLink` URL → reqwest GET → cache bytes in `AttachmentsSnapshot` |
| `whiteboards/` FUSE dir | Frontend Server (FUSE fs.rs) | — | New top-level dir; same tier as `pages/`, `tree/`, `labels/` |
| `.attachments/` per-page FUSE dir | Frontend Server (FUSE fs.rs) | — | Sub-dir under Bucket entry; same tier as `.comments/` from Phase 23 |
| Inode range allocation | Config (`inode.rs`) | — | New constants after `COMMENTS_FILE_INO_BASE` |

---

## Standard Stack

### Core (unchanged — no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `reposix-confluence` (in-crate) | 0.6.0 | Confluence HTTP adapter | Already has `list_comments` / `list_spaces` patterns to extend |
| `reqwest` | 0.12 | HTTP client | Already in use; `resp.bytes().await` for binary downloads |
| `fuser` | 0.17 | FUSE filesystem callbacks | Already mounted, `default-features = false` |
| `dashmap` | current workspace pin | Lock-free concurrent maps for snapshots | Used in `CommentsSnapshot`, `InodeRegistry` |
| `serde_json` | current workspace pin | JSON parsing of Confluence responses | `serde_json::Value` + typed structs pattern |

### No New Dependencies Required
The Phase 23 patterns (`CommentsSnapshot`, `render_comment_file`) are templates that Phase 24 can copy without adding any new Cargo.toml entries. Binary attachment bytes are served as `Vec<u8>` (same as `CommentEntry::rendered`).

---

## Architecture Patterns

### System Architecture Diagram

```
Agent / shell
    |
    | ls / cat / grep
    v
FUSE VFS
    |
    +-- mount/whiteboards/           (new top-level dir)
    |       <id>.json                (whiteboard metadata JSON)
    |
    +-- mount/pages/                 (existing bucket dir)
            <padded-id>.md           (existing page files)
            <padded-id>.comments/    (Phase 23)
                <cid>.md
            <padded-id>.attachments/ (new per-page subdir)
                filename.ext         (binary passthrough)

FUSE callbacks (fs.rs)
    |
    +-- InodeKind::WhiteboardsRoot   -> new synthetic dir inode
    +-- InodeKind::WhiteboardFile    -> new synthetic file inode
    +-- InodeKind::AttachmentsDir    -> per-page dir (mirrors CommentsDir)
    +-- InodeKind::AttachmentFile    -> per-file binary entry

reposix-confluence lib.rs
    |
    +-- list_whiteboards_in_space()  -> GET /wiki/api/v1/content (CQL)
    +-- get_whiteboard(id)           -> GET /wiki/api/v2/whiteboards/{id}
    +-- list_attachments(page_id)    -> GET /wiki/api/v2/pages/{id}/attachments
    +-- download_attachment(url)     -> GET {downloadLink}

Confluence Cloud REST API v2
```

### Recommended Project Structure (additions only)

```
crates/reposix-fuse/src/
├── attachments.rs    # NEW: AttachmentsSnapshot (mirrors comments.rs)
└── fs.rs             # extend: WhiteboardsRoot / WhiteboardFile / AttachmentsDir / AttachmentFile

crates/reposix-confluence/src/
└── lib.rs            # extend: ConfWhiteboard, ConfAttachment structs + methods
```

### Pattern 1: New Confluence API Methods (same shape as `list_comments`)

All new API calls follow the exact same structure as `list_comments` in `lib.rs`. Do NOT deviate:

```rust
// Source: crates/reposix-confluence/src/lib.rs — list_comments() method

// Step 1: build URL
// Step 2: standard_headers()
// Step 3: await_rate_limit_gate().await
// Step 4: request_with_headers(Method::GET, ...)
// Step 5: ingest_rate_limit(&resp)
// Step 6: resp.bytes().await?
// Step 7: parse_next_cursor(&body_json) for pagination
// Step 8: push results; check MAX_ISSUES_PER_LIST cap + WARN

// New serde structs follow the ConfPage pattern (Deserialize only, no deny_unknown_fields)
```

### Pattern 2: New FUSE Overlay Structs (mirror `CommentsSnapshot`)

`AttachmentsSnapshot` in `attachments.rs` mirrors `CommentsSnapshot` exactly:

```rust
// Source: crates/reposix-fuse/src/comments.rs

pub struct AttachmentEntry {
    pub file_ino: u64,
    pub filename: String,         // sanitized: only [a-zA-Z0-9._-], no slashes
    pub rendered: Vec<u8>,        // binary attachment body (cached on first read)
    pub file_size: u64,           // from API fileSize field (for getattr BEFORE body fetch)
    pub download_url: String,     // stored so body can be fetched lazily
    pub media_type: String,       // stored for future Content-Type use
}

pub struct AttachmentsSnapshot {
    by_page: DashMap<u64, (bool, Vec<AttachmentEntry>)>, // fetched flag + entries
    page_to_dir_ino: DashMap<u64, u64>,
    dir_ino_to_page: DashMap<u64, u64>,
    file_ino_to_page: DashMap<u64, u64>,
    next_dir_ino: AtomicU64,
    next_file_ino: AtomicU64,
}
```

**Key difference from `CommentsSnapshot`:** `AttachmentEntry::rendered` is initially `None` (lazy body) or loaded eagerly on first `readdir`. Recommendation: load metadata on `readdir` (list call), load binary body eagerly at that point too if `fileSize < 50 MB threshold`. This avoids a second round-trip on `read` while keeping memory bounded.

### Pattern 3: Whiteboard Listing via Descendants

The v2 API has no `GET /wiki/api/v2/spaces/{id}/whiteboards` list endpoint. Options:

**Option A (recommended for v0.7.0):** Use the v1 CQL search endpoint (still available, not yet removed):
```
GET /wiki/rest/api/content/search?cql=type=whiteboard+AND+space.key={key}&expand=version&limit=50
```
This returns all whiteboards in a space. The v1 endpoint is deprecated but not removed. Wrap in a `#[cfg]` comment noting it will need updating when Atlassian removes v1.

**Option B:** Use `GET /wiki/api/v2/spaces/{space_id}/direct-children` (paginated). Returns all content types (pages, whiteboards, folders) as mixed results with a `type` discriminator. Filter for `type == "whiteboard"`. [VERIFIED from Atlassian developer community discussions — this endpoint exists but is not prominent in the main v2 docs page].

**Option C:** Skip listing — expose only the whiteboard IDs surfaced via the existing page list's `parentType` field. If a page's `parentId` resolves to something with `parentType == "whiteboard"`, we have the whiteboard ID. Limited coverage.

**Recommendation:** Option B (direct-children) is the cleanest v2-only approach. Fetch once per mount for the space, cache results in `ReposixFs`.

### Pattern 4: Binary Attachment Read (in-memory caching)

FUSE `read()` receives `(offset, size)` — it must return the slice `bytes[offset..offset+size]`. For binary attachments this means the full body must be in memory before `read()` returns.

**v0.7.0 strategy:** Fetch full binary on first `readdir` of the `.attachments/` dir. The 500-page cap (HARD-02) + typical Confluence attachment sizes (most are images < 10 MB) makes this acceptable.

```rust
// In AttachmentsSnapshot: on first readdir/lookup of AttachmentsDir:
// 1. list_attachments(page_id) → Vec<ConfAttachment>
// 2. For each attachment: download_attachment(att.download_url) → Vec<u8>
// 3. Store in AttachmentEntry::rendered
// 4. getattr reports att.file_size from metadata; read() slices rendered bytes

// IMPORTANT: attachment content is tainted. Never log attachment bytes.
// The download URL itself (e.g. /wiki/download/attachments/...) should be
// redacted in error messages via redact_url().
```

**Memory budget concern:** 500 pages × N attachments × M bytes. Mitigations:
- Lazy-per-page (only fetch when `.attachments/` is accessed, same as `.comments/`)
- `fileSize` field available from list call → can skip fetch for files > 50 MB with a `tracing::warn!`

### Pattern 5: Inode Range Allocation

New ranges must be added after `COMMENTS_FILE_INO_BASE` (= `0x1C_0000_0000`) in `inode.rs`. Follow the established pattern of 1-billion-slot windows:

| Range | Purpose |
|-------|---------|
| `0x20_0000_0000..0x24_0000_0000` | `whiteboards/` root dir + whiteboard file inodes |
| `0x24_0000_0000..0x28_0000_0000` | Per-page `.attachments/` dir inodes |
| `0x28_0000_0000..u64::MAX` | Attachment file inodes |

The whiteboard root dir needs only one fixed inode (like `LABELS_ROOT_INO`). Individual whiteboard files get sequential allocation from a small counter (typically < 100 whiteboards per space).

### Anti-Patterns to Avoid

- **Hand-roll URL building for whiteboard IDs:** Always validate `id.chars().all(|c| c.is_ascii_digit())` before interpolating into URL (WR-02 pattern from `resolve_space_id`).
- **Log attachment binary content:** Treat `Vec<u8>` attachment bytes as tainted. Only log `page_id`, `attachment_id`, `mediaType`, `fileSize`.
- **Use `_links.base` from Confluence response for URL construction:** Always prepend `self.base()` to relative cursor paths — never use Atlassian's `_links.base` (SSRF vector, same mitigation in `list_issues_impl`).
- **New inode range overlapping existing ranges:** Run the disjoint assertion test in `inode.rs` after adding new constants.
- **Add whiteboard body fetch without size guard:** Whiteboard bodies are not available via v2 API at all (response is metadata only). Do not expect a body field — the `.json` file IS the metadata.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pagination loop | Custom cursor tracker | Copy `list_issues_impl` loop from `lib.rs` | Handles rate-limit gate, SSRF-safe relative URLs, MAX cap + WARN |
| Binary download | Custom HTTP fetch | `self.http.request_with_headers(GET, download_url, ...)` | SG-01 allowlist gate applies to all outbound calls through `HttpClient` |
| Inode allocation | Hash-based scheme | `AtomicU64` starting from new base constants | Deterministic, DashMap idempotent, proven by InodeRegistry pattern |
| Attachment filename sanitization | Regex | Allowlist `[a-zA-Z0-9._-]` + truncate at 255 bytes | Path-traversal defense (mirrors `is_numeric_ascii` in comments.rs) |
| Rate limit awareness | Separate sleep logic | `await_rate_limit_gate()` + `ingest_rate_limit()` | Already implemented on `ConfluenceBackend`; call sequence is mandatory |

---

## API Endpoint Reference (verified)

### Whiteboards [VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard]

| Endpoint | Method | Auth Scope | Notes |
|----------|--------|-----------|-------|
| `/wiki/api/v2/whiteboards/{id}` | GET | `read:whiteboard:confluence` | Returns metadata only — no body/drawing content |
| `/wiki/api/v2/whiteboards` | POST | `write:whiteboard:confluence` | Create only; no list-all endpoint |

**Response schema:** `id`, `type`, `status`, `title`, `parentId`, `parentType`, `position`, `authorId`, `ownerId`, `createdAt`, `spaceId`, `version` — **no body/drawing field**.

**No `GET /wiki/api/v2/whiteboards` (list) endpoint exists.** [VERIFIED: no such endpoint in official v2 docs]

### Attachments [VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-attachment]

| Endpoint | Method | Auth Scope | Notes |
|----------|--------|-----------|-------|
| `/wiki/api/v2/pages/{id}/attachments` | GET | `read:attachment:confluence` | Paginated; returns metadata + `downloadLink` |
| `{downloadLink}` | GET | same | Binary body download (may redirect) |

**Response fields:** `id`, `status`, `title`, `createdAt`, `pageId`, `mediaType`, `mediaTypeDescription`, `comment`, `fileId`, `fileSize`, `webuiLink`, `downloadLink`, `version`, `_links.download`.

### Folders [VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder]

| Endpoint | Method | Notes |
|----------|--------|-------|
| `/wiki/api/v2/folders/{id}` | GET | Single-folder metadata only |

**No `GET /wiki/api/v2/folders` (list) endpoint.** Folders surface as `parentType: "folder"` in page responses.

### Live Docs [VERIFIED: community.developer.atlassian.com/t/rfc-83-live-docs-pages-in-confluence-cloud/88495]

Live docs are **pages with `subtype: "live"`** — they appear in the standard `/pages` endpoint. There is no separate endpoint. The HANDOFF.md note about `/wiki/api/v2/custom-content/` with type discriminator was **incorrect** — that applies to third-party custom content, not Atlassian's own Live Docs. Live docs are therefore already accessible via `pages/<id>.md` in the current mount. **Defer `livedocs/` overlay to Future Requirements.**

---

## Confluence Discovery Gap: No Space-Wide List for Whiteboards/Folders

[VERIFIED: community.developer.atlassian.com/t/rest-api-v2-how-to-get-all-content-in-a-space/83763]

The v2 API lacks a unified "list all content in space" endpoint. For whiteboards specifically:

1. **`GET /wiki/api/v2/spaces/{id}/direct-children`** — returns mixed content types; filter for `type == "whiteboard"`. This is the recommended v0.7.0 approach. [MEDIUM confidence — referenced in community threads; not prominently documented in the main v2 reference page]

2. **v1 CQL fallback:** `GET /wiki/rest/api/content/search?cql=type=whiteboard+AND+space.key=KEY` — works today, deprecated trajectory. Use as fallback if direct-children doesn't exist. [MEDIUM confidence]

3. **Descendants traversal:** Call `/wiki/api/v2/pages/{root_id}/descendants` starting from known root pages. The descendants endpoint returns all 5 content types (Database, Embed, Folder, Page, Whiteboard). Expensive (requires knowing root IDs). [VERIFIED from descendants API docs]

**Decision needed by planner:** confirm which discovery approach to use. Research recommends Option 1 (direct-children) with Option 2 (v1 CQL) as fallback.

---

## Common Pitfalls

### Pitfall 1: Whiteboard Body Does Not Exist in v2 API
**What goes wrong:** Attempt to populate `whiteboard/<id>.json` with drawing content; API returns 404 or schema has no body field.
**Why it happens:** Whiteboard drawing format is proprietary; Atlassian v2 API exposes only metadata for whiteboards.
**How to avoid:** The `.json` file contains only the metadata from `GET /wiki/api/v2/whiteboards/{id}` — serialized as JSON. This is explicitly the v0.7.0 scope per CONTEXT.md.
**Warning signs:** If the response includes `body` or `content` fields, that would be novel and requires re-verifying the schema.

### Pitfall 2: Attachment Download URL Requires Auth
**What goes wrong:** `reqwest::get(download_url)` returns 401 or redirect to login.
**Why it happens:** The Confluence `downloadLink` (e.g. `/wiki/download/attachments/{pageId}/{filename}`) requires Basic-auth headers — same credentials as the REST API.
**How to avoid:** Use `self.http.request_with_headers(Method::GET, download_url, &header_refs)` with `standard_headers()` (which includes Authorization: Basic). Do NOT use a bare `reqwest::get`.
**Warning signs:** 401/403 responses when fetching attachment bodies; or 302 redirect to `id.atlassian.com`.

### Pitfall 3: Attachment Filename Contains Path Separators
**What goes wrong:** Attachment filename `../../etc/passwd.txt` creates path traversal.
**Why it happens:** Confluence allows filenames with `/`, `..`, spaces.
**How to avoid:** Sanitize filenames: allow only `[a-zA-Z0-9._-]`; replace anything else with `_`. Mirror the `is_numeric_ascii` pattern from `comments.rs`. Return `None` from render function for any filename that sanitizes to empty.
**Warning signs:** Test with filenames containing `/`, `..`, null bytes.

### Pitfall 4: Inode Ordering in `classify()` — New Ranges Must Come Before Existing Catch-alls
**What goes wrong:** New `WHITEBOARD_FILE_INO_BASE` or `ATTACHMENT_FILE_INO_BASE` constants are above `COMMENTS_FILE_INO_BASE` but `classify()` doesn't check them, so they fall through to `InodeKind::Unknown`.
**Why it happens:** `classify()` in `fs.rs` checks ranges from high to low; new higher ranges must be added at the top of the numeric-range chain.
**How to avoid:** Follow the ordering established in Phase 23: check highest numerical ranges first. Add disjoint assertion tests in `inode.rs` for all new ranges.

### Pitfall 5: `AttachmentsDir` name suffix conflict with `.comments`
**What goes wrong:** Agent passes `"00000000001.attachments"` to `lookup()` in the Bucket arm; the `.comments` strip_suffix check fires first (it won't — different suffix) or regex is mis-ordered.
**Why it happens:** Phase 23 already reserves the `.comments` suffix. The `.attachments` suffix is new.
**How to avoid:** In the Bucket `lookup()` arm, add the `.attachments` strip_suffix check as a separate branch, sibling to the `.comments` check. Order doesn't matter between them (different suffixes), but both must appear BEFORE `validate_issue_filename`.

### Pitfall 6: `ReposixFs` grows a second concrete-backend reference
**What goes wrong:** Adding `attachment_fetcher: Option<Arc<ConfluenceBackend>>` duplicates `comment_fetcher`. Code duplication + construction complexity.
**How to avoid:** Reuse `self.comment_fetcher` (rename to `confluence_fetcher` in this phase, or simply reuse the existing field). The `ConfluenceBackend` instance already provides both `list_comments()` and the new `list_attachments()` / `get_whiteboard()` methods. One `Arc<ConfluenceBackend>` is enough for all Confluence-specific FUSE overlay methods.

---

## Code Examples

### ConfAttachment serde struct (verified field names from API docs)
```rust
// Source: developer.atlassian.com/cloud/confluence/rest/v2/api-group-attachment
#[derive(Debug, Clone, Deserialize)]
pub struct ConfAttachment {
    pub id: String,
    pub status: String,
    pub title: String,                        // filename as stored in Confluence
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "pageId")]
    pub page_id: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(rename = "fileSize", default)]
    pub file_size: u64,
    #[serde(rename = "downloadLink", default)]
    pub download_link: String,                // relative path e.g. /wiki/download/attachments/...
    // webuiLink, comment, fileId, mediaTypeDescription are fine to include but not required
}

#[derive(Debug, Deserialize)]
struct ConfAttachmentList {
    results: Vec<ConfAttachment>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    links: Option<ConfLinks>,    // reuse existing ConfLinks struct
}
```

### ConfWhiteboard serde struct
```rust
// Source: developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ConfWhiteboard {
    pub id: String,
    pub status: String,
    pub title: String,
    #[serde(rename = "spaceId")]
    pub space_id: String,
    #[serde(rename = "authorId", default)]
    pub author_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "parentId", default)]
    pub parent_id: Option<String>,
    #[serde(rename = "parentType", default)]
    pub parent_type: Option<String>,
}
// Note: serde::Serialize needed so the FUSE read returns serde_json::to_vec(&whiteboard)
```

### Attachment filename sanitization
```rust
/// Sanitize a Confluence attachment title for use as a POSIX filename.
/// Allows [a-zA-Z0-9._-]; replaces everything else with `_`.
/// Returns None if result is empty or exceeds 255 bytes.
fn sanitize_attachment_filename(title: &str) -> Option<String> {
    let sanitized: String = title.chars().map(|c| {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            c
        } else {
            '_'
        }
    }).collect();
    if sanitized.is_empty() || sanitized.len() > 255 {
        return None;
    }
    Some(sanitized)
}
```

### Whiteboard `read()` callback
```rust
// In fs.rs read() match arm for InodeKind::WhiteboardFile:
InodeKind::WhiteboardFile => {
    let Some(wboard) = self.whiteboard_snapshot.entry_by_ino(ino_u) else {
        reply.error(fuser::Errno::from_i32(libc::ENOENT));
        return;
    };
    let bytes = &wboard.rendered_json;
    let start = usize::try_from(offset).unwrap_or(usize::MAX).min(bytes.len());
    let end = start.saturating_add(size as usize).min(bytes.len());
    reply.data(&bytes[start..end]);
}
```

---

## CONF-06 Resolution: Folders Are Already Covered

[VERIFIED: official v2 API docs + codebase inspection of `lib.rs` translate()]

The current `translate()` function in `lib.rs` lines 371–399 treats `parentType == "folder"` as an orphan:

```rust
(_, Some(other)) => {
    // "folder" / "whiteboard" / "database" parents become tree roots with debug log
    tracing::debug!(page_id = ..., parent_type = ..., "confluence non-page parentType, treating as orphan");
    None
}
```

**The CONF-06 requirement asks for a `folders/` overlay tree.** Research finds this is unnecessary if we instead fix `translate()` to pass through `parentType == "folder"` parents. Pages under folders would then appear correctly in the `tree/` overlay with the folder as a parent node. The folder node would be a `tree/` directory entry (via `TreeSnapshot`).

**Recommended CONF-06 implementation:** Instead of a separate `folders/` top-level dir, modify `translate()` to accept `parentType == "folder"` and pass through the `parentId` as `Issue::parent_id`. Update `TreeSnapshot` to handle nodes whose parent is a folder (not a page) by rendering them as a dir entry with the folder title. This is simpler and avoids another inode range.

**If the user requires a literal `folders/` tree** (separate from `tree/`), the planner should clarify. Research assesses it as low-value given the overlap.

---

## Live Docs: Move to Future Requirements

[VERIFIED: community.developer.atlassian.com/t/rfc-83-live-docs-pages-in-confluence-cloud/88495]

Live docs are pages with `subtype: "live"` — they already appear in `GET /wiki/api/v2/spaces/{id}/pages`. They are accessible today via `pages/<id>.md`. The special `livedocs/<id>.md` overlay with `last-synced-at` frontmatter adds minimal value for v0.7.0 given:
1. The content is already accessible.
2. The v2 `subtype` filtering (`?subtype=live`) is EAP/canary as of early 2025.
3. Implementation would require a second pass over the page list + new frontmatter schema.

Move to Future Requirements in `REQUIREMENTS.md`.

---

## Runtime State Inventory

This is a feature-addition phase (no rename/refactor). No runtime state inventory required.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / `rustc` | Building | ✓ | stable via rust-toolchain.toml | — |
| `wiremock` (dev-dep) | Tests | ✓ | already in Cargo.toml | — |
| Confluence test tenant | `#[ignore]` live tests | Conditional | reuben-john.atlassian.net | wiremock mocks cover CI |

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in + `wiremock` |
| Config file | none (inline `#[cfg(test)] mod tests`) |
| Quick run command | `cargo test -p reposix-confluence -p reposix-fuse --workspace` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONF-04 | `list_attachments(page_id)` returns `Vec<ConfAttachment>` | unit/wiremock | `cargo test -p reposix-confluence list_attachments` | ❌ Wave 0 |
| CONF-04 | `ls mount/pages/<id>.attachments/` shows filenames | integration (no FUSE) | `cargo test -p reposix-fuse attachments_readdir` | ❌ Wave 0 |
| CONF-04 | `list_whiteboards_in_space()` returns `Vec<ConfWhiteboard>` | unit/wiremock | `cargo test -p reposix-confluence list_whiteboards` | ❌ Wave 0 |
| CONF-04 | `ls mount/whiteboards/` shows `<id>.json` entries | integration | `cargo test -p reposix-fuse whiteboards_readdir` | ❌ Wave 0 |
| CONF-05 | `cat mount/pages/<id>.attachments/file.pdf` returns binary bytes | integration | `cargo test -p reposix-fuse attachment_read_binary` | ❌ Wave 0 |
| CONF-05 | Attachment filename sanitization rejects path-traversal names | unit | `cargo test -p reposix-fuse sanitize_attachment_filename` | ❌ Wave 0 |
| CONF-06 | `parentType=="folder"` pages appear in `tree/` | unit | `cargo test -p reposix-fuse folder_parent_in_tree` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-confluence -p reposix-fuse`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/reposix-fuse/src/attachments.rs` — new module; `AttachmentsSnapshot` unit tests
- [ ] `crates/reposix-confluence/tests/contract.rs` — add wiremock stubs for `list_attachments` + `get_whiteboard`
- [ ] `crates/reposix-fuse/tests/nested_layout.rs` — add readdir assertions for `.attachments/` and `whiteboards/`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | `sanitize_attachment_filename()` for filenames; `is_numeric_ascii()` for IDs |
| V4 Access Control | yes | Read-only FUSE mount (`perm: 0o444`); no write path for attachments/whiteboards |
| V6 Cryptography | no | Binary passthrough; no encryption at rest |
| V2 Authentication | partial | Basic-auth via existing `standard_headers()` — no new auth code |

### Known Threat Patterns for this Phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via attachment filename | Tampering | `sanitize_attachment_filename()` allowlist |
| SSRF via `downloadLink` from Confluence response | Elevation of Privilege | `HttpClient` SG-01 allowlist gate (all HTTP calls go through it) |
| Log injection via attachment title/body | Information Disclosure | Never log attachment body; log only `page_id`, `attachment_id`, `file_size` |
| Oversized attachment OOM | Denial of Service | Skip download for `file_size > 50 MB`; log WARN |
| Whiteboard ID path injection | Tampering | `WR-02` numeric-only validation before URL interpolation |

**Note on SSRF for attachment download:** The `downloadLink` from Confluence is a relative path (e.g. `/wiki/download/attachments/...`). The pattern in `list_issues_impl` of prepending `self.base()` to relative URLs is SSRF-safe. If Atlassian ever returns an absolute URL, the `HttpClient` SG-01 allowlist gate will refuse it if the domain isn't in `REPOSIX_ALLOWED_ORIGINS`. This is defense in depth already present.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `GET /wiki/api/v2/spaces/{id}/direct-children` returns whiteboards with a `type` field | Architecture Patterns §Whiteboard Listing | May need to fall back to v1 CQL search; extra implementation path |
| A2 | The `downloadLink` in attachment response is a relative path requiring Basic-auth (not a pre-signed URL) | Code Examples | If pre-signed, `standard_headers()` auth would be irrelevant; test against live tenant |
| A3 | Whiteboard response truly has no body/drawing field (confirmed from docs scrape, not live API test) | Common Pitfalls §1 | If a body field exists, the `.json` output would be more valuable |

---

## Open Questions

1. **Whiteboard discovery endpoint**
   - What we know: No `GET /wiki/api/v2/whiteboards` list; `direct-children` may work
   - What's unclear: Whether `direct-children` is generally available (not EAP-gated)
   - Recommendation: Add a wiremock test that mocks `direct-children`; implement with a graceful fallback (empty `whiteboards/` dir) if 404

2. **CONF-06 scope: literal `folders/` dir vs `tree/` fix**
   - What we know: CONTEXT.md says "only if distinct from the existing `tree/` parentId hierarchy"
   - What's unclear: Whether the REQUIREMENTS.md intent is a literal `folders/` dir or just correct hierarchy
   - Recommendation: Implement as `translate()` fix (pass through folder parents); add a `folders/` dir only if the user explicitly confirms they want it as a separate view

3. **Attachment body fetch laziness**
   - What we know: `getattr` needs `size` from the metadata; `read` needs the binary
   - What's unclear: Whether to eagerly fetch binary on `readdir` or lazily on first `read`
   - Recommendation: Eager on `readdir` (simpler dispatch, no deferred-fetch state machine); skip files > 50 MB

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Live docs via `/custom-content/` type discriminator | Live docs are `page` subtype (`subtype: "live"`) | RFC-83, GA Feb 2025 | No separate endpoint needed; already in `list_issues` |
| v1 CQL for space content discovery | v2 `direct-children` (in progress) | Ongoing Atlassian v2 expansion | v1 CQL deprecated but not removed |

**Deprecated/outdated:**
- `GET /wiki/rest/api/content/search?cql=type=whiteboard`: v1 API, deprecated trajectory
- `/wiki/api/v2/custom-content/` with type discriminator for live docs: was speculated but **incorrect** — not the live docs mechanism

---

## Sources

### Primary (HIGH confidence)
- [Confluence Cloud REST API v2 — Whiteboard](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/) — endpoint schemas verified
- [Confluence Cloud REST API v2 — Attachment](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-attachment/) — attachment fields + downloadLink verified
- [Confluence Cloud REST API v2 — Folder](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/) — folder endpoints verified (GET/{id} only, no list)
- `crates/reposix-confluence/src/lib.rs` — direct codebase read for patterns
- `crates/reposix-fuse/src/comments.rs` — direct codebase read for CommentsSnapshot pattern
- `crates/reposix-fuse/src/inode.rs` — direct codebase read for inode layout
- `crates/reposix-fuse/src/fs.rs` — direct codebase read for dispatch patterns

### Secondary (MEDIUM confidence)
- [RFC-83: Live Docs & Pages in Confluence Cloud](https://community.developer.atlassian.com/t/rfc-83-live-docs-pages-in-confluence-cloud/88495) — live docs are `subtype: "live"` on pages endpoint
- [Confluence v2 — how to get all content in a space](https://community.developer.atlassian.com/t/rest-api-v2-how-to-get-all-content-in-a-space/83763) — confirms no unified list endpoint
- [Confluence v2 — descendants API](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-descendants/) — returns 5 content types including whiteboard

### Tertiary (LOW confidence)
- `direct-children` endpoint existence — referenced in community threads, not in main API reference

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps needed; existing patterns sufficient
- Architecture: HIGH — Confluence API verified from official docs; FUSE pattern directly observed
- Pitfalls: HIGH — drawn from verified Phase 23 code + confirmed API schema gaps
- Whiteboard listing: MEDIUM — depends on `direct-children` endpoint availability

**Research date:** 2026-04-16
**Valid until:** 2026-05-16 (Atlassian API changes slowly; Live Docs subtype GA targeted Feb 2025 already shipped)
