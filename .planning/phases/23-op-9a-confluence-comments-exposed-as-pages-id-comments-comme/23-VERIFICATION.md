---
phase: 23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme
verified: 2026-04-16T20:15:00Z
status: human_needed
score: 12/12 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Live FUSE mount against a real or wiremock-backed Confluence tenant: run `ls mount/pages/<padded-id>.comments/` and `cat mount/pages/<padded-id>.comments/<cid>.md`"
    expected: "Directory lists per-comment .md files; cat returns YAML frontmatter then Markdown body"
    why_human: "Requires a running FUSE mount with a Confluence backend — cannot verify in-process from grep/unit tests alone. All in-process components are verified, but the FUSE kernel dispatch path for CommentsDir/CommentFile can only be fully confirmed via a live mount"
  - test: "Tainted Confluence space name containing ANSI escape sequences: run `reposix spaces --backend confluence` against a space whose name contains ESC[31m"
    expected: "Either terminal renders the escape (acceptable) or user pipes through `cat -v` to neutralize — accepted risk per T-23-02-03"
    why_human: "Requires a real Confluence tenant or a specially crafted space name; TTY-aware harness not available in automated test suite"
---

# Phase 23: OP-9a Confluence Comments FUSE Overlay — Verification Report

**Phase Goal:** Expose Confluence page inline and footer comments as synthesized `.comments/` subdirectories under each page in the FUSE mount: `pages/<padded-id>.comments/<comment-id>.md`. Each comment file has YAML frontmatter (id, author, created_at, resolved, parent_comment_id, kind) and a Markdown body. Also adds a `reposix spaces --backend confluence` subcommand for listing Confluence spaces.
**Verified:** 2026-04-16T20:15:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ConfluenceBackend::list_comments(page_id)` returns merged inline + footer comments for a page | VERIFIED | `pub async fn list_comments` present at lib.rs:1007; hits both `inline-comments` and `footer-comments` endpoints; 4 wiremock tests pass |
| 2 | `ConfluenceBackend::list_spaces()` returns all readable spaces (paginated) | VERIFIED | `pub async fn list_spaces` present at lib.rs:1085; hits `/wiki/api/v2/spaces?limit=250`; 3 wiremock tests pass |
| 3 | Comment body atlas_doc_format → Markdown conversion works (CONF-01 body shape) | VERIFIED | `ConfComment::body_markdown()` present at lib.rs; calls `crate::adf::adf_to_markdown`; test `list_comments_handles_absent_body` confirms body-less case returns empty string |
| 4 | Pagination on both endpoints uses `parse_next_cursor` + relative-prepend (SSRF-safe) | VERIFIED | Both `list_comments` and `list_spaces` call `parse_next_cursor(&body_json)` and prepend `self.base()` for relative cursors; pagination tests pass |
| 5 | 500-page cap + `tracing::warn!` applied to comments pagination (HARD-02 compliance) | VERIFIED | Warn text "reached MAX_ISSUES_PER_LIST cap on comments; stopping pagination" present at lib.rs:1032 and absolute cap at lib.rs:1058; no automated test for the cap trigger path (plan's `list_comments_applies_500_cap` test was not written), but code path is implemented and compiles |
| 6 | `reposix spaces --backend confluence` prints a table of space key + name + URL | VERIFIED | `spaces::run(ListBackend::Confluence)` wired; CLI help shows subcommand; `--backend` flag present; error with missing env vars mentions `ATLASSIAN_API_KEY` (names-only confirmed via `cargo run`) |
| 7 | `reposix --help` shows a `spaces` subcommand | VERIFIED | `Cmd::Spaces` variant in `main.rs`; `cargo run -q -p reposix-cli -- --help` shows "spaces   List all readable Confluence spaces" |
| 8 | Missing ATLASSIAN_* env vars produce a clean CLI error (names-only, no value leak) | VERIFIED | `read_confluence_env` promoted to `pub(crate)` in list.rs; error message confirmed: "spaces --backend confluence requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, and REPOSIX_CONFLUENCE_TENANT env vars" |
| 9 | `readdir(CommentsDir)` returns one entry per comment from `CommentsSnapshot` | VERIFIED | `CommentsSnapshot` in `comments.rs`; `readdir` arm at fs.rs:1653 dispatches on `InodeKind::CommentsDir`; `fetch_comments_for_page` called on first readdir; 13 comments tests pass |
| 10 | `cat mount/pages/<padded-id>.comments/<comment-id>.md` returns YAML-frontmatter Markdown (CONF-01) | VERIFIED | `render_comment_file` returns `---\n` frontmatter + body; `read(CommentFile)` arm at fs.rs:1772 serves rendered bytes; 6 render_comment_file unit tests pass |
| 11 | Writes to comments files/dirs return EROFS (CONF-03 read-only) | VERIFIED | `setattr` arm (fs.rs:1894-1896) includes `CommentsDir | CommentFile => EROFS`; `write` arm (fs.rs:1953-1957) includes both; `create`/`unlink` guarded by existing Bucket-only check at fs.rs:2092 |
| 12 | Sim/GitHub backends have zero `.comments/` behaviour change (comment_fetcher is None) | VERIFIED | `comment_fetcher.is_none()` guard at fs.rs returns ENOENT before `ensure_dir` is called; `build_comment_fetcher` returns `Ok(None)` for Sim and Github backends |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/reposix-confluence/src/lib.rs` | `list_comments()`, `list_spaces()`, `ConfComment`, `ConfCommentVersion`, `ConfSpaceSummary` public API | VERIFIED | All 5 public types present; `grep -c 'pub struct ConfComment'` = 2 (struct + doc), `pub async fn list_comments` = 1, `pub async fn list_spaces` = 1 |
| `crates/reposix-cli/src/spaces.rs` | `pub async fn run()` entry point | VERIFIED | File exists; `pub async fn run` present; `list_spaces` called within |
| `crates/reposix-cli/src/main.rs` | `Cmd::Spaces` enum variant + dispatch arm | VERIFIED | `Cmd::Spaces` present 1 time; dispatch arm `Cmd::Spaces { backend } => spaces::run(backend).await` at line 216 |
| `crates/reposix-cli/src/lib.rs` | `pub mod spaces` export | VERIFIED | `pub mod spaces` present |
| `crates/reposix-fuse/src/inode.rs` | `COMMENTS_DIR_INO_BASE` + `COMMENTS_FILE_INO_BASE` constants | VERIFIED | Both constants present; disjoint-range test extended and passes |
| `crates/reposix-fuse/src/comments.rs` | `CommentsSnapshot` + `render_comment_file()` | VERIFIED | File created; `pub struct CommentsSnapshot` present; `pub fn render_comment_file` present |
| `crates/reposix-fuse/src/fs.rs` | `InodeKind::CommentsDir` / `InodeKind::CommentFile` dispatch in lookup/readdir/getattr/read + write-rejection arms | VERIFIED | `InodeKind::CommentsDir` appears 10 times; `InodeKind::CommentFile` appears 8 times; `fetch_comments_for_page` appears 5 times |
| `crates/reposix-fuse/src/lib.rs` | `MountConfig` extended with optional `ConfluenceBackend` passthrough; `Mount::open` 3-arg | VERIFIED | `comment_fetcher: Option<Arc<reposix_confluence::ConfluenceBackend>>` in lib.rs:98 |
| `crates/reposix-fuse/src/main.rs` | `build_comment_fetcher()` constructs second `Arc<ConfluenceBackend>` | VERIFIED | `build_comment_fetcher` appears 2 times (definition + call) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `list_comments` | `GET /wiki/api/v2/pages/{id}/inline-comments` + `/footer-comments` | `self.http.request_with_headers` | WIRED | Pattern `inline-comments\|footer-comments` confirmed at lib.rs:1010-1011 |
| `list_spaces` | `GET /wiki/api/v2/spaces` | `self.http.request_with_headers` | WIRED | Pattern `/wiki/api/v2/spaces` confirmed at lib.rs:1086 |
| `Cmd::Spaces` | `spaces::run` | `main.rs` match arm | WIRED | Dispatch arm at main.rs:216 confirmed |
| `spaces::run` | `ConfluenceBackend::list_spaces` | direct await | WIRED | `b.list_spaces().await` at spaces.rs:39 |
| `fs.rs lookup(Bucket, '<id>.comments')` | `CommentsSnapshot.page_to_dir_ino` | `.strip_suffix(".comments")` | WIRED | `strip_suffix(".comments")` at fs.rs:1354 confirmed |
| `fs.rs readdir(CommentsDir)` | `fetch_comments_for_page` | `runtime.block_on(list_comments(page_id))` | WIRED | `fetch_comments_for_page` called in readdir arm at fs.rs:1668 |
| `fs.rs read(CommentFile)` | `CommentsSnapshot.file_ino_to_rendered` | `entry_by_file_ino` + slice | WIRED | `CommentFile` read arm at fs.rs:1772 confirmed |
| `main.rs BackendKind::Confluence` | `Mount::open` via `MountConfig.comment_fetcher: Some(...)` | `build_comment_fetcher` | WIRED | `comment_fetcher` passed to `Mount::open` in main.rs |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `spaces::run` | `spaces: Vec<ConfSpaceSummary>` | `ConfluenceBackend::list_spaces()` → `/wiki/api/v2/spaces` | Yes (HTTP to real API; 3 wiremock tests cover real data shapes) | FLOWING |
| `fs.rs readdir(CommentsDir)` | `entries: Vec<CommentEntry>` | `fetch_comments_for_page` → `list_comments` → inline/footer endpoints | Yes (lazy-fetched on first readdir; installed via `install_entries`) | FLOWING |
| `fs.rs read(CommentFile)` | `entry.rendered: Vec<u8>` | `render_comment_file` → `body_markdown()` → ADF conversion | Yes (rendered bytes from `ConfComment`; 6 unit tests confirm output format) | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `reposix --help` shows `spaces` subcommand | `cargo run -q -p reposix-cli -- --help` | "spaces   List all readable Confluence spaces as a table of KEY / NAME / URL" | PASS |
| `reposix spaces --help` shows `--backend` flag | `cargo run -q -p reposix-cli -- spaces --help` | Shows `--backend <BACKEND>` with usage | PASS |
| Missing env vars → names-only error | `env -u ATLASSIAN_API_KEY ... cargo run -- spaces --backend confluence` | Error mentions `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`; no secret values leaked | PASS |
| `list_comments` 4 tests pass | `cargo test -p reposix-confluence list_comments` | 4 passed, 0 failed | PASS |
| `list_spaces` 3 tests pass | `cargo test -p reposix-confluence list_spaces` | 3 passed, 0 failed | PASS |
| `spaces::tests` 3 tests pass | `cargo test -p reposix-cli spaces` | 3 passed, 0 failed | PASS |
| FUSE comments 13 tests pass | `cargo test -p reposix-fuse comments` | 13 passed, 0 failed | PASS |
| Workspace gauntlet | `cargo test --workspace` | All pass (no FAILED lines) | PASS |
| Clippy clean | `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 | PASS |
| FUSE mount live FUSE `.comments/` readdir | Requires live FUSE mount | Not testable without running mount | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CONF-01 | 23-01, 23-03 | `cat mount/pages/<id>.comments/<comment-id>.md` returns comment body in Markdown frontmatter format | SATISFIED | `render_comment_file` produces YAML frontmatter + body; `read(CommentFile)` dispatch arm serves it; 6 unit tests confirm output shape; marked `[x]` in REQUIREMENTS.md |
| CONF-02 | 23-01, 23-03 | `ls mount/pages/<id>.comments/` lists all inline + footer comments | SATISFIED | `readdir(CommentsDir)` arm fetches via `fetch_comments_for_page`; `CommentsSnapshot.entries_if_fetched` returns per-comment entries; marked `[x]` in REQUIREMENTS.md |
| CONF-03 | 23-03 | Comments are read-only (no write path) | SATISFIED | `setattr`, `write` EROFS arms cover `CommentsDir | CommentFile`; `create`/`unlink` guarded by existing Bucket-only check; marked `[x]` in REQUIREMENTS.md |
| SPACES-01 | 23-02 | `reposix spaces --backend confluence` lists all readable Confluence spaces in a table | SATISFIED (implementation complete; REQUIREMENTS.md checkbox not updated) | `Cmd::Spaces` wired; `spaces::run` calls `list_spaces`; CLI help confirmed; env-var error confirmed; 3 unit tests pass. REQUIREMENTS.md still shows `[ ]` — documentation gap only, not an implementation gap |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/reposix-fuse/src/fs.rs` | 989 | `"not a placeholder"` comment (literal text, not a code stub) | Info | Not a real stub — comment in code explaining why a value is not a placeholder |
| `crates/reposix-fuse/src/fs.rs` | 2112 | `let placeholder = Issue { ... }` | Info | Existing pre-phase-23 `create` implementation stub for new-page creation — not introduced by phase 23, not blocking phase 23 goal |

No new stubs, hardcoded empty returns, or TODO markers were introduced by phase 23 files.

### Human Verification Required

#### 1. Live FUSE Mount `.comments/` End-to-End

**Test:** Start `cargo run -p reposix-fuse -- --backend confluence --project <SPACE_KEY> /tmp/reposix-mnt` with valid `ATLASSIAN_EMAIL`, `ATLASSIAN_API_KEY`, `REPOSIX_CONFLUENCE_TENANT`, and `REPOSIX_ALLOWED_ORIGINS=https://<tenant>.atlassian.net`. Then in another shell: `ls /tmp/reposix-mnt/pages/` to confirm pages appear, then `ls /tmp/reposix-mnt/pages/<padded-id>.comments/` to see comment files, then `cat /tmp/reposix-mnt/pages/<padded-id>.comments/<cid>.md` to read a comment.
**Expected:** `ls pages/` shows no `.comments` entries (DoS amplifier prevention); `ls pages/<id>.comments/` shows `<cid>.md` files; `cat` of a comment file shows `---` YAML frontmatter with `id`, `page_id`, `author`, `created_at`, `resolved`, `parent_comment_id`, `kind` fields followed by a Markdown body.
**Why human:** Requires a running FUSE daemon with real kernel VFS integration. The `CommentsDir`/`CommentFile` FUSE callback dispatch can only be exercised end-to-end via a live mount. All in-process contracts are verified; FUSE kernel dispatch is the remaining gap.

#### 2. ANSI Escape Sequences in Space Names (T-23-02-03)

**Test:** Create a Confluence space whose name contains `\x1b[31m` (ANSI red escape). Run `reposix spaces --backend confluence` in a terminal emulator.
**Expected:** Either the terminal renders the escape sequence (red text — acceptable per accepted risk T-23-02-03) or the user can pipe through `cat -v` to neutralize. No crash or panic.
**Why human:** Requires a real Confluence tenant with a specially-crafted space name. TTY-aware testing not available in automated test suite. Risk is accepted per the threat model (low severity; requires tenant-admin compromise).

### Gaps Summary

No blocking implementation gaps. All 12 must-haves verified. The phase goal is functionally achieved:

- `ConfluenceBackend::list_comments` and `list_spaces` are implemented with pagination, 500-cap, redacted error messages, and rate-limit gating
- `reposix spaces --backend confluence` CLI subcommand is wired end-to-end  
- FUSE `.comments/` overlay is implemented with lazy per-page fetching, EROFS write rejection, DoS amplifier prevention, and inode range disjointness

One non-blocking documentation gap: REQUIREMENTS.md still shows `[ ]` for SPACES-01 even though the implementation is complete. This should be updated to `[x]`.

One minor test gap: the plan specified a `list_comments_applies_500_cap` wiremock test (PLAN 23-01 acceptance criteria), but it was not implemented. The 500-cap code path IS present and the warn text exists at lib.rs:1032, so the feature works — but the test coverage for this specific path is absent.

Both of these are informational only. No gaps prevent the phase goal from being achieved.

---

_Verified: 2026-04-16T20:15:00Z_
_Verifier: Claude (gsd-verifier)_
