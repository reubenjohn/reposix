# SESSION-HANDOVER.md — v0.15.0 Floor: P116 CLOSED GREEN, GitHub Actions outage
cleared, main certified green — 2026-07-16

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the § 1
verify block yourself first.**

Written by **workhorse #53** (L0 orchestrator), relieving to successor **#54**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#52→#53's
handover, commit `4c069ba`, superseded here — that file was bloated with now-moot
GitHub-Actions-outage diagnostic detail; the outage is fully cleared, so this handover is
deliberately leaner). #53's rotation closed P116 GREEN (verifier 12/12 must-haves PASS, 0
gaps) and advanced STATE/PROGRESS/ROADMAP past it — a clean wave boundary.

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state →
§3 binding constraints (carry verbatim, note the NEW `.git/config` re-corruption hazard)
→ §4 litmus/gate state → §5 noticed/open threads → §6 runbook (P117 planning is the
primary work).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a
leaf. Relieve past ~100k own-context (hard 150k, absolute not %) at a wave boundary.
Every push Bash timeout ≥300s. Refresh `PROGRESS.md`'s `## NOW` at every boundary push.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && \
  git status --porcelain --untracked-files=all && \
  grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json
```

**Expected after THIS handover's commit lands:** `HEAD == origin/main == <this commit>`,
tree CLEAN, `RETIRE_PROPOSED` count `0`.

- **P116 is CLOSED GREEN.** gsd-verifier verdict: **12/12 must-haves PASS, 0 gaps, 0
  blockers** (`.planning/phases/116-adr-010-mirror-fanout-decision-packet-slug-id-durable-create/116-VERIFICATION.md`).
  CI on the phase tip `6825d13` = run `29544462493` concluded `success`
  (`code/ci-green-on-main` P0 PASS). Full pre-push re-run by the verifier: 61 PASS / 0
  FAIL / 1 WAIVED / 0 NOT-VERIFIED.
- P116 delivered (commits `d667eee..6825d13`): 116-01 `a1cc2d4`+`7412833` (non-tautological
  mirror-convergence guard keyed on `"authoritative"` + blessed webhook+cron as
  AUTHORITATIVE in `CLAUDE.md` + `dvcs-topology.md` + bound catalog row), 116-02 `1ea51b3`
  (ADR-010 append-only record: RBF-LR-04 CLOSED, FIX-03 Option B sanctioned target design
  [design-only, zero `crates/` diff], packet cross-link), 116-03 `5ee5e25` (LIVE litmus
  SURPRISES row OPEN→RESOLVED, GOOD-TO-HAVES-09 → sanctioned target design), `6825d13`
  (noticings filed `GTH-V15-41`/`GTH-V15-42`). doc-alignment invariants held: `RETIRE_PROPOSED`=0,
  `RETIRE_CONFIRMED`=68, catalog id count 399→400.
- **3/15 v0.15.0 "Floor" phases now complete** (P114, P115, P116). Next phase: **P117**
  (Doc-truth launch-blocker purge).
- **This commit** — #53's bookkeeping: STATE.md cursor advanced past P116 (completed_phases
  2→3, percent 10→14), ROADMAP.md Phase 116 index + 3 plan checkboxes flipped `[x]`,
  PROGRESS.md SHIPPED bullet + `## NOW`/`## NEXT` refreshed, a HIGH `SURPRISES-INTAKE.md`
  row filed for the `.git/config` corruption incident (see below), plus this handover.
- **A concurrent manager-tier commit landed on `origin/main` mid-rotation, ahead of the
  ground truth #53 was launched against:** `7c46ee4` "docs(planning): encode owner
  delegation-depth directive — 1h+ legs, work 2 levels deep, manager stays meta" —
  touches ONLY `.planning/MANAGER-HANDOVER.md` (12 insertions), authored by the manager
  tier (Co-Authored-By `Claude Fable 5`). #53 did **not** touch that file (guardrail
  honored) and this commit's own tip (`6825d13`) predates it by one commit — `7c46ee4` is
  now the base this handover's commit lands on top of. **Noted for #54, not acted on**:
  the directive text says the workhorse-side encoding ("work 2 levels deep... into
  ORCHESTRATION.md via a tracked quick") is "routed to #54's charter" — if #54's launch
  charter carries that instruction, action it there; this handover does not pre-empt it.
- **Incident this rotation (repaired, filed HIGH):** the shared `.git/config` was
  corrupted (`core.bare = true` + fixture identity `t@t`) by a concurrent sibling
  worktree lane (`gth-hook-curb-capture`, live at the time) partway through this
  rotation, blocking all work-tree git ops until #53 repaired it via a direct config
  edit. `origin/main` + all refs were intact throughout — nothing durable lost. Full
  detail + fix-sketch: `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`
  (2026-07-16 23:50 entry, severity HIGH) — see §3 below for the operational hazard this
  leaves live for #54.

## 2. Wave/cycle state

| Phase | State | Commits / evidence |
|---|---|---|
| P114 | CLOSED (t4 Confluence oid-drift fix-first) | `dc26302` et al. |
| P115 | CLOSED GREEN — verifier GREEN-CHECKPOINT + human confirm-retire gate CLOSED (11 rows retired, `RETIRE_PROPOSED`→0, `RETIRE_CONFIRMED`→68) | `ce4d3b7` (verifier), `4bb0596` (owner confirm-retire batch) |
| P116 | **CLOSED GREEN** — gsd-verifier 12/12 must-haves PASS, 0 gaps, 0 blockers | `116-VERIFICATION.md`; plans `a1cc2d4`/`7412833`, `1ea51b3`, `5ee5e25`; noticings `6825d13` |
| P116 phase-close bookkeeping | STATE/ROADMAP/PROGRESS cursor-advance + SURPRISES filing + this handover | this commit |
| P117 | **NOT STARTED — next.** ROADMAP: "Doc-truth launch-blocker purge"; must fold owner furnished-product mandate (`GTH-V15-36`/`GTH-V15-37`) per ROADMAP annotation + `PROGRESS.md` `## NEXT` item 1 | — |

**3/15 v0.15.0 "Floor" phases complete** (P114, P115, P116); 12 remain (P117–P128).

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate owner); no tag push by any coordinator; no git
surgery (reset/rebase/amend/reorder) on main; leaf isolation in `/tmp`
same-Bash-invocation; opus complex / sonnet default / haiku mechanical, **never fable at
a leaf**; relieve past ~100k own-context (hard 150k, absolute not %) at a wave boundary;
**every push Bash timeout ≥300s**; refresh `PROGRESS.md`'s `## NOW` at every boundary
push; never open the next phase over a red main.

**NEW — shared `.git/config` re-corruption hazard (prominent, read before your first
git op).** A concurrent sibling worktree lane (`gth-hook-curb-capture`, LIVE at handover
time — `worktree-gth-hook-curb-capture` branch present in `.git/config`'s `[branch]`
list) can re-corrupt the SHARED `.git/config` via its leaf-setup, because the
`.claude/hooks/leaf-isolation-guard.sh` PreToolUse hook only covers the Claude Code Bash
*tool* — a subprocess/script write bypasses it. **If ANY work-tree git operation
suddenly fails** ("this operation must be run in a work tree", or the `.githooks/pre-commit`
fixture-identity check rejects a commit under `t@t`): FIRST run `cat .git/config` and
check for `bare = true` or an injected `[user] email = t@t` block. If present, repair via
a direct edit — set `core.bare = false`, remove the `[user]` block (the real identity
comes from the global gitconfig) — this is race-safe: an edit either lands cleanly or
errors if a concurrent write raced it (it does not silently clobber). Filed as
`SURPRISES-INTAKE.md` HIGH this rotation (`2026-07-16 23:50` entry) — a durable fix
(guard-hardening or lane isolation) is still needed, not just the live repair.

## 4. Litmus / gate / REOPEN state

- **P115 human confirm-retire gate: CLOSED.** `RETIRE_PROPOSED` = 0, `RETIRE_CONFIRMED` =
  68 (landed `4bb0596`). No further action needed.
- **P116 verifier verdict: GREEN, 12/12 must-haves, 0 gaps, 0 blockers**
  (`116-VERIFICATION.md`). CI green on the phase tip (`6825d13`, run `29544462493`,
  `success`).
- **CI on main: GREEN.** The GitHub Actions API 503 outage that blocked #51/#52 has
  fully cleared (confirmed live via `gh run view` by #53, not just the check-suites
  fallback) — do not carry forward any "environmental fail" framing from prior
  handovers, it no longer applies.
- **File-size soft-ceiling waiver `GTH-V15-21`** — masking OVER-BUDGET as `--warn-only`
  until **2026-08-08T00:00:00Z**. Now also masks: `PROGRESS.md` (already over the 20k
  ceiling per this file's own header note), `docs/decisions/010-l2-l3-cache-coherence.md`
  (**155% of the 20k ceiling** — `GTH-V15-42`, split candidate), `116-RESEARCH.md`
  (52,340 bytes), `116-PATTERNS.md` (22,259 bytes). Ledger-split owner call still pending
  before the waiver lapses.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **P117/P119 owner "furnished product" quality-bar mandate** (`GTH-V15-36` quality bar
   + `GTH-V15-37` 80s launch-animation embed) is the headline shaping input for P117/P119
   — already annotated on `.planning/ROADMAP.md` Phase 117 + Phase 119 and in
   `PROGRESS.md` `## NEXT` item 1. #54's `/gsd-plan-phase 117` planner MUST fold this in
   as an explicit acceptance-bar input, not an afterthought.
2. **`GTH-V15-41`** (the `structure/banned-words` gate's docs scope excludes
   `docs/decisions/**`, so the ADR banned-word "replace" rule is unenforced there) and
   **`GTH-V15-42`** (`docs/decisions/010-l2-l3-cache-coherence.md` at 155% of the 20k
   soft ceiling, a progressive-disclosure split candidate before the 2026-08-08 waiver
   lapse) — both filed at `6825d13`, both OPEN, tagged for P126 (docs-alignment tooling
   polish). No action needed from #54 unless a slot opens early.
3. **UNFILED candidate, carried from #52's handover:** a resilient CI-status probe (in
   `reposix doctor` or a quality gate) that falls back to the check-suites API when the
   runs API 503s — would have saved real diagnostic effort during the now-cleared
   outage. Still not filed (kept out of intake to conserve context across #52/#53); file
   to `GOOD-TO-HAVES.md` (severity LOW-MEDIUM, tooling/resilience tag) if a slot opens —
   #54 or a later rotation should be the one to close this loose end.
4. **The `.git/config` corruption coverage-boundary FIX** (filed `SURPRISES-INTAKE.md`
   HIGH this rotation, §1/§3 above) needs a durable remedy beyond the live repair — a
   candidate for a hardening phase or a `GOOD-TO-HAVES.md` row. Not yet routed past the
   SURPRISES filing; #54 should not need to re-repair the config if the sibling lane is
   fixed/paused in the meantime, but should know the recovery move (§3) regardless.
5. **Carry-forward from #49/#50/#52 §5 (do NOT re-file, still live):** concepts-page
   four-axis hero coverage gap (only OUTPUT+COST axes are test-pinned on the concepts
   page, CACHE-CREATE+INPUT-CONTEXT are stated-but-untested); `bind --help ::fn`
   Rust-only validator discrepancy (help text implies all file types, only `.rs` actually
   resolves); `test_main_offline_regenerates_doc_from_captures` byte-compare gap (the
   test never diffs against the real committed doc, the exact gap class behind the
   260716-f6o regression); the `GTH-V15-38` copy-paste-bleed false positive is STALE
   (fixed `6d21cae`) — do not chase it a fourth time if a subagent re-raises it.
6. **Sibling-lane ownership:** the manager (w1:p7) owns the `gth-hook-curb-capture`
   sibling lane responsible for this rotation's `.git/config` corruption. #54 should
   ensure the manager is aware — the `SURPRISES-INTAKE.md` HIGH row + this handover are
   the durable surfacing mechanism; #54 is not expected to directly message the manager
   but should not silently absorb a second corruption without escalating if it recurs.

## 6. Precise next steps (successor #54 runbook)

1. **Standard first-act verify block (§1).** Run it yourself; confirm HEAD == this
   handover's own commit == origin/main, tree clean, `RETIRE_PROPOSED` = 0.
2. **Be aware of the sibling-lane re-corruption hazard (§3)** before your first git
   write — if a work-tree git op suddenly fails, check `.git/config` for `bare = true`
   or a `t@t` fixture identity FIRST, before assuming a deeper problem.
3. **Primary work: P117.** Run `/gsd-plan-phase 117` (check `.planning/ROADMAP.md`'s
   Phase 117 entry for its `Execution mode` before dispatching — do not assume
   top-level). The planner MUST fold in the owner "furnished product" quality-bar
   mandate (`GTH-V15-36`/`GTH-V15-37`) as an explicit acceptance-bar input (§5 item 1).
4. **Push cadence.** `git push origin main` BEFORE dispatching any verifier subagent;
   then run `python3 quality/runners/run.py --cadence post-push --persist`
   (`code/ci-green-on-main` is P0 — asserts main's NEWEST run concluded success). Never
   open the next phase over a red main.
5. **Every push Bash timeout ≥300s.**
6. **Refresh `PROGRESS.md`'s `## NOW` at every boundary push** — do not let it go stale.
7. **REPLACE this handover** (not append) at your own relief, following this same
   `.planning/ORCHESTRATION.md` §3 template, with live-verified ground truth — re-check
   every claim live before carrying it forward.
