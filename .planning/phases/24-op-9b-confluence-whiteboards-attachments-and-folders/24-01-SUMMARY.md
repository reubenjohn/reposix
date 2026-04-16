---
phase: 24-op-9b-confluence-whiteboards-attachments-and-folders
plan: "01"
subsystem: api
tags: [confluence, attachments, whiteboards, fuse, rest-v2, wiremock]

requires:
  - phase: 23-op-9a-confluence-comments-exposed-as-pages-id-comments-comme
    provides: list_comments pattern + CommentKind enum (pagination loop to mirror)

provides:
  - "list_attachments(page_id) -> Vec<ConfAttachment> on ConfluenceBackend"
  - "list_whiteboards(space_id) -> Vec<ConfWhiteboard> on ConfluenceBackend"
  - "download_attachment(url) -> Vec<u8> on ConfluenceBackend"
  - "ConfAttachment public struct (Deserialize)"
  - "ConfWhiteboard public struct (Deserialize + Serialize)"
  - "translate() folder-parent fix: parentType=folder now propagates to Issue::parent_id"

affects:
  - 24-op-9b-confluence-whiteboards-attachments-and-folders
  - any FUSE overlay plan consuming attachment/whiteboard metadata

tech-stack:
  added: []
  patterns:
    - "list_*: standard 8-step pagination loop (same as list_comments/list_issues_impl)"
    - "404-graceful: list_whiteboards returns Ok(vec![]) on 404 (MEDIUM-confidence endpoint)"
    - "download_attachment: relative URL prepended with self.base(), Basic auth required"
    - "translate() folder arm: matches before catch-all (_, Some(other)) to propagate folder hierarchy"

key-files:
  created: []
  modified:
    - crates/reposix-confluence/src/lib.rs

key-decisions:
  - "CONF-06: folder parentType now propagates to Issue::parent_id (same logic as page arm) — closes the folder hierarchy gap without a separate folders/ FUSE tree"
  - "list_whiteboards uses direct-children endpoint filtered by type==whiteboard; 404 graceful (MEDIUM-confidence endpoint per RESEARCH.md)"
  - "download_attachment is unbounded by design — caller (FUSE layer) enforces 50 MiB cap (T-24-01-04 accepted)"
  - "Two existing tests updated to reflect CONF-06 behavior change: translate_treats_folder_parent_as_orphan and list_populates_parent_id_end_to_end"

patterns-established:
  - "All three new methods mirror the list_comments pagination loop exactly (standard_headers + await_rate_limit_gate + ingest_rate_limit + parse_next_cursor)"
  - "Internal list structs (ConfAttachmentList, ConfDirectChildrenList, ConfDirectChild) are private; only public structs are exported"

requirements-completed:
  - CONF-04
  - CONF-05
  - CONF-06

duration: 20min
completed: 2026-04-16
---

# Phase 24 Plan 01: Attachments + Whiteboards + Folder Hierarchy Summary

**Three new public methods on `ConfluenceBackend` (list_attachments, list_whiteboards, download_attachment) plus CONF-06 folder-hierarchy fix in translate(), backed by 8 new wiremock unit tests.**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-04-16T20:00:00Z (approx)
- **Completed:** 2026-04-16T20:20:00Z (approx)
- **Tasks:** 1 (TDD: RED + GREEN + fmt/clippy cleanup)
- **Files modified:** 1

## Accomplishments

- `list_attachments(page_id: u64) -> Result<Vec<ConfAttachment>>`: GET /wiki/api/v2/pages/{id}/attachments, paginated, capped at MAX_ISSUES_PER_LIST
- `list_whiteboards(space_id: &str) -> Result<Vec<ConfWhiteboard>>`: GET /wiki/api/v2/spaces/{id}/direct-children filtered by `type == "whiteboard"`, 404-graceful
- `download_attachment(url: &str) -> Result<Vec<u8>>`: relative URL + Basic auth, returns raw bytes
- `translate()` CONF-06 fix: `(Some(pid_str), Some("folder"))` arm added before catch-all, mirrors page arm logic
- 8 new wiremock tests (3 list_attachments + 2 list_whiteboards + 1 download_attachment + 2 translate-folder)
- 83 total tests pass; clippy `-D warnings` clean; `cargo fmt` clean

## Task Commits

TDD commits:

1. **RED: Failing tests** - `a26deb5` (test)
2. **GREEN: Implementation + test corrections** - `7e6ac5c` (feat)

## Files Created/Modified

- `crates/reposix-confluence/src/lib.rs` — Added ConfAttachment, ConfWhiteboard structs + private list structs; list_attachments, list_whiteboards, download_attachment methods; translate() folder arm fix; updated 2 pre-existing tests for CONF-06 behavior change; 8 new wiremock tests

## Decisions Made

- **CONF-06 behavior change acknowledged**: Two existing tests (`translate_treats_folder_parent_as_orphan`, `list_populates_parent_id_end_to_end`) expected folder parents to be orphaned. CONF-06 changes this contract — both tests updated with explanatory comments documenting the intentional change.
- **download_attachment unbounded by design**: The 50 MiB cap belongs at the FUSE layer (Wave 2 plan). The method itself returns all bytes — T-24-01-04 is accepted per threat model.
- **list_whiteboards 404 graceful**: Direct-children endpoint has MEDIUM confidence (not all tenants expose it). 404 returns `Ok(vec![])` with `tracing::warn!` rather than `Err` to avoid breaking the FUSE mount on tenants without the endpoint.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated 2 pre-existing tests for CONF-06 behavior change**
- **Found during:** Task 1 GREEN phase (running tests after translate() fix)
- **Issue:** `translate_treats_folder_parent_as_orphan` and `list_populates_parent_id_end_to_end` asserted `parent_id == None` for folder parents. After CONF-06 fix they correctly return `Some(IssueId(...))` making those tests fail.
- **Fix:** Updated assertions to match new correct behavior; added comments documenting the CONF-06 rationale.
- **Files modified:** `crates/reposix-confluence/src/lib.rs`
- **Verification:** All 83 tests pass after update
- **Committed in:** `7e6ac5c`

---

**Total deviations:** 1 auto-fixed (Rule 1 — existing tests reflected pre-CONF-06 contract)
**Impact on plan:** Necessary correction; no scope creep.

## Issues Encountered

- `cargo fmt` required two rounds: initial write had line-length issues in test assertions; `cargo fmt -p reposix-confluence` auto-corrected all formatting.

## Known Stubs

None — all three methods are fully wired with real HTTP calls (wiremock-tested).

## Threat Flags

No new threat surface beyond what is documented in the plan's threat model (T-24-01-01 through T-24-01-04). The `download_attachment` method uses `self.base()` prepend for relative URLs — absolute URLs pass through the SG-01 `HttpClient` allowlist gate unchanged (T-24-01-01 mitigated by construction).

## TDD Gate Compliance

- RED gate commit: `a26deb5` (`test(24-01): add failing tests...`)
- GREEN gate commit: `7e6ac5c` (`feat(24-01): add list_attachments...`)
- REFACTOR gate: not needed (no structural cleanup required after GREEN)

## Self-Check: PASSED

- `crates/reposix-confluence/src/lib.rs` — FOUND (exists and modified)
- Commit `a26deb5` — FOUND (RED gate)
- Commit `7e6ac5c` — FOUND (GREEN gate)
- `grep -n "pub async fn list_attachments"` — line 1271
- `grep -n "pub async fn list_whiteboards"` — line 1347
- `grep -n "pub async fn download_attachment"` — line 1447
- `grep -n "pub struct ConfAttachment"` — line 402
- `grep -n "pub struct ConfWhiteboard"` — line 433
- `grep -n 'Some("folder")'` — line 642 (translate fix)
- All 83 tests pass

## Next Phase Readiness

- Wave 2 FUSE overlay plans (24-02, 24-03) can now depend on `list_attachments`, `list_whiteboards`, and `download_attachment` being present on `ConfluenceBackend`.
- `translate()` folder fix (CONF-06) is live — the tree/ overlay will correctly show folder hierarchy nodes.

---
*Phase: 24-op-9b-confluence-whiteboards-attachments-and-folders*
*Completed: 2026-04-16*
