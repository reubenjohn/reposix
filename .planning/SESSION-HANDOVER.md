# SESSION-HANDOVER.md — v0.15.0 Floor: PUSH BLOCKED on docs-alignment refresh tail, P115 planning still not started — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #29** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #30**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#28→#29's
handover, superseded here).

**Read order:** this file → §1 (verify live) → the ⛔ BLOCKER at the top of §6 (do this
FIRST, before P115) → §2/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED
staging only, never `git add -A`/`.`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 5
```
**Verified independently this handover (2026-07-15, just now):**
- `origin/main` = `a266582` ("docs(planning): refresh manager handover — rotation
  #8→#9"; **manager-authored**), CI on that tip is **completed/success**
  (`ci.yml` run `29443988376`, 5m23s, concluded `2026-07-15T19:18:08Z`). No red run
  sits on origin/main's recent history (last 5 runs all `completed`/`success`).
- LOCAL `main` = `dff801b`, tree **clean** (`git status --porcelain` empty), **+2 ahead
  / 0 behind** `origin/main` (`git rev-list --left-right --count HEAD...origin/main` →
  `2  0`; `git status --branch` confirms `ahead 2`). This handover commit will make it
  **+3 ahead**.
- **Local-only commits not yet on origin, oldest → newest** (both #29-authored this
  rotation, both from the Directive-2 gsd-quick `260715-h1d`):
  1. `a165d48` — "docs(testing-targets): record reposix-scope-test-DELETEME
     KEEP-policy" (`Claude-Session: gsd-executor`) — the Directive-2 work product:
     adds a KEEP-policy subsection to `docs/reference/testing-targets.md`
     (never-delete / force-push-reset to keep URL stable / unarchive via `gh api`
     before first reuse), PLUS an eager-fix of a stale "Phase 36 cleanup automation"
     forward-reference (verified via git log/grep that no such automation ever
     shipped — rewritten to present-tense manual cleanup).
  2. `dff801b` — "docs(planning): close Directive 2 quick 260715-h1d — scratch-repo
     KEEP-policy + STATE cursor" (`Claude-Session: L0-workhorse-29`) — commits the
     quick's PLAN.md + SUMMARY.md and refreshes `.planning/STATE.md`
     (last_activity + quick-tasks table row).
  3. **`<this handover commit, created below>`** — the #29→#30 handover you are
     reading, will become the new local tip at +3 ahead.
- `.planning/STATE.md` last confirmed `status: in_progress`, `completed_phases: 1/15`,
  P114 CLOSED GREEN, P115 (`/gsd-plan-phase 115`) still the next planning action —
  **UNCHANGED this rotation**, #29 did not reach P115 planning (see §5 for why: the
  Directive-2 push surfaced an unbudgeted docs-alignment refresh tail exactly at
  #29's own ~100k soft-relief line).

**Deviations from the plan #30 MUST know:**
- **The local tree is 2 commits ahead of a CI-green origin/main and CANNOT push
  as-is** — a pre-push quality gate (`docs-alignment/walk`) BLOCKS on 11 drifted
  catalog rows caused by `a165d48`'s edit to `docs/reference/testing-targets.md`. This
  is the load-bearing fact of this rotation — see the ⛔ BLOCKER at the top of §6. It
  is a REAL gate doing its job, not a bug, not a false-positive, not waivable.
- No P115 planning artifacts exist yet — `gsd-sdk query init.plan-phase 115` was run
  by #28 (read-only) but `/gsd-plan-phase 115` itself has never been run. Nothing
  half-built to inspect; P115 planning starts from scratch, AFTER the push unblocks.

## 2. Wave/cycle state

| Wave/Phase | Plan | State | Commits |
|---|---|---|---|
| P114 (all waves + verification + close) | — | **DONE + CI GREEN** | tip carried from prior rotations |
| Directive 2 (scratch-repo KEEP-policy doc, gsd-quick `260715-h1d`) | `260715-h1d-PLAN.md` | **DONE — work committed, ends 5-rotation (now 6) starvation** | `a165d48`, `dff801b` (both **LOCAL ONLY**, not yet on origin) |
| docs-alignment refresh tail (11 drifted rows in `doc-alignment.json`, caused by `a165d48`) | — | **BLOCKING — NOT STARTED**, is the opening move for #30 | — |
| **P115 BENCH-01** (live MCP benchmark re-measurement) | not yet written | **NOT STARTED** — `init.plan-phase` query run by #28, phase dir not yet created, `/gsd-plan-phase 115` never run | — |
| P116 ADR-010 packet (ADR-01 + FIX-03 options) | not yet written | NOT STARTED | — |
| roadmap-diagram gsd-quick (owner-approved) | todo filed | NOT STARTED, queued | — |
| GOOD-TO-HAVES consolidation (needs manager doctrine call) | todo filed | NOT STARTED, blocked on manager ruling | — |

No named incident (no test failure, no corruption, no rollback) this rotation — the
"incident" is purely a quality-gate correctly blocking a push, resolved by running the
gate's own named recovery command. See §5 for the honest account.

## 3. Binding constraints (carried verbatim, unchanged)

- **One tree-writer at a time**; tree-mutating work is serial (no per-agent worktrees —
  owner rejected them as over-engineering for current cadence).
- **ONE cargo invocation machine-wide** (check/build/test/clippy) — prefer `-p <crate>`
  over `--workspace`; VM has OOM-crashed on parallel builds.
- **No `--no-verify`**, ever, on any commit or push — including to work around the
  docs-alignment blocker below; run the gate's named recovery instead.
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
  commits) — the 2 unpushed local commits are NOT yet "already pushed," so they may
  still be amended/reordered if truly needed, but prefer NOT to; the correct fix is to
  add a NEW catalog-refresh commit on top, not rewrite history.
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
  naming successor **#31**.

## 4. Litmus / gate / REOPEN state

- **CI gate:** `origin/main` tip `a266582` — `code/ci-green-on-main` P0 **PASS**
  (newest `ci.yml` run `29443988376`, completed/success). Local tip `dff801b` has NOT
  been pushed, so it has no CI run of its own yet.
- **REOPEN / active gate failure:** pre-push gate `docs-alignment/walk` is **FAILING
  (P0)** against the local tree — **NOT yet run against a push attempt this rotation
  by #29** (the failure was surfaced during the executor's mkdocs-strict + local
  doc-alignment sanity pass inside the Directive-2 quick, per its SUMMARY.md; #29 did
  not force a `git push` to re-confirm the exact stderr, to avoid a failed push attempt
  mid-relief). **#30 MUST re-run the actual `git push origin main` (or the pre-push
  hook standalone) to get the live gate transcript before assuming this handover's
  characterization is exact** — treat "11 drifted rows" as the best-known count, not
  gospel, and let the gate's own output be the final word.
  - Named rows expected to drift (`sources_drifted=[0]`, per doc-alignment catalog
    entries anchored into `docs/reference/testing-targets.md`): `jira-token`,
    `jira-email`, `jira-test-project-override`, `jira-url-format`,
    `github-url-prefix`, `jira-init-success`, `jira-instance`,
    `jira-reposix-override`, `skip-no-panic`, `skip-without-creds`, + 1 more (11
    total) — all in `quality/catalogs/doc-alignment.json`.
  - **Named recovery (repeated per row by the gate itself):** `/reposix-quality-refresh
    docs/reference/testing-targets.md` (top-level slash command). The
    `reposix-quality` binary is already built (`target/release/reposix-quality`, built
    Jul 13) — no rebuild should be needed unless it's stale/missing.
  - This is a REAL gate correctly catching drift from a legitimate doc edit — it is
    NOT a bug, NOT to be waived, NOT to be bypassed with `--no-verify`.
- **Waiver / deadline clocks (carried, unchanged this rotation):**
  - `agent-ux` hero-number doc-alignment rows (8 total) — waiver expires
    **2026-08-15** (P115/BENCH-01 re-measures to lift it).
  - `structure/file-size-limits` — waiver expires **2026-08-08** (`client.rs` split is
    v0.17 scope, do NOT split early).
  - `perf-targets` — self-WAIVED until **2026-07-26**.
  - Pre-push timing WARN is a stale budget doc (GTH-14), NOT a real regression — do
    NOT re-investigate.
- **Milestone-close 9th probe** (`pre-release-real-backend`) not yet due — milestone
  still open, 14 phases remain.
- **Intake already filed — do NOT re-file:** GTH-V15-21 (archived-handover file-size at
  the 08-08 waiver expiry, committed `71e904f`); the 2 todos filed prior rotation
  (roadmap-diagram lane + GOOD-TO-HAVES consolidation); GTH-16.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **What #29 did this rotation (honest account):** re-verified §1 ground truth live
  (HEAD lineage `71e904f`→`26ca703`→`87a4bb2` at start, CI green). **Closed Directive
  2** — a 5-rotation (now 6, counting this one) starvation — via gsd-quick `260715-h1d`
  (opus planner → sonnet executor): recorded the `reposix-scope-test-DELETEME`
  scratch-repo KEEP-policy in `docs/reference/testing-targets.md` (never-delete /
  reset-via-force-push / currently ARCHIVED per LIVE `gh api` check
  `archived:true, private:true, pushed 2026-07-14` / unarchive via `gh api -X PATCH
  repos/reubenjohn/reposix-scope-test-DELETEME -f archived=false` before first reuse).
  **Eager-fixed a genuine lying doc** in the same commit (authorized, OP-8 <1h/
  same-file scope): a stale "Phase 36 cleanup automation will handle this"
  forward-reference — verified via git log/grep that no such automation ever shipped —
  rewritten to present-tense manual cleanup. mkdocs-strict passed GREEN on the diff.
- **Why #29 relieved here (honest, not spun):** the Directive-2 push surfaced the
  docs-alignment refresh tail (a NEW, unbudgeted top-level workload — 11 drifted
  catalog rows requiring an Opus-graded `/reposix-quality-refresh` dispatch) at
  exactly the moment #29's own context reached the ~100k soft-relief threshold. Per
  doctrine §3 (relieve at a wave boundary past ~100k; the runway to 150k is
  CORRECTION MARGIN, not workload budget), #29 relieved rather than either (a)
  hand-bind 11 catalog rows context-fatigued — `reposix-quality doc-alignment bind`
  requires exact `--claim/--source/--test/--rationale` per row, a catalog-corruption
  risk under fatigue — or (b) start heavy P115 planning over-budget (the exact #28
  trap this handover explicitly avoids repeating). Directive-2's WORK is fully
  committed (`a165d48`, `dff801b`); only the push+refresh tail remains — a clean,
  discrete opening unit for a fresh #30. **This is a clean pre-P115 boundary, not a
  mid-flight rescue** — nothing is half-edited, no test is failing, no state is
  corrupted; only a push is pending behind a correctly-firing gate.
- **META-LESSON for #30 (fix-it-twice — this is a NEW instance, not a repeat of the
  #27/#28 plan-phase.md sink):** editing ANY `docs/**/*.md` file that carries
  doc-alignment catalog rows drifts those rows and REQUIRES a top-level
  `/reposix-quality-refresh <doc>` before the NEXT push lands. This tail surfaces
  ONLY at pre-push (`docs-alignment/walk`) — NOT at pre-commit, and NOT at the
  executor's own `mkdocs-strict` gate — so a "cheap docs-only change" framing is a
  trap for ANY tracked doc, not just this one. **Action for #30:** before editing a
  tracked doc in future work, `grep '<doc-path>' quality/catalogs/doc-alignment.json`
  to check if it's tracked, and budget the refresh tail if so. File a
  `GOOD-TO-HAVES` row proposing this pre-check be surfaced in the executor/quick-task
  contract or CLAUDE.md, so future agents stop under-scoping tracked-doc edits. This
  ALSO applies to the queued **roadmap-diagram gsd-quick** if it ends up touching a
  tracked doc — check before scoping it as "small."
- **RAISE LIST / open items carried forward, all still OPEN:**
  - **P116 ADR-010 packet** (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id
    durable-create hazard): after P115, produce options+tradeoffs and **route the
    packet to the MANAGER (w1:p7) for ruling — NO pre-ruling implementation.**
  - **roadmap-diagram gsd-quick** — owner-approved, small; todo
    `.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md` (read it
    for all 5 points before scoping). Interleave opportunistically; carries the
    docs-alignment refresh-tail caveat above if it touches a tracked doc.
  - **GOOD-TO-HAVES consolidation** (two coexisting files: root
    `.planning/GOOD-TO-HAVES.md` vs `.planning/milestones/v0.15.0-phases/
    GOOD-TO-HAVES.md`) — needs a manager/owner **DOCTRINE CALL** before merging; todo
    `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`. Do
    NOT merge unilaterally.
- **Intake already filed — do NOT re-file:** see §4 list above (GTH-V15-21, the 2
  todos, GTH-16). Directive 2's completion itself needs no separate intake filing —
  its resolution is the two local commits `a165d48`/`dff801b`.

## 6. Precise next steps (successor #30 runbook)

**⛔ THE BLOCKER — do this FIRST, before P115 or anything else:**

1. **Re-verify §1 ground truth live**: `git rev-parse HEAD && git status --porcelain
   && git rev-list --left-right --count HEAD...origin/main && gh run list --branch
   main --workflow CI --limit 3`. Confirm local is still `dff801b`+this handover
   commit, ahead of a CI-green `origin/main` tip.
2. **Run `/reposix-quality-refresh docs/reference/testing-targets.md`** (top-level
   slash command) to re-bind the 11 (confirm exact count live) drifted doc-alignment
   catalog rows. It dispatches Opus grader(s) that propose citations; the
   `reposix-quality` binary (`target/release/reposix-quality`) validates and mints —
   **NEVER hand-edit `quality/catalogs/doc-alignment.json` directly.** This command
   commits the minted catalog rows itself.
3. **`git push origin main`** — should now pass the `docs-alignment/walk` gate (lands
   `a165d48` + `dff801b` + this handover commit + the catalog-refresh commit, in that
   order, `--no-verify` NEVER).
4. **Post-push CI verify:** `python3 quality/runners/run.py --cadence post-push
   --persist` — confirm `code/ci-green-on-main` P0 **PASS** (the newest `ci.yml` run
   on the new tip concluded success). **Never open P115 over a red or pending main.**

**Then, P115 planning (the milestone's next substantive work):**

5. **Run `/gsd-plan-phase 115` FRESH from a clean context.** Heed the #27/#28
   meta-lesson (still binding, distinct from this rotation's new one): do **NOT** read
   `$HOME/.claude/get-shit-done/workflows/plan-phase.md` linearly (~1720 lines /
   ~32k-token context sink, burned 2 prior rotations before a single subagent was
   dispatched). Follow it step-by-step, delegate every heavy read to
   `reader-digester`, and let its own dispatched subagents
   (`gsd-phase-researcher`/`gsd-planner`/`gsd-plan-checker`) hold the heavy context.
   If a 3rd rotation sinks here, file a `GOOD-TO-HAVES` for a progressive-disclosure
   pass on that workflow file.
6. **During P115 planning, run `roadmap.update-plan-progress 114`** to clear the
   stale P114 ROADMAP checkbox — NEVER hand-edit `ROADMAP.md`.
7. **Carry the P115 BENCH-01 LOCKED CONSTRAINTS verbatim** into the plan (owner/
   manager-set, do not re-derive): ≤50 benchmark sessions on the EXISTING
   subscription / NO new API spend / escalate past 50 to the MANAGER (w1:p7) /
   hero-number waiver HARD DEADLINE **2026-08-15** (8 `agent-ux` hero-number
   doc-alignment rows) / `Execution mode: top-level` (planning AND execution stay
   top-level — `gsd-executor` lacks the Skill tool a `/gsd-plan-phase` sub-dispatch
   needs). Prior methodology home: `docs/benchmarks/latency.md`. P115 init facts
   (re-run `gsd-sdk query init.plan-phase 115` fresh, do not trust a cached copy):
   `phase_name` "Live MCP benchmark re-measurement", `slug`
   `live-mcp-benchmark-re-measurement`, `phase_req_ids` BENCH-01,
   `has_context`/`has_research`/`has_plans` all false, models researcher=sonnet
   planner=opus checker=sonnet, `phase_dir` NOT yet created (the workflow creates it).
8. **After P115 is planned and executed**, open **P116 ADR-010 packet** — options +
   tradeoffs for BOTH ADR-01 (mirror-fanout) and FIX-03 (GTH-09 slug→id
   durable-create hazard), then route to the **MANAGER (w1:p7) for ruling — no
   pre-ruling implementation.**
9. **roadmap-diagram gsd-quick** (§5) — owner-approved, small; interleave
   opportunistically, mind the docs-alignment refresh-tail caveat if it touches a
   tracked doc.
10. **GOOD-TO-HAVES consolidation** — do NOT merge unilaterally; flag to the manager
    for a doctrine call if it hasn't already happened.
11. **Report to the manager (w1:p7)** at each boundary (blocker resolved + push
    landed, P115 planning start, P115 close, P116 routing to manager) and at any
    owner-blocking moment. The manager POLLS — clear in-pane narration at each
    boundary IS the report.
12. **Relieve past ~100k own-context tokens** (hard stop ~150k, absolute not %) at a
    wave boundary — dispatch `relief-handover-writer`, which writes+commits a fresh
    `.planning/SESSION-HANDOVER.md` that REPLACES this file, naming successor **#31**.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** (current milestone, P114 CLOSED GREEN, 1/15 phases done,
P115 opens next once the push blocker clears) → **v0.17 meta-milestone** (5 gate
shapes: pivot-vocabulary lint, nav-budget, hero-redundancy, framing-claim rows,
persona whole-journey rubric; + subjective-runner Task-dispatch fix unfreezing 3
WAIVED meaning-gates; + waiver-escalation rule; + transcript retention; + bloat
remediation incl. the SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure split) →
**v0.19** truth purge + IA rebuild → **v0.21** benchmark honesty (re-fixture live
baseline, CI job, headline-cross-check verifier) → **v0.23** journey slices →
**v0.25** launch kit → Show-HN. **Q3 launch gate:** Show-HN gated on a walkable
REAL-BACKEND journey (GitHub minimum), not sim-first. **Deep-survey calibration:**
~10% latent work per pass, ~10 passes to converge, recurring deep surveys are
STANDING practice. **Q9 ceiling:** keep v0.15→v0.25 ≈ 6-milestone scale.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
Claude-Session: relief-handover-writer
