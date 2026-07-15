# SESSION-HANDOVER.md — v0.15.0 Floor: P114 Wave 1 done+CI-green, Wave 2 opens next — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #26** (L0
orchestrator, pane w1:p5, herded by the manager in w1:p7), relieving to **successor
#27**. This file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md`
(#25→#26's handover, committed at `fb38189`).

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
- Local `main` HEAD = `57362ff94794080a2c052938eb4fde9daa67ef3b` (short `57362ff`,
  the C1 coordinator's own Wave-1-relief handover doc), tree **clean**
  (`git status --porcelain` empty), **+1 AHEAD** of `origin/main`
  (`origin/main` = `eaf24d944276ac8b43072e6a73360c0b68317fc2`, short `eaf24d9`) —
  `git rev-list --left-right --count HEAD...origin/main` → `1  0`. This relief adds the
  #26→#27 handover commit on top of `57362ff` and pushes both.
- **CI GREEN, verified live via `gh run list`:** newest `ci.yml` run on `main`
  (headSha `eaf24d9…`, run `29431686591`) is `completed` / `success`, 5m32s,
  concluded `2026-07-15T16:16:02Z`. The run before it (`276beb8`, run `29431463121`)
  is `completed` / `cancelled` (superseded by the next push, not a failure). The run
  before that (Wave-1 tip `6f15138`, run `29396167796`) is `completed` / `success`,
  5m9s. **No red/pending run sits on main right now** — #27 may open Wave 2 without
  first chasing a CI resolution (unlike the #25→#26 handover, which had a queued run
  outstanding).
- Last CODE HEAD (non-docs commit) = `6f15138` (Wave-1 114-01 tip: cursor re-append
  lock test). Everything from `276beb8` through `57362ff` is `.planning/`-only docs.

**Commit lineage this rotation (#26), oldest → newest, chronological:**
1. `4652c7d` — docs(planning): file 2 carry-forward noticings (gsd-sdk `--message`
   footgun, stale v0.12.0 catalog example)
2. `88f2e2c` — docs(planning): file pre-push shell-coverage timing-creep noticing
   (amendment 4)
3. `47fa803` — test(114-01): add `list_and_get_render_parity` RED for Confluence
   oid-drift
4. `9908fcc` — fix(114-01): request `body-format=atlas_doc_format` on Confluence
   list path
5. `bf005bc` — docs(114-01): render-parity SUMMARY + RESEARCH OQ fold-in + nextest GTH
6. `db12187` — docs(114-01): correct RESEARCH OQ1 pre-ADF overstatement + file
   cursor-guard GTH
7. `6f15138` — test(114-01): lock cursor re-append carries body-format on >100-page
   follow (**Wave-1 tip, CI GREEN**)
8. `276beb8` — docs(planning): manager watch switches to polling model, ≤1h cap
   (owner directive 2026-07-15) — **manager's commit**, not #26's
9. `eaf24d9` — docs(planning): polling-model doctrine moves to `/herdr-manager`
   skill; handover keeps pointer — **manager's commit**, not #26's
10. `57362ff` — docs(114): C1 relief handover — Wave-1 done+CI-green, Wave-2 not
    started — the P114 C1 coordinator's own relief handover
    (`.planning/phases/114-.../114-HANDOVER.md`)
11. **`<this commit, created below>`** — the #26→#27 handover you are reading

**Deviations from the plan #27 MUST know:**
- The P114 phase is being run under a **C1 phase-coordinator**, not directly by the L0
  workhorse — Wave 2 dispatch is "spawn a fresh C1 seeded from `114-HANDOVER.md`," not
  "implement Wave 2 yourself." Do not collapse the tier.
- A background CI watcher **died mid-rotation** and stalled Wave 2 for ~2h before the
  manager caught it idle — see §5's new liveness doctrine before you dispatch/monitor
  anything that self-resumes in the background.

## 2. Wave/cycle state

| Wave | Plan | State | Commits |
|---|---|---|---|
| 114-01 (FIX-01, Confluence adapter render-parity) | `114-01-PLAN.md` | **DONE + CI GREEN** | `47fa803` (RED test) → `9908fcc` (fix: `body-format=atlas_doc_format` on list path) → `bf005bc` → `db12187` → `6f15138` (tip) |
| 114-02 (FIX-02, reconcile-audit) | `114-02-PLAN.md` | **NOT STARTED — #27's opening move** | — |

**Wave 1 summary:** Confluence adapter's `list_issues_impl` LIST url was missing
`&body-format=atlas_doc_format`, so list-render diverged from get-render, producing a
deterministic OidDrift. Fixed; RED→GREEN `list_and_get_render_parity` contract test
added; cursor/`next_url` pagination re-append locked with a follow-up test.
`gsd-code-reviewer` verdict: **APPROVE-WITH-NITS** — all 4 critical checks PASS; nit
#1/#2 absorbed pre-push; nit #3 filed as GTH-11 (cursor-guard false-skip). §5(a)
RESEARCH Open-Questions RESOLVED marker folded into `114-01-SUMMARY.md`.

**Named incident to read before dispatching the next executor:** the P114 C1
coordinator's background self-resume CI watcher (used to confirm `eaf24d9` went green
before opening Wave 2) **died silently** and never re-invoked itself. Wave 2 stalled
~2h until the manager polled and caught the coordinator idle. This is the trigger for
the new §5 liveness doctrine — read it before you spawn or monitor the Wave-2 C1.

**Wave 2 brief location:** the relieved P114 coordinator wrote
`.planning/phases/114-t4-confluence-oid-drift-fix-first-reconcile-audit/114-HANDOVER.md`
— it contains the full Wave-2-onward brief (the `DriftingMock` 3-test spec: drift
repro + reconcile-NON-recovery + aligned-resolves; the `error.rs`/`sync.rs`/`main.rs`
doc corrections with verbatim replacement strings; the `114-01-SUMMARY.md` L100
overstatement fold-in; cadence; verifier; SC1/SC2; STATE advance; constraints).
`cache.rs` was confirmed accurate during Wave 1 — leave it untouched. Do not re-derive
this brief yourself; hand the file to the fresh C1 and let it read its own plans.

## 3. Binding constraints (unchanged, carried)

- **One tree-writer at a time**; tree-mutating work is serial (no per-agent worktrees
  — owner rejected them as over-engineering for current cadence).
- **ONE cargo invocation machine-wide** (check/build/test/clippy) — prefer `-p <crate>`
  over `--workspace`; VM has OOM-crashed on parallel builds.
- **No `--no-verify`**, ever, on any commit or push.
- **Push at green, then confirm CI green on `main` AFTER the push** — run
  `python3 quality/runners/run.py --cadence post-push --persist`; the
  `code/ci-green-on-main` (P0) probe asserts the NEWEST `ci.yml` run on `main`
  concluded success, not merely that some older green run exists. Never open the next
  phase/wave over a red or pending main.
- **Commit-trailer format:** `Co-Authored-By: Claude <Model> <noreply@anthropic.com>`
  + `Claude-Session: <role-or-session-id>`.
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
- **Relieve past ~100k tokens of own context** (hard stop ~150k; **absolute, not %**
  of the window) at a wave boundary — write+commit a fresh handover, REPLACING this
  file, naming successor **#28**.

## 4. Litmus / gate / REOPEN state

- **Wave-1 code review:** `gsd-code-reviewer` → **APPROVE-WITH-NITS**. 4/4 critical
  checks PASS. Nit #1/#2 absorbed pre-push (folded into `bf005bc`/`db12187`). Nit #3 →
  filed as **GTH-11** (cursor-guard false-skip) in `GOOD-TO-HAVES.md`.
- **SC1/SC2 real-backend acceptance: NOT-VERIFIED** (Wave-2-dependent — no run has
  been attempted yet this rotation). Cadence per manager amendment: source `.env` in
  the SAME invocation + run `scripts/refresh-tokenworld-mirror.sh` as a PRE-STEP
  before any litmus/real-backend run; TokenWorld PROTECTED PAIR `7766017`/`7798785`
  is NEVER deleted (repair tool: `scripts/confluence_tokenworld.py`). Report
  NOT-VERIFIED honestly if creds/substrate are absent — never fake or skip-as-pass.
- **KEY SC1 RISK — flag prominently to the Wave-2 C1:** if TokenWorld page `7766017`
  is PRE-ADF (Confluence *storage* format, not ADF-native), the Wave-1 fix
  (`body-format=atlas_doc_format`) does **not** cover it → OidDrift persists → **SC1
  goes RED**, requiring a NEW list-path storage-fallback follow-up fix (research OQ1
  in `114-RESEARCH.md`). This is resolvable only by the live run. If SC1 is RED for
  this reason, the C1 must report honestly and file the follow-up as a
  phase/surprise — **do not hack around it**.
- **Pre-push timing WARN:** Wave-1's push took **127s** (WARN, > 60s budget) vs the
  already-filed ~91s baseline — a further +40% degradation. The WARN (`took ~91s+`)
  now trips on essentially every push. Root-cause noticing already filed at `88f2e2c`
  (kcov `code/shell-coverage` crept ~29s→~56s). #27: watch the Wave-2 and subsequent
  pushes; if they consistently land ~127s, this needs a REAL investigation (not just
  another WARN) plus a § Cadences baseline reconciliation soon — WARN fatigue erodes
  signal. **Do not re-file the base noticing**, just escalate if the trend holds.
- **Waiver / deadline clocks (carried, unchanged this rotation):**
  - `agent-ux` hero-number doc-alignment rows (8 total) — waiver expires
    **2026-08-15**.
  - `structure/file-size-limits` — waiver expires **2026-08-08**.
  - `perf-targets` — self-WAIVED until **2026-07-26**.
  - **P115 BENCH-01** hero-number waiver — **HARD DEADLINE 2026-08-15**; ≤50
    benchmark sessions on the subscription budget; escalate to the manager past 50
    (never exceed without a manager GO). Schedule early — see §6.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **NEW LIVENESS DOCTRINE (owner directive 2026-07-15, now durable in the
  `/herdr-manager` skill — carry forward every rotation until superseded):**
  background self-resume watchers are a **liveness risk**. This rotation, the P114
  C1 coordinator's background CI watcher died and never re-invoked itself, stalling
  Wave 2 for ~2h until the manager caught the idle child. Rule: (a) dispatchers MUST
  bound their wait and HEALTH-CHECK a quiet child — never idle-trust a background
  self-resume; (b) children MUST poll CI INLINE (≤1h cap) or run synchronously —
  never end their turn to wait on a background watcher alone. **#27: apply this
  directly when monitoring the Wave-2 C1** — bound the wait, proactively health-check,
  do not assume silence means progress.
- `client.rs` is ~130k chars — a genuine progressive-disclosure split candidate, but
  currently under the WAIVED `structure/file-size-limits` gate (expires 2026-08-08).
  Do NOT split early; this is v0.17 scope.
- **Intake already filed this rotation — do NOT re-file:**
  - gsd-sdk `--message` footgun + stale `v0.12.0` catalog example (`4652c7d`).
  - Pre-push timing-creep noticing / amendment 4 (`88f2e2c`).
  - GTH-10 (nextest absent in the executor's environment) + GTH-11 (cursor-guard
    false-skip) — filed by the Wave-1 executors during `bf005bc`/`db12187`.
- **P116 ADR-010 packet** (top-level, decision-only, unstarted): produce
  options+tradeoffs for BOTH **ADR-01** (mirror-fanout) and **FIX-03** (GTH-09
  slug→id durable-create hazard — the ADR-010 convergence contract is FALSE for
  CREATEs on id-assigning backends; an interrupted create can duplicate on retry).
  Route the packet to the **MANAGER (w1:p7) for ruling** — NO pre-ruling
  implementation of either option.
- **Directive 2** (GSD-quick, low urgency, now **3 rotations pending** — #24→#25,
  #25→#26, #26→#27 all carried it forward without picking it up): document the
  scratch-repo `reposix-scope-test-DELETEME` KEEP-policy into
  `docs/reference/testing-targets.md` (reset via force-push, never delete). Consider
  picking this up opportunistically if a gap opens — it is cheap and has been
  deferred three times.

## 6. Precise next steps (successor #27 runbook)

1. **Re-verify §1 ground truth live** before doing anything else: `git rev-parse
   HEAD`, `git status --porcelain`, `git rev-list --left-right --count
   HEAD...origin/main`, `gh run list --branch main --workflow CI --limit 3`. Do not
   trust the timestamps in this file — confirm independently.
2. **OPENING MOVE:** dispatch a **fresh C1 `phase-coordinator` (opus, clean
   context)** for **P114 Wave 2**, seeded from
   `.planning/phases/114-t4-confluence-oid-drift-fix-first-reconcile-audit/114-HANDOVER.md`
   (full Wave-2-onward brief — see §2). **Do NOT re-read the context-heavy
   `plan-phase.md`/`execute-phase.md` workflows yourself** — delegate from the start;
   the coordinator reads its own plans. **Bound your wait and proactively
   health-check the C1** per the §5 liveness doctrine — do not idle-trust a
   background self-resume.
3. **Wave-2 flow the C1 should run:** implement FIX-02 (reconcile-audit, per the
   `DriftingMock` 3-test spec + doc corrections in `114-HANDOVER.md`) → dispatch
   `gsd-code-reviewer` → push (TARGETED staging; carry forward any unpushed commits;
   `git pull --rebase origin main && git push` if origin advanced, never force) →
   confirm post-push `code/ci-green-on-main` GREEN → dispatch `gsd-verifier`
   (`VERIFICATION.md`; RED loops back to the C1, not to #27) → run real-backend
   SC1/SC2 acceptance (§4 cadence: `.env` + `scripts/refresh-tokenworld-mirror.sh`
   pre-step; watch the §3 KEY SC1 RISK — pre-ADF page → RED → file follow-up, don't
   hack around it) → advance `.planning/STATE.md` to P114 complete.
4. **After P114 closes**, in order: **P115 BENCH-01** (schedule EARLY — hard deadline
   2026-08-15, ≤50 benchmark sessions on the subscription, escalate to the manager
   past 50), then **P116 ADR-010 packet** (ADR-01 mirror-fanout + FIX-03 slug→id
   hazard, options+tradeoffs only, route to the manager w1:p7 for ruling, no
   pre-ruling implementation), then **Directive 2** (GSD-quick — scratch-repo
   KEEP-policy doc, 3 rotations pending).
5. **Report to the manager (w1:p7)** at each boundary (Wave-2 dispatch, Wave-2 close,
   P114 close, each subsequent phase close) and at any owner-blocking moment.
6. **Relieve past ~100k own-context tokens** (hard stop ~150k, absolute not %) at a
   wave boundary — dispatch `relief-handover-writer`, which writes+commits a fresh
   `.planning/SESSION-HANDOVER.md` that REPLACES this file, naming successor **#28**.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** (current milestone, P114 Wave 1 done+CI-green, Wave 2
opens next) → **v0.17 meta-milestone** (5 gate shapes: pivot-vocabulary lint,
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
