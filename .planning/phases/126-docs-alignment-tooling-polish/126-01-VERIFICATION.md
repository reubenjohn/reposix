---
phase: 126-docs-alignment-tooling-polish
subphase: 126-01
verified: 2026-07-19T00:00:00Z
verifier: Claude (gsd-verifier, fresh zero-context leaf)
verdict: GREEN
status: passed
score: 11/11 must-have truths verified
mode: standard-goal-backward
evidence:
  head: 06781ad767510d07dc3624bc329383ea7f78b9d4
  ci_green_on_main_run: 29690120484        # ci.yml on head 06781ad7 -> success (all 15 jobs)
  real_git_push_e2e_runs:
    - run: 29690120484                      # newest main: PASS 0.64s
    - run: 29688725032                      # ba13553f: PASS 4.28s (SIGKILL rerun clean = confirmed flake)
  commits_confirmed: [ba13553f, 7f70b0de, dc60cc21, 588c1546, 65e8c497, 5d097937, f1959373]
warnings:
  - id: WARN-1
    row: structure/hermetic-test-network-isolation
    severity: P2 (non-blocking)
    finding: >-
      Brief claimed this PASSED on the clean CI attempt; LIVE evidence contradicts that.
      It FAILS deterministically at 0.02s (collection/setup-time fast-fail) on the newest
      main run 29690120484 AND on the superseded dc60cc21 run. It is NOT transient (same
      0.02s both runs) and NOT a P126 deliverable — the gate + its test were created by
      f1959373 (Cycle-2 task (d), 2026-07-19 01:21), an ancestor of P126's first commit
      44783ebe. Passes locally per handover; a pre-existing CI-portability bug in a sibling
      gate. Does NOT gate P126 close (P2, pre-pr exits 0; the P0 ci-green-on-main is GREEN).
    also: >-
      Catalog row committed status is PASS (last_verified 2026-07-19T08:12:32Z) while CI
      grades it FAIL — a stale local --persist mint that does not reflect CI reality.
    recommend: >-
      Coordinator FILE to SURPRISES-INTAKE as a standing (not transient) CI-portability
      bug; do not let the row's committed PASS ride as green.
  - id: WARN-2
    finding: >-
      REQUIREMENTS.md definition-list checkboxes for DRAIN-15..21 (lines 254-272) remain
      "[ ]" while the traceability table (lines 336-342) correctly reads Complete and the
      ROADMAP P126 box is [x]. Minor internal inconsistency; traceability table is
      status-of-record. Non-blocking.
---

# Phase 126: Docs-alignment tooling polish (DRAIN-15..21 + landmine + RAISE-3) — Verification Report

## VERDICT: GREEN

**Phase goal (ROADMAP.md:79):** the doc-alignment skill/tooling surface is more reliable
and less confusing — DRAIN-15..21, the HIGH `agent-ux/real-git-push-e2e` load-crash
landmine defused repro-first with a read-only write-path guard, and RAISE-3 removing the
stale-roadmap active-milestone lie.

**Verified:** 2026-07-19 · **Re-verification:** No — initial verification · **Verifier:**
fresh zero-context leaf, graded from committed artifacts + live `gh` ground truth.

GREEN because the two phase-close P0 gates are genuinely passing against LIVE CI evidence,
all 11 must-have truths (DRAIN-15..21 + landmine + RAISE-3 + W6 close) are verified in the
committed tree, and the only failing rows are P2 non-blocking: transient/owner-gated
docs-build badge rows and a pre-existing (non-P126) hermetic gate with a CI-portability
bug. Per the verdict rule, a P2 row does not roll the phase RED when the P0 rows are green.

## P0 gates (phase-close bar)

| Row | Status | Live evidence |
| --- | --- | --- |
| `agent-ux/real-git-push-e2e` (P0) | ✓ GREEN | run 29690120484 (head 06781ad7): `[PASS] agent-ux/real-git-push-e2e (P0, 0.64s)`; run 29688725032 (ba13553f): PASS 4.28s. Row shape correct: `minted_at` = 2026-07-19T11:05:43Z present; `expected.asserts` reduced to the 2 PASS-path claims (git<2.34 3rd assert removed = the F-K4b fix in ba13553f) |
| `code/ci-green-on-main` (P0) | ✓ GREEN | Newest ci.yml run on head 06781ad7 = 29690120484 = `success`; all 15 jobs green incl. `quality gates (pre-pr)` and `runner unit tests (hermetic)` |

The F-K4b defusal (`ba13553f`) is a real mechanism-level fix: it removed a
mutually-exclusive `git<2.34 → NOT-VERIFIED` entry from `expected.asserts` (that entry was
structurally uncoverable on the PASS path and silently demoted PASS→FAIL once `minted_at`
armed the grade-time F-K4b congruence check). Regression-locked by
`test_audit_field.py::TestFK4bMutuallyExclusiveBranch` via
`structure/asserts-congruence-grade-time`. Confirmed PASS on CI, twice.

## Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | W1 landmine: `minted_at` on `real-git-push-e2e`, stale `git 2.25.1` refreshed, read-only non-persist write guard | ✓ VERIFIED | `minted_at` present; `2.25.1` count in agent-ux.json = 0; DP-2 review `126-01-REVIEW.md` PASS (5d097937): `save_catalog(persist=)` raises on non-persist; regression tests `TestSaveCatalogPersistGuard`/`TestValidateOnlyMultiCatalogByteIdentical` |
| 2 | W1 RED-repro committed before fix (DP-2 catalog-first) | ✓ VERIFIED | 44783ebe (RED) precedes 65e8c497 (GREEN) in git log |
| 3 | DRAIN-17: walk BLOCK names blocking row-STATE(s) | ✓ VERIFIED | `block_state_summary()` (doc_alignment.rs:1399) emits `docs-alignment BLOCK: N row(s) blocking across M state(s):`; unit test at :1671-1744 |
| 4 | DRAIN-18: grader binds only when drift fails the test + grep src/ | ✓ VERIFIED | grader.md carries drift/FAILS/grep-src guidance (e693deeb) |
| 5 | DRAIN-20: `status` prints `waived_active` recomputed vs now | ✓ VERIFIED | `waived_active` x9 in doc_alignment.rs (0270f91c) |
| 6 | DRAIN-21: out-of-eligible coverage warnings dispositioned + implemented (17→2) | ✓ VERIFIED | `eligible_files()` extended (benchmarks/README.md + archived .planning + backend.rs/main.rs), e8823049; dispositions recorded inline in SUMMARY |
| 7 | DRAIN-16: README expands MCP on first use | ✓ VERIFIED | "Model Context Protocol (MCP)" present in README.md (1ef508bf) |
| 8 | DRAIN-19: refresh.md cold-plan-refresh under-reports note | ✓ VERIFIED | cold/under-report/walk-first note present (1ef508bf) |
| 9 | DRAIN-15: in-repo doc-clarity-review subscription/canary caveat; out-of-repo skill surfaced to L0 | ✓ VERIFIED | CLAUDE.md § Cold-reader caveat (1ef508bf); SUMMARY records GTH-V15-98 out-of-repo surfacing |
| 10 | RAISE-3: stale active-milestone lie removed; 5 rows re-cited in one commit; binding-free strip intact | ✓ VERIFIED (via documented deviation) | 588c1546 (ONE commit): docs/development/roadmap.md line 26 now "v0.15.0 'Floor' — ACTIVE" (was "v0.11.0 … PLANNING"); doc-alignment.json 5-row re-cite (66 lines); active-milestone gate header refreshed. Deviation: fixed IN-PLACE not deleted (confirm-retire is human-gated) — SUMMARY deviation #1, defensible |
| 11 | W6 close: STATE/ROADMAP/REQUIREMENTS advanced, part-08 CLOSED, roadmap strip refreshed, pushed | ✓ VERIFIED | ROADMAP:79 `[x]`; REQUIREMENTS traceability DRAIN-15..21 = Complete; part-08 STATUS: CLOSED (P126 W1); 7f70b0de close bookkeeping; HEAD=origin/main pushed |

**Score:** 11/11 truths verified.

## P2 rows (non-blocking — do not roll phase RED)

| Row | CI state | Classification |
| --- | --- | --- |
| `docs-build/badges-resolve` | FAIL 30s | Known-transient live-network. Legitimately transient. Verify-by-reobservation only. |
| `docs-build/p94-badges-real-vs-transient` | FAIL 67s | Known-transient live-network. Legitimately transient. |
| `structure/badges-resolve` | FAIL 30s | Same badge-network transient class. |
| `docs-build/animation-renders` | NOT-VERIFIED | Owner-gated stub (verifier absent, GTH-V15-37). Deliberate. |
| `structure/hermetic-test-network-isolation` | FAIL 0.02s | See WARN-1: deterministic CI-portability bug in a PRE-EXISTING (non-P126) gate. Passes locally. P2, does not block. |

Pre-pr summary on the newest main run: `79 PASS, 4 FAIL, 0 PARTIAL, 1 WAIVED, 1
NOT-VERIFIED -> exit=0`. All 4 FAILs are P2; every P0 row passed.

## Commits confirmed (spot-checked diffs)

- `ba13553f` — F-K4b grade-time demote defusal: removes git<2.34 assert from
  expected.asserts + adds `TestFK4bMutuallyExclusiveBranch`. Confirmed on CI (PASS 0.64s/4.28s).
- `588c1546` — RAISE-3: ONE commit, roadmap fix-in-place + 5-row re-cite + gate header refresh.
- `7f70b0de` — close bookkeeping: STATE/ROADMAP/REQUIREMENTS + part-08 CLOSED + docs/roadmap.md strip + SUMMARY.
- `dc60cc21` — GTH-V15-94..98 filing + PROTOCOL bare-session verdict note.

## NOTICED (verifier deliverable)

1. **WARN-1 (material):** The dispatch brief asserted `structure/hermetic-test-network-isolation`
   "ran and PASSED on the clean CI attempt." **This is false against live evidence** — it
   FAILS at 0.02s on the newest main run (and on dc60cc21). It is a P2 non-blocking row and
   NOT a P126 deliverable (created by `f1959373`, ancestor of P126's first commit), so it
   does not gate the close. But it is DETERMINISTIC, not transient (same 0.02s twice, passes
   locally) — a standing CI-portability bug (poisoned-proxy gate collection/setup fast-fails
   in the CI sandbox). The prior HANDOVER conflated it with the `container-rehearse-sigkill-safe`
   SIGKILL (which WAS a confirmed flake); these are two separate items — the hermetic 0.02s
   fast-fail is still live. Its catalog status is a stale local PASS mint. **Recommend the
   coordinator file it to SURPRISES-INTAKE** and not let the committed PASS ride as green.
2. **WARN-2 (minor):** REQUIREMENTS.md DRAIN-15..21 definition-list checkboxes (lines
   254-272) are still `[ ]` while the traceability table (336-342) reads Complete. Flip the
   definition checkboxes for internal consistency.
3. **Carried from DP-2 REVIEW WR-01 (non-blocking):** `test_freshness_synth.py:97,126` write
   catalogs via raw `json.dumps` (bypasses the new `save_catalog(persist=)` guard); bounded
   by the `backup_catalogs` restore fixture. This is the same test the hermetic gate runs —
   worth folding into the WARN-1 investigation, though WR-01 is a byte-identity concern, not
   the network/collection failure.
4. SUMMARY-filed good-to-haves (GTH-V15-94..98) and the `minted_at`/F-K4b coupling footgun
   are correctly carried forward, not silently dropped.

## Gaps Summary

None blocking. Phase goal achieved: DRAIN-15..21 landed, the HIGH load-crash landmine is
defused repro-first with a mechanism-level fix and a read-only write-boundary guard
(DP-2 review PASS), RAISE-3 removed the stale active-milestone lie (fix-in-place, atomic,
binding-free strip intact), and the phase-close P0 gates are green on live CI. The
outstanding P2 hermetic-isolation CI failure is a pre-existing sibling-gate portability
bug outside P126's scope — surfaced to the coordinator as WARN-1, not a P126 gap.

---

_Verified: 2026-07-19_
_Verifier: Claude (gsd-verifier) — fresh zero-context leaf, graded from committed artifacts + live gh ground truth_
