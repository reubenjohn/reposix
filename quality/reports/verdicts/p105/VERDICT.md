---
phase: 105-rbf-lr-03-rebase-recovery
milestone: v0.14.0
verified: 2026-07-12T09:22:19Z
status: passed
verdict: GREEN
score: 1/1 gradeable row PASS (13/13 asserts)
verifier: unbiased phase-close verifier (no session context — graded from real gate execution)
head: 8afb52de9a9eeb7ee478f73be1b4b459fe433d74
row: agent-ux/rebase-recovery-reconciles
observed_exit: 0
transcript: quality/reports/transcripts/rebase-recovery-reconciles-2026-07-12T09-22-19Z.txt
constraint_notes:
  - "ONE-cargo budget honored: no concurrent cargo/sim at run time (pgrep clean). Held the single slot, CARGO_BUILD_JOBS=2."
  - "Leaf isolation: the gate ran its own mktemp /tmp run-dir; I created no shared-tree git setup."
  - "Did not touch STATE.md / MANAGER-HANDOVER.md, the 3 fleet-safety JSONs, the concurrent-session untracked dirs (phases/21, phases/22, scripts/demos, scripts/dev), or stash@{0}."
---

# Phase 105 "RBF-LR-03 rebase-recovery reconciliation" — Verification Report

Graded against reality, not the executor's word. I ran the gate myself against
the shipped binary at HEAD 8afb52d and confirmed exit 0 (`OBSERVED_EXIT=0`), then
audited the row for honesty.

## Row — agent-ux/rebase-recovery-reconciles — PASS

**Gate ran for real (I ran it).** `bash quality/gates/agent-ux/rebase-recovery-reconciles.sh`
→ exit 0. Rebuilt reposix-remote / reposix-cli / reposix-sim from current source,
spun a live sim on :7988, drove the REAL git-remote-reposix helper through all three
drift scenarios. Fresh transcript `rebase-recovery-reconciles-2026-07-12T09-22-19Z.txt`.
13/13 asserts PASS, `asserts_failed: []`.

**All 3 scenarios verified against reality (not stubs):**

- **Scenario A (peer git-push drift).** Clone B's single documented
  `git pull --rebase origin main && git push origin main` exits 0; issue2 SoT
  version converged v1→v2 (read live via `curl .../issues/2 | json['version']`,
  before+1). No `fatal: error while running fast-import` / `does not contain`
  (layer-1 fix holds); no `cannot lock ref 'refs/reposix/origin/main'` (layer-2
  fix holds). CLOBBER-GUARD: local `refs/heads/main` tip is `B edits issue2`
  (helper never wrote refs/heads/*); private `refs/reposix-import/main` present.
  IMPORT-CHAIN: `refs/reposix-import/main` is a 3-commit `from`-chained history
  (a parentless re-mint would be 1) — assert #7 concretely satisfied.
- **Scenario B (external REST PATCH drift).** `curl -X PATCH` moves the SoT (not a
  git push); clone C's documented recovery exits 0; issue2 converged v2→v3. Same
  layer-1 / layer-2 / clobber guards pass.
- **Scenario C (record DELETED at SoT).** REST DELETE issue2 (204); clone D's
  documented recovery exits 0; the deletion PROPAGATED (issues/2.md left the
  working tree after rebase) AND the push did NOT resurrect it (SoT stays 404,
  version -1). CR-01 `deleteall` full-rebuild holds.

**Negative guards genuinely BITE (Rule-1 clear — they read git's own state, not the
code's echo):**

- **Layer-1 NEGATIVE GUARD.** Drives `git fast-import` directly with a parentless
  non-descendant snapshot against a seeded tracking ref → reproduces the exact
  pre-fix RED baseline `does not contain`, and asserts the ref stayed put
  (`git rev-parse` before==after). If this string does not appear the gate
  `finish 1`s — it cannot silently pass.
- **DELETION NEGATIVE GUARD (CR-01).** Feeds `git fast-import` both emission shapes
  and reads git's OWN `git ls-tree -r --name-only`: the pre-fix overlay
  (`from`+`M`, no deleteall) RESURRECTS issues/2.md; the post-fix rebuild
  (`from`+`deleteall`+`M`) DROPS it. Distinguishes the regression via real tree
  state, not an echoed string. `finish 1`s if it can't distinguish.

**Fix is in the shipped source at 8afb52d (not a stale run):**

- `crates/reposix-remote/src/fast_import.rs` — writes `commit`/`reset
  refs/reposix-import/main` (private ns, lines 159/195), chains `from <parent>`
  (line 206), and emits `deleteall` for CR-01 (line 207+). `minted_at`
  2026-07-12T07:59:33Z, `last_verified` refreshed by real runs today.
- `crates/reposix-remote/src/main.rs:202` advertises
  `refspec refs/heads/*:refs/reposix-import/*`, so git fetch remains the SOLE
  writer of `refs/reposix/origin/*` — the two-namespace remote-helper contract.

**Row is HONEST — expected.asserts map 1:1 to gate checks:**
all 9 expected.asserts (overall-exit-0, Scenario A, Scenario B, layer-2 no-lock
guard both scenarios, clobber guard, layer-1 negative guard, import-chain assert #7,
Scenario C, deletion negative guard) each map to a live gate assertion, and every one
appears in `asserts_passed`. No claim-vs-assertion mismatch. No test-that-can't-fail:
each positive assert has a paired concrete falsifier, and a non-converging recovery
routes to BLOCKED→exit 75 (NOT-VERIFIED with a filed reason), never a silent PASS.

**§5 stateless-connect skip is LABELLED, not swallowed.** git 2.25.1 here; the gate
forces `protocol.version=0` to hit the `import` path deterministically and records a
LABELLED SKIP in the transcript ("STATELESS-CONNECT: SKIPPED — git 2.25 < 2.34 …
Not faked."). The row's `transport_claim: false` / `coverage_kind: mechanical` are
honest: it claims client-side fast-import + fetch ref-update mechanics against the
sim, NOT a modern-git transport-layer or latency guarantee. The row asserts nothing
about stateless-connect, so nothing is over-claimed.

## Noticed

- **exit-75 on convergence failure is lenient-but-honest.** A regressed fix routes
  through `BLOCKED=1 → finish 75 → NOT-VERIFIED` (with a reason filed to
  SURPRISES-INTAKE), not a hard FAIL. For a P1 pre-push row this still blocks (does
  not silently pass), and it correctly distinguishes "documented recovery does not
  converge" from "gate errored". Acceptable; flagged so the next reader knows a
  future regression surfaces as NOT-VERIFIED, not FAIL.
- The Scenario-C precondition (`issues/2.md must exist before deletion`) hard-`finish
  1`s if the clone shape drifts — good defensive check, prevents a vacuous deletion
  pass.
- `_provenance_note` honestly records the hand-edited agent-ux mint (bind only
  supports docs-alignment) and the catalog-first NOT-VERIFIED→PASS Lane-0→Lane-2
  path. No phantom-green.

---

**VERDICT: GREEN.** I observed exit 0 running the gate against 8afb52d. All three
drift scenarios (peer git-push, external REST PATCH, SoT deletion) converge via the
SINGLE documented `git pull --rebase && git push`; both negative guards bite against
real git state; the two-namespace + deleteall fix is in the shipped source; the row's
asserts match the gate and the git-2.25 stateless-connect skip is labelled, not faked.

_Verifier: Claude (unbiased phase-close). Real execution only._
