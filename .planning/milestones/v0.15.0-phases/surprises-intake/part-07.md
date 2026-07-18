# v0.15.0 Surprises Intake — Part 7 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-18 | discovered-by: P123 close wave (push cadence measurement, coordinator-filed) | severity: MEDIUM

> **Fourth corroborating data point for the existing pre-push-timing entries above
> (`2026-07-15 06:35`, `2026-07-15 17:18`, `2026-07-16 12:00`).** Does NOT duplicate —
> ties the creep directly to P123's own new gates. The drain phase should resolve all
> four together.

**What:** The P123 phase-close `git push` measured the pre-push hook at **~109s wall-clock**
against the `quality/CLAUDE.md` § Cadences documented **≈55s** budget — roughly 2×, and
still above even the `2026-07-15 17:18` entry's proposed **~75s** re-baseline. Likely
driver: P123 itself landed 4 new catalog rows/gates this phase
(`structure/verifier-script-exists` + its selftest, `structure/persist-refuses-downgrade`,
`structure/persist-catalog-write-locked`) plus the associated Python unittest suites
(`TestEnvSelfSourcing`, `TestPersistDowngradeGuard` [9 tests], `TestPersistCatalogLock`
[4 tests, ~4.3s of REAL cross-process flock contention by design]) — all whole-repo fixed
costs per `quality/CLAUDE.md`'s "runtime does NOT scale with diff size" rule, so this
creep compounds onto the pre-existing kcov/clippy baseline rather than replacing it.

**Why out-of-scope for the discovering session:** The P123 close-wave charter was
grading/advancing state and filing intake, not profiling or re-baselining the pre-push
cadence — a genuine investigation (stage-by-stage timing, deciding whether the new
selftests/tests can be cadence-scoped to `pre-commit`-exempt or optimized) is a bounded
lane of its own, best paired with the three prior corroborating entries.

**Sketched resolution (P124-investigation candidate):** Profile `python3
quality/runners/run.py --cadence pre-push` stage-by-stage (per-row timing, as the
`2026-07-15 17:18` entry already demonstrated) on current `main` to isolate how much of
the 109s is P123's 4 new gates/selftests vs. continued kcov/clippy baseline creep. Then
decide: (a) can any of the new selftests/unittests run at a cheaper cadence (e.g.
`pre-commit`-only, skipping `pre-push` re-execution) without weakening the guarantee; (b)
should `TestPersistCatalogLock`'s ~4.3s real-concurrency wall-clock proof be gated behind
a `--slow` opt-out for pre-push while staying mandatory at `pre-pr`/CI; (c) once measured,
update the `quality/CLAUDE.md` § Cadences budget to the real current baseline (resolve
jointly with the three prior entries rather than re-baselining three times).

**STATUS:** OPEN

## 2026-07-18 | discovered-by: gsd-verifier P123 phase-close verdict (`quality/reports/verdicts/p123/VERDICT.md`) | severity: LOW-MEDIUM

**What:** `code/shell-coverage` (kcov, P2 blast radius) shows a counter-validation drift
between its two coverage-counting mechanisms: the harness-side `transcript.sh` invocation
counter reports **34** covered call-sites while the underlying `kcov` line-coverage
instrumentation reports **27** — a **25.9% relative gap**, well above the gate's own
**15%** cross-check threshold. The row's AGGREGATE coverage floor still PASSED (the
gate's primary pass/fail signal), so this is non-blocking today, but the two counters
disagreeing by more than the gate's own tolerance for its own health-check is itself a
signal the counting mechanisms have drifted apart (e.g. transcript-side double-counts a
call-site kcov attributes once, or vice versa).

**Why out-of-scope for the discovering session:** The independent verifier's charter was
grading P123's 5 SCs against reality, not reconciling an unrelated P2 counter-validation
drift in a pre-existing gate outside P123's own SC set; the P123 close-wave executor's
charter was STATE-advance + intake filing, not a shell-coverage internals investigation.
Reconciling `transcript.sh`'s counting logic against kcov's (or deciding the per-file
threshold itself needs adjusting) is a bounded diagnostic lane of its own.

**Sketched resolution (P124-investigation candidate):** Read `transcript.sh`'s
call-site-counting logic side-by-side with kcov's line-coverage output for the same
run and determine which one over/under-counts (a plausible cause: the transcript counter
counts distinct invocation SITES in source while kcov counts distinct EXECUTED lines,
which naturally diverge when a single call-site spans multiple source lines or a single
source line is invoked from multiple call-sites) — then either (a) fix the counting logic
that's wrong so the two converge under 15%, or (b) if both counters are individually
correct but measuring genuinely different things, adjust the cross-check threshold (or
document why a >15% divergence between the two is expected and non-alarming for this
gate specifically) so a future run doesn't need to re-diagnose this from scratch.

**STATUS:** OPEN

## 2026-07-18 | discovered-by: gsd-verifier P123 phase-close verdict, NOTICED #1 (`quality/reports/verdicts/p123/VERDICT.md`) | severity: LOW / INFO

**What:** P123 SC1 (`run.py` self-sources `./.env` when present, DRAIN-03) makes the
operator's real backend credentials (`ATLASSIAN_API_KEY`, `GITHUB_TOKEN`,
`JIRA_API_TOKEN`, `REPOSIX_ALLOWED_ORIGINS`, `CARGO_REGISTRY_TOKEN`, …) live in EVERY
`run.py` process's environment whenever `./.env` is present — not just the specific
real-backend cadences that need them. The verifier judged this correct-and-intentional
(not a new attack surface: creds already sit on disk, gates are trusted first-party code,
and OP-1 fail-closed egress is unchanged — a real backend is still hit only when creds are
present AND `REPOSIX_ALLOWED_ORIGINS` is non-default), and the immediate gh-auth-shadowing
slice of this property is already separately audited (the P127 gh-auth-audit entry
elsewhere in this file). What remains outstanding is a one-line explicit security
sign-off on the broader property itself — "every `run.py` invocation now hydrates real
creds for all downstream gates, reviewed and accepted" — which the verifier flagged as
INFO-severity and appropriate for milestone-close, not a P123 blocker.

**Why out-of-scope for the discovering session:** This is a milestone-scoped security
review item (assessing a property that spans the whole quality-runner surface, not one
gate), not a P123 SC gap — the verifier explicitly graded P123 GREEN with this as a
non-blocking NOTICED item, and the close-wave executor's charter was STATE-advance +
intake filing, not a security-review lane.

**Sketched resolution (P128 / milestone-close-security candidate):** As part of
milestone-close (P128, OP-9 distillation), add an explicit one-line security sign-off
confirming: (1) no gate's stdout/stderr/persisted catalog output ever echoes a
`.env`-sourced credential VALUE (only key names, per the existing `_env_load.py`
docstring contract); (2) the gh-auth-shadowing P127 slice landed or is tracked; (3) the
broader "creds now hydrate into every `run.py` process" property is a reviewed, accepted
tradeoff, not an overlooked expansion of blast radius. Cross-ref `_env_load.py`'s
docstring (already documents the non-clobbering / present-only contract) so the sign-off
cites the real implementation, not just this intake row's paraphrase.

**STATUS:** OPEN

## 2026-07-18 15:47 | discovered-by: quick-260718-fork (fork-anti-pattern doctrine + intake-filing lane) | severity: MEDIUM

**What:** The P123→P124 split-archive lane (commit `f654cfc3`) split the two v0.15.0
intake ledgers (`GOOD-TO-HAVES.md`, `SURPRISES-INTAKE.md`) under the 20k budget — but that
did NOT de-risk the `structure/file-size-limits` waiver deadline. That waiver is a
**SINGLE GLOBAL row** in `quality/catalogs/freshness-invariants.json`
(`waiver.until: 2026-08-08T00:00:00Z`) still covering **91 over-budget files** (re-counted
2026-07-18 via the gate; the row's own historical "56 files" 45/6/5 breakdown is now
**STALE ON THE COUNT** — the reason text self-notes this). Splitting the two intakes
removed exactly those two ledgers from the set; the remaining **91** still ride the global
waiver, including STATE.md, ORCHESTRATION.md (now 21733B — grew further via THIS quick's
own §11 edit), the three milestones' ROADMAP/REQUIREMENTS pairs, archived phase/handover
bundles, and GTH-V15-78's `quality/gates/agent-ux/rebase-recovery-reconciles.sh` (~42k, 4×
the 10k `.sh` ceiling). Several — STATE.md (concurrent-writer, never hand-edit) and
ORCHESTRATION.md (read-first entry-point contract) — **cannot** be ledger-split without
breaking their "read-first entry-point" contract, so no amount of `split_ledger.py` sweeps
clears them; they need a per-file permanent waiver or an accepted-residual decision.

**Why out-of-scope for the discovering session:** This quick's charter was the
fork-to-resume anti-pattern doctrine (ORCHESTRATION §11 + coordinator-dispatch §6a) plus
this intake filing — not a repo-wide waiver-remediation sweep. The remediation decision
spans three milestones' planning artifacts plus repo-wide docs/scripts, needs an L0/owner
call on which residuals are genuinely un-splittable, and is broader than v0.15.0.

**Sketched resolution (needs L0/owner decision — broader than v0.15.0):** Three options.
(a) **Extend** the global waiver `until` past 2026-08-08 — buys time, defers the real work,
lowest effort. (b) **Split-sweep + permanent per-file waiver:** run `scripts/split_ledger.py`
across the genuinely-splittable `## `-delimited ledgers (the mechanic already proven on the
v0.13.0 + v0.15.0 intake twins), AND grant a **permanent** per-file waiver for the
genuinely-un-splittable entry-point docs (STATE.md, ORCHESTRATION.md — splitting breaks
their read-first contract) plus the large single-purpose scripts (rebase-recovery-reconciles.sh
needs a `lib/` source-helper convention before factoring — GTH-V15-78). (c) **Accept** the
residual set as permanently waived and downgrade the gate to print-only WARN for those
specific paths. Cross-ref **GTH-V15-21** (archived-milestone handover files start BLOCKING
pushes when this waiver expires) and **GTH-V15-78** (rebase-recovery-reconciles.sh 42k `.sh`).

**STATUS:** OPEN

## 2026-07-18 | discovered-by: P124 W1a + W2 (source=P124, migrated at phase close from phase-local `deferred-items.md`) | severity: LOW-MEDIUM

> **Third + fourth corroborating data points for the existing `code/shell-coverage`
> counter-validation-drift entry above (`2026-07-18 | gsd-verifier P123 phase-close
> verdict | LOW-MEDIUM`).** Does NOT duplicate — confirms the drift is stable across two
> more independent P124-wave runs on the SAME box, tightening the "environmental / local
> kcov, not a regression" diagnosis. Drain jointly with the P123-verifier entry.

**What:** Two P124 waves independently observed `code/shell-coverage` grade **FAIL (P2)**
locally, each time from the SAME anti-gaming counter-validation flip, NOT the aggregate
floor: `quality/gates/agent-ux/lib/transcript.sh` measures `counter=34 vs kcov=27 = 25.9%`
(> the gate's 15% cross-check), so the assert "coverable-line counter validated within 15%
of kcov on all executed scripts" fails. Wave 1a (SC1/DRAIN-22): FAIL at 64.81s, aggregate
17.78% ≥ 13.0% floor (aggregate assert PASSES). Wave 2 (SC2/DRAIN-23): FAIL at 62.10s,
aggregate 17.52% ≥ floor. The P124 close-wave re-measured a THIRD time (this migration
run, 2026-07-18): identical FAIL at 64.97s, same 34-vs-27 flip. Committed HEAD status is
**PASS** (last CI-verified 2026-07-16); `transcript.sh` + `scripts/shell_coverage.py` are
untouched by every P124 wave (last touched 2026-07-13, pre-P124); the new P124 scripts
only enter the aggregate denominator (which stays above floor), never the counter-validated
harness set. P2 → does not gate the pre-push P0/P1 exit code; `run.py` here is validate-only
(no `--persist`) so the FAIL is never written back to the committed catalog. It is case (c)
in the row's own `owner_hint` (kcov line-counter drift) and a documented local-vs-CI kcov
discrepancy (`quality/CLAUDE.md` § Shell-coverage ratchet — the two-honesty-layer note added
at P124 close explains *why* a small script legitimately breaches the 15% relative threshold).

**Why out-of-scope for the discovering session:** Each P124 wave's charter was its own SC
(container-rehearse harness hardening), not reconciling a pre-existing P2 counter drift in a
gate outside every wave's touched-file set. SCOPE BOUNDARY: only auto-fix issues DIRECTLY
caused by the wave's changes; this FAIL predates P124 and reproduces on unchanged files.

**Sketched resolution (drain jointly with the P123-verifier entry above):** If the drift
recurs in **CI** (not just this box's local kcov), retune `scripts/shell_coverage.py`'s
`coverable_line_count` bash-aware skip rules for `transcript.sh` so the static counter
converges under 15% of kcov's coverable total — OR, if both counters are individually
correct but measuring genuinely different "coverable" heuristics (the P124-close honesty
note argues this for small scripts), raise the per-file cross-check threshold or exempt
scripts under N coverable lines from the relative-drift assert. Do NOT lower the aggregate
floor — that assert is healthy.

**STATUS:** OPEN
