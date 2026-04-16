---
phase: 23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme
plan: "03"
subsystem: fuse
tags: [fuse, confluence, comments, inode, dashmap, erofs]

# Dependency graph
requires:
  - phase: 23-01
    provides: "ConfluenceBackend::list_comments API + ConfComment/CommentKind types"
  - phase: 19
    provides: "LabelSnapshot lazy-overlay pattern (CommentsSnapshot mirrors this design)"
provides:
  - "CommentsSnapshot: lazy per-page comment cache with DashMap, inode allocators, render_comment_file"
  - "COMMENTS_DIR_INO_BASE (0x18_0000_0000) + COMMENTS_FILE_INO_BASE (0x1C_0000_0000) inode constants"
  - "InodeKind::CommentsDir + InodeKind::CommentFile dispatch arms in all 6 FUSE callbacks"
  - "Mount::open 3-arg signature (comment_fetcher: Option<Arc<ConfluenceBackend>>)"
  - "reposix-fuse binary build_comment_fetcher() function for --backend-kind confluence"
affects:
  - "phase-24"
  - "reposix-cli"
  - "any integration test that calls Mount::open (updated to 3-arg form)"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Lazy per-page fetch: CommentsDir inode allocated on first lookup; comments fetched on first readdir/lookup of that dir"
    - "Separate Arc<ConfluenceBackend> alongside Arc<dyn IssueBackend> (list_comments not on IssueBackend trait)"
    - "is_numeric_ascii validator for tainted string-to-filename conversion (WR-02 pattern)"
    - "Body-after-fence: tainted comment body placed after closing ---\n\n to prevent YAML injection (T-23-03-01)"
    - "5-second timeout on all list_comments calls (SG-07 contract)"

key-files:
  created:
    - crates/reposix-fuse/src/comments.rs
  modified:
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/lib.rs
    - crates/reposix-fuse/src/main.rs
    - crates/reposix-fuse/tests/readdir.rs
    - crates/reposix-fuse/tests/sim_death_no_hang.rs
    - crates/reposix-fuse/tests/nested_layout.rs

key-decisions:
  - "Separate comment_fetcher parameter on Mount::open rather than extending MountConfig — MountConfig derives Serialize/Deserialize which Arc<ConfluenceBackend> cannot satisfy"
  - "CommentsDir inode allocated only after registry.lookup_id confirms the page exists — garbage .comments lookups never allocate (T-23-03-07 DoS guard)"
  - "readdir(Bucket) does NOT emit .comments entries — only explicit lookup materializes CommentsDir (T-23-03-04 DoS amplifier prevention)"
  - "Comment cache NOT invalidated on refresh_issues — lazy-once-per-page semantics intentional; remount to force refresh (documented on field)"
  - "build_comment_fetcher is a separate function from build_backend, reading env vars a second time — avoids Arc<dyn Any> downcast which IssueBackend trait does not expose"

patterns-established:
  - "CommentsSnapshot mirrors LabelSnapshot overlay pattern but with lazy per-page fetching"
  - "#[allow(clippy::too_many_lines)] on FUSE callbacks that are genuinely large match tables — not a code smell here"

requirements-completed:
  - CONF-01
  - CONF-02
  - CONF-03

# Metrics
duration: ~90min (multi-session, resumed from context summary)
completed: "2026-04-16"
---

# Phase 23 Plan 03: Confluence Comments FUSE Overlay Summary

**CommentsSnapshot lazy per-page overlay wired into all 6 FUSE callbacks: pages/<id>.comments/ dirs materialize on lookup with comment files rendered as YAML-frontmatter Markdown, all writes rejected EROFS, readdir(Bucket) unchanged**

## Performance

- **Duration:** ~90 min (split across two sessions)
- **Started:** 2026-04-16
- **Completed:** 2026-04-16
- **Tasks:** 3 (Task 1 completed in prior session; Tasks 2+3 completed in this session)
- **Files modified:** 9 (1 created, 8 modified)
- **LOC delta:** +793 / -27 across 9 files

## Accomplishments

- New `crates/reposix-fuse/src/comments.rs` module (420 LOC): `CommentsSnapshot` struct with DashMap-based lazy per-page cache, monotonic inode allocators, 10 unit tests, `render_comment_file` with numeric-ASCII ID validation and body-after-fence YAML injection prevention
- Two new inode constants `COMMENTS_DIR_INO_BASE` (0x18_0000_0000) and `COMMENTS_FILE_INO_BASE` (0x1C_0000_0000) added to `inode.rs` with disjoint-range assertions in existing test
- `InodeKind::CommentsDir` and `InodeKind::CommentFile` variants wired into all FUSE callbacks: `getattr`, `lookup`, `readdir`, `read`, `setattr`, `write` — plus the existing `create`/`unlink` guards already covered CommentsDir
- `Mount::open` extended to 3-arg form; `ReposixFs::new` accepts `Option<Arc<ConfluenceBackend>>` as comment fetcher
- `reposix-fuse` binary: `build_comment_fetcher()` constructs a second `Arc<ConfluenceBackend>` only for `--backend-kind confluence`; sim/github receive `None`
- All 3 integration test files updated to pass `None` as the new 3rd argument

## Task Commits

Each task was committed atomically:

1. **Task 1: CommentsSnapshot + render_comment_file + inode constants** - `9baca41` (test — TDD RED/GREEN)
2. **Task 2: Wire CommentsSnapshot into ReposixFs dispatch** - `ece7161` (feat)
3. **Task 3: Wire comment_fetcher into reposix-fuse binary** - `3dbd40b` (feat)
4. **Fmt fixes: apply rustfmt to all modified files** - `7d99cbb` (style)

## Files Created/Modified

- `crates/reposix-fuse/src/comments.rs` — new: CommentsSnapshot, render_comment_file, 10 unit tests
- `crates/reposix-fuse/src/inode.rs` — +2 inode constants, extended disjoint-range test
- `crates/reposix-fuse/src/fs.rs` — +263 LOC: CommentsDir/CommentFile dispatch in all callbacks, fetch_comments_for_page helper, 3-test comments_dispatch_tests module
- `crates/reposix-fuse/src/lib.rs` — pub mod comments, re-exports, Mount::open 3-arg signature
- `crates/reposix-fuse/src/main.rs` — +28 LOC: build_comment_fetcher(), wired into main()
- `crates/reposix-fuse/tests/readdir.rs` — updated Mount::open call to pass None
- `crates/reposix-fuse/tests/sim_death_no_hang.rs` — updated Mount::open call to pass None
- `crates/reposix-fuse/tests/nested_layout.rs` — updated Mount::open call to pass None
- `crates/reposix-confluence/src/lib.rs` — reformatted by rustfmt (test assertions)

## Decisions Made

- `comment_fetcher` as separate parameter on `Mount::open` rather than inside `MountConfig`: `MountConfig` derives `Serialize/Deserialize` which `Arc<ConfluenceBackend>` cannot implement, and the field should not persist to disk config anyway.
- CommentsDir inode only allocated after `registry.lookup_id` confirms the page exists: prevents inode table growth from malicious `.comments` lookups (T-23-03-07).
- `readdir(Bucket)` does NOT emit `.comments` entries: materializing entries would trigger one `list_comments` HTTP call per page on any `grep -r mount/pages/`, creating a DoS amplifier (T-23-03-04).
- Comment cache intentionally not invalidated on `refresh_issues`: page inodes are stable via `InodeRegistry::intern`; TTL-based invalidation deferred to v0.8.0.
- `build_comment_fetcher` reads env vars independently rather than downcasting from `Arc<dyn IssueBackend>`: the IssueBackend trait does not expose `&dyn Any`, and downcasting would couple the binary to `reposix-confluence` at the trait level.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy: option_as_ref_cloned**
- **Found during:** Task 2 completion verification (cargo clippy)
- **Issue:** `self.comment_fetcher.as_ref().cloned()` flagged by `clippy::option_as_ref_cloned`
- **Fix:** Changed to `self.comment_fetcher.clone()`
- **Files modified:** crates/reposix-fuse/src/fs.rs
- **Committed in:** ece7161 (style fix included in Task 2 commit)

**2. [Rule 1 - Bug] Clippy: match_same_arms on setattr**
- **Found during:** Task 2 completion verification (cargo clippy)
- **Issue:** Labels EROFS arm and CommentsDir EROFS arm in `setattr` had identical bodies
- **Fix:** Merged into single `LabelsRoot | LabelDir | LabelSymlink | CommentsDir | CommentFile` arm
- **Files modified:** crates/reposix-fuse/src/fs.rs
- **Committed in:** ece7161

**3. [Rule 2 - Missing Critical] Clippy: too_many_lines on getattr**
- **Found during:** Task 2 completion verification (cargo clippy)
- **Issue:** `getattr` function has 111 lines, exceeding the 100-line pedantic limit
- **Fix:** Added `#[allow(clippy::too_many_lines)]` with rationale comment (FUSE callback covering every inode kind in a single dispatch table — splitting would require forwarding state)
- **Files modified:** crates/reposix-fuse/src/fs.rs
- **Committed in:** ece7161

**4. [Rule 1 - Style] rustfmt formatting**
- **Found during:** Final verification (cargo fmt --all --check)
- **Issue:** Multiple files had lines exceeding rustfmt's preferred width: long assert_eq! chains in tests, long method-chain expressions in fs.rs
- **Fix:** `cargo fmt --all` applied
- **Files modified:** fs.rs, lib.rs, main.rs, comments.rs, reposix-confluence/src/lib.rs
- **Committed in:** 7d99cbb (style commit)

---

**Total deviations:** 4 auto-fixed (3 clippy Rule 1/2, 1 fmt Rule 1)
**Impact on plan:** All auto-fixes were mechanical correctness/style fixes. No scope creep. No plan deviations.

## Threat Register Coverage

| Threat ID | Mitigation | How Verified |
|-----------|------------|--------------|
| T-23-03-01 | Body after `---\n\n` fence | `render_comment_file_body_after_frontmatter_fence` test |
| T-23-03-02 | `is_numeric_ascii` validator returns None for non-numeric id/parent_id | `render_comment_file_rejects_non_numeric_id` + `_parent_id` tests |
| T-23-03-03 | Tracing spans log only page_ino/comment_id/kind | Code-review invariant — no body text in any `tracing::*` call in comments.rs or fs.rs dispatch arms |
| T-23-03-04 | `readdir(Bucket)` does not emit `.comments` entries | Code inspection: Bucket readdir arm only emits `*.md` + `_INDEX.md` filenames |
| T-23-03-05 | 5-second `tokio::time::timeout` in `fetch_comments_for_page` | Code inspection + SG-07 contract (same pattern as list_issues) |
| T-23-03-06 | Inode range disjoint assertions | `fixed_inodes_are_disjoint_from_dynamic_ranges` test extended |
| T-23-03-07 | CommentsDir allocated only after `registry.lookup_id` confirms page exists | Code inspection: `ENOENT` returned before `ensure_dir` on unknown page |
| T-23-03-08 | `write`/`setattr`/`create`/`unlink` all reject CommentsDir/CommentFile with EROFS | `classify_returns_commentsdir_in_range` + existing Bucket-only guards |
| T-23-03-09 | `comment_fetcher.is_none()` guard in Bucket lookup returns ENOENT before allocating CommentsDir | Code inspection (`grep -c 'comment_fetcher.is_none' fs.rs` = 1) |
| T-23-03-10 | author_id PII: accepted | Documented — mirrors Confluence's own access model |

## Issues Encountered

- **Context-window split**: This plan executed across two sessions (context summary forced mid-Task-2). Task 1 was completed in the first session; Tasks 2 and 3 were completed in this continuation session. No work was lost; the continuation resumed correctly from the Task 2 compilation state.

## Known Stubs

None — all comment dispatch arms are wired to live `CommentsSnapshot` + `ConfluenceBackend::list_comments`. The `comment_fetcher: None` path (sim/github) fast-fails with `ENOENT` before any stub behavior.

## Next Phase Readiness

- CONF-01, CONF-02, CONF-03 are materially implemented at the code level; full end-to-end verification (live FUSE mount against a wiremock Confluence server) deferred to v0.8.0 integration test harness
- Comment cache TTL-based invalidation deferred to v0.8.0 (CONF-04)
- `reposix-cli spaces` command (Plan 23-02) ships alongside this plan

## Open Follow-ups for v0.8.0

1. **TTL-based cache invalidation**: `CommentsSnapshot` entries cached for mount lifetime; add configurable TTL (e.g. 60s) to re-fetch on `readdir` after expiry
2. **Integration test**: `cargo test -p reposix-fuse --features fuse-mount-tests` test that spins up wiremock Confluence mock, mounts, and verifies `ls mount/pages/<id>.comments/` + `cat` round-trip
3. **`create_comment` write path**: Currently EROFS everywhere — a future write-enabled Confluence backend would need a comment-file parse/POST path

---
*Phase: 23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme*
*Completed: 2026-04-16*
