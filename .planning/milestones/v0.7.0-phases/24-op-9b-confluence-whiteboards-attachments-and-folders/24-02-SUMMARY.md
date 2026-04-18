---
phase: 24-op-9b-confluence-whiteboards-attachments-and-folders
plan: "02"
subsystem: reposix-fuse
tags: [fuse, confluence, whiteboards, attachments, inode, phase-24]
dependency_graph:
  requires: [24-01-PLAN.md]
  provides: [CONF-04, CONF-05]
  affects:
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/attachments.rs
    - crates/reposix-fuse/src/lib.rs
    - crates/reposix-confluence/src/lib.rs
tech_stack:
  added: []
  patterns:
    - AttachmentsSnapshot mirrors CommentsSnapshot (lazy per-page cache, DashMap, AtomicU64 allocators)
    - InodeKind::classify() highest-first range dispatch for new inode ranges
    - 50 MiB attachment cap enforced at both fetch_attachments_for_page and read(AttachmentFile)
    - whiteboards/ emitted in Root readdir only when comment_fetcher is Some
key_files:
  created:
    - crates/reposix-fuse/src/attachments.rs
  modified:
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/lib.rs
    - crates/reposix-confluence/src/lib.rs
decisions:
  - "AttachmentsSnapshot.is_fetched uses is_some_and() per clippy::unnecessary_map_or"
  - "const MAX_ATTACHMENT_BYTES hoisted to function top to satisfy clippy::items_after_statements"
  - "fetch_whiteboards uses self.project as space key (resolve_space_id is private to confluence crate)"
  - "translate_treats_folder_parent_as_orphan test updated to reflect CONF-06 folder propagation"
  - "GITIGNORE_BYTES extended to include whiteboards/ (28 bytes total)"
metrics:
  duration: ~25m
  completed: "2026-04-16"
  tasks: 2
  files: 5
---

# Phase 24 Plan 02: FUSE overlay for whiteboards and attachments

## One-liner

FUSE overlay for Confluence whiteboards (`whiteboards/<id>.json`) and per-page attachments (`pages/<id>.attachments/<filename>`) with 50 MiB cap, binary body caching, and POSIX filename sanitization.

## What was built

### Task 1: Inode constants + AttachmentsSnapshot module

**`crates/reposix-fuse/src/inode.rs`**
- Added four new `pub const` values after `COMMENTS_FILE_INO_BASE`:
  - `WHITEBOARDS_ROOT_INO = 0x20_0000_0000` (fixed dir inode)
  - `WHITEBOARD_FILE_INO_BASE = 0x20_0000_0001`
  - `ATTACHMENTS_DIR_INO_BASE = 0x24_0000_0000`
  - `ATTACHMENTS_FILE_INO_BASE = 0x28_0000_0000`
- Added disjoint-range assertions in `fixed_inodes_are_disjoint_from_dynamic_ranges` test
- Updated module-level doc table to include new Phase 24 ranges

**`crates/reposix-fuse/src/attachments.rs`** (new file)
- `AttachmentEntry` struct: `file_ino`, `filename` (sanitized), `rendered` (binary), `file_size`, `download_url`, `media_type`
- `AttachmentsSnapshot` struct: mirrors `CommentsSnapshot` with per-page `DashMap` + `AtomicU64` allocators
- Public API: `new()`, `ensure_dir()`, `mark_fetched()`, `is_fetched()`, `entries_for_page()`, `page_of_dir()`, `entry_by_file_ino()`, `alloc_file_ino()`, `update_entry_rendered()`
- `sanitize_attachment_filename()`: allowlist `[a-zA-Z0-9._-]`, replace others with `_`, return `None` for empty/>255 byte results
- 7 unit tests covering idempotency, reverse lookup, entry retrieval, sanitization edge cases

**`crates/reposix-fuse/src/lib.rs`**
- Added `pub mod attachments;`

### Task 2: FUSE dispatch wiring in fs.rs

**`crates/reposix-fuse/src/fs.rs`**
- Imports: `sanitize_attachment_filename`, `AttachmentEntry`, `AttachmentsSnapshot`, new inode constants
- Updated `GITIGNORE_BYTES` to include `whiteboards/` (28 bytes total)
- Added 4 `InodeKind` variants: `WhiteboardsRoot`, `WhiteboardFile`, `AttachmentsDir`, `AttachmentFile`
- Updated `InodeKind::classify()` with highest-first range ordering (Attachment > Whiteboard > Comments)
- Added 3 new `ReposixFs` fields: `attachment_snapshot`, `whiteboard_snapshot`, `next_whiteboard_ino`
- Initialized all three in `ReposixFs::new()`
- Full dispatch in all 5 FUSE callbacks:
  - `getattr`: all 4 new variants
  - `lookup`: `whiteboards` in Root arm; `.attachments` suffix in Bucket arm; `WhiteboardsRoot` + `AttachmentsDir` arms
  - `readdir`: `whiteboards/` in Root (Confluence-only); `WhiteboardsRoot` + `AttachmentsDir` arms
  - `read`: `WhiteboardFile` (JSON serialize); `AttachmentFile` (binary bytes + 50 MiB cap)
  - `write`/`setattr`: new variants marked EROFS
- Helper methods: `whiteboards_attr()`, `whiteboard_entry_by_ino()`, `fetch_whiteboards()`, `fetch_attachments_for_page()`, `fetch_attachment_body()`

**`crates/reposix-confluence/src/lib.rs`**
- Fixed doc_markdown clippy lint (`52_428_800` → backticked)
- Fixed fmt diff in `download_attachment` function
- Updated two tests to reflect CONF-06 folder parent propagation (see Deviations)

## Verification results

- `cargo test --workspace`: all test suites pass (0 failures across all crates)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `cargo fmt --all -- --check`: clean
- Grep: 37 occurrences of new InodeKind variants in fs.rs
- Grep: 15 occurrences of new inode constants in inode.rs
- attachments.rs: 3 public items (AttachmentsSnapshot, AttachmentEntry, sanitize_attachment_filename) confirmed

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed clippy::unnecessary_map_or in AttachmentsSnapshot::is_fetched**
- **Found during:** Task 1 clippy check
- **Issue:** `map_or(false, |e| e.0)` flagged by clippy as `is_some_and` preferred
- **Fix:** Changed to `.is_some_and(|e| e.0)`
- **Files modified:** `crates/reposix-fuse/src/attachments.rs`
- **Commit:** 94dd435

**2. [Rule 1 - Bug] Fixed clippy::items_after_statements for MAX_ATTACHMENT_BYTES const**
- **Found during:** Task 2 clippy check
- **Issue:** `const MAX_ATTACHMENT_BYTES` declared after statements in two places
- **Fix:** Hoisted to function top (`fetch_attachments_for_page`) and `read` callback
- **Files modified:** `crates/reposix-fuse/src/fs.rs`
- **Commit:** 6f800ce

**3. [Rule 1 - Bug] Fixed clippy::doc_markdown lints**
- **Found during:** Task 1 and Task 2 clippy checks
- **Issue:** `Ok(vec![])`, `list_whiteboards`, `52_428_800` without backticks in doc comments
- **Fix:** Added backticks to all flagged identifiers
- **Files modified:** `crates/reposix-confluence/src/lib.rs`, `crates/reposix-fuse/src/fs.rs`
- **Commit:** 94dd435, 6f800ce

**4. [Rule 1 - Bug] Updated tests to match CONF-06 folder-parent propagation**
- **Found during:** Task 2 full workspace test run
- **Issue:** `translate_treats_folder_parent_as_orphan` and `list_populates_parent_id_end_to_end` expected `parent_id = None` for `parentType="folder"`, but Plan 01 added CONF-06 propagation
- **Fix:** Updated both tests to assert `Some(IssueId(...))` for folder parents
- **Files modified:** `crates/reposix-confluence/src/lib.rs`
- **Commit:** 6f800ce

**5. [Rule 1 - Bug] Updated gitignore_bytes_is_correct test**
- **Found during:** Task 2 test run
- **Issue:** Existing test asserted `GITIGNORE_BYTES == b"/tree/\nlabels/\n"` (15 bytes), but Plan 02 adds `whiteboards/\n` making it 28 bytes
- **Fix:** Updated test to assert new 28-byte value
- **Files modified:** `crates/reposix-fuse/src/fs.rs`
- **Commit:** 6f800ce

## Known Stubs

None — all four InodeKind variants have full dispatch. Whiteboard and attachment bodies are fetched from the real Confluence API via `comment_fetcher`.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: EFBIG-bypass | `crates/reposix-fuse/src/fs.rs` | Attachment read enforces 50 MiB cap but relies on `file_size` from API metadata, not actual downloaded size. If Confluence returns a wrong `file_size`, the body could exceed the cap. Accepted risk (T-24-02-02 mitigated at API boundary). |

## Self-Check: PASSED

- `crates/reposix-fuse/src/attachments.rs`: exists
- `crates/reposix-fuse/src/inode.rs`: contains 4 new constants
- `crates/reposix-fuse/src/fs.rs`: contains WhiteboardsRoot, WhiteboardFile, AttachmentsDir, AttachmentFile (37 occurrences)
- Commits 94dd435 and 6f800ce: both present in git log
