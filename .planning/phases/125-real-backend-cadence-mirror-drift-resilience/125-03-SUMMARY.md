---
phase: 125-real-backend-cadence-mirror-drift-resilience
plan: 03
subsystem: docs
tags: [testing-targets, mirror-refresh, doc-alignment, dvcs, troubleshooting, real-backend-cadence]

# Dependency graph
requires:
  - phase: 125-02
    provides: "litmus mirror-drift self-heal (_litmus_mirror_reconcile, DRAIN-02) that the pre-step doc cross-references as 'usually unnecessary'"
  - phase: 125-01
    provides: "write_loop.rs remote-explicit teaching string + the additive Pattern-C note the PART 2 blockquote reword is made consistent with"
provides:
  - "docs/reference/testing-targets.md now names scripts/refresh-tokenworld-mirror.sh as the mirror-refresh pre-step for the pre-release-real-backend cadence (closed the SC1/DRAIN-02 doc gap: previously ZERO mentions)"
  - "docs/guides/troubleshooting.md v0.14.0 blockquote's Pattern-C attach recovery is now remote-explicit (SC3/DRAIN-12 completion)"
affects: [125-close, milestone-close-vision-litmus, real-backend-cadence-operators]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Drift-safe doc insertion: append below the highest doc-alignment-bound line (source_hash is fixed-line-range, no content re-anchor) to avoid a STALE_DOCS_DRIFT re-mint cascade"

key-files:
  created: []
  modified:
    - docs/reference/testing-targets.md
    - docs/guides/troubleshooting.md
    - .planning/milestones/v0.15.0-phases/surprises-intake/part-07.md
    - .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md

key-decisions:
  - "Placed the new pre-step section at end-of-doc (drift-safe) instead of the plan's preferred near-top sibling, to avoid an 18-row STALE_DOCS_DRIFT cascade — pivot toward intention (discoverable named pre-step) over literal placement wording"
  - "Cross-referenced the self-heal as P125/DRAIN-02 (the mirror-drift self-reconcile), correcting the plan Task-1's literal 'DRAIN-12' (which is the helper teaching-string requirement PART 2 completes)"
  - "PART 2 escape valve: verified zero doc-alignment binding drift (all 6 troubleshooting bindings at line <=227, above the edit) so fixed + no rebind needed in the same commit — did NOT defer"

patterns-established:
  - "Before rewording/inserting in a doc-alignment-bound file, enumerate the file's bindings' line ranges; edit strictly below the max bound line to guarantee zero source_hash drift"

requirements-completed: [DRAIN-02]

# Metrics
duration: ~40min
completed: 2026-07-18
---

# Phase 125 Plan 03: Mirror-refresh pre-step doc (SC1/DRAIN-02) Summary

**testing-targets.md now names `scripts/refresh-tokenworld-mirror.sh` as the mirror-refresh pre-step for the `pre-release-real-backend` cadence (closing the zero-mention SC1 gap), cross-referencing the P125/DRAIN-02 litmus self-reconcile; plus a coordinator-absorbed SC3/DRAIN-12 fix making the troubleshooting.md v0.14.0 attach-tree recovery remote-explicit.**

## Performance

- **Duration:** ~40 min
- **Completed:** 2026-07-18
- **Tasks:** 2 (Task 1 committed; Task 2 = walk-gate PASS clean, no binding needed) + 1 coordinator-absorbed PART 2
- **Files modified:** 4

## Accomplishments
- **SC1/DRAIN-02:** Added a top-level `## Mirror-refresh pre-step (GitHub-mirror drift)` section to `docs/reference/testing-targets.md` — copy-paste `bash scripts/refresh-tokenworld-mirror.sh`, an accurate what-it-does line (overlays the backend-materialized `pages/` tree onto the external GitHub mirror; explicitly distinct from `reposix sync --reconcile`, which rebuilds only the local cache), and a `>` callout that the P125/DRAIN-02 litmus self-reconcile usually makes it unnecessary.
- **Task 2 (doc-alignment):** `bash quality/gates/docs-alignment/walk.sh` PASSES clean (rc=0) — the pre-step prose is an operational instruction, not an eligible testable claim; no binding minted, committed catalog untouched.
- **PART 2 (SC3/DRAIN-12):** Reworded the `troubleshooting.md` v0.14.0 blockquote's Pattern-C attach paragraph to name the bus remote explicitly, consistent with 125-01's `write_loop.rs` teaching string and the note directly below. Filed intake entry marked RESOLVED.

## Task Commits

1. **Task 1: Mirror-refresh pre-step section** - `cdaee30a` (docs)
2. **PART 2: attach-tree recovery remote-explicit + intake resolution** - `0808d48f` (docs)

_Task 2 produced no commit (walk gate PASS clean; doc-alignment.json unchanged — plan specifies "commit only if doc-alignment.json changed")._

**Plan metadata (this SUMMARY):** committed separately below.

## Files Created/Modified
- `docs/reference/testing-targets.md` - Added the mirror-refresh pre-step section (SC1/DRAIN-02)
- `docs/guides/troubleshooting.md` - v0.14.0 blockquote Pattern-C recovery made remote-explicit (SC3/DRAIN-12)
- `.planning/milestones/v0.15.0-phases/surprises-intake/part-07.md` - marked the 125-01-filed entry RESOLVED
- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` - index line annotated RESOLVED

## Decisions Made
- **Placement pivot (drift-safety over literal wording):** `source_hash` hashes bytes at a fixed 1-based line range with no content re-anchor (`crates/reposix-quality/src/hash.rs`), so inserting the plan's preferred near-top sibling would shift all 18 testing-targets.md bindings below it and trigger a STALE_DOCS_DRIFT re-mint cascade (P117 W3 doctrine forbids). Placed the section at end-of-doc (below the max bound line 251) → zero drift, walk gate clean. Intention (a discoverable, named pre-step + the self-heal caveat) fully honored.
- **DRAIN tag correction:** The plan Task-1 callout cited "DRAIN-12", but the mirror-drift self-reconcile the caveat describes is `_litmus_mirror_reconcile` = **DRAIN-02** (litmus-self-heal.sh line 20; RESEARCH.md req table). DRAIN-12 is the helper teaching-string requirement that PART 2 completes. Used P125/DRAIN-02 in the doc (consistent with the doc's existing `P123/DRAIN-03` traceability style).

## Deviations from Plan

### Adjustments (intention over literal wording)

**1. [Rule 3-adjacent — blocking-avoidance] Section placed at end-of-doc, not near-top sibling**
- **Found during:** Task 1 / Task 2 walk-gate reasoning
- **Issue:** The plan's preferred near-top placement would shift 18 doc-alignment bindings → STALE_DOCS_DRIFT cascade + 18-row re-mint (the exact hazard P117 W3 and the escape valve forbid).
- **Fix:** Appended the section below the highest bound line (251); verified walk gate exits 0 with no drift and no binding needed.
- **Files modified:** docs/reference/testing-targets.md
- **Verification:** `bash quality/gates/docs-alignment/walk.sh` rc=0, no STALE_DOCS_DRIFT; committed catalog clean.
- **Committed in:** cdaee30a

**2. [Doc accuracy] Cross-reference tagged DRAIN-02 (not the plan's literal DRAIN-12)**
- **Found during:** Task 1 drafting
- **Issue:** Plan Task-1 point-4 said "self-heal, DRAIN-12"; the mirror self-reconcile is DRAIN-02.
- **Fix:** Cited P125/DRAIN-02 (the accurate mirror-drift self-heal / this plan's own requirement).
- **Verification:** litmus-self-heal.sh line 20 + 125-RESEARCH.md req table.
- **Committed in:** cdaee30a

---

**Total deviations:** 2 adjustments (both toward project intention: drift-safety + doc accuracy). No scope creep.
**Impact on plan:** SC1/DRAIN-02 fully delivered; the placement pivot avoided a self-inflicted drift cascade.

## Doc-truth guardrail honored
Introduced **no** fresh unqualified claim that the external 30-minute cron converger is live. The new pre-step section describes only the manual script + the P125/DRAIN-02 litmus self-heal; the qualified mirror-head framing (root CLAUDE.md, ADR-010 RBF-LR-04) is untouched.

## Issues Encountered
- **File-size WARN (print-only, non-blocking, pre-existing):** `troubleshooting.md` (28659) and `surprises-intake/part-07.md` (21674) exceed the 20000 `.md` ceiling — both under the global `structure/file-size-limits` waiver (until 2026-08-08) and both were already over budget before my small edits. Already tracked (fork-anti-pattern intake + GTH-V15 waiver-remediation). Not introduced by this plan; noted, not fixed (out of scope).

## Known Stubs
None.

## Gate Results
- `bash quality/gates/docs-build/mkdocs-strict.sh` → rc=0 ("OK: docs site clean")
- `bash quality/gates/docs-build/mermaid-renders.sh` → rc=0 (7 pages valid)
- `bash quality/gates/structure/banned-words.sh` → rc=0 (all mode; Layer-2 troubleshooting.md clean)
- `bash quality/gates/docs-alignment/walk.sh` → rc=0 (no STALE_DOCS_DRIFT; no binding minted)
- `bash quality/gates/docs-alignment/dvcs-troubleshooting-matrix.sh` → rc=0 (structural section intact)

## Next Phase Readiness
- SC1/DRAIN-02 doc gap closed and SC3/DRAIN-12 doc residual completed — ready for phase-125 close.
- Coordinator owns: STATE/ROADMAP advance, `git push origin main`, and the unbiased verifier dispatch. No push performed by this executor.

## Self-Check: PASSED
- FOUND: docs/reference/testing-targets.md, docs/guides/troubleshooting.md, 125-03-SUMMARY.md
- FOUND commits: cdaee30a (Task 1), 0808d48f (PART 2)

---
*Phase: 125-real-backend-cadence-mirror-drift-resilience*
*Completed: 2026-07-18*
