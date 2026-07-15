# SESSION-HANDOVER.md — v0.15.0 Floor: P115 BENCH-01 PLANNING COMPLETE, EXECUTION is the next substantive work — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #31** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #32**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#30→#31's
handover, superseded here).

**Read order:** this file → §1 (verify live) → §6 runbook (P115 EXECUTION is the
opening move) → §3/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED
staging only, never `git add -A`/`.`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```
**Verified independently this handover (2026-07-15, just now):**
- `HEAD` = `8e1e9708e24ab0973eac9e5f56b252fbf64797d1` ("docs(115): create phase plan —
  BENCH-01 live MCP benchmark re-measurement"). Tree **clean**, local `main` ==
  `origin/main` — **0 ahead / 0 behind**.
- CI on `8e1e970`: `ci.yml` run `29451252463`, **completed/success**, 5m16s, concluded
  `2026-07-15T21:15:01Z`. Prior tip `68fcbca` also `completed/success` (run
  `29448482364`). **This handover's own commit will move `HEAD`/`origin/main` again and
  kick off a fresh CI run** — #32 MUST re-run `gh run list --branch main --workflow CI
  --limit 3` after this commit lands and confirm that run is green (or at minimum
  in-flight, not failed) before opening P115 execution. Never open work over a red or
  pending main.
- `.planning/STATE.md`: `completed_phases: 1/15` (**still 1** — P115 is PLANNED, not
  yet EXECUTED, so the counter has not moved); last_activity "Phase 115 planning
  complete"; P114's ROADMAP checkbox cleared (`roadmap.update-plan-progress 114` ran
  during planning). Phase dir `.planning/phases/115-live-mcp-benchmark-re-measurement/`
  contains `115-RESEARCH.md` (`cd0c951`), `115-PLAN.md` + `115-VALIDATION.md`
  (`8e1e970`) — confirmed present on disk this handover.
- **Known flaky CI job (see §4):** the commit BEFORE `68fcbca` (`6114a6f`, doc-only)
  ran `ci.yml` `29448217566` and came back **failure** (3m43s) — but the very next
  doc-only commit `68fcbca` and this rotation's `8e1e970` both came back green on the
  same `test` job. Treated as a flake, not a regression — see §4 watch item.

## 2. Wave/cycle state

| Wave/Phase | Plan | State | Commits |
|---|---|---|---|
| P114 (all waves + verification + close) | — | **DONE + CI GREEN** | tip carried from prior rotations |
| Push-unblock (docs-alignment refresh tail) | — | **DONE — pushed, CI green** | `be6f1bf` (carried from #30) |
| **P115 BENCH-01 planning** (research → plan → plan-check → revision) | `115-PLAN.md` (1 plan, 6 tasks, 5 waves) | **PLANNING COMPLETE, pushed, CI GREEN** | `cd0c951` (research), `8e1e970` (plan+validation, post plan-checker revision) |
| **P115 BENCH-01 execution** | `115-PLAN.md` | **NOT STARTED — THE OPENING MOVE FOR #32** | — |
| P116 ADR-010 packet (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id) | not yet written | NOT STARTED, after P115 execution, routes to MANAGER for ruling | — |
| roadmap-diagram gsd-quick (owner-approved) | todo filed | NOT STARTED, queued, interleave opportunistically | — |
| GOOD-TO-HAVES consolidation (needs manager doctrine call) | todo filed | NOT STARTED, blocked on manager ruling | — |

**P115 plan shape** (`115-PLAN.md`, 6 tasks / 5 waves): W1 — T1 A1-gate (session-unit
`checkpoint:decision`, autonomous:false) ‖ T2 latency re-measure (independent, zero
session budget); W2 — T3 ledger scaffold; W3 — T4 live-MCP capture; W4 — T5
`token-economy.md` regen; W5 — T6 un-waive path + consolidation.

No named incident this rotation — #31's plan-phase run was clean top-level work (see
§5 for the honest account).

## 3. Binding constraints (carried verbatim, unchanged)

- **One tree-writer at a time**; tree-mutating work is serial (no per-agent worktrees —
  owner rejected them as over-engineering for current cadence).
- **ONE cargo invocation machine-wide** (check/build/test/clippy) — prefer `-p <crate>`
  over `--workspace`; VM has OOM-crashed on parallel builds.
- **No `--no-verify`**, ever, on any commit or push.
- **Push at green, then confirm CI green on `main` AFTER the push** — run
  `python3 quality/runners/run.py --cadence post-push --persist`; the
  `code/ci-green-on-main` (P0) probe asserts the NEWEST `ci.yml` run on `main`
  concluded success, not merely that some older green run exists. Never open the next
  phase over a red or pending main.
- **Commit-trailer format:** `Co-Authored-By: Claude <Model> <noreply@anthropic.com>` +
  `Claude-Session: <role-or-session-id>`.
- **Model tiering:** opus for complex/security work, sonnet for default execution,
  haiku for mechanical tasks; **never dispatch `fable` at a leaf**.
- **Leaf isolation:** any `reposix init`/sim-seed/`git commit`/`config` test setup runs
  in a throwaway `/tmp` clone, `cd`'d into in the SAME Bash invocation — never the
  shared repo. Mechanically enforced by `.claude/hooks/leaf-isolation-guard.sh`.
- **No tag push by any coordinator** — the manager cuts tags.
- **No git surgery on `main`** (no reset/rebase/reorder/amend of already-pushed
  commits).
- **Shared tree with the manager** — TARGETED staging only (`git add <path>`, never
  `-A`/`.`); do not touch `.planning/MANAGER-HANDOVER.md`.
- **LIVENESS doctrine:** bound every wait on a dispatched child; health-check quiet
  children on a ≤30min/≤1h timer; children poll CI INLINE or run synchronously — never
  idle-trust a background self-resume watcher alone.
- **Real-backend cadence:** source `.env` in the SAME invocation as `run.py`;
  `scripts/refresh-tokenworld-mirror.sh` as a pre-step; TokenWorld protected pair
  `7766017`/`7798785` NEVER deleted.
- **Before ending any turn with background shells/monitors running**, note their task
  ids in visible output. **This rotation: none running** — verified clean at handover
  time, nothing to note for #32.
- **Manager (w1:p7) uses a POLLING model** — clear in-pane narration at each boundary
  IS the report; escalate actively only for owner-blocking moments.
- **Relieve past ~100k tokens of own context** (hard stop ~150k; **absolute, not %** of
  the window) at a wave boundary — write+commit a fresh handover, REPLACING this file,
  naming successor **#33**.

## 4. Litmus / gate / REOPEN state

- **CI gate:** `origin/main` tip `8e1e970` — `code/ci-green-on-main` P0 **PASS**
  (`ci.yml` run `29451252463`, completed/success). No REOPEN, no active gate failure.
- **NEW watch item (#31): flaky `test` job.** `ci.yml` run `29448217566` on
  `6114a6f` (doc-only commit, superseded by `68fcbca`) concluded **failure** at
  3m43s. The very next doc-only commit `68fcbca` ran the same job GREEN, and this
  rotation's `8e1e970` also came back GREEN. No code changed between the failing and
  passing runs that would plausibly cause a real regression on a docs-only diff —
  carried as a **FLAKE**, not a regression. If a P115-execution push sees the `test`
  job go red, **re-run it once before treating it as real**; a REPEATED/reproducible
  failure across re-runs IS a real signal and must not be waved through.
- **Pre-push timing:** not separately re-measured this rotation (no code push
  happened, only docs commits); the prior two rotations' elevated ~84-91s
  observations (vs documented ~55-60s) both attributed to cold cargo/kcov caches, NOT
  yet a warm-cache signal. Still worth a warm-cache re-check at the next code push.
- **Waiver / deadline clocks (carried, unchanged this rotation):**
  - `agent-ux` hero-number doc-alignment rows (8 total) — waiver expires
    **2026-08-15**. **P115 BENCH-01 lifts these, but only after EXECUTION re-measures
    — planning alone does NOT lift them.** The waiver clock is still live and is the
    reason P115 was front-loaded in the milestone.
  - `structure/file-size-limits` — waiver expires **2026-08-08** (`client.rs` split is
    v0.17 scope, do NOT split early). Gate-wide WARN-not-block: this handover and
    `115-PLAN.md` (~29.4k chars) both sit over the 20000 B `*.md` ceiling
    non-blockingly under the same waiver.
  - `perf-targets` — self-WAIVED until **2026-07-26**.
- **Milestone-close 9th probe** (`pre-release-real-backend`) not yet due — 14 phases
  remain.
- **Intake already filed — do NOT re-file:** GTH-V15-21, the 2 todos
  (roadmap-diagram, GOOD-TO-HAVES consolidation), GTH-16, GTH-V15-22, GTH-V15-23,
  GTH-V15-16 (the plan-refresh-cold-under-report lesson). **#31 filed NOTHING new** —
  all noticing was folded into `115-PLAN.md`'s deferral notes, see §5.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **Honest account of #31:** opened P115 and ran the full plan-phase flow at
  top-level (`Execution mode: top-level`, per the orchestration-shaped-phase rule),
  delegating every heavy read to `reader-digester`/dispatched subagents rather than
  linear-reading `plan-phase.md` (~1720 lines) — the anti-sink lesson from #27/#28 held.
  Sequence: `gsd-phase-researcher` (sonnet, → `cd0c951`) → `gsd-planner` (opus) →
  `gsd-plan-checker` (sonnet, ISSUES: 2 MEDIUM + 1 LOW) → `gsd-planner` revision (opus,
  all 3 fixes verified landed) → committed + pushed (`8e1e970`) → CI confirmed green.
  **No P115 execution was started** — correctly deferred; it is a large separate wave
  with a spend ceiling and an owner-escalation path (A1, below).
- **A1 — OPEN, decision-gating, ROUTED TO MANAGER (w1:p7):** "benchmark session" is
  UNDEFINED anywhere in-repo. Two readings: (H1) one live agentic conversation vs (H2)
  one metered API call. The ≤50-session ledger cannot be designed without a ruling.
  Plan T1 is a blocking `checkpoint:decision` (`autonomous:false`) — **NO ledger row /
  NO session spend before the manager rules.** **#32 MUST get the manager's A1 ruling
  before opening T3/T4** (ledger scaffold / live-MCP capture). Prompt the manager if
  not yet ruled.
- **Research finding surfaced to manager (de-risks the ceiling):** the 8 waived rows
  split into 2 tracks — the **latency track** (5 rows: 27ms cold-init, 8ms read)
  re-runs the EXISTING real `quality/gates/perf/latency-bench.sh` at **ZERO session
  budget** (already run by CI's `bench-latency-v09` job); only the **token-economy
  track** (3 rows: 89.1%) needs live MCP sessions. The original blocker (Atlassian MCP
  not GA, per `docs/concepts/reposix-vs-mcp-and-sdks.md:30-32`) reportedly **no longer
  holds** — Atlassian + GitHub official remote MCP servers are now GA — but this is
  MEDIUM-confidence WebSearch; plan T1 re-verifies GA at execution start, with fallback
  `sooperset/mcp-atlassian`.
- **Noticing (folded into the plan, NOT separately filed):**
  1. Row-ids say "24ms" while claims say "27ms" cold-init — T2 resolves the NUMBER;
     the separate prose relabel + a 9th near-miss row (`docs/why/cold-init-24ms-sim`)
     is handed to Phase 117/118, out of P115 scope.
  2. `headline-numbers-cross-check.py` confirmed ABSENT from the repo — named in the
     un-waive-path doc (T6), authoring it is DEFERRED, not P115 scope.
  3. All research file:line citations were verified true against the live source.
- **Capture artifacts** (so P118/DOCS-05 consume directly, per the plan): regenerate
  `docs/benchmarks/latency.md` + `token-economy.md` (committed markdown), replacing the
  synthetic `mcp_jira_catalog.json` + FUSE-era `reposix_session.txt`; plus a NEW
  session-spend ledger at `benchmarks/bench-session-ledger.md` (deliberately outside
  `docs/` to dodge the mkdocs orphan-doc invariant), monotonic, with a mechanical ≤50
  check via `tail -1 … | awk -F'|' '{v=$(NF-2); gsub(/ /,"",v); exit (v+0>50)}'`.
- **RAISE LIST / open items carried forward, all still OPEN:**
  - **P116 ADR-010 packet** — after P115 execution, produce options+tradeoffs for
    ADR-01 (mirror-fanout) and FIX-03 (GTH-09 slug→id durable-create hazard), route to
    the **MANAGER (w1:p7) for ruling — no pre-ruling implementation.**
  - **roadmap-diagram gsd-quick** — owner-approved, small; todo
    `.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md`.
    Interleave opportunistically; touching any tracked doc in
    `quality/catalogs/doc-alignment.json` requires a `/reposix-quality-refresh` pass
    before the next push.
  - **GOOD-TO-HAVES consolidation** (two coexisting files: root
    `.planning/GOOD-TO-HAVES.md` vs `.planning/milestones/v0.15.0-phases/
    GOOD-TO-HAVES.md`) — needs a manager/owner **DOCTRINE CALL** before merging; todo
    `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`. Do
    NOT merge unilaterally.

## 6. Precise next steps (successor #32 runbook)

1. **Re-verify §1 ground truth live**: `git rev-parse HEAD && git status --porcelain
   && git rev-list --left-right --count HEAD...origin/main && gh run list --branch
   main --workflow CI --limit 3`. Confirm THIS handover's own commit's CI run
   concluded green (or is in-flight-not-failed) before proceeding.
2. **Get the MANAGER's A1 ruling** (session-unit definition, H1 vs H2 above) — this
   gates P115 execution's T3/T4. If unruled, prompt the manager (w1:p7) directly.
3. **Open P115 EXECUTION at top-level** (`Execution mode: top-level`; do NOT route via
   `/gsd-execute-phase`/`gsd-executor`, which lacks the `Skill`/top-level tools AND the
   ≤50-session-ceiling + owner-escalation judgment needs top-level authority). Plan =
   `115-PLAN.md` (6 tasks, 5 waves, see §2). **T2 (latency re-measure, zero session
   budget) can start IMMEDIATELY, independent of the A1 gate.** T1's A1-gate blocks the
   token-economy track (T3/T4) on the manager's ruling. Anti-sink lesson STILL
   BINDING: don't linear-read `plan-phase.md`/`execute-phase.md`; delegate heavy reads
   to `reader-digester`; let dispatched subagents hold heavy context.
4. **Push cadence at execution close:** `git push origin main` BEFORE the verifier
   subagent; then `python3 quality/runners/run.py --cadence post-push --persist` —
   `code/ci-green-on-main` P0 must be green.
5. **After P115 execution:** open **P116 ADR-010 packet** (ADR-01 + FIX-03), route to
   the MANAGER for ruling, NO pre-ruling implementation.
6. **roadmap-diagram gsd-quick + GOOD-TO-HAVES consolidation** — interleave/flag to
   the manager for a doctrine call, don't merge unilaterally.
7. **Report to the manager (w1:p7)** at each boundary (A1 escalation, P115 execution
   start/close, P116 routing). The manager POLLS — clear in-pane narration at each
   boundary IS the report; escalate actively only for owner-blocking moments.
8. **Relieve past ~100k own-context tokens** (hard stop ~150k, absolute not %) at a
   wave boundary — dispatch `relief-handover-writer`, which writes+commits a fresh
   `.planning/SESSION-HANDOVER.md` that REPLACES this file, naming successor **#33**.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** (current milestone, P114 CLOSED GREEN, P115 PLANNED →
executes next, 1/15 phases done) → **v0.17 meta-milestone** (5 gate shapes: pivot-
vocabulary lint, nav-budget, hero-redundancy, framing-claim rows, persona whole-journey
rubric; + subjective-runner Task-dispatch fix unfreezing 3 WAIVED meaning-gates; +
waiver-escalation rule; + transcript retention; + bloat remediation incl. the
SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure split) → **v0.19** truth purge +
IA rebuild → **v0.21** benchmark honesty (re-fixture live baseline, CI job,
headline-cross-check verifier) → **v0.23** journey slices → **v0.25** launch kit →
Show-HN. **Q3 launch gate:** Show-HN gated on a walkable REAL-BACKEND journey (GitHub
minimum), not sim-first. **Deep-survey calibration:** ~10% latent work per pass, ~10
passes to converge, recurring deep surveys are STANDING practice. **Q9 ceiling:** keep
v0.15→v0.25 ≈ 6-milestone scale.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
Claude-Session: relief-handover-writer
