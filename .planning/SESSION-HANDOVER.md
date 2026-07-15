# SESSION-HANDOVER.md — v0.15.0 Floor: P114 CLOSED GREEN, P115 planning opens next — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #28** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #29**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#27→#28's
handover, committed at `29470e2`, superseded here).

**Read order:** this file → §1 (verify live, do not trust timestamps) → §6 (runbook) →
dip into §2/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
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
**Verified independently this handover (2026-07-15, just now, before writing this
file):**
- Local `main` HEAD = `87a4bb24bec872033a1fce501b377b1d1b322ad9` (short `87a4bb2`,
  "docs(planning): prune manager handover to size budget (git history is the
  archive)"), tree **clean** (`git status --porcelain` empty), **IN SYNC** with
  `origin/main` (`git rev-list --left-right --count HEAD...origin/main` → `0  0`).
- **`87a4bb2` is a MANAGER commit, not #28's work** — the manager (w1:p7) pruned
  `.planning/MANAGER-HANDOVER.md` between #28's own verification pass and this
  handover being written. Expected on a shared tree; not a deviation to chase.
- **CI status on the CURRENT TIP (`87a4bb2`) was IN_PROGRESS, not yet concluded, at
  the moment this handover was verified** (`ci.yml` run `29442201377`, started
  `18:50:51Z`, still `in_progress` at `18:53:12Z`; `release-plz` run `29442203689`
  also `in_progress`; only `Push on main` had completed/success on this sha). **#29
  MUST re-run `gh run list --branch main --workflow CI --limit 3` as its first action
  and confirm the newest run on `87a4bb2` (or whatever the tip is by then) concluded
  `success` before doing anything else** — do not assume green from this document.
- **The prior tip `f2f0f01` (manager's earlier handover refresh) IS confirmed GREEN**:
  `ci.yml` run `29441724976`, `completed`/`success`, concluded `18:49:16Z`. The tip
  before that, `29470e2` (the #27→#28 relief handover), is also `completed`/`success`
  (run `29440972233`). No red run sits on main's recent history.
- `.planning/STATE.md` last confirmed (by #27→#28's handover) `status: in_progress`,
  `completed_phases: 1/15`, current position "P114 CLOSED GREEN", next action
  `/gsd-plan-phase 115` — unchanged this rotation, #28 did not execute planning (see
  §5 for why).

**Commit lineage since the last known-clean sha (`dc26302`), oldest → newest:**
1. `e039bb7` — docs: capture 2 todos — roadmap-diagram lane + GOOD-TO-HAVES
   consolidation (prior rotation, #27 closeout)
2. `29470e2` — docs(planning): L0 relief handover #27→#28 (prior rotation's handover)
3. `f2f0f01` — docs(planning): refresh manager handover — P114 closed green,
   pane-clear rotation rule, #8 session state (**manager-authored**)
4. `87a4bb2` — docs(planning): prune manager handover to size budget
   (**manager-authored, current HEAD**)
5. **`<this commit, created below>`** — the #28→#29 handover you are reading

**Deviations from the plan #29 MUST know:**
- **No P115 planning artifacts exist yet.** #28 ran `gsd-sdk query init.plan-phase 115`
  (read-only, produced no file writes — see §3/§5) then relieved at that clean
  pre-research boundary. `/gsd-plan-phase 115` has NOT been run. There is nothing
  half-built to inspect or resume — the next unit of work is starting P115 planning
  from scratch.
- P114 remains fully closed (both waves shipped, verification tail ran, real-backend
  SC1/SC2 PASSED) — carried unchanged from the #27→#28 handover, no new information
  this rotation.

## 2. Wave/cycle state

| Wave/Phase | Plan | State | Commits |
|---|---|---|---|
| P114 (all waves + verification + close) | — | **DONE + CI GREEN** | tip `dc26302` (prior rotations) |
| P114 closeout (2 todos filed) | — | **DONE** | `e039bb7` |
| Manager handover refresh + prune | — | DONE (manager's own artifact) | `f2f0f01`, `87a4bb2` |
| **P115 BENCH-01** (live MCP benchmark re-measurement) | not yet written | **NOT STARTED — `init.plan-phase` query run, phase dir NOT yet created, `/gsd-plan-phase 115` NOT yet run** | — |
| P116 ADR-010 packet (ADR-01 + FIX-03 options) | not yet written | NOT STARTED | — |
| roadmap-diagram gsd-quick (owner-approved) | todo filed | NOT STARTED, queued | — |
| Directive 2 (scratch-repo KEEP-policy doc, gsd-quick) | todo | NOT STARTED, **5 rotations pending** | — |
| GOOD-TO-HAVES consolidation (needs manager doctrine call) | todo filed | NOT STARTED, blocked on manager ruling | — |

No named incident this rotation — #28's turn was short and clean (init query → early
relief). See §5 for the honest account of why relief happened this early.

## 3. Binding constraints (unchanged, carried)

- **One tree-writer at a time**; tree-mutating work is serial (no per-agent worktrees —
  owner rejected them as over-engineering for current cadence).
- **ONE cargo invocation machine-wide** (check/build/test/clippy) — prefer `-p <crate>`
  over `--workspace`; VM has OOM-crashed on parallel builds.
- **No `--no-verify`**, ever, on any commit or push.
- **Push at green, then confirm CI green on `main` AFTER the push** — run
  `python3 quality/runners/run.py --cadence post-push --persist`; the
  `code/ci-green-on-main` (P0) probe asserts the NEWEST `ci.yml` run on `main`
  concluded success, not merely that some older green run exists. Never open the next
  phase/wave over a red or pending main.
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
  ids in visible output (`/clear` does not kill them; successors can't enumerate them).
- **Manager (w1:p7) uses a POLLING model** — clear in-pane narration at each boundary
  IS the report; escalate actively only for owner-blocking moments.
- **Relieve past ~100k tokens of own context** (hard stop ~150k; **absolute, not %** of
  the window) at a wave boundary — write+commit a fresh handover, REPLACING this file,
  naming successor **#30**.

## 4. Litmus / gate / REOPEN state

- **CI gate run history:** newest run on main's tip (`87a4bb2`) was **IN_PROGRESS at
  verification time** — see §1, #29 must confirm conclusion before proceeding. The
  preceding two tips (`f2f0f01` run `29441724976`, `29470e2` run `29440972233`) are
  both `completed`/`success`. `code/ci-green-on-main` P0 status: **PASS on `f2f0f01`,
  PENDING re-check on `87a4bb2`.**
- **P115 BENCH-01 LOCKED CONSTRAINTS** (owner/manager-set — #29 MUST carry these into
  planning as locked decisions, not re-derive them):
  - Funded Q1 re-measurement.
  - Ceiling **≤50 benchmark sessions on the EXISTING subscription**; **NO new API
    spend**.
  - **Escalate past 50 sessions to the MANAGER (w1:p7)** — never exceed 50 without an
    explicit manager GO.
  - Hero-number waiver **HARD DEADLINE 2026-08-15** (the `agent-ux` hero-number
    doc-alignment rows, 8 total, whose waiver expires that date — BENCH-01
    re-measures the hero numbers so the waiver can be lifted). Schedule early.
  - `Execution mode: top-level` — P115 is orchestration/measurement-shaped, runs at
    top-level (NOT under `/gsd-execute-phase`; a `phase-coordinator`/`gsd-executor`
    lacks the Skill tool needed to run `/gsd-plan-phase`). Planning MUST stay
    top-level (L0/#29).
  - Prior benchmark methodology / latency envelope home: `docs/benchmarks/latency.md`.
- **P115 init facts** (from `gsd-sdk query init.plan-phase 115`, captured by #28 —
  re-run this query fresh, do not trust a cached copy, none was committed):
  - `phase_found: true`; `phase_name: "Live MCP benchmark re-measurement"`;
    `phase_slug: live-mcp-benchmark-re-measurement`; `padded_phase: 115`.
  - `phase_dir` does NOT exist yet — the plan-phase workflow (step 2) creates
    `.planning/phases/115-live-mcp-benchmark-re-measurement` itself.
  - `has_context`, `has_research`, `has_plans` all **false**; `phase_req_ids:
    BENCH-01`.
  - `research_enabled`, `plan_checker_enabled`, `nyquist_validation_enabled`,
    `commit_docs` all **true**.
  - Models: researcher=**sonnet**, planner=**opus**, checker=**sonnet**.
- **Waiver / deadline clocks (carried, unchanged this rotation):**
  - `agent-ux` hero-number doc-alignment rows (8 total) — waiver expires
    **2026-08-15**.
  - `structure/file-size-limits` — waiver expires **2026-08-08** (`client.rs` split is
    v0.17 scope, do NOT split early).
  - `perf-targets` — self-WAIVED until **2026-07-26**.
  - Pre-push timing WARN is a stale budget doc (GTH-14), NOT a real regression — do
    NOT re-investigate.
- **Milestone-close 9th probe** (`pre-release-real-backend`) not yet due — milestone
  still open, 14 phases remain.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **What #28 did this rotation (honest account):** verified §1 ground truth live,
  confirmed CI green on the tip it inherited (the manager had landed `f2f0f01`
  "refresh manager handover" on top of the `29470e2` relief handover — expected, the
  manager owns that doc). Then initialized P115 planning by running `gsd-sdk query
  init.plan-phase 115` (read-only, captured in §4 above). #28 then **relieved at the
  clean pre-research boundary BEFORE dispatching any planning subagent.**
- **Why #28 relieved this early (honest, not spun):** #28 over-read the plan-phase
  workflow file (`$HOME/.claude/get-shit-done/workflows/plan-phase.md`, ~1720 lines /
  ~32k tokens) **linearly**, pushing its own context to ~96k (the ~100k soft-relief
  threshold) before dispatching a single planning subagent. Per the scoping rule
  (don't plan workload into correction margin — the runway to the 150k hard stop is
  correction margin, NOT workload budget), #28 relieved rather than gamble a degraded
  150k-context run. No planning artifacts were written; no half-state was left behind
  (the workflow creates the phase dir itself at its own step 2). This is a clean
  pre-wave boundary, not a mid-flight rescue.
- **META-LESSON for #29 (fix-it-twice — apply this, don't just read it):** do **NOT**
  read `$HOME/.claude/get-shit-done/workflows/plan-phase.md` linearly — it is a
  ~32k-token context sink that will burn the same budget #28 burned. Follow it
  step-by-step, **delegate every heavy read to `reader-digester`**, and let the
  workflow's OWN dispatched subagents (`gsd-phase-researcher`, `gsd-planner`,
  `gsd-plan-checker`) hold the heavy context — the orchestrator's own context must
  stay lean. If #29 (or #30, #31…) hits the same workflow-file-linear-read failure
  mode again, that is a signal the workflow file itself needs a progressive-disclosure
  pass (per-OP-4 doctrine) — file it as a `GOOD-TO-HAVES` row rather than re-suffering
  it silently a third time.
- **RAISE LIST / open items carried forward, all still OPEN** (pull verbatim detail
  from `git log` on `29470e2` if deeper context is needed — that handover has the full
  narrative for each):
  - **P116 ADR-010 packet** (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id
    durable-create hazard): after P115, produce options+tradeoffs and **route the
    packet to the MANAGER (w1:p7) for ruling — NO pre-ruling implementation.**
  - **roadmap-diagram gsd-quick** — owner-approved, cheap; todo
    `.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md` (read it
    for all 5 points before scoping). Interleave opportunistically; gsd-quick scale.
  - **Directive 2** — scratch-repo `reposix-scope-test-DELETEME` KEEP-policy into
    `docs/reference/testing-targets.md` (reset via force-push, never delete). Now **5
    rotations pending** (#24→#25 through #28→#29 all carried it forward without
    picking it up) — pick up opportunistically, it is cheap.
  - **GOOD-TO-HAVES consolidation** (two coexisting files: root
    `.planning/GOOD-TO-HAVES.md` vs `.planning/milestones/v0.15.0-phases/
    GOOD-TO-HAVES.md`, distinct prefixes, ambiguity confirmed real) — needs a
    manager/owner **DOCTRINE CALL** before merging; todo
    `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`. Do
    NOT merge unilaterally.
- **Intake already filed — do NOT re-file:** the 2 todos from `e039bb7`
  (roadmap-diagram lane + GOOD-TO-HAVES consolidation); GTH-16 (filed prior rotation);
  the SC1 false-GREEN command fix (already shipped, informational only).

## 6. Precise next steps (successor #29 runbook)

1. **Re-verify §1 ground truth live** before doing anything else: `git rev-parse
   HEAD`, `git status --porcelain`, `git rev-list --left-right --count
   HEAD...origin/main`, `gh run list --branch main --workflow CI --limit 3`. Confirm
   the newest CI run on main's current tip concluded `success` — the tip inherited
   here (`87a4bb2`) had an IN_PROGRESS run at verification time; do not assume it
   finished green without checking.
2. **OPENING MOVE — run `/gsd-plan-phase 115` FRESH from a clean context.** Heed §5's
   meta-lesson: do NOT read `plan-phase.md` linearly — follow it step-by-step,
   delegate heavy reads to `reader-digester`, let its own dispatched subagents
   (`gsd-phase-researcher`/`gsd-planner`/`gsd-plan-checker`) hold the heavy context.
   The workflow will create the phase dir itself, then hit the context gate (no
   CONTEXT.md exists yet for P115): either "continue without context" (the BENCH-01
   constraints in §4 are already known/locked and can be carried in) or run
   `/gsd-discuss-phase 115` first — #29's call.
3. **During planning, run `roadmap.update-plan-progress 114`** to clear the stale P114
   ROADMAP checkbox — do NOT hand-edit `ROADMAP.md`.
4. **Carry the P115 BENCH-01 locked constraints verbatim** (§4) into the plan: ≤50
   benchmark sessions on the existing subscription / no new API spend / escalate to
   manager past 50 / hero-number waiver hard deadline 2026-08-15 / execution mode
   top-level.
5. **After P115 is planned and executed**, open **P116 ADR-010 packet** — produce
   options+tradeoffs for BOTH ADR-01 (mirror-fanout) and FIX-03 (GTH-09 slug→id
   durable-create hazard), then **route to the MANAGER (w1:p7) for ruling — no
   pre-ruling implementation.**
6. **roadmap-diagram gsd-quick** (§5) — owner-approved, small; interleave
   opportunistically, does not need to block P115/P116.
7. **Directive 2** — scratch-repo KEEP-policy doc, now 5 rotations pending; pick up
   opportunistically if a gap opens.
8. **GOOD-TO-HAVES consolidation** — do NOT merge unilaterally; flag to the manager
   for a doctrine call if it hasn't already happened.
9. **Report to the manager (w1:p7)** at each boundary (P115 planning start, P115
   close, P116 routing to manager, each subsequent phase close) and at any
   owner-blocking moment. Remember the manager POLLS — clear in-pane narration at each
   boundary IS the report.
10. **Relieve past ~100k own-context tokens** (hard stop ~150k, absolute not %) at a
    wave boundary — dispatch `relief-handover-writer`, which writes+commits a fresh
    `.planning/SESSION-HANDOVER.md` that REPLACES this file, naming successor **#30**.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** (current milestone, P114 CLOSED GREEN, 1/15 phases done,
P115 opens next) → **v0.17 meta-milestone** (5 gate shapes: pivot-vocabulary lint,
nav-budget, hero-redundancy, framing-claim rows, persona whole-journey rubric; +
subjective-runner Task-dispatch fix unfreezing 3 WAIVED meaning-gates; +
waiver-escalation rule; + transcript retention; + bloat remediation incl. the
SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure split) → **v0.19** truth purge +
IA rebuild → **v0.21** benchmark honesty (re-fixture live baseline, CI job,
headline-cross-check verifier) → **v0.23** journey slices → **v0.25** launch kit →
Show-HN. **Q3 launch gate:** Show-HN gated on a walkable REAL-BACKEND journey (GitHub
minimum), not sim-first. **Deep-survey calibration:** ~10% latent work per pass, ~10
passes to converge, recurring deep surveys are STANDING practice. **Q9 ceiling:** keep
v0.15→v0.25 ≈ 6-milestone scale.

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
Claude-Session: relief-handover-writer
