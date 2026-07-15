# SESSION-HANDOVER.md — Arc D pipeline ACTIVE; t4 real-backend re-run pending manager GO — 2026-07-14

Written by the **relief-handover-writer** on behalf of **workhorse successor #21** (L0
orchestrator, pane w1:p5, herded by the manager in w1:p7), relieving to **successor
#22**. This file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md`
(#20's CHARTER-COMPLETE handover, item 0 of that charter closed this rotation).

**Read order:** this file → §1 (verify live, do not trust timestamps) → §6 (runbook) →
dip into §2/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3 --json headSha,status,conclusion
```
**Verified independently by this handover-writer (2026-07-14, ~just now):**
- `HEAD` = `9468898` (full: `94688982f23ececd744cee38aa690841fafe8f4f`), tree **clean**
  (`git status --porcelain` empty — no output; the `code.json` modification noted in an
  earlier snapshot is no longer present in the working tree).
- `git rev-list --left-right --count HEAD...origin/main` → **`0  0`** — local `main` is
  **EVEN** with `origin/main`. This handover's commit will put local 1 ahead the instant
  it lands; the successor pushes it immediately after (§6) — do not treat a
  post-commit "1 ahead" reading as drift.
- `9468898` = `docs(planning): refresh manager handover — rotation #6→#7 (Arc D
  ratified, #21 live, credential audit complete, owner commitments)` — a
  **manager-authored** commit (not authored by this L0 rotation).
- **CI status on the 3 most recent `main` runs** (`gh run list`, verified live):
  | headSha | status | conclusion |
  |---|---|---|
  | `9468898...` (current HEAD) | `in_progress` | *(not yet concluded)* |
  | `3d0184f...` (item-0 commit) | `completed` | **success** |
  | `7ebdaa6...` | `completed` | `cancelled` (superseded by the next push before it finished) |
  **Actionable fact:** CI on the CURRENT HEAD (`9468898`) had NOT concluded at
  verification time — it was still `in_progress`. #22's first action (§6 step 1) must
  re-poll and confirm it lands `success` before treating main as a green base for new
  pushes, per the standing "never open next work over a red/pending main" rule. The
  last CONFIRMED-green run was `3d0184f` (item-0's commit, run `29385269398`), one
  commit behind current HEAD.
- A harmless orphaned background `gh`-poll (`b91lg45pm`, item-0 executor's CI watch) may
  still notify — ignore it, it is not actionable.

## 2. Wave/cycle state

| Item | Artifact | State | Commit(s) |
|---|---|---|---|
| 0 — STATE.md cursor refresh + carried-noticing intake filing | `/gsd-quick 260714-rcv` (`.planning/quick/260714-rcv-cursor-refresh-intake/`) | **DONE, pushed, CI-verified green** (run `29385269398`, `3d0184f`). Cursor now reads post-tag queue 0–5 CLOSED, Arc D ratified, pipeline active on new-milestone prep. #20's two carried noticings (GOOD-TO-HAVES oversize; v0.13.0-phases ROADMAP `NN-PLAN-OVERVIEW.md` broken links) filed as intake rows — exact file/row IDs are in `260714-rcv-SUMMARY.md`. | `3d0184f` |
| Directive 1 — pre-release-real-backend t4 (`agent-ux/t4-conflict-rebase-ancestry-real-backend`, P0) | `quality/runners/run.py --cadence pre-release-real-backend` | **NOT-VERIFIED — caveat NOT retired.** Root cause diagnosed (mechanical env-propagation gap, not a product failure — see §5). Correct command identified, NOT yet re-run. Footgun (env-gap skip silently downgrading 5 real-green rows, 2 of them P0) was caught and REVERTED (`git restore`, no bad commit, no push). | none this rotation |
| Directive 2 — scratch-repo `reposix-scope-test-DELETEME` policy doc | `docs/reference/testing-targets.md` | **NOT STARTED.** Manager commit `7ebdaa6` retired the RAISE in planning docs (git 2.50.1 verified; repo kept as reusable scratch target, reset via force-push, never deleted). Confirmed live (grep, this handover): `docs/reference/testing-targets.md` does **NOT** yet mention `DELETEME`/scratch — the doc-level policy note is still an open TODO, not a duplicate. | none |
| Item 1 — `/gsd-new-milestone` PROJECT.md re-anchor | — | **NOT STARTED.** Gated on Arc D (now RATIFIED, pipeline ACTIVE) — no longer blocked, just not yet begun. | — |
| Item 2 — v0.15 floor milestone definition + planning | — | **NOT STARTED.** Same gate status as item 1. | — |

**No named-incident / diagnostic pending for the next successor to read before acting**
beyond the t4 env-gap root-cause writeup in §5 (read before touching Directive 1).

## 3. Binding constraints (unchanged)

- **ONE cargo invocation machine-wide** (prefer `-p <crate>`). Leaf isolation: `/tmp`
  clones, `cd` in the SAME Bash invocation, never the shared tree.
- **Uncommitted = didn't happen.** Push per queue-item/phase cadence → confirm
  `code/ci-green-on-main` (P0) green → **never proceed over a red or pending main.**
- You **route, don't work**: delegate opus (complex/security), sonnet (default), haiku
  (mechanical); never fable at a leaf. Report to the manager (w1:p7) at each boundary
  or when blocked. Relieve past ~100k own-context tokens (hard stop ~150k) at a wave
  boundary — write+commit a handover first.
- **No `--no-verify`. No tag push by any coordinator** — the MANAGER cuts tags.
- **Owner-only stays owner-only.** The Directive-1 destructive real-backend re-run needs
  explicit manager/owner GO with live env confirmed (see §5/§6) before it executes — this
  is NOT yet obtained. New creds/scopes/spend, interactive sudo, outward publishing are
  all owner-gated.
- Do NOT touch `.planning/MANAGER-HANDOVER.md` (separate owner). Do NOT do git surgery
  on `main`.
- **Arc D is RATIFIED** (`6aa734a`, under owner delegation; canonical record: the
  ADDENDUM in `.planning/milestones/audits/2026-07-12-reality-check.md`). Pipeline pause
  is LIFTED, "no new lanes" is DISSOLVED — normal GSD gates apply to items 1/2 below.

## 4. Litmus / gate / REOPEN state

- `ci.yml` on `origin/main` — newest run (`9468898`, current HEAD) was **`in_progress`**
  at verification time; last CONFIRMED-`success` run is `3d0184f` (run `29385269398`).
  #22 must re-poll and confirm the newer run lands green (§1/§6).
- **`pre-release-real-backend` cadence — t4 row (`agent-ux/t4-conflict-rebase-ancestry-real-backend`,
  P0) has NEVER run green.** Last attempt this rotation: `0 PASS / 6 NOT-VERIFIED`, exit
  1, because `REPOSIX_ALLOWED_ORIGINS` and creds were unset in the subagent's process
  (root cause in §5). No catalog mutation landed from that attempt — the opus subagent
  `git restore`d `quality/catalogs/agent-ux.json` after noticing `--persist` had
  downgraded 5 genuinely-GREEN rows (2 P0: `milestone-close-vision-litmus`,
  `p93-partial-failure-recovery`) to NOT-VERIFIED. Working tree confirmed clean this
  handover — no lingering uncommitted catalog drift.
- **Open waiver clock (pre-existing, unrelated to this rotation):** 8 hero-number
  doc-alignment rows waived, expiring **2026-08-15** (funded live MCP re-measurement,
  Q1 2026-07-12). This is also the HARD DEADLINE the funded re-measurement must beat —
  see Item 2 in §6. Also pre-existing: token-economy perf-targets catalog self-declares
  `WAIVED until 2026-07-26` (`quality/catalogs/perf-targets.json`), not part of this
  rotation's work.
- No other open REOPEN-gate clock from this rotation's work.

## 5. Mid-execution decisions + noticed-not-filed

**Decisions/diagnoses made live this rotation:**
- **Directive-1 t4 root cause is a mechanical env-propagation gap, NOT a product
  failure.** `scripts/preflight-real-backends.sh` auto-sources `./.env` (so preflight
  reports 0/reachable), but `quality/runners/run.py` reads `os.environ` directly and
  does **not** source `.env`. In the dispatched subagent's process
  `REPOSIX_ALLOWED_ORIGINS` and backend creds were unset, so the `pre-release-real-backend`
  cadence skipped all 6 rows instead of exercising them. This means t4 (and the other 5
  rows) simply never ran, green or red — "preflight green" was silently read as "cadence
  executed," which is false.
- **Corrected command for the re-run** (authorization already exists — owner unblocked
  t4, manager-directed, owner's `.env` holds creds + `REPOSIX_ALLOWED_ORIGINS` includes
  `https://reuben-john.atlassian.net`):
  ```
  set -a; . ./.env; set +a; python3 quality/runners/run.py --cadence pre-release-real-backend --persist
  ```
  Before running, VERIFY IN-PROCESS that `REPOSIX_ALLOWED_ORIGINS` is set and creds are
  present (fail LOUD, do not let it silently skip again). Protected-pair guardrail:
  Confluence ids `7766017`/`7798785` must NEVER be deleted (use
  `scripts/confluence_tokenworld.py restore`/`reparent` if the fixture pair drifts).
- **This is an IRREVERSIBLE DESTRUCTIVE real-backend op.** The opus subagent that
  diagnosed the gap recommended a human/owner green-light with live env confirmed before
  firing it. #21 surfaced this to the manager as a decision point (fire-now vs defer)
  and did **not** receive/execute a GO before this handover. **#22 must obtain the
  manager's/owner's explicit GO first**, then delegate the corrected run to an **OPUS**
  subagent (complexity/security tier). A product-FAIL result → file a v0.15 fix-first
  item. A PASS retires the longest-standing release caveat.
- **Two infra fix-first noticings surfaced this rotation — NOT YET FILED as of this
  handover.** #22's first action per the manager's directive is to file these (v0.15
  candidates, OP-8 eager-fix-or-file):
  1. `quality/runners/run.py` does not source `./.env` while
     `preflight-real-backends.sh` does, causing the false-green-preflight/silent-skip
     gap above. Fix-it-twice: make `run.py` source `.env` (or bake
     `set -a; . ./.env; set +a` into the documented invocation) AND update the
     `pre-release-real-backend` doc references in `.planning/CLAUDE.md` /
     `docs/reference/testing-targets.md`.
  2. `--persist` on an env-gap SKIP silently rewrites genuinely-GREEN catalog rows to
     NOT-VERIFIED (downgraded 2 P0 rows before the subagent caught and reverted it via
     `git restore`). Fix: gate skip-driven writes behind an opt-in flag, or don't rewrite
     `status` on a skip outcome.

**Noticings carried forward from #20, already filed (do NOT re-file):**
- `GOOD-TO-HAVES.md` itself over its own size ceilings, masked by the repo-wide
  `structure/file-size-limits` waiver (expires 2026-08-08) — filed as an intake row this
  rotation via item 0 (`260714-rcv`); exact row ID in that quick's `-SUMMARY.md`.
- `v0.13.0-phases/ROADMAP.md` phase-index links pointing at non-existent
  `NN-PLAN-OVERVIEW.md` files (real artifact is a `NN-PLAN-OVERVIEW/` directory) — also
  filed via item 0 this rotation.

## 6. Precise next steps (successor #22 runbook)

1. **Re-verify §1 ground truth live first.** In particular re-poll CI on current HEAD
   (`9468898` or later if the manager moved main again) — confirm it concluded
   `success` before treating main as a clean base for new pushes. If it's still
   `in_progress`, wait/re-poll; if it concluded `failure`, treat as a red main (stop,
   diagnose, do not open new work over it).
2. **File the two infra fix-first noticings** from §5 into the intake mechanism (GSD
   `/gsd-quick` or direct intake-file entry per OP-8 convention) — this is the manager's
   named first action for this rotation.
3. **Obtain the manager's/owner's explicit GO on the corrected Directive-1 t4 re-run**
   (the destructive real-backend op, §5). Do not fire it without that GO. Once given,
   delegate to an **opus** subagent with: the `.env`-sourced command from §5, an
   in-process fail-loud check that `REPOSIX_ALLOWED_ORIGINS`/creds are actually set
   before proceeding, and the protected-pair guardrail (Confluence `7766017`/`7798785`
   never deleted). Report the verdict (PASS retires the t4 caveat; FAIL → file a v0.15
   fix-first item) to the manager.
4. **Proceed to Item 1 — `/gsd-new-milestone` PROJECT.md re-anchor.** Fold the Arc D
   ADDENDUM (`.planning/milestones/audits/2026-07-12-reality-check.md`) in FULL;
   reconcile the P112 ROADMAP prose-vs-artifact divergence (standing RAISE = docs/planning
   simplification as a first-class roadmap goal); replace the PROJECT.md truth banner
   with real re-anchored content. Reconcile the ADDENDUM's "cut two stalled tags
   (v0.13.0, v0.14.0)" phrasing against live reality — **v0.14.0 has already SHIPPED
   publicly**, so that phrasing predates the ship and needs correcting, not literal
   execution.
5. **Then Item 2 — v0.15 floor milestone definition + planning.** Route OPEN v0.15.0
   intakes + GOOD-TO-HAVES rows in (OP-8), INCLUDING the two infra fix-first noticings
   filed in step 2 and any Directive-1 product-FAIL item from step 3. **HARD DEADLINE —
   schedule EARLY:** the funded Q1 live MCP re-measurement must land before
   **2026-08-15** (8 hero-number doc-alignment waiver rows expire then; a late
   re-measurement re-blocks every docs push — §4). Include an **ADR-010 mirror fan-out
   decision packet** (options + tradeoffs prepared by a lane; the MANAGER decides — do
   NOT implement before that ruling).
   - Ratchet-first sequence for reference (canonical: the Arc D ADDENDUM — do not
     re-fetch, this is the digest): **v0.15** floor (kill 4 LAUNCH-BLOCKERs: index.md
     category, filesystem-layer rewrite, `reposix list/refresh` errors, `reposix detach`
     fix/delete, token-fixture provenance; cut the two stalled tags — reconcile vs live)
     → **v0.17** meta-milestone (5 gate shapes: pivot-vocabulary lint, nav-budget,
     hero-redundancy, framing-claim rows, persona whole-journey rubric; + subjective-runner
     Task-dispatch fix unfreezing 3 WAIVED meaning-gates; + waiver-escalation rule; +
     transcript retention; + bloat remediation, natural home for the GOOD-TO-HAVES split)
     → **v0.19** truth purge + IA rebuild (behind ratchets) → **v0.21** benchmark honesty
     (re-fixture live baseline, CI job, headline-cross-check verifier) → **v0.23** journey
     slices (connect-backend, agent-integration) → **v0.25** launch kit → Show-HN launch.
     Even numbers v0.16–v0.26 = small stub milestones (ratchets catch regressions at push
     time). **Q3 launch gate:** Show-HN gated on a walkable REAL-BACKEND journey (GitHub
     minimum), not sim-first. **Q5/Q7 mandate:** DELETE legacy/historical files outright,
     no keep-with-banners (git history is the archive). **Deep-survey calibration:** one
     pass surfaces ~10% latent work, expect ~10 passes to converge, recurring deep
     surveys are STANDING practice. **Q9 ceiling:** keep v0.15→v0.25 ≈ 6-milestone scale.
6. **Directive 2 doc note** — write the scratch-repo (`reposix-scope-test-DELETEME`)
   KEEP-as-reusable-scratch-target policy into `docs/reference/testing-targets.md` (owner
   policy: reset via force-push, never delete; currently archived, unarchive via API on
   first reuse) — confirmed NOT yet present (§2). Do this at GSD-quick scale when
   convenient; not urgent/blocking relative to steps 2–5.
7. **Report to the manager (w1:p7)** at each boundary above or when blocked — do not
   silently continue past a checkpoint.
8. **Relieve past ~100k own-context tokens at the next clean wave boundary** — write and
   commit a fresh `.planning/SESSION-HANDOVER.md` (REPLACE, not append) naming successor
   #23, following this same §3 (of `ORCHESTRATION.md`) template.
