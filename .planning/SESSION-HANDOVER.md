# SESSION-HANDOVER.md — v0.15.0 Floor: P115 BENCH-01 refresh-recovery wave CLOSED + PUSHED, Wave 2/T3 next — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #33** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #34**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#32→#33's
handover, superseded here).

**Read order:** this file → §1 (verify live, note CI is IN PROGRESS) → §6 runbook
(confirm CI green is act ONE, then Wave 2 / T3 ledger scaffold) → §3/§4/§5 as needed.
**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager, pane w1:p7). No tag push by any coordinator —
the manager cuts tags, never L0. Do NOT do git surgery (reset/rebase/reorder/amend) on
`main`. Shared tree with the manager — TARGETED staging only, never `git add -A`/`.`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```
**Verified independently this handover (2026-07-15, just now):**
- `HEAD` = `92c3ab50df31e903f3dfd2d61915dccf9b483aa4` ("fix(latency-bench): enforce
  real-backend 3s WARN + refresh latency doc-alignment rows (P115)"). Tree **CLEAN**
  (`git status --porcelain` empty).
- `git rev-list --left-right --count HEAD...origin/main` → **`0  0`** — local `main` is
  **even with `origin/main`, fully pushed**. #33 pushed 5 commits this rotation
  (`9384ca6`, `3845b13`, `eac08a1`, `af4bbd4`, `92c3ab5`); the push landed
  `3278abc..92c3ab5 main -> main`.
- **`gh run list --branch main --workflow CI --limit 3`** (verified this handover):
  ```
  in_progress   —          fix(latency-bench): enforce real-backend 3s WARN…  CI  main  push  29455190072  1m39s  2026-07-15T22:22:38Z
  completed     success    docs(planning): A1 ruled [SELF] — benchmark…       CI  main  push  29452237641  5m28s  2026-07-15T21:31:34Z
  completed     cancelled  docs(planning): L0 relief handover #31→#32…        CI  main  push  29451926899  5m28s  2026-07-15T21:26:20Z
  ```
  **The newest run (on `92c3ab5`, the tip #33 pushed) was still `in_progress` at
  handover time — NOT YET CONFIRMED GREEN.** This is the single most important fact in
  this handover: #34 inherits an unresolved CI-green-on-main check, not a clean floor.
- Recent commit history (`git log --oneline -8` from HEAD):
  ```
  92c3ab5 fix(latency-bench): enforce real-backend 3s WARN + refresh latency doc-alignment rows (P115)
  af4bbd4 docs(planning): T6 headline-framing ruled [SELF] — honest CI-canonical reframe proceeds; manager handover → #33
  eac08a1 docs(planning): L0 relief handover #32→#33 — P115 Wave-1 CLOSED (T2 latency corrected), T3→T6 next
  3845b13 docs(115): correct sim latency to CI canonical figures, ledger + defect filing
  9384ca6 docs(benchmarks): re-measure v0.9.0 latency envelope (P115 BENCH-01 T2)
  3278abc docs(planning): A1 ruled [SELF] — benchmark session = one agentic conversation; manager handover → #32
  804f5b0 docs(planning): L0 relief handover #31→#32 — P115 planning DONE, execution next
  8e1e970 docs(115): create phase plan — BENCH-01 live MCP benchmark re-measurement
  ```
- **After this handover's own commit lands**, local `main` will be **1 ahead of
  origin/main** (the handover commit itself). This is **INTENTIONAL** per dispatch
  instructions — the handover commit stays local; #34 or phase-close pushes it. Do NOT
  treat that `ahead=1` as an anomaly, and do NOT assume it means work is unpushed —
  only the handover commit itself is unpushed at that point.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE | `9384ca6`, `3845b13` |
| Refresh-recovery (this rotation, #33) | `/reposix-quality-refresh docs/benchmarks/latency.md` — clear `STALE_TEST_DRIFT` on 14 rows + real-backend 3s WARN gap fix | **DONE + PUSHED** | `92c3ab5` |
| Wave 2 / T3 | Session-spend ledger scaffold (`benchmarks/bench-session-ledger.md`), ZERO session budget | **NOT STARTED — #34's opening move (after CI-green check)** | — |
| Wave 3 / T4 | Live capture (both fixtures, real sessions) | NOT STARTED | — |
| Wave 4 / T5 | Token-economy regen (`token-economy.md`) | NOT STARTED | — |
| Wave 5 / T6 | Un-waive path + consolidation, phase-close push+verify | NOT STARTED | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | NOT STARTED (blocked on P115 close) | — |

**Named-incident post-mortem to read before dispatching an executor:** none new this
rotation — no incident, only a planned recovery (`/reposix-quality-refresh`) triggered
by #32's prior CI-canonical rewrite leaving 14 doc-alignment rows stale. That recovery
is now closed. The one open thread is the **latency.md regeneration-clobber tension**
(see §5) — not an incident, a design note to carry into T5/T6.

### What #33 did this rotation — the refresh-recovery wave (COMPLETE + PUSHED)

- Opening move: `/reposix-quality-refresh docs/benchmarks/latency.md` — the named
  recovery for the pre-push `STALE_DOCS_DRIFT` block caused by #32's CI-canonical
  latency rewrite.
- `plan-refresh` surfaced **14 stale rows** (`STALE_TEST_DRIFT`). Dispatched **3 Opus
  graders** batched by claim family (sim+capabilities / real-backend cells /
  soft-thresholds) — a deliberate ROI call given the 75% weekly-limit warning,
  preserving unbiased per-row Opus judgment.
- 13 rows rebound GREEN to CI-canonical figures. **Grader C found a real lying-doc
  defect:** the claim "real-backend step < 3s — regression-flagged via WARN" was
  documented (`latency-bench.sh:35`, `emit-markdown.sh:101`) but UNENFORCED — only sim
  had a WARN (`sim.sh:48-49`).
- **Eager-fixed** (OP-8, <1h, no new dependency) via a sonnet executor: added
  `warn_if_over_3s()` to `quality/gates/perf/latency-bench/lib.sh`, wired per-step into
  `github/jira/confluence.sh` (guard fires >3000ms, dormant at current ≤1136ms
  figures). Rebound the row GREEN. **Dispatcher `latency-bench.sh` +
  `docs/benchmarks/latency.md` untouched** (12 rows hash the dispatcher byte-for-byte).
- `reposix-quality doc-alignment walk` → **EXIT=0** (only the 8 pre-existing WAIVED
  hero rows + informational `coverage:` lines remain).
- Committed **`92c3ab5`** (5 files: catalog + 4 helper scripts; targeted staging).
  Pre-commit exit 0.
- **PUSHED:** `3278abc..92c3ab5 main -> main`. Pushed 5 commits (`9384ca6`, `3845b13`,
  `eac08a1`, `af4bbd4`, `92c3ab5`). Pre-push **61 PASS, 0 FAIL, 1 WAIVED, exit=0**;
  secret scan clean.

## 3. Binding constraints (unchanged — carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push by any coordinator; no git surgery on
main; leaf isolation in `/tmp` same-invocation; opus complex / sonnet default / haiku
mechanical, never fable at a leaf; relieve past ~100k own-context (hard 150k, absolute
not %) at a wave boundary; manager (w1:p7) POLLS — clear in-pane narration at
boundaries IS the report, escalate actively only for owner-blocking moments; push at
green, then confirm CI green on main AFTER the push (`code/ci-green-on-main` P0
post-push probe — never open the next phase/wave over a red or unresolved main);
watch item: flaky `test` CI job (re-run once before treating as real).

## 4. Litmus / gate / REOPEN state

- Pre-push (on `92c3ab5`): **61 PASS, 0 FAIL, 1 WAIVED, exit=0**; secret scan clean.
- `reposix-quality doc-alignment walk`: **EXIT=0** — only the 8 pre-existing WAIVED
  hero rows (docs/index + README, waived until 2026-08-15, see §5/§6) + informational
  `coverage:` lines remain outstanding. No open STALE_* rows.
- CI on `92c3ab5` (the pushed tip): **`in_progress`** at handover time, run id
  `29455190072`, started `2026-07-15T22:22:38Z`. **NOT YET CONFIRMED GREEN — this is
  #34's first required check (§6 step 1).**
- No open waiver clocks expiring imminently beyond the standing 8 hero rows
  (2026-08-15, unaffected by this rotation).
- No REOPEN state pending.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **This rotation's real-backend-3s-WARN gap was EAGER-FIXED in `92c3ab5`, NOT
  filed** — it is CLOSED, do not re-file.
- **latency.md regeneration-clobber tension (OPEN, unresolved):**
  `emit-markdown.sh` REGENERATES `latency.md` (incl. the soft-threshold section) from a
  LOCAL bench run — a local run WILL clobber #32's CI-canonical figures. Reconcile in
  T5/T6 (options: teach the generator to pull CI figures / move generation into CI /
  header-warn against local regen).
- **latency.md is doc-alignment-TRACKED** — editing it AGAIN for the T6 headline
  reframe RE-DRIFTS its 14 rows. **Budget a SECOND
  `/reposix-quality-refresh docs/benchmarks/latency.md` before the T6 phase-close
  push.** Grep `quality/catalogs/doc-alignment.json` for ANY doc before editing it.
- **T6 headline framing RULED [SELF]** (`.planning/CONSULT-DECISIONS.md`): honest
  CI-canonical reframe proceeds NOW, no owner gate; cherry-picked hero numbers = a
  lying-doc defect.
- **8 hero-number doc-alignment rows** (docs/index + README) are
  WAIVED-MISSING_TEST until **2026-08-15** — T6 un-waives them AFTER T4/T5 re-measure
  (confirmed present in this rotation's walk output).
- Ledger entries to **DELETE at P115 close** (per each entry's own instruction in
  `.planning/CONSULT-DECISIONS.md`): A1 [SELF] (once T3 encodes it verbatim), P115-T2
  canonical-CI methodology [SELF] (once T6 encodes into un-waive path), P115-T6
  headline-framing [SELF] (once T6 lands). **Don't delete before the named consumer
  lands.**
- **Intakes — do NOT re-file:** all prior (GTH-V15-* etc.) + the sim
  `expected_version` PATCH defect (`SURPRISES-INTAKE.md`, MEDIUM, OPEN, from `3845b13`).
- **Weekly-limit watch:** pane showed **75% weekly limit used** at this rotation's
  start; #33 spent ~5 subagents (3 graders + 1 executor + 1 digester). Surface a
  limit-stall to the MANAGER immediately if hit — this budget pressure did not ease on
  its own and should be assumed still tight for #34.
- **Background shells/monitors: NONE running** — all graders + executor + digester
  from #33's rotation completed and reported. Nothing left running for #34 to inherit.

## 6. Precise next steps (successor runbook)

1. **FIRST ACT — confirm CI green on `92c3ab5` before opening T3.** CI run
   `29455190072` was `in_progress` at handover time.
   - `gh run list --branch main --workflow CI --limit 3` — wait for the top row
     (`92c3ab5`) to reach `completed`.
   - Then `python3 quality/runners/run.py --cadence post-push --persist` — the
     `code/ci-green-on-main` (P0) probe asserts main's NEWEST `ci.yml` run concluded
     success (not merely that some older green run exists).
   - **If the flaky `test` job goes red, re-run it ONCE** (carried watch item) before
     treating it as a real regression. If it's still red after one re-run, STOP — do
     not open T3 over a red main; escalate per the retro/incident norms.
   - Only once green is confirmed: proceed to step 2.

2. **Wave 2 / T3 — session-spend ledger scaffold (ZERO session budget, no live
   sessions spent yet).**
   - File: `benchmarks/bench-session-ledger.md` (OUTSIDE `docs/` — dodges the mkdocs
     orphan-doc invariant).
   - 8 columns, EXACT order: `# | timestamp (UTC, ISO-8601) | backend | arm
     (mcp-mediated / reposix-mediated) | task | unit_consumed (per ruling) |
     running_total | artifact_produced (which fixture)`.
   - Header MUST record VERBATIM the A1 ruling: **one session = one live agentic
     conversation/task run** (failed/aborted runs count); **≤50 ceiling**; **escalate
     to MANAGER past 50**; flag any session >~5x median token spend. Full text:
     `.planning/CONSULT-DECISIONS.md` 2026-07-15 A1 entry.
   - Commit the EMPTY schema (zero data rows) BEFORE any session spend — never
     backfill.
   - Verify: `grep -qiE 'running_total' benchmarks/bench-session-ledger.md &&
     grep -qiE '50' benchmarks/bench-session-ledger.md && echo LEDGER_SCAFFOLD_OK`.
   - Plan refs: `.planning/phases/115-*/115-PLAN.md` (or wherever P115's plan lives —
     confirm exact path via `.planning/STATE.md`) ~lines 252-272 (T3), 260-261
     (header), 264 (columns), 269-272 (verify/done).
   - Once T3 lands, delete the A1 [SELF] entry from `CONSULT-DECISIONS.md` per its own
     instruction (now that T3 has encoded it verbatim).

3. **Wave 3 / T4 — live capture (the expensive session-spending wave).**
   - Both fixtures must be REAL live captures (no `/mnt`/`scripts/demo.sh`), one ledger
     row per session, final `running_total` ≤50 (or MANAGER escalation), no leaked
     creds.
   - §5 GA intel from a prior handover still applies — **re-verify at T4 start, don't
     trust staleness:** Rovo MCP GA, **API-token path not OAuth**;
     `MAX_MCP_OUTPUT_TOKENS` identical across arms; instrument BOTH transcripts via
     `session-analyzer`.
   - **#34 should relieve BEFORE T4 if deep in context** — T4 is explicitly flagged as
     the wave most likely to blow the ~100k line given its session-spend nature.

4. **Wave 4 / T5 — token-economy regen.**
   - `token-economy.md` regenerated, offline-cache-stable, honest provenance (no
     `scripts/demo.sh`/"modeled on"), methodology note (MCP server + task), README
     matched.
   - RECONCILE the latency.md regeneration-clobber tension here (§5) or explicitly
     hand it to T6 — do not leave it silently unresolved past T5.

5. **Wave 5 / T6 — un-waive path + consolidation + phase-close.**
   - `115-UNWAIVE-PATH.md` names both perf-targets rows + the confirmed-absent
     cross-check script (authoring it is future work, NOT P115 scope), maps the 3
     fresh figures to the 8 doc-alignment rows, confirms docs consumable, re-asserts
     `running_total` ≤50, encodes canonical-CI methodology into the un-waive path.
   - Budget the SECOND `/reposix-quality-refresh docs/benchmarks/latency.md` here
     (§5) before the phase-close push.
   - Delete the P115-T2 and P115-T6 [SELF] ledger entries per their own instructions
     once encoded.
   - Phase-close ritual: `git push origin main`, confirm `code/ci-green-on-main`
     green, run the verifier subagent for a catalog-row PASS verdict, advance
     `.planning/STATE.md` cursor, RAISE LIST / intake disposition, final report.

6. **After P115 closes — P116 ADR-010 packet.** Produce options+tradeoffs for
   ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id, route to **MANAGER (w1:p7) for
   ruling, NO pre-ruling implementation.**

7. **Open items to carry (all OPEN, not yet actioned):**
   - **roadmap-diagram gsd-quick** — owner-approved todo
     (`.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md`);
     interleave opportunistically; refresh any doc-alignment-tracked doc it touches.
   - **GOOD-TO-HAVES consolidation** (two coexisting files) — needs a manager/owner
     DOCTRINE CALL; do NOT merge unilaterally.
