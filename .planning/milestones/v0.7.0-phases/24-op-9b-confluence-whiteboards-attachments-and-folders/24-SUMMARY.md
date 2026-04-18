---
phase: 24-op-9b-confluence-whiteboards-attachments-and-folders
plan: "00"
subsystem: api + reposix-fuse
tags: [confluence, whiteboards, attachments, fuse, rest-v2, folder-hierarchy, phase-24]
dependency_graph:
  requires:
    - 23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme
  provides:
    - list_attachments(page_id) on ConfluenceBackend
    - list_whiteboards(space_id) on ConfluenceBackend
    - download_attachment(url) on ConfluenceBackend
    - whiteboards/ FUSE top-level directory (Confluence-only)
    - pages/<id>.attachments/ FUSE per-page subdirectory
    - folder hierarchy propagation via translate() CONF-06 fix
  affects:
    - crates/reposix-confluence/src/lib.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/attachments.rs
    - crates/reposix-fuse/src/lib.rs
    - CHANGELOG.md
tech-stack:
  added: []
  patterns:
    - "Pagination loop: standard 8-step loop (same as list_comments/list_issues_impl)"
    - "404-graceful: list_whiteboards returns Ok(vec![]) on 404 (MEDIUM-confidence endpoint)"
    - "AttachmentsSnapshot mirrors CommentsSnapshot (lazy per-page DashMap cache)"
    - "InodeKind::classify() highest-first range dispatch"
    - "50 MiB attachment cap enforced at both fetch and read layers"
    - "sanitize_attachment_filename() allowlist [a-zA-Z0-9._-]"
key-files:
  created:
    - crates/reposix-fuse/src/attachments.rs
  modified:
    - crates/reposix-confluence/src/lib.rs
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/lib.rs
    - CHANGELOG.md
    - .planning/STATE.md
decisions:
  - "CONF-06 resolved via translate() fix (folder parents → parent_id) rather than a separate folders/ FUSE tree"
  - "Whiteboard listing uses direct-children endpoint (graceful 404 → Ok(vec![]))"
  - "Attachment binary fetch is eager on readdir for files ≤ 50 MiB; lazy fallback on read()"
  - "AttachmentsSnapshot mirrors CommentsSnapshot exactly (new pattern is established)"
  - "comment_fetcher reused for all Confluence-specific methods (no new field duplication)"
metrics:
  duration: ~45min
  completed: "2026-04-16"
  tasks: 3
  files: 6
  tests_before: 318
  tests_after: 397
---

# Phase 24: OP-9b — Confluence Whiteboards, Attachments, and Folder Hierarchy — Phase Summary

**Confluence whiteboards (`whiteboards/<id>.json`), per-page attachments (`pages/<id>.attachments/<file>`), and folder hierarchy propagation via `translate()` CONF-06 fix — 397 workspace tests, all green.**

## Requirements Closed

| Requirement | Description | Plan |
|-------------|-------------|------|
| CONF-04 | `list_whiteboards` + `whiteboards/` FUSE directory | 24-01, 24-02 |
| CONF-05 | `list_attachments` + `download_attachment` + `.attachments/` FUSE subdir | 24-01, 24-02 |
| CONF-06 | `translate()` folder-parent fix — folder hierarchy in `tree/` overlay | 24-01 |

## Wave Structure

| Wave | Plan | Scope |
|------|------|-------|
| Wave 1 | 24-01 | `lib.rs` — new API methods + structs + CONF-06 translate() fix + 8 wiremock tests |
| Wave 2 | 24-02 | FUSE overlay — inode constants, `attachments.rs` module, `fs.rs` dispatch wiring |
| Wave 3 | 24-03 | Green gauntlet (fmt + clippy + test) + CHANGELOG + STATE.md update |

## Test Count

| Milestone | Count |
|-----------|-------|
| Phase 23 baseline | 318 |
| After Phase 24 Wave 1 | 326 (318 + 8 new wiremock tests) |
| After Phase 24 Wave 2 | 397 (326 + 71: 7 AttachmentsSnapshot unit tests + 64 full workspace re-count) |
| Phase 24 final (`cargo test --workspace`) | **397 (0 failures)** |

## Files Modified

| File | Change |
|------|--------|
| `crates/reposix-confluence/src/lib.rs` | Added `ConfAttachment`, `ConfWhiteboard` structs; `list_attachments`, `list_whiteboards`, `download_attachment` methods; `translate()` CONF-06 folder arm; 8 new wiremock tests; clippy/fmt fixes |
| `crates/reposix-fuse/src/attachments.rs` | **New file** — `AttachmentsSnapshot`, `AttachmentEntry`, `sanitize_attachment_filename()` + 7 unit tests |
| `crates/reposix-fuse/src/inode.rs` | Added 4 new inode constants: `WHITEBOARDS_ROOT_INO`, `WHITEBOARD_FILE_INO_BASE`, `ATTACHMENTS_DIR_INO_BASE`, `ATTACHMENTS_FILE_INO_BASE`; updated disjoint-range test + doc table |
| `crates/reposix-fuse/src/fs.rs` | Added 4 `InodeKind` variants + classify dispatch; 3 `ReposixFs` fields; full FUSE callback dispatch; `GITIGNORE_BYTES` extended to include `whiteboards/`; helper methods |
| `crates/reposix-fuse/src/lib.rs` | Added `pub mod attachments;` |
| `CHANGELOG.md` | Added Phase 24 entries for CONF-04, CONF-05, CONF-06 under `[Unreleased]` |
| `.planning/STATE.md` | Updated cursor to Phase 24 SHIPPED |

## Key Decisions

1. **CONF-06 via translate() fix**: Resolved folder hierarchy by adding `(Some(pid_str), Some("folder"))` arm in `translate()` before the catch-all — folder-parented pages propagate to `Issue::parent_id` exactly like page-parented ones. No separate `folders/` FUSE tree needed.

2. **Whiteboard listing — direct-children endpoint**: `GET /wiki/api/v2/spaces/{id}/direct-children` filtered by `type == "whiteboard"`. 404 returns `Ok(vec![])` with `tracing::warn!` (MEDIUM-confidence endpoint — not all tenants expose it).

3. **AttachmentsSnapshot mirrors CommentsSnapshot**: New module `attachments.rs` follows the exact same `DashMap` + `AtomicU64` allocator pattern established in Phase 23. This is now a reusable established pattern for per-page lazy caches in the FUSE layer.

4. **comment_fetcher reuse**: All Confluence-specific methods (`list_whiteboards`, `list_attachments`, `download_attachment`) are called via the existing `comment_fetcher` closure — no new `Arc<ConfluenceBackend>` field added to `ReposixFs`.

5. **50 MiB cap — two layers**: Cap enforced both at `fetch_attachments_for_page` (skips fetching) and at `read()` callback (returns `EFBIG`). Belt-and-suspenders for the T-24-02-02 threat.

6. **GITIGNORE_BYTES extended**: The synthesized `.gitignore` now contains `/tree/\nlabels/\nwhiteboards/\n` (28 bytes) so the `whiteboards/` FUSE directory is never accidentally `git add`ed.

## Green Gauntlet Results

```
cargo fmt --all -- --check    → PASSED
cargo clippy --workspace --all-targets -- -D warnings → PASSED (0 warnings)
cargo test --workspace        → 397 passed, 0 failed
```

## Deviations Across Phase

All deviations were auto-fixed (Rules 1 and 2); no architectural decisions required (no Rule 4 stops).

| Rule | Description | Plan | Commit |
|------|-------------|------|--------|
| Rule 1 — Bug | Updated 2 pre-existing tests for CONF-06 behavior change (`translate_treats_folder_parent_as_orphan`, `list_populates_parent_id_end_to_end`) | 24-01 | 7e6ac5c |
| Rule 1 — Bug | `clippy::unnecessary_map_or` in `AttachmentsSnapshot::is_fetched` — changed to `.is_some_and()` | 24-02 | 94dd435 |
| Rule 1 — Bug | `clippy::items_after_statements` for `MAX_ATTACHMENT_BYTES` const — hoisted to function top | 24-02 | 6f800ce |
| Rule 1 — Bug | `clippy::doc_markdown` lints — backticked identifiers in doc comments | 24-01, 24-02 | 94dd435, 6f800ce |
| Rule 1 — Bug | Updated `gitignore_bytes_is_correct` test for 28-byte value (was 15 bytes before `whiteboards/` added) | 24-02 | 6f800ce |

## Known Stubs

None — all API methods are fully wired and wiremock-tested. All four FUSE `InodeKind` variants have full dispatch. No placeholder text or hardcoded empty returns that affect plan goals.

## Threat Surface

| Mitigation | Status |
|------------|--------|
| T-24-02-01: `sanitize_attachment_filename()` allowlist prevents path traversal | Implemented and unit-tested |
| T-24-02-02: 50 MiB cap at fetch + read layers | Implemented (belt-and-suspenders) |
| T-24-02-04: `download_attachment` relative URL prepend via `self.base()` + SG-01 gate | Implemented |
| T-24-01-04: `download_attachment` unbounded bytes — accepted risk (cap belongs at FUSE layer) | Resolved: cap now in FUSE (Wave 2) |

## Self-Check: PASSED

- `crates/reposix-fuse/src/attachments.rs`: FOUND
- `crates/reposix-fuse/src/inode.rs`: FOUND with 4 new constants
- `crates/reposix-fuse/src/fs.rs`: FOUND with 4 new InodeKind variants
- `CHANGELOG.md`: CONF-04/05/06 entries present (9 occurrences)
- `cargo test --workspace`: 397 passed, 0 failed
- `cargo clippy --workspace --all-targets -- -D warnings`: PASSED
- `cargo fmt --all -- --check`: PASSED

---
*Phase: 24-op-9b-confluence-whiteboards-attachments-and-folders*
*Completed: 2026-04-16*
