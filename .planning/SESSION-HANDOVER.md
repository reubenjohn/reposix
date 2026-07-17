# SESSION-HANDOVER.md — v0.15.0 Floor: P117 CLOSED GREEN, counters
reconciled, next = P118 — 2026-07-17

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the
ground-truth block yourself first.**

Written by **workhorse seat #59** (L0 ROUTER), relieving to successor **seat #60**
(fresh L0 ROUTER — `.planning/ORCHESTRATION.md` § "L0 is a ROUTER"). This file
**REPLACES** the prior `#58→#59` handover (last reachable at commit `08cb62b`) — that
handover's runbook is fully executed and DONE, do not re-run it. Milestone
**v0.15.0 "Floor"**. STATE cursor: **P117 CLOSED GREEN; next = P118.** Router ROUTES
ONLY — delegate reads through a reader-digester, cap subagent reports.

**Read order:** this file → §1 ground truth (verify live) → §2 wave/phase state → §3
binding constraints → §4 litmus/gate state → §5 mid-execution decisions + noticed-not-
filed → §6 runbook (start at step 1).

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && git status --porcelain
git rev-list --left-right --count origin/main...HEAD
gh run list --workflow=ci.yml --branch main -L 5 \
  --json headSha,conclusion,status,createdAt \
  --jq '.[]|"\(.headSha[0:7]) \(.status)/\(.conclusion) \(.createdAt)"'
```

**Live-verified by #59 immediately before writing this handover (2026-07-17):**

- `HEAD` = `origin/main` = `b0c1c12` (`docs(planning): reconcile STATE frontmatter
  counters (15/4/27) + flip stale P115/P117 ROADMAP checkboxes`). 0 ahead, 0 behind.
  `git status --porcelain` → empty, tree clean.
- Commits since the last confirmed-green tip, newest-first: `b0c1c12` (this rotation's
  fix), `08cb62b` (#58→#59 relief handover), `97a4008` (honesty fix — E1 deferral was
  MANAGER not OWNER), `df5bdc6` (P117 CLOSED GREEN), `cdf5557` (filed ci-green-on-main
  race surprise), `698293f` (P117 phase-close verdict GREEN) — all confirmed present
  in `git log`, no gaps.
- **CI, `ci.yml` specifically:**
  - `08cb62b` → run `29572779674`, `completed/success` (confirmed by #59, satisfies
    the inherited runbook step-1 gate).
  - `97a4008` → run `29572436284`, `completed/success`.
  - `b0c1c12` (current tip) → run `29573427893`, was **`in_progress`** at the moment
    of this check — **NOT YET RESOLVED. Seat #60's first live-check obligation:**
    re-run the `gh run list` command above and confirm it resolved to
    `completed/success` before opening P118 or trusting main as CI-green. `b0c1c12`
    is docs-only (2 files, `ROADMAP.md`/`STATE.md`), so a red result would signal
    infra/flake, not a content regression — investigate, don't bury under new
    commits, don't open P118 over it.
- `quality/catalogs/doc-alignment.json` — check before trusting; if dirty and you did
  not run a `doc-alignment` verb yourself, `git checkout -- quality/catalogs/doc-alignment.json`
  (regenerable `summary.last_walked` noise, not real work — bit #57 and #58 both).

## 2. Wave/cycle state

| Item | State | Evidence |
|---|---|---|
| P117 (doc-truth launch-blocker purge) | CLOSED GREEN (non-blocked scope) | `quality/reports/verdicts/p117/VERDICT.md`; CI green on `cdf5557`/`c3b4d5c` |
| STATE.md counter reconciliation | DONE this rotation | `b0c1c12`: frontmatter was `total_phases:21 completed_phases:3 percent:14` (WRONG) → fixed to `15/4/27` (ground truth: 15 total P114–P128, P114/P115/P116/P117 all CLOSED = 4). Root cause: P115 and P117 ROADMAP checkboxes were never flipped `[ ]`→`[x]` at their closes, which mis-fed a phase-count verifier this session. Both flipped in the same commit. |
| Honesty note | A fresh reader-digester was fooled by the stale P115 checkbox mid-shift this rotation; direct `git log` verification (P115 CLOSED @ `d667eee`) was decisive. **Trust `git log` over ROADMAP checkboxes** when the two disagree. | n/a |
| Milestone v0.15.0 "Floor" | 4/15 phases complete (P114–P117); 11 remain; **NEXT = P118** | `.planning/ROADMAP.md` §"Phase index (P114–P128)", now consistent with `STATE.md` frontmatter and prose |
| P118 launch | NOT YET DISPATCHED — directed by this handover (see §6 step 3), gated on §1 step-1 CI confirmation | `.planning/ROADMAP.md:71` |

## 3. Binding constraints (carry forward, unchanged)

- One tree-writer at a time; **ONE cargo invocation machine-wide** (prefer `-p`); no
  `--no-verify`; targeted staging only (never `-A`/`.`); no tag push by any
  coordinator; no git surgery (reset/rebase/amend/reorder) on shared/pushed `main`.
- Leaf isolation: `reposix`/sim/git test setup in a `/tmp` clone, `cd` in the SAME
  Bash invocation as the mutating command — never the shared repo.
- Push cadence: `git push origin main` BEFORE any verifier-subagent dispatch, then
  `python3 quality/runners/run.py --cadence post-push --persist` — the
  `code/ci-green-on-main` (P0) probe must pass. Never open the next phase/wave over a
  red main.
- **GAUGE NOTE:** relieve at ~100k soft / 150k hard ABSOLUTE own-context (not % of
  window), at a wave boundary, with a committed handover.
- **THE OPEN OWNER GATE (carry forward, unresolved):** launch-animation E1 publish
  (`gh release create docs-assets --latest=false` + mp4 upload + live
  `animation-renders` playwright verify — second half of `117-07`) is
  **MANAGER-DEFERRED under standing doctrine (outward publishing = owner-only), with
  OWNER APPROVAL STILL PENDING.** Ledger: `.planning/CONSULT-DECISIONS.md` `## 2026-07-17
  [MANAGER] launch-animation publish held (117-07 second half)`, tracked
  **GTH-V15-37**. Never self-authorize this action, never tag it `[OWNER]` without
  genuine owner input arriving — re-raise to the OWNER (not the manager) when
  reachable. `docs-build/animation-renders` reading `NOT-VERIFIED` is a PENDING gate,
  not an accepted deferral.

## 4. Litmus / gate / REOPEN state

- **`code/ci-green-on-main` (P0):** green through `97a4008`/`08cb62b`; **NOT yet
  confirmed for `b0c1c12`** — seat #60's first obligation (§1, §6 step 1).
- **`docs-alignment` walk:** clean at last check (0 unwaived `STALE_DOCS_DRIFT`).
- **`docs-build/animation-renders`:** `NOT-VERIFIED`, `blast_radius: P2`, correctly
  documented as intentionally absent pending the §3 owner gate. Only open/pending
  gate in the tree.
- **`structure/file-size-limits` on `.planning/ROADMAP.md`:** 34,207 chars vs the 20k
  soft ceiling (warn-only, WAIVED until 2026-08-08, part of the broader GTH-V15-21
  waiver class). Natural landing spot for the split is **P119** ("Docs/planning
  simplification / the P112 RAISE"). Do not split it now — not this rotation's job.

## 5. Mid-execution decisions + noticed-not-filed

**Manager deltas still in force (carry forward verbatim):**

- ALL >100-line reads via reader-digester; cap subagent report size at ≤300-word
  digests — an earlier grader-fanout wave ballooned a coordinator's context from
  ~96k→~165k on report size alone.
- Trust the context gauge %; relieve ~100k soft / ~150k hard ABSOLUTE own-context (not
  % of window) at a wave boundary; write+commit a handover first.
- **P118 has no standing pre-authorization beyond this handover's directive** — it IS
  directed here (§6 step 3), so dispatch it, gated only on the §1/§6-step-1 CI check.
- Animation `gh release` upload stays HELD pending OWNER approval (MANAGER-deferred,
  not owner-decided — do not re-litigate, just hold per §3).

**Honesty-defect lesson (carry verbatim — why #58 corrected its own prior commit):**
AskUserQuestion answers in this setup may route through the **MANAGER**, not the human
owner. The recurring system-reminder "No genuine owner input has been received" is
authoritative. Never tag a decision `[OWNER]` or write "owner approved/deferred" unless
genuine owner input actually arrived — tag `[MANAGER]` and mark the gate PENDING/open
otherwise.

**Pre-existing debt noted (NOT blocking, do not eager-fix now):**

1. `.planning/ROADMAP.md` 34,207 chars vs 20k soft limit, `structure/file-size-limits`
   row WAIVED until 2026-08-08 — see §4. Natural home for the split is P119. Leave it.
2. `GOOD-TO-HAVES.md`/`SURPRISES-INTAKE.md` (v0.15.0 milestone ledgers) both over the
   20KB structure ceiling, same waiver class, targeted for an OP-9 milestone-close
   split — carried from prior rotations, still true, not this rotation's problem.
3. GTH-V15-57 (doc-alignment rows bind to line numbers, causing repeated
   push-blocker cascades) — highest-leverage fix candidate in the ledger, still open.

**Noticed-not-filed:** none new this rotation beyond what's already in the v0.15.0
ledgers (see item 2/3 above, both confirmed still filed, no new items surfaced during
the counter-reconciliation work).

## 6. Precise next steps (successor seat #60 runbook)

1. **CI gate.** Confirm HEAD = `origin/main` = `b0c1c12` (or newer), clean tree, AND
   `b0c1c12`'s `ci.yml` run (`29573427893`) resolved `completed/success` — it was
   IN-FLIGHT at this handover's write time. `b0c1c12` is docs-only, so a red result
   would be infra/flake: investigate it directly, do NOT bury it under new commits,
   do NOT open P118 over a red/unresolved main.
2. If `quality/catalogs/doc-alignment.json` shows dirty and you did not run a
   doc-alignment verb yourself, `git checkout -- quality/catalogs/doc-alignment.json`
   — regenerable noise, not real work.
3. **PRIMARY WORK — dispatch P118** ("Post-bench honesty corrections",
   `.planning/ROADMAP.md:71` / §Phase index: correct the disputed token-count figure +
   the stale tag-cut premise). Dispatch a FRESH **opus** `phase-coordinator`
   (`subagent_type: phase-coordinator`, `model: opus`) as a per-phase C1 that OWNS
   P118 end-to-end by DELEGATING (reader-digester → gsd-executor → gsd-code-reviewer
   → gsd-verifier — never leaf work itself). Enforce push-before-verifier cadence +
   the `code/ci-green-on-main` post-push P0 probe. Embed the full ownership charter
   (OD-3, root `CLAUDE.md` § "Ownership charter for dispatched subagents"). Pull the
   `coordinator-dispatch` skill for the exact charter + role→subagent_type mapping
   before dispatching. **IMPORTANT:** the seat that dispatches the P118 C1 should own
   it through checkpoint/close — do NOT spawn it and immediately relieve (that orphans
   the background C1 across the session boundary).
4. **HOLD the E1 animation owner-gate** (§3). Never self-authorize, never tag
   `[OWNER]` without genuine owner input. `animation-renders` staying NOT-VERIFIED is
   a pending gate, not an owner-accepted deferral.
5. Carry §5's pre-existing debt items forward unchanged unless drained in an OP-8/OP-9
   absorption slot — none are urgent enough to interrupt P118 dispatch.
6. **REPLACE this handover** (not append) at your own relief, re-verifying every claim
   live before carrying it forward — an uncommitted handover didn't happen.
