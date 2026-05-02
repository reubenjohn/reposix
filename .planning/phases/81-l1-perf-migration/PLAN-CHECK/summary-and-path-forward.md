# Summary table and recommended path forward

← [back to index](./index.md)

## Summary table

| Question | Status |
|----------|--------|
| 1. Goal achievable? | NO at executable level (H1) |
| 2. SC1–SC7 covered? | YES at planning level |
| 3. Catalog-first? | YES |
| 4. D-01 ratified in 3 places? | YES |
| 5. First-push fallback? | YES |
| 6. Cargo discipline? | YES |
| 7. STRIDE register? | PARTIAL (T-81-02 cites non-existent peek()) |
| 8. Plan size budget? | YES (1,954 lines, ~150 lines trim available) |
| 9. Open questions deferrable? | NO for Q2 + Q3; partial for Q6 |
| 10. Phase shape sound? | YES |
| 11. P82 contract integrity? | PARTIAL (signature couples to State) |
| 12. Build memory budget? | YES |

## HIGH issue count: 4
## MEDIUM issue count: 4
## LOW issue count: 3

---

## Recommended path forward

The phase **goal is sound** (L1 conflict detection, single shared module, sync --reconcile escape hatch, D-01 ratified). The plans **understand the architecture correctly** at the strategic level. But the four HIGH issues are concrete API contract errors that the planner could have caught with `grep read_blob`, `grep peek`, `grep "pub enum.*Error" remote/src/`, and reading 4 lines of `crates/reposix-remote/src/main.rs:42–71`. The planner asserted contracts and the source disproves them.

**Re-plan with these specific fixes:**

1. (H1) Add `Cache::read_blob_cached(...)` to `cache.rs` (sync, gix-local, returns `Option<Tainted<Vec<u8>>>`); use it in precheck.rs. OR rewrite precheck to read version from backend GET (one per `changed_set ∩ push_set` record) instead of cache. Either way, do NOT call `read_blob` in the precheck.
2. (H2) Replace `.peek()` with `.inner_ref()` in 4 plan locations + must_haves + canonical_refs + threat_model. Patch RESEARCH.md in same PR.
3. (H3) Widen `State` to `pub(crate)` (struct + 4 fields); change `precheck.rs` import to `use crate::{State, issue_id_from_path};` (drop `::main::`).
4. (H4) Rewrite precheck sketch to use `anyhow::Result` + `anyhow::Error` (no typed variants). Add a 4-line subsection in T02 § 2b titled "Error flow" specifying anyhow throughout.
5. (M1) Refactor `precheck_export_against_changed_set` signature to accept narrow dependencies (`cache: &Cache, backend: &dyn BackendConnector, project: &str, rt: &Runtime, parsed: &ParsedExport`). 10 lines of plumbing in `handle_export` cost; unlocks P82 reuse cleanly.
6. (M2) Size `drive_export_verb_single_record_edit` at 60–80 lines OR pivot to subprocess invocation; add "read mirror_refs.rs FIRST" hard-block to T04 read_first.
7. (M3) Promote `mod common;` requirement check to T03 + T04 hard-block.
8. (M4) Rewrite the "≥1 list_changed_since call" matcher with a custom `Match` impl symmetric to `NoSinceQueryParam`.
9. (L1, L2, L3) Optional polish — tighten if revising anyway.

After these revisions, the plan should be executable end-to-end without surprise rework. Estimated revision effort: 1 hour (planner-time grep + 10-line edits + 1 new function in cache.rs sketch).
