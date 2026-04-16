---
phase: 23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme
reviewed: 2026-04-16T00:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-cli/src/spaces.rs
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/lib.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-fuse/src/comments.rs
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/lib.rs
  - crates/reposix-fuse/src/main.rs
  - crates/reposix-fuse/tests/readdir.rs
  - crates/reposix-fuse/tests/sim_death_no_hang.rs
  - crates/reposix-fuse/tests/nested_layout.rs
findings:
  critical: 0
  warning: 4
  info: 4
  total: 8
status: issues_found
---

# Phase 23: Code Review Report

**Reviewed:** 2026-04-16T00:00:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Phase 23 adds lazy per-page comment directories (`pages/<id>.comments/`) for the Confluence FUSE backend. The core design is sound: the `CommentsSnapshot` struct in `comments.rs` uses lock-free `DashMap` structures with atomic inode allocators, the path-traversal mitigations for non-numeric comment IDs are tested, the YAML injection defence (body after closing fence) is in place, and the inode range ordering is verified by compile-time-style assertions.

The security-sensitive pieces (T-23-03-01 through T-23-03-03) appear correctly implemented. No critical vulnerabilities were found.

Four warnings surface around correctness risks: an inode leak in `ensure_dir`'s TOCTOU race, an unguarded `trim_start_matches` that allows the empty-string page ID `"0"` to resolve to a real page (`0.comments`), a missing backend call for the `delete_or_close` CONFLICT branch in `update_issue`, and a comment-cache that is never invalidated across `refresh_issues` calls (documented but worth flagging). Four info items flag dead/stale code, a missing `.comments` entry in the `nested_layout` test, and cosmetic matters.

## Warnings

### WR-01: `ensure_dir` leaks an inode slot on every concurrent race

**File:** `crates/reposix-fuse/src/comments.rs:94-108`
**Issue:** `ensure_dir` reads `page_to_dir_ino`, finds nothing, then `fetch_add`s `next_dir_ino`, then tries a `DashMap::entry`. If two threads race between the read and the `fetch_add`, both threads increment the counter, but only one wins the `Vacant` arm. The losing thread's allocated inode is permanently consumed from the `COMMENTS_DIR_INO_BASE..COMMENTS_FILE_INO_BASE` range (4 billion slots) without being inserted into any map — it is unreachable and unrecoverable for the lifetime of the mount. This mirrors the same documented race in `InodeRegistry::intern`, but there the comment says "we accept the wasted inode." That acceptance is reasonable for `InodeRegistry` (u64 space) but was not explicitly acknowledged for `CommentsSnapshot`.

With the current 500-page Confluence cap (HARD-02) the leak is bounded: at most 500 × (workers racing) leaked slots, well within the 4-billion range. However, the race is undocumented here unlike `InodeRegistry`.

**Fix:** Add an inline comment matching the one in `InodeRegistry::intern` (line 152-154 of `inode.rs`) so future readers understand this is an intentional trade-off:
```rust
// Race: two threads may both read None and both fetch_add.
// The DashMap::entry below resolves the race; the losing
// thread's counter increment is wasted (max 500 * worker_count
// slots; the 4B range absorbs this easily — same as InodeRegistry::intern).
let dir_ino = self.next_dir_ino.fetch_add(1, Ordering::SeqCst);
```

---

### WR-02: `.comments` lookup accepts `"0"` as a valid page ID

**File:** `crates/reposix-fuse/src/fs.rs:1360-1363`
**Issue:** When the kernel looks up `"00000000000.comments"` (11-zero padded), `trim_start_matches('0')` produces an empty string `""`, and `"".parse::<u64>()` succeeds with value `0`. `IssueId(0)` is not a valid Confluence page ID (Atlassian IDs start at 1), but if any issue was ever interned with `IssueId(0)` (e.g. during the `create` placeholder path in `fs.rs:2112`), `registry.lookup_id(IssueId(0))` would return `Some(page_ino)` and a `CommentsDir` would be materialised for it.

```rust
// Current — allows IssueId(0):
let Ok(page_id_num) = page_id_str.trim_start_matches('0').parse::<u64>() else {
    reply.error(fuser::Errno::from_i32(libc::ENOENT));
    return;
};
```

**Fix:** Parse the full `page_id_str` as a decimal (no leading-zero strip) and reject zero:
```rust
let Ok(page_id_num) = page_id_str.parse::<u64>() else {
    reply.error(fuser::Errno::from_i32(libc::ENOENT));
    return;
};
if page_id_num == 0 {
    reply.error(fuser::Errno::from_i32(libc::ENOENT));
    return;
}
```
This is consistent with `issue_filename` using `{:011}` (which never produces all-zeros for a real ID) and avoids the edge case entirely.

---

### WR-03: `update_issue` CONFLICT branch in `ConfluenceBackend` drops the `version_mismatch:` signal

**File:** `crates/reposix-confluence/src/lib.rs:1351-1357`
**Issue:** When Confluence returns HTTP 409 on a `PUT`, the code returns:
```rust
return Err(Error::Other(format!(
    "confluence version conflict for PUT {}: {body_preview}",
    redact_url(&url),
)));
```
The `fs.rs` caller (`release`) handles `FetchError::Conflict` specifically and logs a user-friendly "git pull --rebase" message (`fs.rs:2066-2072`). However, `backend_err_to_fetch` only recognises the `"version mismatch:"` prefix (set by the sim backend), not the `"confluence version conflict"` prefix set here. As a result, a Confluence 409 surfaces as `FetchError::Core` → `libc::EIO` instead of `FetchError::Conflict` — the user never sees the helpful rebase message.

**Fix:** Use the same prefix the `backend_err_to_fetch` decoder expects, or change `backend_err_to_fetch` to also handle the Confluence prefix:
```rust
// Option A: match the existing decoder's expected prefix
return Err(Error::Other(format!(
    "version mismatch: {{\"current\":{}}}",
    /* extract version from body if parseable, else 0 */
    serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|v| v["version"]["number"].as_u64())
        .unwrap_or(0)
)));
```
Or alternatively (cleaner): add a `"confluence version conflict"` arm to `backend_err_to_fetch` in `fs.rs:186-199`.

---

### WR-04: Comment cache is silently stale after `refresh_issues`

**File:** `crates/reposix-fuse/src/fs.rs:644-650` (field comment) and `crates/reposix-fuse/src/fs.rs:893-950` (`refresh_issues`)
**Issue:** The field comment documents that `comment_snapshot` is NOT invalidated on `refresh_issues`. This is intentional for v0.7.0 (lazy-once-per-page). However, this means that if a user:
1. Opens `pages/00000131192.comments/` (comments are fetched and cached).
2. Adds a comment in Confluence.
3. Runs `git diff` or any operation that triggers `refresh_issues`.
4. Re-reads the `.comments/` directory — they see stale data (no new comment).

The problem is that `refresh_issues` invalidates the bucket index, tree index, and label snapshot, but not the comment snapshot. A partial invalidation creates an internally inconsistent view: `_INDEX.md` shows the updated page list but `.comments/` shows pre-update comments. This is not a security issue but is a correctness gap that could confuse autonomous agents.

**Fix:** Minimally, the comment in the field doc should reference this inconsistency explicitly and suggest a user-visible workaround (e.g. unmount and remount to flush comments). Ideally, `refresh_issues` should call `comment_snapshot.invalidate()` (a new method that sets all `by_page` entries back to `(false, vec![])` so they are re-fetched on next access, preserving the dir inode assignments so no inode numbers change).

## Info

### IN-01: `ConfLinks` `next` field is `#[allow(dead_code)]` in two places

**File:** `crates/reposix-confluence/src/lib.rs:219-223` and `crates/reposix-confluence/src/lib.rs:391-394`
**Issue:** The `ConfLinks.next` field is marked `#[allow(dead_code)]` in both `ConfPageList` and `ConfCommentList` because pagination is driven via the `parse_next_cursor` pure helper operating on a raw `serde_json::Value`, not on the typed field. The typed struct therefore has a field that is never read.

**Fix:** Remove the `next` field from both `ConfLinks` instances (keep only the struct for serde skipping if `deny_unknown_fields` is ever added), or remove `ConfLinks` entirely and rely solely on `parse_next_cursor`. Eliminates the `#[allow(dead_code)]` suppression.

---

### IN-02: `nested_layout` tests do not assert `.comments` directories

**File:** `crates/reposix-fuse/tests/nested_layout.rs` (entire file)
**Issue:** Phase 23's primary deliverable is exposing `.comments/` directories, but none of the integration tests in `nested_layout.rs` assert their existence. The `boot_mount` helper passes `comment_fetcher: None` (line 160), which means `CommentsDir` inodes are never materialised. Tests confirm the tree overlay, gitignore, and bucket listing — but zero coverage of the new comment path under a real (wiremock) Confluence fixture.

**Fix:** Add a test that:
1. Registers a comment endpoint: `GET /wiki/api/v2/pages/{id}/footer-comments?...` → returns a comment JSON object.
2. Passes `comment_fetcher: Some(Arc::new(ConfluenceBackend::new_with_base_url(...)))` to `Mount::open`.
3. Asserts `pages/00000360556.comments/` is reachable after `readdir`.

---

### IN-03: `list_spaces` has no pagination cap

**File:** `crates/reposix-confluence/src/lib.rs:1085-1135`
**Issue:** `list_issues_impl` enforces `MAX_ISSUES_PER_LIST` (500) and warns on overflow. `list_comments` enforces the same cap. `list_spaces`, however, paginates without any cap or warning — it will loop forever on a response stream that always returns `_links.next`. For well-behaved tenants this is unlikely, but the code's own convention is to cap all paginated calls.

**Fix:** Add a `pages` counter and a warn-then-break guard matching the pattern in `list_issues_impl:709-720`:
```rust
let mut pages: usize = 0;
while let Some(url) = next_url.take() {
    pages += 1;
    if pages > 20 { // 20 * 250 = 5000 spaces, more than any real tenant
        tracing::warn!(pages, "list_spaces: pagination cap reached");
        break;
    }
    // ...
}
```

---

### IN-04: `readdir` for `CommentsDir` hardcodes `BUCKET_DIR_INO` as the `..` entry

**File:** `crates/reposix-fuse/src/fs.rs:1667`
**Issue:** The `..` entry for a `CommentsDir` is always emitted as `BUCKET_DIR_INO` (inode 2, the `pages/` directory). This is correct for Confluence (comments dirs live under `pages/`), but the code comment in the analogous `TreeDir` arm (line 1612) acknowledges the same limitation for tree dirs: "we don't track the reverse relation in the snapshot, so use `TREE_ROOT_INO` as a conservative default." For comment dirs the parent is always `BUCKET_DIR_INO`, which happens to be correct, but the reasoning should be made explicit with a comment to prevent confusion if a future backend stores comments elsewhere.

**Fix:** Add a brief comment:
```rust
// Parent is always the bucket directory (pages/) — comments
// live under pages/<id>.comments/, never at the mount root.
(BUCKET_DIR_INO, FileType::Directory, "..".to_owned()),
```

---

_Reviewed: 2026-04-16T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
