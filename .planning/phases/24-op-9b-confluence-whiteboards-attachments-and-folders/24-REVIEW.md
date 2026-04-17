---
phase: 24-op-9b-confluence-whiteboards-attachments-and-folders
reviewed: 2026-04-16T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-fuse/src/attachments.rs
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/lib.rs
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: issues_found
---

# Phase 24: Code Review Report

**Reviewed:** 2026-04-16T00:00:00Z
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Phase 24 adds `list_attachments`, `list_whiteboards`, `download_attachment` on `ConfluenceBackend`, plus the `AttachmentsSnapshot`/`AttachmentEntry`/`sanitize_attachment_filename` module and four new `InodeKind` variants wired into all FUSE callbacks. The CONF-06 folder-parent propagation fix in `translate()` is also included.

The code is structurally sound and the primary mitigations (filename sanitization, 50 MiB cap at fetch time, SG-01 allowlist) are correctly placed. Two critical issues were found: a path-traversal bypass in `download_attachment` that circumvents the SG-01 allowlist, and the 50 MiB cap not being re-checked at `read()` time when the body was fetched lazily with an empty `file_size`. Four warnings cover the inode leak in `ensure_dir` under concurrent calls, an implicit double-fetch window in `readdir(AttachmentsDir)`, and missing input validation on `list_whiteboards`'s `space_id` URL parameter. Three info items cover O(n) scan cost, absent sanitization of whiteboard filenames, and a missing EFBIG test.

---

## Critical Issues

### CR-01: `download_attachment` accepts absolute URLs from the server, bypassing the SG-01 allowlist

**File:** `crates/reposix-confluence/src/lib.rs:1448-1452`

**Issue:** When `download_url` starts with `http://` or `https://`, `download_attachment` uses it verbatim without any allowlist check:

```rust
let full_url =
    if download_url.starts_with("http://") || download_url.starts_with("https://") {
        download_url.to_owned()
    } else {
        format!("{}{}", self.base(), download_url)
    };
```

The `download_link` field on `ConfAttachment` is server-controlled (comes from the Confluence JSON response, which is tainted). The SG-01 allowlist lives inside `HttpClient::request_with_headers`, but that guard only blocks origins not in `REPOSIX_ALLOWED_ORIGINS`. If the Confluence API returns an absolute URL pointing at a different host (e.g. via a compromised CDN, a malicious test fixture, or a future API change), `download_attachment` will follow it — potentially exfiltrating auth headers (`Authorization: Basic ...`) to an attacker-controlled server. The same bypass exists identically in `list_comments` and `list_whiteboards` pagination cursors, but those only appear after an operator explicitly sets `REPOSIX_ALLOWED_ORIGINS`; attachment download URLs appear in routine operation with the API credentials in the request.

The comment in the SSRF mitigation doc (module-level comment) says "relative-only by construction defeats SSRF by design", but that invariant is violated here for absolute download links.

**Fix:** Never trust an absolute URL from the server. Validate that any absolute URL in `download_link` has the same origin as `self.base()`, or strip and re-join:

```rust
let full_url = if download_url.starts_with("http://") || download_url.starts_with("https://") {
    // SSRF guard: reject absolute URLs pointing outside our tenant base.
    let expected = self.base();
    if !download_url.starts_with(expected) {
        return Err(Error::Other(format!(
            "attachment download_link has unexpected origin (SSRF guard): {}",
            redact_url(download_url)
        )));
    }
    download_url.to_owned()
} else {
    format!("{}{}", self.base(), download_url)
};
```

---

### CR-02: 50 MiB cap not enforced when lazy fetch runs with `file_size == 0`

**File:** `crates/reposix-fuse/src/fs.rs:2161-2194` and `fs.rs:1294-1329`

**Issue:** In `fetch_attachments_for_page` (fs.rs:1294) the eager download is correctly skipped when `att.file_size > MAX_ATTACHMENT_BYTES`. However the eager download is also skipped when `att.file_size == 0` (the `att.file_size > 0 &&` guard at line 1294). If Confluence returns `fileSize: 0` (the `serde default` on the field — see `lib.rs:419`) for an attachment that is actually large, `entry.rendered` stays empty and `entry.file_size` stays 0.

Later, in `read(AttachmentFile)` at fs.rs:2167, the cap check is:

```rust
if entry.file_size > MAX_ATTACHMENT_BYTES {
    reply.error(fuser::Errno::from_i32(libc::EFBIG));
    return;
}
```

When `file_size == 0` this check passes, and the code falls through to `fetch_attachment_body` (the lazy path at line 2179) which calls `download_attachment` with no size limit whatsoever. `download_attachment` does `resp.bytes().await?` — fully buffering the response in memory with no cap. A server that lies about `fileSize: 0` (or an attachment endpoint that omits the field) can cause unbounded heap allocation in the FUSE daemon.

**Fix:** After the lazy fetch in `read(AttachmentFile)`, re-check the actual byte count before returning:

```rust
Ok(bytes) => {
    // Re-check cap after lazy fetch (file_size may have been 0 from API).
    if bytes.len() as u64 > MAX_ATTACHMENT_BYTES {
        tracing::warn!(
            file_ino = ino_u,
            actual_size = bytes.len(),
            "lazily-fetched attachment exceeds 50 MiB; returning EFBIG"
        );
        reply.error(fuser::Errno::from_i32(libc::EFBIG));
        return;
    }
    bytes
}
```

Additionally, `download_attachment` itself should use `reqwest`'s `content_length()` to abort before buffering if the server reports a large body.

---

## Warnings

### WR-01: `ensure_dir` inode leak under concurrent calls (wasted inodes accumulate)

**File:** `crates/reposix-fuse/src/attachments.rs:96-111`

**Issue:** The code explicitly acknowledges the race:

```rust
let dir_ino = self.next_dir_ino.fetch_add(1, Ordering::SeqCst);
// Race: two threads could both bump; the DashMap insert below picks one winner.
match self.page_to_dir_ino.entry(page_ino) {
    dashmap::Entry::Occupied(e) => *e.get(),   // winner already inserted
    dashmap::Entry::Vacant(e) => { ... }
}
```

The losing thread's `dir_ino` is permanently consumed from `next_dir_ino` but never inserted into `dir_ino_to_page`. Since `InodeKind::classify` routes any inode in `ATTACHMENTS_DIR_INO_BASE..ATTACHMENTS_FILE_INO_BASE` to `InodeKind::AttachmentsDir`, that orphaned inode will return `ENOENT` from `getattr` and `page_of_dir`. That is safe — but the orphaned inode can never be reclaimed. If many pages are first-accessed concurrently (e.g. a parallel agent doing `stat` on many `.attachments` dirs), inode slots drain faster than expected. With the 4 GiB range `0x24_0000_0000..0x28_0000_0000` this is not practically exhaustible, but the comment says "race is harmless" without documenting the inode drain. The same pattern exists in `InodeRegistry::intern` and is accepted there too.

More importantly: the `dir_ino_to_page` reverse-map is only written by the **winning** thread (the `Vacant` arm). If the losing thread later calls `page_of_dir(dir_ino)` on the leaked inode (e.g., if the kernel handed back a dir inode that FUSE returned before the race resolved), it gets `None`, causing a spurious ENOENT. In practice the kernel will not reuse an inode it was never told about, so this is theoretical — but it should be explicitly documented.

**Fix:** Document the invariant clearly. If the cost of the race is truly acceptable, add a comment explaining why the orphaned dir_ino is safe (the kernel never learns about it, so it is never sent to `getattr` or `readdir`). Alternatively, initialize `by_page` unconditionally before the map-entry race to avoid a partial-state window.

---

### WR-02: Double-fetch window in `readdir(AttachmentsDir)` when concurrent threads race on `is_fetched`

**File:** `crates/reposix-fuse/src/fs.rs:2011-2018`

**Issue:**

```rust
if !self.attachment_snapshot.is_fetched(page_ino) {
    if let Err(e) = self.fetch_attachments_for_page(page_ino) {
        ...
    }
}
```

`is_fetched` and the subsequent `fetch_attachments_for_page` are not atomic. Two concurrent `readdir` calls on the same `AttachmentsDir` can both observe `is_fetched == false` and both call `fetch_attachments_for_page`. `fetch_attachments_for_page` calls `snap.mark_fetched(page_ino, entries)` which does `self.by_page.insert(page_ino, (true, entries))` — the second write overwrites the first. The result is duplicate file inodes in `file_ino_to_page` (each call allocates fresh `file_ino`s via `alloc_file_ino`), producing two distinct inodes for the same logical attachment. Any `read` on the later-returned inode will find the entry correctly (the second `mark_fetched` replaces the first), but inodes from the first call that the kernel cached in its dentry cache will become permanently orphaned after the second `mark_fetched` discards them — those inodes will return ENOENT on subsequent `getattr`. The same window exists for comments (Phase 23) and is pre-existing, but attaching a note here because the attachment download is expensive (multiple HTTP requests) so the double-fetch has a real cost beyond just inode leakage.

**Fix:** In `mark_fetched`, check `is_fetched` inside the DashMap entry lock before inserting:

```rust
pub fn mark_fetched(&self, page_ino: u64, entries: Vec<AttachmentEntry>) {
    use dashmap::Entry;
    match self.by_page.entry(page_ino) {
        Entry::Occupied(mut e) if e.get().0 => return, // already fetched by another thread
        Entry::Occupied(mut e) => {
            for entry in &entries {
                self.file_ino_to_page.insert(entry.file_ino, page_ino);
            }
            e.insert((true, entries));
        }
        Entry::Vacant(e) => {
            for entry in &entries {
                self.file_ino_to_page.insert(entry.file_ino, page_ino);
            }
            e.insert((true, entries));
        }
    }
}
```

---

### WR-03: `list_whiteboards` URL-path-injects the `space_id` parameter without validation

**File:** `crates/reposix-confluence/src/lib.rs:1348-1352`

**Issue:**

```rust
let first = format!(
    "{}/wiki/api/v2/spaces/{}/direct-children?limit={}",
    self.base(),
    space_id,   // ← tainted: comes from self.project, set by the user or operator
    PAGE_SIZE
);
```

`space_id` here is `self.project`, passed in from `fetch_whiteboards` (fs.rs:1228), which is set from `MountConfig::project`. In production that is the `--project` CLI flag from the operator. However this is a URL path segment, not a query parameter. A `space_id` of `"REPOSIX/../admin"` or `"REPOSIX?injected=true"` would corrupt the path. Compare `resolve_space_id` which correctly uses `url::Url::query_pairs_mut` for the `keys=` parameter (with an explanatory `WR-01` comment). The same protection is missing here.

This is a lower-severity instance than CR-01 because `project` comes from the operator rather than the server, but in an automated CI/CD environment where `--project` might come from a git branch name or environment variable, it is an operator-controlled injection vector.

**Fix:** URL-encode `space_id` when embedding it in the path:

```rust
let encoded_space_id = percent_encoding::utf8_percent_encode(
    space_id,
    percent_encoding::NON_ALPHANUMERIC,
);
let first = format!(
    "{}/wiki/api/v2/spaces/{}/direct-children?limit={}",
    self.base(),
    encoded_space_id,
    PAGE_SIZE
);
```

Or, use `url::Url`'s path-segment API. The same issue exists in `list_issues_impl` at the `spaces/{space_id}/pages` URL (line 815), though `space_id` there is the numeric-validated result of `resolve_space_id` which already enforces `all(is_ascii_digit)` — that validation is the correct existing mitigation for that path.

---

### WR-04: `readdir(Bucket)` does not emit `.comments` or `.attachments` virtual directories

**File:** `crates/reposix-fuse/src/fs.rs:1854-1880`

**Issue:** `readdir(Bucket)` lists `_INDEX.md` and `<padded-id>.md` entries, but never emits the `<padded-id>.comments` or `<padded-id>.attachments` virtual directory entries. A user running `ls pages/` will not see `00000001234.attachments` or `00000001234.comments` even if those have been materialized. They are only discoverable via explicit `stat`/`ls` on the known name.

This is a usability issue that also affects agents relying on directory listing to discover available resources. It is not a security issue, but it is incorrect POSIX semantics: a directory should enumerate all entries reachable within it.

**Fix:** Emit virtual `.comments` and `.attachments` entries in `readdir(Bucket)` by iterating `comment_snapshot` and `attachment_snapshot` for any dirs already materialized, or by emitting synthetic entries for all known page inodes when the fetcher is present:

```rust
// After emitting real page entries:
if self.comment_fetcher.is_some() {
    for issue in &sorted {
        let page_ino = self.registry.intern(issue.id);
        let dir_ino = self.comment_snapshot.ensure_dir(page_ino);
        let padded = format!("{:011}", issue.id.0);
        out.push((dir_ino, FileType::Directory, format!("{padded}.comments")));
        let att_dir_ino = self.attachment_snapshot.ensure_dir(page_ino);
        out.push((att_dir_ino, FileType::Directory, format!("{padded}.attachments")));
    }
}
```

---

## Info

### IN-01: `whiteboard_entry_by_ino` is O(n) over the full whiteboard set

**File:** `crates/reposix-fuse/src/fs.rs:1199-1210`

**Issue:**

```rust
fn whiteboard_entry_by_ino(&self, file_ino: u64) -> Option<(String, ConfWhiteboard)> {
    for entry in self.whiteboard_snapshot.iter() {
        let (ino, wb) = entry.value();
        if *ino == file_ino { return Some(...); }
    }
    None
}
```

Called on every `getattr(WhiteboardFile)` and `read(WhiteboardFile)`. The doc comment says "typical space has < 100 whiteboards so this is acceptable" — that is plausible, but the scan is also called from `getattr` which can be invoked by the kernel very frequently (stat on every `open`). For spaces with large whiteboard counts the cost grows linearly. The comment is also not enforced anywhere.

**Fix (non-blocking):** Add a reverse map `file_ino → whiteboard_id` (analogous to `dir_ino_to_page` in `AttachmentsSnapshot`) when whiteboards are populated in `fetch_whiteboards`. This would make the lookup O(1):

```rust
// In ReposixFs:
whiteboard_ino_to_id: DashMap<u64, String>,

// In fetch_whiteboards:
self.whiteboard_ino_to_id.insert(ino, wb.id.clone());

// In whiteboard_entry_by_ino:
fn whiteboard_entry_by_ino(&self, file_ino: u64) -> Option<(...)> {
    let id = self.whiteboard_ino_to_id.get(&file_ino)?;
    self.whiteboard_snapshot.get(id.value()).map(|e| (id.value().clone(), e.value().1.clone()))
}
```

---

### IN-02: Whiteboard filenames (`.json` entries in `readdir`) are not sanitized

**File:** `crates/reposix-fuse/src/fs.rs:2002`

**Issue:**

```rust
out.push((*file_ino, FileType::RegularFile, format!("{}.json", wb.id)));
```

`wb.id` is a server-supplied numeric string (Confluence whiteboard IDs are documented as numeric), but this is not validated. If the API returned a non-numeric `id` containing `/`, `.`, or other path characters, the resulting filename would be a malformed or traversal-capable POSIX name in the directory entry. Compare attachment filenames which go through `sanitize_attachment_filename`. Whiteboard IDs in practice are numeric strings (like page IDs), but there is no compile-time or runtime assertion of this.

**Fix:** Validate that `wb.id` is purely numeric before using it as a filename, or apply a sanitization step analogous to `sanitize_attachment_filename`. A simple guard:

```rust
if wb.id.chars().all(|c| c.is_ascii_digit()) {
    out.push((*file_ino, FileType::RegularFile, format!("{}.json", wb.id)));
} else {
    tracing::warn!(whiteboard_id = %wb.id, "whiteboard id is not numeric; skipping");
}
```

---

### IN-03: No test for `read(AttachmentFile)` returning `EFBIG`

**File:** `crates/reposix-fuse/src/fs.rs:2161-2194` and `crates/reposix-fuse/src/attachments.rs`

**Issue:** The 50 MiB cap in `read(AttachmentFile)` (line 2167) is not tested. The unit tests in `attachments.rs` cover `ensure_dir`, `entry_by_file_ino`, and `sanitize_attachment_filename`, but there is no test that verifies `EFBIG` is returned when `entry.file_size > MAX_ATTACHMENT_BYTES`. Given that CR-02 identifies a bypass when `file_size == 0`, the absence of a test for the main path makes future regressions harder to detect.

**Fix:** Add a unit/integration test that creates an `AttachmentEntry` with `file_size = 52_428_801` and verifies that `read(AttachmentFile)` replies `EFBIG`. This can be done at the `ReposixFs` level with a mock backend or at the attachment snapshot level by inspecting the cap logic directly.

---

_Reviewed: 2026-04-16T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
