---
phase: 114-t4-confluence-oid-drift-fix-first-reconcile-audit
plan: 02
subsystem: testing
tags: [oid-drift, reconcile, confluence, backend-connector, mock, doc-honesty, sc4]

# Dependency graph
requires:
  - phase: 114-01
    provides: FIX-01 adapter render-parity (closes ADF-native list-vs-get drift); post-FIX-01 state the corrected docs reference
  - phase: 114-RESEARCH
    provides: FIX-02 three-class drift table + Assumption A3 (reconcile == a forced build_from, cannot heal the systematic list-vs-get class)
provides:
  - reproduction-backed proof that divergent list/get bodies → Error::OidDrift (DriftingMock, backend-agnostic)
  - empirical proof that a second build_from (== sync --reconcile) leaves the stale list-oid unchanged and read_blob still errors (backs SC4)
  - corrected Error::OidDrift + sync --reconcile doc scope naming the three coexisting drift classes precisely (not over-corrected)
affects: [phase-close verifier (SC4 doc-accuracy grade), real-backend t4 gate SC1/SC2, future list-path storage-fallback follow-up]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "reproduction-before-claim: a doc scope statement about recovery behavior is backed by a mock that empirically reproduces the non-recovery, not asserted"
    - "backend-agnostic body-divergence mock (DriftingMock): list render diverges from get render via Record.body, complementing CappingMock's membership divergence"

key-files:
  created:
    - crates/reposix-cache/tests/oid_drift_reconcile.rs
  modified:
    - crates/reposix-cache/src/error.rs
    - crates/reposix-cli/src/sync.rs
    - crates/reposix-cli/src/main.rs
    - .planning/phases/114-t4-confluence-oid-drift-fix-first-reconcile-audit/114-01-SUMMARY.md

key-decisions:
  - "Precision over reversal (Pitfall 3): the docs now name BOTH drift causes with opposite recovery answers — reconcile CAN heal an eventual-consistency race, CANNOT heal a systematic rendering mismatch — never flipped to a blanket 'reconcile never heals oid-drift'"
  - "cache.rs::write_last_fetched_at left untouched: its 'cursor drift is recoverable via --reconcile' claim is about temporal CURSOR drift, a different (accurate) class"
  - "Corrected main.rs stale 'Cache::sync' → 'Cache::build_from' (sync.rs::run calls build_from directly) — a lying-doc fix noticed within the consistency-pass file"

patterns-established:
  - "A recovery-scope doc claim must be reproduction-backed: oid_drift_reconcile.rs's reconcile-non-recovery test is the empirical basis for the error.rs/sync.rs 'systematic class is NOT reconcile-recoverable' statement (SC4)"

requirements-completed: [FIX-02]

# Metrics
duration: 18min
completed: 2026-07-15
---

# Phase 114 Plan 02: reconcile-audit (FIX-02) Summary

**A backend-agnostic `DriftingMock` proves — against a reproduction, not an assumption — that divergent list/get bodies abort `read_blob` with `Error::OidDrift` and that a second `build_from` (exactly what `reposix sync --reconcile` runs) leaves the stale list-oid UNCHANGED; the `Error::OidDrift` / `--reconcile` doc comments are corrected to name the three coexisting drift classes and scope reconcile's recovery to the two it can actually heal (SC4).**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-07-15T16:39Z
- **Completed:** 2026-07-15T16:57Z
- **Tasks:** 2/2
- **Files modified:** 5 (1 test created, 3 code doc-comments, 1 Wave-1 SUMMARY fold-in)

## Accomplishments
- **Reproduction-backed the FIX-02 doc claim.** `crates/reposix-cache/tests/oid_drift_reconcile.rs` (`DriftingMock` + 3 tests, all GREEN) proves the list-vs-get render-drift class empirically rather than by assertion — the empirical floor SC4 asks for.
- **Corrected the reconcile-recovery overclaim** in `error.rs` (`Error::OidDrift`), `sync.rs` (module + `run` fn doc), and `main.rs` (`Sync` clap doc), naming all three coexisting drift classes precisely: (1) tree↔oid_map coherence drift, (2) eventual-consistency race (both reconcile-recoverable), (3) systematic backend rendering-representation mismatch (NOT reconcile-recoverable — needs the adapter fix).
- **Held Pitfall 3**: the docs still document reconcile as able to heal the eventual-consistency-race class — the fix is precision, not a blanket claim reversal.
- **Confirmed `read_blob`'s drift guard stays live** (T-114-04): the reproduction tests would turn RED if the `written_oid != oid` check were silenced.
- **Folded in the §5-item-1 doc-honesty correction** to `114-01-SUMMARY.md`'s "Residual gap": pre-ADF drift persists (list path now ADF-only, no storage fallback; get_record does fall back), answerable only by the live SC1 gate — mirroring the OQ1 honesty already at `db12187`.

## Task Commits

1. **Task 1: reproduction-backed oid_drift_reconcile test (DriftingMock)** — `9915953` (test)
2. **Task 2: correct reconcile-recovery doc comments + SUMMARY fold-in** — `0e200f9` (docs)

_(This plan is test + doc only; there is no source-behavior change to implement, so no RED→GREEN implementation pair — the 3 tests characterize/lock the already-fixed post-FIX-01 behavior and pass GREEN from creation, which is the plan's stated acceptance shape.)_

## The three tests (each asserts its named claim)

- **`pre_fix_divergent_bodies_trigger_oid_drift`** — mock NOT aligned; `build_from` stores the empty-body-derived oid; `read_blob(oid)` returns `Err(Error::OidDrift { issue_id, .. })` and the test asserts `issue_id == "1"`. Asserts the drift REPRODUCTION its name promises.
- **`reconcile_does_not_clear_stale_list_oid_while_bodies_diverge`** — `build_from` → `oid_a`; a SECOND `build_from` (the `sync --reconcile` path) → `oid_b`; asserts `oid_a == oid_b` (stale oid UNCHANGED) AND `read_blob` still returns `OidDrift`. Asserts the NON-RECOVERY its name promises — the empirical backing for SC4.
- **`aligned_bodies_resolve_without_drift`** — mock aligned (list body == get body); `read_blob(oid)` returns `Ok`, and the bytes are asserted equal to `frontmatter::render(&full[0])`. Asserts the RESOLUTION its name promises.

## Corrected doc text (verbatim excerpts)

- **`error.rs::Error::OidDrift`** — now names two causes with OPPOSITE recovery answers: cause 2 reads "A **systematic backend rendering-representation mismatch** — the same id renders to DIFFERENT bytes via `list_records` vs `get_record` regardless of timing … where it is NOT closed, `--reconcile` CANNOT heal it — a re-list reproduces the SAME mismatched oid deterministically". Contains the verbatim `systematic backend rendering-representation mismatch` and `CANNOT`.
- **`sync.rs`** (both module doc and `run` fn doc) — each gained a caveat: reconcile "heals tree↔`oid_map` coherence … and genuine eventual-consistency races, but a `systematic` backend-side rendering mismatch … requires the adapter/backend fix itself, not a reconcile." Contains `systematic` at both sites (grep count = 2).
- **`main.rs::Sync`** clap doc — "It does NOT repair a `systematic` backend rendering mismatch … that class needs the adapter/backend fix, not a reconcile." Contains `systematic`; also corrected the stale `Cache::sync` → `Cache::build_from`.

## cache.rs review (read-only, no edit)
`crates/reposix-cache/src/cache.rs::write_last_fetched_at` (lines 575-592) reviewed and left UNCHANGED. Its "Cursor drift is recoverable via `reposix sync --reconcile`" claim is about temporal CURSOR drift — a distinct, accurate class (`--reconcile` bumps `last_fetched_at` to `Utc::now()`), not the list-vs-get render class. `git diff --stat` shows zero change to cache.rs.

## Files Created/Modified
- `crates/reposix-cache/tests/oid_drift_reconcile.rs` (created) — `DriftingMock` BackendConnector + 3 reproduction/characterization tests.
- `crates/reposix-cache/src/error.rs` — `Error::OidDrift` doc names both drift causes (systematic class = not reconcile-recoverable).
- `crates/reposix-cli/src/sync.rs` — module + `run` fn doc scope `--reconcile`'s recovery, with the systematic-mismatch caveat.
- `crates/reposix-cli/src/main.rs` — `Sync` clap doc consistency pass + `Cache::sync`→`Cache::build_from` accuracy fix.
- `.planning/phases/114-.../114-01-SUMMARY.md` — Residual-gap section tightened (pre-ADF drift persists; SC1-gate answers it).

## Decisions Made
See frontmatter `key-decisions`. Headline: precision, not reversal (Pitfall 3 held); cache.rs cursor-drift claim confirmed accurate and untouched.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug / doc-accuracy] Corrected stale `Cache::sync` reference in the `Sync` clap doc**
- **Found during:** Task 2 (main.rs consistency pass)
- **Issue:** The `main.rs` `Sync` doc said `--reconcile` "runs a full `Cache::sync`", but `sync.rs::run` calls `Cache::build_from` directly (NOT `Cache::sync`, whose delta path it deliberately bypasses per ADR-010/RBF-LR-01). A lying-doc claim (OD-3 #2), inside the exact file the plan already scoped for a consistency pass.
- **Fix:** Changed `Cache::sync` → `Cache::build_from` in the same edit.
- **Files modified:** crates/reposix-cli/src/main.rs
- **Verification:** matches sync.rs::run source (`cache.build_from().await`).
- **Committed in:** `0e200f9` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 doc-accuracy bug). In-scope (same file as the planned consistency pass), no new dependency, no scope creep.
**Impact on plan:** Strengthens the family-alignment goal the main.rs pass exists for.

## Issues Encountered
- `cargo nextest` is not installed on this VM (already filed as GTH-10) — ran the plan's Task-1 verification via `cargo test -p reposix-cache --test oid_drift_reconcile` instead (equivalent coverage, one cargo invocation). Not re-filed.

## Known Stubs
None.

## Next Phase Readiness
- FIX-02 (doc-truth + reproduction) complete. The corrected docs and the reconcile-non-recovery test are the autonomous-testable floor.
- **Still gating phase-close (NOT run here — env-gated, coordinator/verifier scope):** SC1 (live TokenWorld checkout including page `7766017`, zero oid-drift) and SC2 (`t4-conflict-rebase-ancestry-real-backend.sh`). If SC1 trips `OidDrift` on a DIFFERENT page id, that page is likely pre-ADF (RESEARCH OQ1 / handover §5-item-4) — a named risk, escalate as a list-path storage-fallback follow-up, not a silent patch.

## Self-Check: PASSED
- `crates/reposix-cache/tests/oid_drift_reconcile.rs` — present, `struct DriftingMock` + all 3 named fns present, 3 tests GREEN via `cargo test`.
- `error.rs` — `systematic backend rendering-representation mismatch` + `CANNOT` present.
- `sync.rs` — `systematic` count = 2; `main.rs` — `systematic` present.
- `cache.rs` — `git diff --stat` shows no change.
- Commits `9915953` (test) and `0e200f9` (docs) present in git log.

---
*Phase: 114-t4-confluence-oid-drift-fix-first-reconcile-audit*
*Completed: 2026-07-15*
