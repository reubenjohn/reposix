---
phase: 105-rbf-lr-03-rebase-recovery
verified: 2026-07-12T09:22:19Z
status: passed
verdict: GREEN
score: 1/1 catalog row PASS (13/13 asserts)
head: 8afb52de9a9eeb7ee478f73be1b4b459fe433d74
row: agent-ux/rebase-recovery-reconciles
observed_exit: 0
transcript: quality/reports/transcripts/rebase-recovery-reconciles-2026-07-12T09-22-19Z.txt
verifier: unbiased phase-close verifier (no session context)
---

# Phase 105 — RBF-LR-03 rebase-recovery reconciliation — VERIFICATION

**Phase goal:** the DOCUMENTED recovery move — the single command
`git pull --rebase && git push` — genuinely reconciles after a push is rejected on
remote drift, across peer-git-push drift (A), external-REST-PATCH drift (B), and
record-deletion-at-SoT (C, must not resurrect). Two-layer fix: (1) `emit_import_stream`
emits `from <tracking-tip>` + `deleteall` (full rebuild); (2) helper import writes the
disjoint private ns `refs/reposix-import/*` so git fetch is sole writer of the tracking
ref.

**Status:** GREEN — graded from real gate execution at HEAD 8afb52d, not the executor's word.

## Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Gate exits 0 against local workspace + live sim | VERIFIED | I ran it: `OBSERVED_EXIT=0`, transcript 2026-07-12T09-22-19Z |
| 2 | Scenario A (peer git-push drift) converges via single documented cmd | VERIFIED | recovery exit 0, issue2 v1→v2 (live curl), no fatal fast-import, no ref-lock |
| 3 | Scenario B (external REST PATCH drift) converges | VERIFIED | recovery exit 0, issue2 v2→v3 |
| 4 | Scenario C (SoT deletion) propagates, does NOT resurrect | VERIFIED | exit 0, issues/2.md gone from worktree, SoT 404 (version -1) |
| 5 | Layer-2 guard: `cannot lock ref 'refs/reposix/origin/main'` absent both scenarios | VERIFIED | grep-absence asserts pass A+B; helper writes private ns |
| 6 | Layer-1 negative guard bites (parentless → `does not contain`) | VERIFIED | drives real `git fast-import`, reads `git rev-parse` (ref unchanged) |
| 7 | Deletion negative guard bites (overlay resurrects vs deleteall drops) | VERIFIED | reads git's OWN `git ls-tree -r --name-only`, distinguishes both shapes |
| 8 | Import-chain: `from <parent>` to private ns, ≥2-commit chain | VERIFIED | `git rev-list --count refs/reposix-import/main` = 3 |
| 9 | Clobber guard: helper never wrote refs/heads/* | VERIFIED | local main tip = user's own commit both scenarios |

**Score:** 13/13 asserts PASS, `asserts_failed: []`.

## Fix present in shipped source (8afb52d)

| Artifact | Provides | Status |
|----------|----------|--------|
| `crates/reposix-remote/src/fast_import.rs` | commit/reset → private `refs/reposix-import/main` (159/195), `from <parent>` chain (206), `deleteall` CR-01 (207+) | VERIFIED |
| `crates/reposix-remote/src/main.rs:202` | advertises `refspec refs/heads/*:refs/reposix-import/*` — git fetch sole writer of tracking ns | VERIFIED |

## Honesty audit

- **Asserts match gate 1:1.** All 9 `expected.asserts` map to live gate checks; all in `asserts_passed`. No claim-vs-assertion mismatch, no test-that-can't-fail (each positive assert paired with a concrete falsifier; convergence failure routes BLOCKED→exit 75, never a silent pass).
- **Negative guards read real git state** (`fast-import` + `rev-parse` + `ls-tree`), not the code's echo.
- **§5 stateless-connect SKIP is labelled** in the transcript (git 2.25.1 < 2.34, "Not faked"). Row `transport_claim: false` / `coverage_kind: mechanical` — no modern-git transport guarantee claimed. Honest.

## Noticed

- exit-75-on-convergence-failure is lenient-but-honest: a regressed fix surfaces as NOT-VERIFIED (with a filed reason), not FAIL — still blocks a P1 pre-push row, never silent-passes.
- Scenario-C precondition hard-fails if clone shape drifts (prevents vacuous deletion pass).
- `_provenance_note` honestly records hand-edited agent-ux mint + catalog-first NOT-VERIFIED→PASS path.

---

**VERDICT: GREEN.** Full write-up + constraint notes: `quality/reports/verdicts/p105/VERDICT.md`.

_Verifier: Claude (unbiased phase-close). Real execution only._
