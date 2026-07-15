# SESSION-HANDOVER.md — v0.15.0 Floor: push-unblock COMPLETE, P115 planning is the next substantive work — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #30** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #31**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#29→#30's
handover, superseded here).

**Read order:** this file → §1 (verify live) → §6 runbook (the ONLY next task is
P115 planning — the push-unblock objective #29 handed off is DONE, do NOT repeat it)
→ §3/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
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
- `HEAD` = `be6f1bf431dcb133c816d567b473ffa05d69c17d` ("refresh(doc-alignment): re-bind
  10 stale rows in docs/reference/testing-targets.md"). Tree **clean**
  (`git status --porcelain` empty). Local `main` == `origin/main` — **0 ahead / 0
  behind** (`git rev-list --left-right --count HEAD...origin/main` → `0  0`).
- CI on `be6f1bf`: `ci.yml` run `29447017101`, **completed/success**, 5m39s, concluded
  `2026-07-15T20:06:04Z`. The two runs before it (`29443988376` on `a266582`,
  `29443163657` on `71e904f`) are also `completed`/`success` — no red run anywhere in
  recent history. Post-push `code/ci-green-on-main` (P0) probe **PASS**.
- **This handover's own commit will move `HEAD`/`origin/main` again and kick off a
  fresh CI run for it** — re-run `gh run list` per §1 above after pushing and confirm
  that run is green (or at minimum in-flight, not failed) before #31 opens P115. Never
  open a phase over a red or pending main.
- `.planning/STATE.md` last confirmed `status: in_progress`, `completed_phases: 1/15`,
  P114 CLOSED GREEN. P115 (`/gsd-plan-phase 115`) is the next planning action — the
  phase dir does not exist yet, `/gsd-plan-phase 115` has never been run.

**The push-unblock objective handed to #30 is COMPLETE — do not repeat it:**
- #29's blocker (11-ish drifted `docs-alignment` catalog rows caused by `a165d48`'s edit
  to `docs/reference/testing-targets.md`) is **resolved and pushed** as `be6f1bf`. The
  local tree that was previously "+2/+3 ahead, cannot push" is now **fully synced with
  origin, 0/0**. There is no pending push, no pending catalog refresh, no open gate
  failure. §6 below starts directly at P115 planning.

## 2. Wave/cycle state

| Wave/Phase | Plan | State | Commits |
|---|---|---|---|
| P114 (all waves + verification + close) | — | **DONE + CI GREEN** | tip carried from prior rotations |
| Directive 2 (scratch-repo KEEP-policy doc, gsd-quick `260715-h1d`) | `260715-h1d-PLAN.md` | **DONE** | `a165d48`, `dff801b` — pushed, part of `origin/main` |
| docs-alignment refresh tail (10 drifted rows in `doc-alignment.json`, caused by `a165d48`) | — | **DONE — pushed, CI green** | `be6f1bf` |
| **P115 BENCH-01** (live MCP benchmark re-measurement) | not yet written | **NOT STARTED** — `init.plan-phase` query run by #28 (stale, re-run fresh), phase dir not yet created, `/gsd-plan-phase 115` never run — **THE OPENING MOVE FOR #31** | — |
| P116 ADR-010 packet (ADR-01 + FIX-03 options) | not yet written | NOT STARTED, comes after P115, routes to MANAGER for ruling | — |
| roadmap-diagram gsd-quick (owner-approved) | todo filed | NOT STARTED, queued, interleave opportunistically | — |
| GOOD-TO-HAVES consolidation (needs manager doctrine call) | todo filed | NOT STARTED, blocked on manager ruling | — |

No named incident this rotation — #30's push-unblock work was a clean, correctly-scoped
catalog refresh; no test failure, no corruption, no rollback. See §5 for the honest
account of what #30 did.

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
  ids in visible output (`/clear` does not kill them; successors can't enumerate them).
  **This rotation: none running** — verified `jobs -l` and `ps aux` clean at handover
  time, nothing to note for #31.
- **Manager (w1:p7) uses a POLLING model** — clear in-pane narration at each boundary
  IS the report; escalate actively only for owner-blocking moments.
- **Relieve past ~100k tokens of own context** (hard stop ~150k; **absolute, not %** of
  the window) at a wave boundary — write+commit a fresh handover, REPLACING this file,
  naming successor **#32**.

## 4. Litmus / gate / REOPEN state

- **CI gate:** `origin/main` tip `be6f1bf` — `code/ci-green-on-main` P0 **PASS**
  (`ci.yml` run `29447017101`, completed/success). No REOPEN, no active gate failure.
- **What was REOPEN last rotation, now CLOSED:** the `docs-alignment/walk` pre-push
  block (10 drifted rows in `quality/catalogs/doc-alignment.json`, anchored into
  `docs/reference/testing-targets.md`, caused by `a165d48`'s KEEP-policy insertion).
  Resolved via `/reposix-quality-refresh docs/reference/testing-targets.md` (Opus
  grader re-bound all 10 rows GREEN with corrected source ranges, e.g. `jira-token`
  `:186 → :209`), committed `be6f1bf`, pushed, CI verified green. `walk` now exits 0,
  no blocking states remain.
- **Waiver / deadline clocks (carried, unchanged this rotation):**
  - `agent-ux` hero-number doc-alignment rows (8 total) — waiver expires
    **2026-08-15** (P115/BENCH-01 re-measures to lift it).
  - `structure/file-size-limits` — waiver expires **2026-08-08** (`client.rs` split is
    v0.17 scope, do NOT split early). This waiver is gate-wide (over-budget tier
    warns, does not block) — this handover file itself and its predecessor both sit
    slightly over the 20000 B `*.md` ceiling; non-blocking under the same waiver, but
    do not treat that as license to bloat further.
  - `perf-targets` — self-WAIVED until **2026-07-26**.
  - Pre-push timing WARN is a stale budget doc (GTH-14), NOT a real regression — do
    NOT re-investigate.
- **NEW watch item (#30, not yet a filed defect):** #30's pre-push cadence measured
  **~91s vs the documented ~55-60s budget** (WARN-level, non-blocking). Likely cause:
  cold cargo/kcov caches after a `reposix-quality --release` rebuild earlier this
  session — NOT the GTH-14 stale-budget-doc case above (that one is about the budget
  number being wrong; this is a live timing observation on a specific run). Not a
  blocker. **If #31 (or a later successor) sees this recur on a WARM cache**, treat it
  as a real signal and check for a newly-added whole-repo pre-push gate rather than
  assuming diff size — do not dismiss a second occurrence as noise.
- **Milestone-close 9th probe** (`pre-release-real-backend`) not yet due — milestone
  still open, 14 phases remain.
- **Intake already filed — do NOT re-file:** GTH-V15-21 (archived-handover file-size at
  the 08-08 waiver expiry, committed `71e904f`); the 2 todos filed prior rotation
  (roadmap-diagram lane + GOOD-TO-HAVES consolidation); GTH-16; **and, new this
  rotation, GTH-V15-22 + GTH-V15-23** (both filed below in §5/committed to
  `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` in the same commit as this
  handover).

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **What #30 did this rotation (honest account):** ran the push-unblock runbook #29
  handed off, end to end, and nothing else. Re-verified §1 ground truth live. Diagnosed
  the exact drift: `a165d48` had inserted a 23-line "Scratch repo —
  reposix-scope-test-DELETEME" subsection at doc line 166 in
  `docs/reference/testing-targets.md`, uniformly shifting every cited JIRA/GitHub/
  skip-pattern claim below it by +23 lines (STALE_DOCS_DRIFT, confirmed **10** rows,
  not the "11-ish" estimate #29's handover carried — the exact count was resolved by
  the live `walk`, not by trusting the prior handover's characterization, per that
  handover's own instruction). Ran `/reposix-quality-refresh
  docs/reference/testing-targets.md`; an Opus grader re-read each claim's prose AND
  its asserting test fn body (not just the claim text) and re-bound all 10 GREEN with
  corrected source ranges. Committed `be6f1bf`, pushed, confirmed CI green
  (`29447017101`) and the post-push `code/ci-green-on-main` P0 probe PASS. **No P115
  work was started this rotation** — the push-unblock tail consumed the rotation as
  scoped; P115 planning is entirely #31's to open fresh.
- **NEW operational meta-lesson for #31 (add near the #29/#30 plan-refresh lesson,
  this is a NEW instance discovered fixing the blocker above):**
  `reposix-quality doc-alignment plan-refresh <doc>` builds its stale-row manifest
  from the catalog's **PERSISTED** `last_verdict` — on a clean tree it returns an
  **EMPTY** `stale_rows` list until a `walk` has run and persisted `STALE_DOCS_DRIFT`.
  `walk` "updates `last_verdict` only" — i.e. it is the one that mutates/dirties the
  catalog with the live drift state. So when diagnosing a `STALE_DOCS_DRIFT` block
  manually: run **`walk` FIRST** (authoritative — exits 1 and lists the drifted rows +
  persists their state), **THEN** `plan-refresh` will return them. **Do NOT misread an
  empty `plan-refresh` as "nothing to do"** — cross-check against `walk`'s exit code.
  (The pre-push gate's own internal `walk` normally pre-populates this automatically;
  it is only a fresh MANUAL diagnosis on an otherwise-clean tree that can be fooled by
  an empty `plan-refresh`.) This is closely related to, but a sharper/more specific
  instance of, the already-filed **GTH-V15-16** ("`plan-refresh` under-reports drift
  when invoked cold") — do NOT file a duplicate GTH row for this; the existing
  GTH-V15-16 fix-sketch already covers it, this bullet is operational guidance for
  agents until that GTH lands.
- **Two GOOD-TO-HAVES rows filed this rotation** (from the Opus grader's noticing
  during the refresh above), appended to
  `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` in the SAME commit as this
  handover — **do NOT re-file, see full text there:**
  - **GTH-V15-22** — `prior_rationale` line-refs in `doc-alignment.json` rot silently
    (hardcoded line numbers drift even when the underlying binding is sound, because
    nothing validates them against the live source; e.g. all JIRA rows cited
    `agent_flow_real.rs:296` while the real fn/assertions sit at `:298`-`:311`).
  - **GTH-V15-23** — the `github-url-prefix` claim (row
    `docs/reference/testing-targets/github-url-prefix`) is bound to prose in the
    ADR-008 dispatch-note blockquote (`docs/reference/testing-targets.md:245-251`),
    not stated in the GitHub testing section proper — binding is sound, but a reader
    scanning that section won't find the URL contract there.
- **RAISE LIST / open items carried forward, all still OPEN:**
  - **P116 ADR-010 packet** (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id
    durable-create hazard): after P115, produce options+tradeoffs and **route the
    packet to the MANAGER (w1:p7) for ruling — NO pre-ruling implementation.**
  - **roadmap-diagram gsd-quick** — owner-approved, small; todo
    `.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md` (read it
    for all 5 points before scoping). Interleave opportunistically; carries the
    docs-alignment refresh-tail caveat (editing ANY tracked doc in
    `quality/catalogs/doc-alignment.json` requires a `/reposix-quality-refresh` pass
    before the next push) if it touches a tracked doc.
  - **GOOD-TO-HAVES consolidation** (two coexisting files: root
    `.planning/GOOD-TO-HAVES.md` vs `.planning/milestones/v0.15.0-phases/
    GOOD-TO-HAVES.md`) — needs a manager/owner **DOCTRINE CALL** before merging; todo
    `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`. Do
    NOT merge unilaterally.
- **Intake already filed — do NOT re-file:** see §4 list above (GTH-V15-21, the 2
  todos, GTH-16, and this rotation's GTH-V15-22 + GTH-V15-23).

## 6. Precise next steps (successor #31 runbook)

**The push-unblock blocker is CLOSED. There is no ⛔ blocker at the top of this
runbook — start directly at step 1.**

1. **Re-verify §1 ground truth live**: `git rev-parse HEAD && git status --porcelain
   && git rev-list --left-right --count HEAD...origin/main && gh run list --branch
   main --workflow CI --limit 3`. Expect `HEAD` at or after `be6f1bf` (this handover's
   own commit will be the new tip), clean tree, `0 0` ahead/behind, newest CI run
   green. **If the newest run is still `in_progress`** (this handover's push will have
   just kicked one off), wait for it to conclude green before proceeding — never open
   a phase over a red or pending main.
2. **Run `/gsd-plan-phase 115` FRESH from a clean context.** `Execution mode:
   top-level` (planning AND execution stay top-level — `gsd-executor` lacks the Skill
   tool a `/gsd-plan-phase` sub-dispatch needs; orchestration-shaped-phase rule,
   `.planning/CLAUDE.md`). The phase dir does not exist yet — re-run `gsd-sdk query
   init.plan-phase 115` fresh (do not trust any cached copy) to confirm: `phase_name`
   "Live MCP benchmark re-measurement", `slug` `live-mcp-benchmark-re-measurement`,
   `phase_req_ids` BENCH-01, `has_context`/`has_research`/`has_plans` all false, models
   researcher=sonnet planner=opus checker=sonnet.
   - **Anti-sink lesson (carried, still binding — distinct from #30's new lesson
     above):** do **NOT** read `$HOME/.claude/get-shit-done/workflows/plan-phase.md`
     linearly (~1720 lines / ~32k-token context sink — burned 2 prior rotations
     before a single subagent was dispatched, the #27/#28 trap). Follow it
     step-by-step, delegate EVERY heavy read to `reader-digester`, and let its own
     dispatched subagents (`gsd-phase-researcher`/`gsd-planner`/`gsd-plan-checker`)
     hold the heavy context. If a 3rd rotation sinks here, file a `GOOD-TO-HAVES` for
     a progressive-disclosure pass on that workflow file.
3. **During P115 planning, run `roadmap.update-plan-progress 114`** to clear the stale
   P114 ROADMAP checkbox — NEVER hand-edit `ROADMAP.md`.
4. **Carry the P115 BENCH-01 LOCKED CONSTRAINTS verbatim** into the plan (owner/
   manager-set, do not re-derive): ≤50 benchmark sessions on the EXISTING
   subscription / NO new API spend / escalate past 50 to the MANAGER (w1:p7) /
   hero-number waiver HARD DEADLINE **2026-08-15** (8 `agent-ux` hero-number
   doc-alignment rows) / `Execution mode: top-level`. Prior methodology home:
   `docs/benchmarks/latency.md`.
5. **After P115 is planned and executed**, open **P116 ADR-010 packet** — options +
   tradeoffs for BOTH ADR-01 (mirror-fanout) and FIX-03 (GTH-09 slug→id
   durable-create hazard), then route to the **MANAGER (w1:p7) for ruling — no
   pre-ruling implementation.**
6. **roadmap-diagram gsd-quick** (§5) — owner-approved, small; interleave
   opportunistically, mind the docs-alignment refresh-tail caveat if it touches a
   tracked doc.
7. **GOOD-TO-HAVES consolidation** — do NOT merge unilaterally; flag to the manager
   for a doctrine call if it hasn't already happened.
8. **Report to the manager (w1:p7)** at each boundary (P115 planning start, P115
   close, P116 routing to manager) and at any owner-blocking moment. The manager
   POLLS — clear in-pane narration at each boundary IS the report.
9. **Relieve past ~100k own-context tokens** (hard stop ~150k, absolute not %) at a
   wave boundary — dispatch `relief-handover-writer`, which writes+commits a fresh
   `.planning/SESSION-HANDOVER.md` that REPLACES this file, naming successor **#32**.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** (current milestone, P114 CLOSED GREEN, 1/15 phases done,
P115 opens next — the push blocker that gated it is now clear) → **v0.17
meta-milestone** (5 gate shapes: pivot-vocabulary lint, nav-budget, hero-redundancy,
framing-claim rows, persona whole-journey rubric; + subjective-runner Task-dispatch fix
unfreezing 3 WAIVED meaning-gates; + waiver-escalation rule; + transcript retention; +
bloat remediation incl. the SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure
split) → **v0.19** truth purge + IA rebuild → **v0.21** benchmark honesty (re-fixture
live baseline, CI job, headline-cross-check verifier) → **v0.23** journey slices →
**v0.25** launch kit → Show-HN. **Q3 launch gate:** Show-HN gated on a walkable
REAL-BACKEND journey (GitHub minimum), not sim-first. **Deep-survey calibration:**
~10% latent work per pass, ~10 passes to converge, recurring deep surveys are
STANDING practice. **Q9 ceiling:** keep v0.15→v0.25 ≈ 6-milestone scale.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
Claude-Session: relief-handover-writer
