# P93 phase-close verdict (unbiased verifier subagent) — 2026-07-05T21:05:11Z

- **Overall: RED** — P93 not closeable at `bf3bc9c`.
- Verifier: unbiased subagent, zero implementation context. Graded against reality
  (ran every gate, read every transcript) + the catalog contracts, not anyone's word.
- Auto-rollup companion: `quality/reports/verdicts/p93/2026-07-05T20-57-23Z.md`
  (`verdict.py --phase 93` → red, 103/112 P0/P1 green).

## RED reason (ONE root cause, trivially fixable — substance already holds)

**The P93 phase-close skipped PROTOCOL Step 6 (run the runner) for its own rows.**
No verification artifact exists for ANY of the six P93 rows —
`quality/reports/verifications/agent-ux/p93-*.json` are **all MISSING** (none
git-tracked), and `last_verified: null` on all six. The runner never minted a PASS:

- **RBF-LR-01 / RBF-LR-02 / D-P92-03** carry an invalid `status: WAIVED` with
  `waiver: null`. Commit `3976789` deleted the waiver *block* but never ran the
  runner to perform the WAIVED→PASS flip the deleted waiver text itself documented
  ("flips WAIVED->PASS then [when the verifier lands]"). Result: a phantom-green
  status with no waiver justification AND no runner-minted PASS.
- **RBF-LR-04 / RBF-LR-05** carry `status: NOT-VERIFIED`, `last_verified: null`.

Per the PROTOCOL's foundational principle — *"If the verifier reads the catalog and
sees no artifact dated this session, the row is RED"* — every P93 row is ungraded.
`verdict.py`'s own rollup agrees (RED). The executing agent's word ("all three grade
PASS", `3976789` message) is not the verdict; the runner is, and it never ran.

**The substance is fully present and PASSES** — I ran all five gates and confirmed
RBF-LR-05's evidence. The fix is a re-run, NOT re-implementation (see § Fix).

## Per-row grades

| Row | Catalog id | Substance (ran it) | Catalog contract | Grade |
|---|---|---|---|---|
| RBF-LR-01 | p93-l2-l3-coherence-adr | gate **exit 0** (ADR-010: ACCEPTED, names L2+L3, Option C + trade-off, v0.14.0 deferral + owner-signoff, x-refs RBF-LR-02) | no artifact; `status=WAIVED/waiver=null`; `last_verified=null` | **NOT-VERIFIED** (blocks) |
| RBF-LR-02 | p93-cache-coherence-refresh-honest | gate **exit 0** (`cargo test -p reposix-cache --test cache_coherence` 3/3 green; `reposix-remote --test partial_failure_recovery` 1/1 green; refresh honesty holds) | no artifact; `status=WAIVED/waiver=null`; `last_verified=null` | **NOT-VERIFIED** (blocks) |
| D-P92-03 | p93-delta-sync-coherence-invariant | gate **exit 0** (`#[ignore]` removed; `delta_sync` 4 passed, 0 ignored; target fn ran + passed) | no artifact; `status=WAIVED/waiver=null`; `last_verified=null` | **NOT-VERIFIED** (blocks) |
| RBF-LR-03 | p93-partial-failure-recovery-real-confluence | gate **exit 75** (creds absent, fail-closed OD-2) | no artifact; `status=NOT-VERIFIED` | **NOT-VERIFIED-expected** (does NOT block phase-close) |
| RBF-LR-04 | p93-l1-promise-reconciled | gate **exit 0** (keep+qualify branch consistent across CLAUDE.md + dvcs-topology.md + troubleshooting.md; reconcile pointer + RBF-LR-04 cites present; no lying-doc token) | no artifact; `status=NOT-VERIFIED`; `last_verified=null` | **NOT-VERIFIED** (blocks) |
| RBF-LR-05 | p93-mid-stream-litmus-t1-t4 | **PASS from committed transcripts** (T1–T3 sim `dark-factory.sh sim` exit 0 → 0 HIGH; T4 4/4 PASS in git-2.54.0 container; TokenWorld arm NOT-VERIFIED-expected) | no artifact JSON; `status=NOT-VERIFIED`; `last_verified=null` | **NOT-VERIFIED** (evidence exists; row never graded/minted) |

### RBF-LR-03 cadence determination (independent, per PROTOCOL Step 6 / OD-2)
Row cadence = `["pre-release-real-backend"]` only (not pre-pr/pre-push). At **phase**-close
with creds absent, `exit 75 → NOT-VERIFIED` is the honest SLOT state and does **not** block
P93. The OD-2 hard-RED rule ("creds-missing ⇒ RED, no waiver") fires at **v0.13.0
milestone-close** (9th probe, `--cadence pre-release-real-backend`), not here. Its own
`expected.asserts[2]` codifies this ("without creds/allowlist: … NOT-VERIFIED … NOT PASS,
NOT FAIL"). Correctly deferred.

### RBF-LR-05 manual-kind grading (from committed evidence, per dispatch instruction)
- `rbf-lr-05-t1-t3-dark-factory-sim.txt`: `dark-factory.sh sim` **exit 0** → 0 HIGH frictions (T1 baseline).
- `rbf-lr-05-t4-git254-container.txt`: host git 2.25.1 (T4 needs ≥2.34), so run in a
  **git 2.54.0** container. All **4 T4 assertions PASS** — baseline push; stale-base push
  correctly rejected; refetched root IDENTICAL before/after (HIGH-1 stays fixed);
  ancestry advanced past shared root (count=2, non-vacuous) — **exit 0, not env-skipped.**
  Not RED'd for old on-box git per dispatch. TokenWorld arm NOT-VERIFIED-expected (creds absent).

## Gate-honesty findings (per script — the axis the dispatch flagged as RED-triggering)

**All five newly-landed scripts (`75bdd21`) GENUINELY assert their row's contract —
no trivial `exit 0`, no check weaker than the row's `expected.asserts`.** Verified
line-by-line:

- **p93-l2-l3-coherence-adr.sh** — greps the real ADR for both option ids (`re-fetch-on-cache-miss`(L2) + `transactional-cache-writes`(L3)), `Status ACCEPTED`, `## Decision`, chosen `Option C`, `v0.14.0` deferral + `owner sign-off` rule, x-ref `RBF-LR-02`. Maps 5/5 asserts. HONEST.
- **p93-cache-coherence.sh** — runs two **real** cargo test binaries sequentially (one-cargo budget honored), greps `partial_fail…replan…converg` fn, checks `refresh_for_mirror_head` files_touched-gate honesty in write_loop.rs + CLAUDE.md + dvcs-topology.md. Maps 3/3 asserts. HONEST.
- **p93-delta-sync-coherence.sh** — walks attribute lines to confirm `#[ignore]` truly removed from the target fn, runs real cargo, asserts `test <fn> … ok` observed AND `4 passed; 0 ignored`. Maps 3/3 asserts. HONEST (not vacuous — checks fn actually ran, not just suite exit 0).
- **p93-l1-promise-reconciled.sh** — branch-detects (files_touched gate present ⇒ keep+qualify), asserts matching qualifiers + `reposix sync --reconcile` pointer + `RBF-LR-04` cites in both docs, and troubleshooting.md doesn't lie ("L3 defers to v0.14.0"). Honestly leaves process-claim #4 (same-PR) to the subagent per row owner_hint. HONEST.
- **p93-partial-failure-recovery-real-confluence.sh** — OD-2 env-gate (exit 75), sanctioned-target hard-FAIL (exit 1, never 75) for non-TokenWorld, drives the real `--ignored` smoke, emits a `lib/transcript.sh` transcript. HONEST.

RBF-LR-05's verifier is the pre-existing `dark-factory.sh sim` (already-PASSING
`agent-ux/dark-factory-sim`); its T4 container evidence is genuine + complete.

**No gate-honesty RED.** The RED is purely the un-run-runner / stale-catalog defect.

## Phase-close hygiene
- HEAD == origin/main == `bf3bc9c` — push-before-verifier landed. ✓
- Un-waive `3976789` removed waiver blocks from **exactly** RBF-LR-01/02/D-P92-03 (3 rows,
  18 deletions / 3 `waiver:null` insertions); no other row touched. ✓ (but see RED reason —
  removal without runner flip is the defect, not the row scope)
- CLAUDE.md + docs/concepts/dvcs-topology.md + docs/guides/troubleshooting.md +
  docs/decisions/010-l2-l3-cache-coherence.md reconciled in-phase (QG-07); RBF-LR-04 gate
  PASS proves CLAUDE.md carries the RBF-LR-04-qualified "semantic no-op" caveat. ✓

## Fix (substance already verified GREEN by this verifier — expect green on re-run)
1. `python3 quality/runners/run.py --cadence pre-pr` — executes + mints PASS artifacts for
   RBF-LR-01/02/D-P92-03 (I confirmed each exits 0). Also refreshes the pre-existing
   mechanical rows currently stale-NOT-VERIFIED (real-git-push-e2e, t4-conflict-rebase-ancestry,
   p92-mid-stream-litmus, etc.) that also drag the rollup red.
2. `python3 quality/runners/run.py --cadence on-demand` — mints RBF-LR-04 (grep PASS) +
   RBF-LR-05 (dark-factory sim exit 0). Confirm the manual-kind LR-05 handling preserves the
   committed git-2.54 T4 transcript as its evidence.
3. RBF-LR-03 stays NOT-VERIFIED (exit 75, creds absent) — correct; defer to milestone-close 9th probe.
4. Commit the runner-minted catalog + artifacts, push, re-run `verdict.py --phase 93` → GREEN for P93 rows.

## NOTICED (ownership charter OD-3)
1. **`verdict.py --phase 93` is a pure rollup, NOT a grader, and `--phase` does not scope the
   P0/P1 gate to phase-93 rows** — the RED mixes P93 rows with unrelated pre-existing
   NOT-VERIFIED rows (real-git-push-e2e, cargo-binstall-resolves, subjective/dvcs-cold-reader,
   p92-*). A phase verifier could misread the global RED as a P93 failure, or rubber-stamp
   green by dismissing it. Suggest: `--phase N` emits a "phase-N rows: X/Y" sub-line, or scopes
   the gate. Filed candidate for GOOD-TO-HAVES.
2. **`status:WAIVED` + `waiver:null` loads silently and counts under "Waivers"** (toward green)
   with empty `until`/`tracked_in` ("?"). A deleted waiver leaves a phantom-green row.
   `_audit_field.py` should reject `status==WAIVED ⟺ waiver present` at load — mirroring the
   pre-release-real-backend waiver-refuses-to-load guard. This is a honesty hole. Filed candidate
   for SURPRISES-INTAKE (severity: MEDIUM — enables silent-descope).
3. **`3976789` commit message overstates** — "all three grade PASS" is false at `bf3bc9c`;
   deleting a waiver does not grade. The rows never graded PASS (no runner run).
4. **T1–T3 sim transcript** (line 11) shows the on-box init fetch failing with
   `blocked origin: http://127.0.0.1:7878` while the sim was spawned on :7779 (port/origin
   mismatch on git-2.25.1). `dark-factory.sh sim` still exits 0 — it validates config +
   recovery-hint messages, not a full fetch (consistent with the PASSING dark-factory-sim
   row). The real end-to-end fetch is exercised in the git-2.54 T4 container. Not a P93
   regression; worth documenting that the on-box sim arm is a config/message check.

---
*Verifier: Claude (unbiased P93 phase-close verifier). Evidence: gates run live + transcripts
read this session. verdict.py rollup: 2026-07-05T20-57-23Z.md.*
