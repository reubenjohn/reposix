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
