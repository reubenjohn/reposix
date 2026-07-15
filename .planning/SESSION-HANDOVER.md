# SESSION-HANDOVER.md — v0.15.0 Floor: P114 CLOSED GREEN, P115 planning opens next — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #27** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #28**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#26→#27's
handover, committed at `276beb8`'s successor chain, superseded here).

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
- Local `main` HEAD = `e039bb79b2a5992016228445ea62f468685f7e5e` (short `e039bb7`, "docs:
  capture 2 todos — roadmap-diagram lane + GOOD-TO-HAVES consolidation"), tree
  **clean** (`git status --porcelain` empty), **+1 AHEAD** of `origin/main`
  (`origin/main` = `dc26302fefebb83da5e185e069f01592d30d2741`, short `dc26302`) —
  `git rev-list --left-right --count HEAD...origin/main` → `1  0`.
- **After this handover commit lands, local HEAD will be 2 AHEAD of origin**
  (`e039bb7` + this handover doc). **L0 pushes both after this commit** — #28 must
  re-verify CI green live on the pushed tip as runbook step 1 (docs-only diff,
  expected green, but confirm, don't assume).
- **CI GREEN, verified live via `gh run list --branch main --workflow CI --limit 3`:**
  newest `ci.yml` run on `main` (headSha `dc26302`, run `29439318240`) is
  `completed`/`success`, 5m37s, concluded 2026-07-15T18:08:15Z. The two runs before it
  (`5de29d4` run `29437799441`, success; `12a0f57` run `29435883070`, success) are also
  green — no red/pending run sits on main right now.
- `.planning/STATE.md` (`last_updated: 2026-07-15T18:10:00Z`) independently confirms:
  `status: in_progress`, `completed_phases: 1/15`, current position "P114 CLOSED
  GREEN", next action `/gsd-plan-phase 115`. This matches the git lineage below —
  cross-checked, not just trusted.

**Commit lineage this rotation (#27), oldest → newest, chronological:**
1. `9915953` — test(114-02): reproduction-backed oid-drift + reconcile-non-recovery
2. `0e200f9` — docs(114-02): scope `--reconcile`'s oid-drift recovery to what it heals
3. `de87650` — docs(114-02): Wave-2 reconcile-audit SUMMARY (FIX-02)
4. `9bed65a` — docs(114-02): align `read_blob` `# Errors` OidDrift framing (code-review
   nit)
5. `12a0f57` — docs(114-02): re-bind `cli-subcommand-surface` doc-alignment hash after
   FIX-02 doc-comment drift
6. `dde50fa` — docs(114): C1 relief handover — Wave-2 done+pushed+CI-green,
   verification tail opens next
7. `5de29d4` — docs(114): phase-close verification + batched GTH intake
   (FIX-01/FIX-02)
8. `dc26302` — docs(114): close phase — SC1/SC2 real-backend GREEN, cursor advance,
   SC1-cmd fix
9. `e039bb7` — docs: capture 2 todos — roadmap-diagram lane (owner-approved) +
   GOOD-TO-HAVES consolidation (**L0 closeout, current HEAD**)
10. **`<this commit, created below>`** — the #27→#28 handover you are reading

**Deviations from the plan #28 MUST know:**
- P114 is now **fully closed** — both waves shipped, verification tail ran, real-backend
  SC1/SC2 acceptance ran live and PASSED. There is no open Wave for #28 to resume; the
  next unit of work is **planning P115**, not executing an in-flight phase.
- The SC1 acceptance command in the plan/verification docs had a **latent false-GREEN**
  (a bare `git checkout -B main` materializes nothing, so the OidDrift check never
  actually ran) — this was caught and corrected to `git checkout -B main
  refs/reposix/origin/main` across 5 occurrences before close. Read §2/§3 before
  reusing that acceptance command anywhere else.

## 2. Wave/cycle state

| Wave/Phase | Plan | State | Commits |
|---|---|---|---|
| P114-01 (FIX-01, Confluence render-parity) | `114-01-PLAN.md` | **DONE + CI GREEN** | tip `6f15138` (prior rotation) |
| P114-02 (FIX-02, reconcile-audit) | `114-02-PLAN.md` | **DONE + CI GREEN** | `9915953` → `0e200f9` → `de87650` → `9bed65a` → `12a0f57` |
| P114 verification tail | `114-VERIFICATION.md` | **DONE + CI GREEN** | `5de29d4` |
| P114 phase-close (SC1/SC2 real-backend, cursor advance) | — | **DONE + CI GREEN** | `dc26302` |
| P114 closeout (2 todos filed) | — | **DONE** | `e039bb7` |
| **P115 BENCH-01** (live MCP benchmark re-measurement) | not yet written | **NOT STARTED — #28's opening move** | — |
| P116 ADR-010 packet (ADR-01 + FIX-03 options) | not yet written | NOT STARTED | — |
| roadmap-diagram gsd-quick (owner-approved, §4) | todo filed | NOT STARTED, queued | — |
| Directive 2 (scratch-repo KEEP-policy doc, gsd-quick) | todo | NOT STARTED, **4 rotations pending** | — |

**P114 close-out summary (milestone v0.15.0 Floor — 1 of 15 phases done):**
- Wave-1 FIX-01 (Confluence render-parity): done+green (tip `6f15138`).
- Wave-2 FIX-02 (reconcile-audit): done+green (`9915953`/`0e200f9`/`de87650`/`9bed65a`/
  `12a0f57`).
- Verification tail: `5de29d4` + `dc26302`, both CI-green.
- **SC results:** SC3/SC4 GREEN (`oid_drift_reconcile` 3/3, `list_and_get_render_parity`
  1/1). **SC1 PASS live** — TokenWorld page `7766017` materialized ADF-native (664B),
  zero OidDrift; the pre-ADF risk carried from the prior rotation did NOT fire (OQ1
  residual latent-but-dormant, tracked, not re-opened). **SC2 PASS**
  (conflict-rebase-ancestry real-backend, exit 0, 7 asserts). Protected TokenWorld pair
  `7766017`/`7798785` intact throughout.

**Named incident carried from the prior rotation (already resolved, informational
only):** a background self-resume CI watcher died mid-Wave-2 and stalled progress ~2h
before the manager caught it idle. This produced the standing liveness doctrine in §5 —
already applied for the rest of this rotation, no outstanding action.

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
- **Relieve past ~100k tokens of own context** (hard stop ~150k; **absolute, not %** of
  the window) at a wave boundary — write+commit a fresh handover, REPLACING this file,
  naming successor **#29**.

## 4. Litmus / gate / REOPEN state

- **CI gate run history:** `29439318240` (headSha `dc26302`) `completed`/`success`,
  5m37s — the current tip of `origin/main`, verified live this handover via `gh run
  list`. Preceding two runs (`29437799441` on `5de29d4`, `29435883070` on `12a0f57`)
  also `completed`/`success`. `code/ci-green-on-main` P0 = **PASS** on `dc26302`.
- **P114 real-backend acceptance (SC1–SC4), all GREEN, run live 2026-07-15 ~17:56Z**
  (tenant `reuben-john` / space `REPOSIX`): SC1 PASS (TokenWorld page `7766017`
  materialized ADF-native, 664B, zero OidDrift — the corrected `git checkout -B main
  refs/reposix/origin/main` command was used, not the earlier false-GREEN form); SC2
  PASS (`agent-ux/t4-conflict-rebase-ancestry-real-backend`, exit 0, 7 asserts); SC3/SC4
  GREEN via artifact-verifiable tests (`oid_drift_reconcile` 3/3,
  `list_and_get_render_parity` 1/1). Full transcript: `114-VERIFICATION.md` in
  `.planning/phases/114-t4-confluence-oid-drift-fix-first-reconcile-audit/`.
- **Waiver / deadline clocks (carried, unchanged this rotation):**
  - `agent-ux` hero-number doc-alignment rows (8 total) — waiver expires
    **2026-08-15**.
  - `structure/file-size-limits` — waiver expires **2026-08-08** (`client.rs` ~130k
    chars split is v0.17 scope, do NOT split early).
  - `perf-targets` — self-WAIVED until **2026-07-26**.
  - **P115 BENCH-01** hero-number waiver — **HARD DEADLINE 2026-08-15**; ≤50 benchmark
    sessions on the subscription budget; escalate to the manager past 50 (never exceed
    without a manager GO). Schedule early — see §6.
  - Pre-push timing WARN (was trending up rotation-over-rotation) is now understood as
    a **stale budget doc** (GTH-14), not a real regression — do NOT re-investigate.
- **Real-backend cadence (unchanged):** source `.env` in the SAME invocation as
  `run.py`; `scripts/refresh-tokenworld-mirror.sh` as a PRE-STEP before any
  litmus/real-backend run; TokenWorld protected pair `7766017`/`7798785` NEVER deleted
  (repair tool: `scripts/confluence_tokenworld.py`). Milestone-close needs the
  non-skippable 9th probe `pre-release-real-backend` — not yet due (milestone still
  open, 14 phases remain).

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **RAISE LIST triaged this rotation:**
  - **#1 ROADMAP.md checkbox stale** (line ~67 `- [ ] Phase 114` + progress row "Not
    started" — contradicts `STATE.md`). Deliberately NOT hand-edited (tool-owned).
    **FIX during P115 planning via `roadmap.update-plan-progress 114`** — #28 must
    ensure it runs as part of `/gsd-plan-phase 115`.
  - **#2 Two coexisting GOOD-TO-HAVES.md files** — root
    `.planning/GOOD-TO-HAVES.md` (prefix `GOOD-TO-HAVES-01,-09..16`) vs
    `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (prefix
    `GTH-V15-01..20`) — DISTINCT prefixes, ambiguity CONFIRMED real. Filed as todo
    `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`.
    **NEEDS a manager/owner DOCTRINE CALL before consolidating** — do NOT merge
    unilaterally. Flag to manager.
  - **#3 (informational, already fixed):** SC1 acceptance shipped a latent
    false-GREEN (bare `git checkout -B main` materializes nothing) — caught and fixed
    across 5 occurrences before close. Also independently confirmed by the closeout
    executor: a `link-resolution.py` `DEFAULT_GLOBS` gap and a GTH-numbering
    correction. No further action — informational only.
- **OWNER-APPROVED lane queued (manager w1:p7, 2026-07-15) — public birds-eye roadmap
  diagram.** Now a committed todo:
  `.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md` (all 5
  points verbatim — read it before scoping). **gsd-quick scale, schedule AFTER P114
  (now eligible), enter via `/gsd-quick`.** Summary: (1) new `docs/roadmap.md` in
  mkdocs nav — ONE color-coded mermaid, shipped/active(v0.15.0)/future arcs to OD-4
  golden end state, ARCS/CAPABILITIES not phase#/dates; (2) OWNER REQ — bidirectional
  `<!-- SYNC: -->` cross-links `docs/roadmap.md`↔`.planning/PROJECT.md` + add re-color
  line to OP-9 distill checklist in `.planning/PRACTICES.md`; (3) gates: mkdocs-strict,
  mermaid-renders, mcp-mermaid render-review, reposix-banned-words, docs→.planning
  link-resolution; (4) optional structure-gate row asserting the SYNC marker pair; (5)
  REQUIRED — extend `DEFAULT_GLOBS` in `quality/gates/docs-build/link-resolution.py`
  to cover `docs/*.md` + `.planning/PROJECT.md` (catalog-first if a row contract
  changes). **#28: ack to manager + schedule.**
- **Structural recommendation (not yet a decision — advisory only):** 14 phases remain
  (P115–P128). Consider launching a **milestone-scoped C2 coordinator-of-coordinators**
  so per-phase C1 rotations absorb below L0 rather than relieving L0 every ~100k tokens.
  **Caveat:** top-level PLANNING (`/gsd-plan-phase`) can't run inside a
  `phase-coordinator` (no Skill tool) — planning stays top-level (L0/#28); only
  execution would delegate to C1/C2. #28 to weigh this before P115 planning if context
  budget is a concern.
- **NEW LIVENESS DOCTRINE (owner directive 2026-07-15, durable in the `/herdr-manager`
  skill — carry forward every rotation until superseded):** background self-resume
  watchers are a liveness risk (a P114 C1's background CI watcher died mid-Wave-2 and
  stalled progress ~2h until the manager caught it idle). Rule: (a) dispatchers MUST
  bound their wait and HEALTH-CHECK a quiet child — never idle-trust a background
  self-resume; (b) children MUST poll CI INLINE (≤1h cap) or run synchronously — never
  end their turn to wait on a background watcher alone. Already applied for the rest of
  this rotation with no incident; #28 must keep applying it to any child it dispatches.
- **Manager (w1:p7) uses a POLLING model** (polls L0's pane, ≤1h cap) — clear in-pane
  narration at each boundary IS the report; escalate actively only for owner-blocking
  moments.
- **Directive 2** (GSD-quick, low urgency, now **4 rotations pending** — #24→#25,
  #25→#26, #26→#27, #27→#28 all carried it forward without picking it up): document the
  scratch-repo `reposix-scope-test-DELETEME` KEEP-policy into
  `docs/reference/testing-targets.md` (reset via force-push, never delete). Consider
  picking this up opportunistically — it is cheap and has been deferred four times now.
- **Intake already filed this rotation — do NOT re-file:** the 2 todos in `e039bb7`
  (roadmap-diagram lane + GOOD-TO-HAVES consolidation); GTH-16 (filed at `dc26302`); the
  SC1 false-GREEN command fix (already shipped, not an open item).

## 6. Precise next steps (successor #28 runbook)

1. **Re-verify §1 ground truth live** before doing anything else: `git rev-parse HEAD`,
   `git status --porcelain`, `git rev-list --left-right --count HEAD...origin/main`,
   `gh run list --branch main --workflow CI --limit 3`. Confirm HEAD is now 2 ahead of
   the `dc26302` origin recorded here (L0's push should have landed both `e039bb7` and
   this handover) and that CI is green on the new tip (docs-only diff, expected green,
   but confirm — do not assume).
2. **OPENING MOVE — P115 BENCH-01 planning.** `Execution mode: top-level`. Requires a
   **top-level `/gsd-plan-phase 115` FIRST** (heavy fresh charter — the reason this
   relief happened at a phase-boundary rather than mid-phase). **HARD DEADLINE
   2026-08-15**; ≤50 benchmark sessions on the subscription — escalate to the manager
   past 50, never exceed without a GO. Schedule EARLY. During this planning, also run
   `roadmap.update-plan-progress 114` to fix the stale ROADMAP checkbox (RAISE #1 in
   §5) — do not hand-edit ROADMAP.md directly.
3. **After P115 is planned and executed**, open **P116 ADR-010 packet** — produce
   options+tradeoffs for BOTH ADR-01 (mirror-fanout) and FIX-03 (GTH-09 slug→id
   durable-create hazard). **Route the packet to the MANAGER (w1:p7) for ruling — NO
   pre-ruling implementation** of either option.
4. **roadmap-diagram gsd-quick** (§5) — owner-approved, small; interleave
   opportunistically between P115/P116, does not need to block either.
5. **Directive 2** — scratch-repo KEEP-policy doc into
   `docs/reference/testing-targets.md`, now 4 rotations pending; pick up
   opportunistically if a gap opens.
6. **Weigh the §5 structural recommendation** (milestone-scoped C2
   coordinator-of-coordinators) before or during P115 planning if own-context budget is
   a live concern — not mandatory, advisory only.
7. **Report to the manager (w1:p7)** at each boundary (P115 planning start, P115 close,
   P116 routing to manager, each subsequent phase close) and at any owner-blocking
   moment.
8. **Relieve past ~100k own-context tokens** (hard stop ~150k, absolute not %) at a
   wave boundary — dispatch `relief-handover-writer`, which writes+commits a fresh
   `.planning/SESSION-HANDOVER.md` that REPLACES this file, naming successor **#29**.

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
