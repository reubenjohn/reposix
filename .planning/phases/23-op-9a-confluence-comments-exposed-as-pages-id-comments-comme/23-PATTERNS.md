# Phase 23: OP-9a Confluence Comments — Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 6 new/modified files
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/reposix-confluence/src/lib.rs` | service (API adapter) | request-response + CRUD | `lib.rs` itself (`list_issues_impl`, `resolve_space_id`) | exact — same file, adding two new methods |
| `crates/reposix-fuse/src/inode.rs` | config (constants) | — | `inode.rs` itself (`LABELS_DIR_INO_BASE`, `LABELS_SYMLINK_INO_BASE`) | exact — same file, adding two new constants + assertions |
| `crates/reposix-fuse/src/comments.rs` | service (in-memory overlay) | event-driven (lazy-loaded per lookup) | `crates/reposix-fuse/src/labels.rs` (`LabelSnapshot`) | exact role-match |
| `crates/reposix-fuse/src/fs.rs` | controller (FUSE dispatch) | request-response | `fs.rs` itself (`InodeKind::LabelDir` / `InodeKind::LabelSymlink` dispatch arms) | exact — same file, adding two new `InodeKind` variants + dispatch |
| `crates/reposix-cli/src/spaces.rs` | service (CLI output) | request-response | `crates/reposix-cli/src/list.rs` (`render_table`, `run`) | exact role-match |
| `crates/reposix-cli/src/main.rs` | controller (CLI dispatch) | request-response | `main.rs` itself (`Cmd::List` arm) | exact — same file, adding `Cmd::Spaces` arm |

---

## Pattern Assignments

### `crates/reposix-confluence/src/lib.rs` — add `list_comments()` + `list_spaces()` (service, request-response)

**Analog:** the same file — `list_issues_impl` (lines 546–629) and `resolve_space_id` (lines 736–782)

**New serde structs pattern** (after existing `ConfPageBody`, ~line 285):

```rust
// Pattern: same as ConfPage / ConfVersion — Deserialize only, no deny_unknown_fields
#[derive(Debug, Deserialize)]
struct ConfCommentList {
    results: Vec<ConfComment>,
    #[serde(default, rename = "_links")]
    links: Option<ConfLinks>,
}

#[derive(Debug, Deserialize)]
pub struct ConfComment {
    pub id: String,
    #[serde(rename = "pageId")]
    pub page_id: String,
    pub version: ConfCommentVersion,
    #[serde(default, rename = "parentCommentId")]
    pub parent_comment_id: Option<String>,
    #[serde(default, rename = "resolutionStatus")]
    pub resolution_status: Option<String>,
    #[serde(default)]
    pub body: Option<ConfPageBody>,  // reuses existing ConfPageBody for ADF
}

#[derive(Debug, Deserialize)]
pub struct ConfCommentVersion {
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "authorId")]
    pub author_id: String,
}

// For list_spaces():
#[derive(Debug, Deserialize)]
struct ConfSpaceSummaryList {
    results: Vec<ConfSpaceSummary>,
    #[serde(default, rename = "_links")]
    links: Option<ConfLinks>,
}

#[derive(Debug)]
pub struct ConfSpaceSummary {
    pub key: String,
    pub name: String,
    pub webui_url: String,  // built from _links.webui
}

// Internal: the raw serde shape for a single space in list_spaces() response
#[derive(Debug, Deserialize)]
struct ConfSpaceRaw {
    key: String,
    name: String,
    #[serde(default, rename = "_links")]
    links: serde_json::Value,
}
```

**`list_comments()` pagination pattern** — copy shape from `list_issues_impl` (lines 546–629):

```rust
// In impl ConfluenceBackend (NOT on IssueBackend trait)
pub async fn list_comments(&self, page_id: u64) -> Result<Vec<ConfComment>> {
    // Fetch both inline-comments and footer-comments; merge and return.
    let mut out = Vec::new();
    for kind in &["inline-comments", "footer-comments"] {
        let first = format!(
            "{}/wiki/api/v2/pages/{}/{}?limit={}&body-format=atlas_doc_format",
            self.base(), page_id, kind, PAGE_SIZE
        );
        let mut next_url: Option<String> = Some(first);
        let mut count: usize = 0;
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        while let Some(url) = next_url.take() {
            count += 1;
            if count > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
                tracing::warn!(
                    page_id,
                    kind,
                    "reached MAX_ISSUES_PER_LIST cap on comments; stopping"
                );
                break;
            }
            self.await_rate_limit_gate().await;
            let resp = self
                .http
                .request_with_headers(Method::GET, url.as_str(), &header_refs)
                .await?;
            self.ingest_rate_limit(&resp);
            let status = resp.status();
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "confluence returned {status} for GET {}: {}",
                    redact_url(&url),
                    String::from_utf8_lossy(&bytes)
                )));
            }
            let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
            let next_cursor = parse_next_cursor(&body_json);
            let list: ConfCommentList = serde_json::from_value(body_json)?;
            for comment in list.results {
                out.push(comment);
                if out.len() >= MAX_ISSUES_PER_LIST {
                    return Ok(out);
                }
            }
            next_url = next_cursor.map(|relative| {
                if relative.starts_with("http://") || relative.starts_with("https://") {
                    relative
                } else {
                    format!("{}{}", self.base(), relative)
                }
            });
        }
    }
    Ok(out)
}
```

Key reused helpers (do NOT re-implement):
- `self.standard_headers()` — lines 640–649
- `parse_next_cursor(&body_json)` — lines 309–314
- `self.await_rate_limit_gate().await` — lines 706–724
- `self.ingest_rate_limit(&resp)` — lines 678–704
- `redact_url(&url)` — lines 427–438

**`list_spaces()` pattern** — copy shape from `resolve_space_id` (lines 736–782) but without `keys=` filter and with pagination:

```rust
pub async fn list_spaces(&self) -> Result<Vec<ConfSpaceSummary>> {
    let first = format!("{}/wiki/api/v2/spaces?limit=250", self.base());
    let mut next_url: Option<String> = Some(first);
    let mut out: Vec<ConfSpaceSummary> = Vec::new();
    let header_owned = self.standard_headers();
    let header_refs: Vec<(&str, &str)> =
        header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
    while let Some(url) = next_url.take() {
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::GET, url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for GET {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
        let next_cursor = parse_next_cursor(&body_json);
        // ... deserialize results, push to out ...
        next_url = next_cursor.map(|rel| {
            if rel.starts_with("http://") || rel.starts_with("https://") {
                rel
            } else {
                format!("{}{}", self.base(), rel)
            }
        });
    }
    Ok(out)
}
```

**Wiremock test pattern** — copy from `list_resolves_space_key_and_fetches_pages` and `list_paginates_via_links_next` (lines 1210–1275):

```rust
// At top of #[cfg(test)] mod tests:
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use serde_json::json;

fn creds() -> ConfluenceCreds { /* ... same as existing creds() helper line 1135 */ }

#[tokio::test]
async fn list_comments_returns_inline_and_footer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/98765/inline-comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ /* ... */ }],
            "_links": {}
        })))
        .mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/98765/footer-comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ /* ... */ }],
            "_links": {}
        })))
        .mount(&server).await;
    let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
    let comments = backend.list_comments(98765).await.expect("comments");
    assert_eq!(comments.len(), 2);
}
```

---

### `crates/reposix-fuse/src/inode.rs` — add `COMMENTS_DIR_INO_BASE` + `COMMENTS_FILE_INO_BASE` (config)

**Analog:** the same file — `LABELS_DIR_INO_BASE` and `LABELS_SYMLINK_INO_BASE` (lines 98–104) and the `fixed_inodes_are_disjoint_from_dynamic_ranges` test (lines 267–307)

**New constants pattern** (after `LABELS_SYMLINK_INO_BASE`, ~line 104):

```rust
// Source: inode.rs lines 98-104 — copy this exact pattern, increment bases
/// Start of the per-page `.comments/` directory inode range.
///
/// One `CommentsDir` inode allocated per page on first access. Range:
/// `COMMENTS_DIR_INO_BASE .. COMMENTS_FILE_INO_BASE`.
/// 4-billion slot window — vastly exceeds the 500-page cap.
pub const COMMENTS_DIR_INO_BASE: u64 = 0x18_0000_0000;

/// Start of the per-comment file inode range.
///
/// One `CommentFile` inode allocated per comment. Range:
/// `COMMENTS_FILE_INO_BASE .. u64::MAX` (in practice far less).
pub const COMMENTS_FILE_INO_BASE: u64 = 0x1C_0000_0000;
```

**Module doc update** — extend the layout table (lines 6–21) to include the two new ranges.

**Disjoint assertions** — extend `fixed_inodes_are_disjoint_from_dynamic_ranges` test (lines 267–307):

```rust
// Add after the existing labels assertions (after line 306):
assert!(COMMENTS_DIR_INO_BASE > LABELS_SYMLINK_INO_BASE,
    "COMMENTS_DIR_INO_BASE must be above LABELS_SYMLINK_INO_BASE");
assert!(COMMENTS_DIR_INO_BASE < COMMENTS_FILE_INO_BASE,
    "COMMENTS_DIR_INO_BASE must be below COMMENTS_FILE_INO_BASE");
```

---

### `crates/reposix-fuse/src/comments.rs` — NEW: `CommentsSnapshot`, `render_comment_file()` (service, event-driven/lazy)

**Analog:** `crates/reposix-fuse/src/labels.rs` (`LabelSnapshot` — full file, lines 1–312)

**File-level structure pattern** (copy from `labels.rs` lines 1–55):

```rust
//! Pure in-memory comment overlay for the Phase-23 `.comments/` per-page dirs.
//!
//! Mirrors `labels.rs` but with lazy per-page fetching instead of a bulk build.
//!
//! # Security
//!
//! T-23-01: comment body text is tainted content; never logged.
//! T-23-02: comment_id and page_id are validated numeric before use in filenames.

#![allow(clippy::module_name_repetitions)]

use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;
use chrono::{DateTime, Utc};
pub use crate::inode::{COMMENTS_DIR_INO_BASE, COMMENTS_FILE_INO_BASE};
```

**`CommentEntry` struct pattern** (copy structure from `LabelEntry`, lines 27–35):

```rust
/// A single comment file entry inside a `.comments/` directory.
#[derive(Debug, Clone)]
pub struct CommentEntry {
    /// Inode for this comment file.
    pub file_ino: u64,
    /// Filename: `<comment-id>.md`
    pub filename: String,
    /// Rendered bytes (YAML frontmatter + Markdown body).
    pub rendered: Vec<u8>,
}
```

**`CommentsSnapshot` struct** — the lazy-cache design described in RESEARCH.md Pattern 1. There is no exact analog in the codebase since `LabelSnapshot` is built eagerly from `issues: &[Issue]`. The key difference: comments are fetched per-page on demand. Use `DashMap` throughout (same as `InodeRegistry` uses `DashMap`, lines 109–112 of `inode.rs`):

```rust
/// Lazy per-page comment cache. Populated on first `CommentsDir` access per page.
///
/// Uses `DashMap` throughout so FUSE callbacks (which run on fuser threads)
/// can write without holding a coarse write lock.
#[derive(Debug, Default)]
pub struct CommentsSnapshot {
    /// page_ino → (fetched: bool, comments: Vec<CommentEntry>)
    by_page: DashMap<u64, (bool, Vec<CommentEntry>)>,
    /// page_ino → CommentsDir inode (lazily allocated on first lookup)
    page_to_dir_ino: DashMap<u64, u64>,
    /// CommentsDir inode → page_ino (reverse map for getattr/readdir)
    dir_ino_to_page: DashMap<u64, u64>,
    /// comment file_ino → page_ino (for getattr/read dispatch)
    file_ino_to_page: DashMap<u64, u64>,
    /// Monotonic allocator for CommentsDir inodes.
    next_dir_ino: AtomicU64,
    /// Monotonic allocator for CommentFile inodes.
    next_file_ino: AtomicU64,
}
```

**`render_comment_file()` pure fn pattern** — copy body-rendering approach from `translate()` in `reposix-confluence/src/lib.rs` lines 342–416 (ADF to markdown) and frontmatter emission from existing issue rendering in `reposix-core`:

```rust
/// Render a `ConfComment` as a YAML-frontmatter Markdown file.
///
/// Body conversion uses `adf::adf_to_markdown`; unknown ADF degrades to empty string.
/// The body is appended AFTER the closing `---\n\n` so attacker-controlled body text
/// cannot inject into the frontmatter (YAML fence injection mitigation).
///
/// # Arguments
/// - `comment` — the raw comment from the Confluence API.
/// - `kind` — `"inline"` or `"footer"` (determines `resolved` semantics).
pub fn render_comment_file(
    comment: &reposix_confluence::ConfComment,
    kind: &str,
) -> Vec<u8> {
    // ... YAML frontmatter then body after closing ---
}
```

**Unit test pattern** (copy from `labels.rs` lines 149–312):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_comment_file_produces_valid_frontmatter() { /* ... */ }

    #[test]
    fn comments_dir_ino_constants_ordering() {
        assert!(COMMENTS_DIR_INO_BASE > crate::inode::LABELS_SYMLINK_INO_BASE);
        assert!(COMMENTS_DIR_INO_BASE < COMMENTS_FILE_INO_BASE);
    }
}
```

---

### `crates/reposix-fuse/src/fs.rs` — add `InodeKind::CommentsDir` + `InodeKind::CommentFile` dispatch (controller, request-response)

**Analog:** the same file — `InodeKind::LabelDir` / `InodeKind::LabelSymlink` arms, which are the direct precedent for a new two-variant synthesized overlay. Lines 486–540 (enum + classify), 1167–1192 (getattr), 1321–1350 (lookup), 1491–1508 (readdir), 1572–1576 (read).

**`InodeKind` enum additions** (after `LabelSymlink` at line 512):

```rust
// Source: fs.rs lines 486-515 — add two variants at end (before Unknown)
/// A synthesized `.comments/` directory for a specific page.
/// `COMMENTS_DIR_INO_BASE..COMMENTS_FILE_INO_BASE`.
CommentsDir,
/// A read-only comment file inside a `.comments/` directory.
/// `COMMENTS_FILE_INO_BASE..`.
CommentFile,
```

**`classify()` additions** (after the `LabelSymlink` arm at line 533):

```rust
// Source: fs.rs lines 517-540 — CRITICAL ordering: comment ranges must appear
// BEFORE LabelSymlink catch-all since they are numerically above it.
// Copy the ordering comment from the labels section (lines 528-532).
n if n >= COMMENTS_FILE_INO_BASE => Self::CommentFile,
n if n >= COMMENTS_DIR_INO_BASE => Self::CommentsDir,
// existing: n if n >= LABELS_SYMLINK_INO_BASE => Self::LabelSymlink,
```

**`ReposixFs` struct field additions** (after `label_snapshot` at line 587):

```rust
// Source: fs.rs lines 583-630 — same Arc<RwLock<...>> pattern as label_snapshot
/// Comment overlay snapshot. Lazily populated per page on first `.comments/`
/// access. `None` when backend does not support comments (sim/github).
comment_snapshot: Option<Arc<CommentsSnapshot>>,
/// The `ConfluenceBackend` needed for `list_comments()` calls.
/// Stored separately because `list_comments` is NOT on `IssueBackend` trait.
/// `None` for sim/github backends.
confluence_backend: Option<Arc<reposix_confluence::ConfluenceBackend>>,
```

**`getattr` dispatch additions** (after `LabelSymlink` arm at lines 1181–1192):

```rust
// Source: fs.rs lines 1167-1192 (LabelDir / LabelSymlink getattr)
// Copy pattern: read lock on snapshot, look up by ino, build attr.
InodeKind::CommentsDir => {
    if let Some(snap) = &self.comment_snapshot {
        if snap.dir_ino_to_page.contains_key(&ino_u) {
            let mut attr = self.labels_attr;  // reuse read-only dir attr shape
            attr.ino = INodeNo(ino_u);
            reply.attr(&ATTR_TTL, &attr);
        } else {
            reply.error(fuser::Errno::from_i32(libc::ENOENT));
        }
    } else {
        reply.error(fuser::Errno::from_i32(libc::ENOENT));
    }
}
InodeKind::CommentFile => {
    if let Some(snap) = &self.comment_snapshot {
        // look up page_ino from file_ino_to_page, then get rendered bytes length
        // ... reply.attr with synthetic_file_attr(ino_u, size)
    } else {
        reply.error(fuser::Errno::from_i32(libc::ENOENT));
    }
}
```

**`lookup` Bucket arm addition** — MUST come BEFORE `validate_issue_filename` call (lines 1254–1276). The RESEARCH.md Pitfall 1 governs this ordering:

```rust
// Source: fs.rs lines 1254-1276 (InodeKind::Bucket lookup arm)
// Insert this block AFTER the BUCKET_INDEX_FILENAME check and BEFORE
// the validate_issue_filename call:
if let Some(page_id_str) = name_str.strip_suffix(".comments") {
    if let Ok(page_id) = page_id_str.trim_start_matches('0').parse::<u64>() {
        // look up the page inode by page_id via registry.lookup_id(IssueId(page_id))
        // then allocate or retrieve CommentsDir inode from comment_snapshot
        // reply.entry(&ENTRY_TTL, &dir_attr, fuser::Generation(0))
        return;
    }
    reply.error(fuser::Errno::from_i32(libc::ENOENT));
    return;
}
// existing: if validate_issue_filename(name_str).is_err() { ... }
```

**`lookup` CommentsDir arm addition** (after `LabelDir` arm at lines 1321–1340):

```rust
// Source: fs.rs lines 1321-1340 (InodeKind::LabelDir lookup arm)
// Copy pattern: look up filename in entries, reply with file attr.
InodeKind::CommentsDir => {
    // fetch comments lazily if not yet loaded (runtime.block_on(confluence_backend.list_comments(...)))
    // find entry by filename, reply with synthetic_file_attr
}
```

**`readdir` CommentsDir addition** (after `LabelDir` arm at lines 1491–1508):

```rust
// Source: fs.rs lines 1491-1508 (InodeKind::LabelDir readdir)
// Copy pattern exactly: dot/dotdot, then iterate entries.
InodeKind::CommentsDir => {
    let mut entries: Vec<(u64, FileType, String)> = vec![
        (ino_u, FileType::Directory, ".".to_owned()),
        // use BUCKET_DIR_INO as ".." (CommentsDir parent is the Bucket)
        (BUCKET_DIR_INO, FileType::Directory, "..".to_owned()),
    ];
    if let Some(snap) = &self.comment_snapshot {
        // ... push (file_ino, FileType::RegularFile, filename) for each comment
    }
    entries
}
```

**`read` CommentFile addition** (after `BucketIndex` arm at lines 1564–1570):

```rust
// Source: fs.rs lines 1578-1601 (InodeKind::RealFile read)
// Copy the offset/size slice pattern exactly.
InodeKind::CommentFile => {
    // retrieve rendered bytes from CommentsSnapshot
    // reply.data(&bytes[start..end])
}
```

**`read` and `write` rejection for CommentFile** — add `InodeKind::CommentFile` to the `EISDIR`/`EROFS` arms in `create`/`write`/`release` callbacks so comment files are read-only (CONF-03).

---

### `crates/reposix-cli/src/spaces.rs` — NEW: `spaces::run()` table rendering (CLI output, request-response)

**Analog:** `crates/reposix-cli/src/list.rs` — full file, especially `run()` (lines 62–116) and `render_table()` (lines 118–136)

**Module structure pattern** (copy from `list.rs` lines 1–25):

```rust
//! `reposix spaces` — list all readable Confluence spaces as a table.
//!
//! Calls `ConfluenceBackend::list_spaces()` and renders key + name + URL.

use anyhow::{Context, Result};
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
```

**`run()` function pattern** (copy from `list.rs::run()` lines 62–116):

```rust
// Source: list.rs lines 62-116
pub async fn run(origin: String) -> Result<()> {
    // read_confluence_env() — identical to list.rs; expose from reposix_cli::list
    // or duplicate the small helper here.
    let (email, token, tenant) = crate::list::read_confluence_env()
        .context("spaces requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT")?;
    let creds = ConfluenceCreds { email, api_token: token };
    let b = ConfluenceBackend::new(creds, &tenant).context("build ConfluenceBackend")?;
    let spaces = b.list_spaces().await.with_context(|| {
        format!("list_spaces (REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net)")
    })?;
    render_spaces_table(&spaces);
    Ok(())
}
```

**`render_spaces_table()` pattern** (copy from `list.rs::render_table()` lines 118–136):

```rust
// Source: list.rs lines 118-136 — same fixed-width column approach
fn render_spaces_table(spaces: &[reposix_confluence::ConfSpaceSummary]) {
    let key_col = "KEY";
    let name_col = "NAME";
    println!("{key_col:<12} {name_col:<30} URL");
    println!("{} {} {}", "------------", "------------------------------", "---");
    for s in spaces {
        println!("{:<12} {:<30} {}", s.key, s.name, s.webui_url);
    }
}
```

**NOTE:** `read_confluence_env` is private in `list.rs`. Either make it `pub(crate)` or duplicate it in `spaces.rs`. Prefer `pub(crate)` — cleaner, avoids duplication.

---

### `crates/reposix-cli/src/main.rs` — add `Cmd::Spaces` arm (CLI dispatch, request-response)

**Analog:** the same file — `Cmd::List` arm (lines 106–123, 181–187)

**Enum variant pattern** (copy from `Cmd::List`, lines 98–123):

```rust
// Source: main.rs lines 98-123 (Cmd::List variant)
/// List all readable Confluence spaces.
///
/// Requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT
/// and REPOSIX_ALLOWED_ORIGINS including the tenant origin.
Spaces {
    /// Sim/github backends not applicable; flag reserved for future backends.
    #[arg(long, default_value = "http://127.0.0.1:7878")]
    origin: String,
},
```

**`mod spaces;` declaration** (after `mod list;` reference in `use` block, around line 30):

```rust
// Source: main.rs lines 24-31 — add alongside existing module declarations
mod spaces;
```

**Dispatch arm pattern** (copy from `Cmd::List` dispatch at lines 181–187):

```rust
// Source: main.rs lines 181-187
Cmd::Spaces { origin } => spaces::run(origin).await,
```

---

## Shared Patterns

### Confluence HTTP method shape (standard_headers + await_rate_limit + ingest_rate_limit)
**Source:** `crates/reposix-confluence/src/lib.rs` lines 558–582, 640–649, 678–724
**Apply to:** `list_comments()` and `list_spaces()` — both are GET endpoints on `ConfluenceBackend`

```rust
// ALWAYS use this three-part call sequence for any new GET on ConfluenceBackend:
let header_owned = self.standard_headers();
let header_refs: Vec<(&str, &str)> =
    header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
// ...
self.await_rate_limit_gate().await;
let resp = self.http.request_with_headers(Method::GET, url.as_str(), &header_refs).await?;
self.ingest_rate_limit(&resp);
```

### Pagination cursor extraction + SSRF-safe URL construction
**Source:** `crates/reposix-confluence/src/lib.rs` lines 309–314 (`parse_next_cursor`) and lines 616–626 (relative-prepend)
**Apply to:** `list_comments()` and `list_spaces()` — both paginate via `_links.next`

```rust
// Source: lib.rs lines 595, 616-626
let next_cursor = parse_next_cursor(&body_json);
// ...
next_url = next_cursor.map(|relative| {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        relative
    } else {
        format!("{}{}", self.base(), relative)
    }
});
```

### 500-item cap + `tracing::warn!` (HARD-02 compliance)
**Source:** `crates/reposix-confluence/src/lib.rs` lines 564–575
**Apply to:** `list_comments()` — same loop shape, same cap, same warn

```rust
// Source: lib.rs lines 564-575
if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
    tracing::warn!(
        pages,
        "reached MAX_ISSUES_PER_LIST cap; stopping pagination"
    );
    break;
}
```

### DashMap-based lazy per-kind state on `ReposixFs`
**Source:** `crates/reposix-fuse/src/fs.rs` lines 583–631 (`label_snapshot`, `tree_dir_index_cache`, `tree_index_inodes`)
**Apply to:** `CommentsSnapshot` fields in `ReposixFs`

```rust
// Pattern: Arc<RwLock<...>> for snapshots rebuilt on refresh,
//          DashMap<u64, ...> for lazily-populated per-inode caches.
// label_snapshot uses Arc<RwLock<LabelSnapshot>> — CommentsSnapshot follows same.
comment_snapshot: Option<Arc<CommentsSnapshot>>,
```

### `InodeKind` ordering constraint (new ranges must appear before existing catch-alls)
**Source:** `crates/reposix-fuse/src/fs.rs` lines 528–532 (comment explaining label ordering)
**Apply to:** `classify()` additions for `CommentsDir` and `CommentFile`

```rust
// Source: fs.rs lines 528-532 — copy this comment and extend it:
// Comment ranges MUST appear before the LabelSymlink/LabelDir catch-alls because
// COMMENTS_DIR_INO_BASE (0x18_0000_0000) and COMMENTS_FILE_INO_BASE (0x1C_0000_0000)
// are numerically above LABELS_SYMLINK_INO_BASE (0x14_0000_0000).
n if n >= COMMENTS_FILE_INO_BASE => Self::CommentFile,
n if n >= COMMENTS_DIR_INO_BASE => Self::CommentsDir,
n if n >= LABELS_SYMLINK_INO_BASE => Self::LabelSymlink,
```

### Tainted content / no body text in logs (SG-05 / T-16-C-04)
**Source:** `crates/reposix-confluence/src/lib.rs` lines 381–403 (IN-01 cap on attacker strings in tracing)
**Apply to:** any `tracing::warn!` / `tracing::debug!` in `list_comments()` and `CommentsSnapshot`

```rust
// NEVER: tracing::warn!(body = %comment.body, ...)
// ALWAYS: only log page_id, comment_id (numeric), kind ("inline"/"footer"), count
tracing::warn!(page_id, kind, count = out.len(), "comments cap reached");
```

### ADF body conversion for comment bodies
**Source:** `crates/reposix-confluence/src/lib.rs` lines 349–365 (`translate()` ADF branch)
**Apply to:** `render_comment_file()` in `comments.rs`

```rust
// Source: lib.rs lines 349-365 — copy this exact degradation pattern
let body_md = match crate::adf::adf_to_markdown(&adf_value) {
    Ok(md) => md,
    Err(e) => {
        tracing::warn!(error = %e, "adf_to_markdown failed on comment; using empty body");
        String::new()
    }
};
```

### WR-01/WR-02 numeric-only validation for URL path components
**Source:** `crates/reposix-confluence/src/lib.rs` lines 776–780 (`resolve_space_id` WR-02 guard)
**Apply to:** `list_comments(page_id: u64)` — `page_id` is already `u64` so the type system handles WR-01. `parentCommentId` and `comment.id` embedded in filenames must be validated as numeric strings before use.

```rust
// Source: lib.rs lines 776-780
if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
    return Err(Error::Other(format!("malformed comment id from server: {id:?}")));
}
```

---

## No Analog Found

All new files have direct analogs. The following design choices have no close codebase analog (use RESEARCH.md patterns):

| Aspect | Reason |
|--------|--------|
| Lazy per-page fetch on first `readdir(CommentsDir)` | `LabelSnapshot` is built eagerly from a `&[Issue]` slice; there is no precedent for a lazily-fetched FUSE overlay that triggers backend I/O on first directory access. The `CommentsSnapshot` design is novel to this phase. |
| `confluence_backend: Option<Arc<ConfluenceBackend>>` on `ReposixFs` | No existing field stores a concrete backend type alongside the `Arc<dyn IssueBackend>`. This is the simplest pattern per RESEARCH.md Open Question 1 option (a). |
| YAML frontmatter format for comment files | Page files render via `reposix-core`'s `frontmatter` module; comment files introduce new frontmatter fields (`kind`, `resolved`, `parent_comment_id`). No existing comment frontmatter template exists. |

---

## Metadata

**Analog search scope:** `crates/reposix-fuse/src/`, `crates/reposix-confluence/src/`, `crates/reposix-cli/src/`
**Files scanned:** 11 source files read directly
**Pattern extraction date:** 2026-04-16
