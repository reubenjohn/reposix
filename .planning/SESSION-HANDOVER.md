# SESSION-HANDOVER.md — v0.15.0 Floor: P120 implemented+pushed, CI
in-flight, verify+close next — 2026-07-17

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the
ground-truth block yourself first.**

Written by **workhorse seat #60** (L0 ROUTER), relieving to successor **seat #61**
(fresh L0 ROUTER — `.planning/ORCHESTRATION.md` § "L0 is a ROUTER"). This file
**REPLACES** the prior `#59→#60` handover (last reachable at commit `80a4282`) — that
handover's runbook (P118 dispatch) is fully executed and DONE, do not re-run it.
Milestone **v0.15.0 "Floor"**. STATE.md cursor currently still reads **6/15 (40%),
P119 last CLOSED** — this is STALE-BY-DESIGN: P120 is implemented and pushed but NOT
yet verified/closed, so the STATE advance has not happened yet. That advance is
seat #61's job (see §6). Router ROUTES ONLY — delegate reads through a
reader-digester, cap subagent reports.

**Read order:** this file → §1 ground truth (verify live) → §2 wave/phase state → §3
binding constraints → §4 litmus/gate state → §5 mid-execution decisions + noticed-not-
filed → §6 runbook (start at step 1).

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && git status --porcelain
git rev-list --left-right --count HEAD...origin/main
gh run view 29601596722 --json status,conclusion,headSha
```

**Live-verified by #60 immediately before writing this handover (2026-07-17):**

- `HEAD` = `origin/main` = `859ba0e3` (full: `859ba0e310b0a19aa544b96e2af780b962c24b85`,
  `docs(120-close): de-literal ghp_ token in GTH-V15-67 to clear cred-hygiene P0
  gate`). 0 ahead, 0 behind. `git status --porcelain` → empty, tree clean.
- `git log --oneline -8` (newest first): `859ba0e3` de-literal ghp_ token /
  `660c5285` file GTH-V15-67 / `695a8213` trim cargo-mutex.sh under shim-shape ceiling /
  `c34f7772` test-name-honesty markers (3 P120-W3 tests) / `e28268f4` backtick SoT doc
  comment for clippy / `2af20e15` non-ghp_ WR-01 fixture token / `5e372d14` commit
  REVIEW.md as phase evidence / `01d7cf63` file 3 GOOD-TO-HAVES intakes. All 20 commits
  from `9b514980..859ba0e3` are the P120 close tail (implementation + code-review-fix +
  W6 + push); confirmed present in `git log`, no gaps.
- **CI, `ci.yml` specifically (the critical open item):**
  - `d3268a5e` (P119 tip) → run `29582783259`, `completed/success`.
  - `9b514980` (PR #77 merge, landed mid-P120 from another session) → run
    `29587173796`, `completed/success`.
  - `859ba0e3` (CURRENT HEAD, P120 push) → run **`29601596722`, `status=in_progress`,
    conclusion empty** at the moment of this check — **NOT YET RESOLVED. Seat #61's
    first live-check obligation** (§6 step 1). Local pre-push was GREEN before this
    push landed; this is the post-push confirmation, not yet in hand.
- The P120 W0–W5 handover (prior C1, relieved at all-impl-complete) is committed at
  `8d7832ca` (`docs(planning): P120 C1 relief handover — W0-W5 code-complete, W6+push+
  verifier remain`) — **note:** the dispatching context named this commit `a0b07f28`;
  that SHA does not exist in `git log --all`. `8d7832ca` is the ground-truthed one; use
  it if you need to read that handover
  (`.planning/phases/120-cli-helper-error-hardening/120-HANDOVER.md`, 17,809 bytes —
  route through a reader-digester, do not read raw).

## 2. Wave/cycle state

| Item | State | Evidence |
|---|---|---|
| P118 (Post-bench honesty corrections) | CLOSED GREEN | `quality/reports/verdicts/p118/VERDICT.md`; CI confirmed |
| P119 (docs/planning simplification, "the P112 RAISE") | CLOSED GREEN, DP-4 intention-preserving pivot | `quality/reports/verdicts/p119/VERDICT.md` (`# P119 VERDICT — GREEN`); tip `d3268a5e`, CI run `29582783259` success. ROADMAP SC-1/SC-2 "deletions" were found LIVE targets at exec time → deferred+filed, not blindly executed. Manager P117-parenthetical reword ("owner-deferred"→"owner-approval PENDING") landed in `0dea3474`. |
| P120 (CLI+helper error hardening, UX-01) — implementation | DONE, pushed | W0–W5 by a prior C1 (relief handover `8d7832ca`); code-review + W6 + rebase + push by a successor C1 → tip `859ba0e3` on `origin/main` |
| P120 — verify + close | **NOT STARTED** — gated on CI `29601596722` resolving green | This handover's primary directive, §6 |
| Milestone v0.15.0 "Floor" | 6/15 phases complete (P114–P119); STATE.md advance to 7/15 (47%) is PENDING P120 close | `.planning/STATE.md` frontmatter (`completed_phases: 6`, `percent: 40`) — will read stale until seat #61 (or the woken C1) advances it at P120 close |

## 3. Binding constraints (carry forward, unchanged)

- One tree-writer at a time; **ONE cargo invocation machine-wide** (prefer `-p`, jobs=2,
  **cargo is FOREGROUND-only — NEVER `run_in_background`/detached**, root-caused a
  machine-wide deadlock this milestone that burned ~180k tokens); no `--no-verify`;
  targeted staging only (never `-A`/`.`); no tag push by any coordinator; no git surgery
  (reset/rebase/amend/reorder) on shared/pushed `main`.
- **Commit-before-stop**: an executor/coordinator that ends its turn without committing
  leaves orphaned work — this is the other half of the cargo-deadlock root cause.
- Leaf isolation: `reposix`/sim/git test setup in a `/tmp` clone, `cd` in the SAME
  Bash invocation as the mutating command — never the shared repo.
- Push cadence: `git push origin main` BEFORE any verifier-subagent dispatch, then
  `python3 quality/runners/run.py --cadence post-push --persist` — the
  `code/ci-green-on-main` (P0) probe must pass. Never open the next phase/wave over a
  red main.
- **GAUGE NOTE:** relieve at ~100k soft / ~150k hard ABSOLUTE own-context (not % of
  window), at a wave boundary, with a committed handover. Seat #60's own calibration:
  fresh-seat gauge baseline is ~6% (~60k overhead); relieve at ~18% gauge soft / ~22%
  hard (NOT the naive 10–13% read), ~1h+ routed work per leg.
- **Concurrency reality — OTHER sessions push to `origin/main` concurrently.** PR #77
  (`9b514980`) landed mid-P120 from another session; a cargo-mutex orphan this milestone
  came from session `claude-84b9`. Fetch-rebase-before-every-push is mandatory; expect
  divergence. Never re-wake a C1 while it may still have live children — confirm via
  `git log`/`git status` that the prior writer's work landed and the tree is quiescent
  before re-dispatching (a prior rotation had a benign push race in P119 from exactly
  this failure mode).
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

- **`code/ci-green-on-main` (P0):** green through `9b514980`/`d3268a5e`; **NOT yet
  confirmed for `859ba0e3`** — run `29601596722`, `in_progress` at write time. Seat
  #61's first obligation (§1, §6 step 1).
- **P120 code-review (WARNING-class, all fixed pre-push):** `REVIEW.md` (deep pass
  across `errmsg`/`backend_dispatch`/`bus_handler`/`bus_url`/`stateless_connect`/
  `errors.rs`/`sync.rs`/`worktree_helpers.rs`/`init.rs`, committed at `5e372d14` as
  phase evidence) found **3 real credential-leak findings**, all fixed before push:
  - **WR-01** `bus_handler` — token leaking into audit row + stderr (the worst one).
  - **WR-02** `sync.rs`.
  - **WR-03** `worktree_helpers.rs`.
  Fixes use `redact_userinfo` (NOT `strip_url_userinfo`, which passes
  `reposix::`-prefixed URLs through unredacted — a footgun filed as **GTH-V15-62**).
  **The gsd-verifier at close must confirm the regression tests actually ASSERT
  non-leakage** (tainted-byte / OP-2 doctrine), not merely that the code compiles.
- **`cred-hygiene.sh` (P0 pre-push gate):** tripped twice this rotation on
  test-fixture `ghp_`-shaped tokens (WR-01 fixture, then the GTH-V15-67 intake text
  itself) — both cleared by de-literalizing the fixture strings (`2af20e15`,
  `859ba0e3`). **GTH-V15-67** filed: a P0 security-gate needs a careful inline
  test-fixture allow-marker mechanism (deliberately NOT implemented now — needs
  careful review before landing).
- **`cargo-mutex.sh` false-match:** FIXED this rotation (P120 W6, `695a8213` trims it
  under the 34-line shim-shape ceiling); "foreground-only cargo + commit-before-stop"
  codified into ORCHESTRATION.md/CLAUDE.md (§3 above).
- **`docs-build/animation-renders`:** still `NOT-VERIFIED`, `blast_radius: P2`, correctly
  documented as intentionally absent pending the §3 owner gate. Only open/pending
  gate in the tree unrelated to P120.
- **`structure/file-size-limits` waiver (planning artifacts + `*.rs`):** WAIVED until
  **2026-08-08** — SURPRISES-INTAKE ~84KB, GOOD-TO-HAVES ~101–103KB,
  `reality-check.md` 47.9KB, `STATE.md` >20KB, `ROADMAP.md` 34.2KB all still over the
  20KB soft ceiling. An OP-9 milestone-close distill is needed BEFORE this waiver
  expires and starts BLOCKING pushes.

## 5. Mid-execution decisions + noticed-not-filed

**Manager deltas still in force (carry forward verbatim):**

- ALL >100-line reads via reader-digester; cap subagent report size at ≤400-word
  digests.
- Trust the context gauge; relieve ~100k soft / ~150k hard ABSOLUTE own-context (not
  % of window) at a wave boundary; write+commit a handover first.
- **CRITICAL shepherding-pattern lesson (why this handover exists at this exact
  boundary):** phase-coordinator C1s reliably END THEIR TURN after each delegation /
  at each CI gate; their own `gh run watch` invocation does NOT re-invoke them
  afterward. **The L0 seat MUST own every CI watch** (`run_in_background`, or Monitor
  with an until-loop) and wake the sleeping C1 via SendMessage when the run resolves
  green. Do not assume a dispatched C1 will notice CI finished on its own.
- Animation `gh release` upload stays HELD pending OWNER approval (MANAGER-deferred,
  not owner-decided — do not re-litigate, just hold per §3).

**Noticed-not-filed:** none new this rotation beyond what's already ledgered (see §4
GTH-V15-62, GTH-V15-67, and the pre-push time regression / oversized-artifacts items
below) — all were filed live during the P120 close tail, not left dangling.

**Pre-existing / carried debt (NOT blocking, triage on receipt per OP-8, never
silently skip):**

1. **Pre-push time regression** ~97–103s vs ~60s budget — 3-observation trend with
   2026-07-15 kcov-creep; filed MEDIUM (GTH-V15-56/related). Orthogonal to phases;
   needs a dedicated quality-gates/OP-9 pass.
2. **Oversized planning artifacts** (§4 waiver list) — expires 2026-08-08, needs an
   OP-9 milestone-close distill before it starts blocking.
3. **Top-level `GOOD-TO-HAVES.md` stale-scheme migration** (GTH-V15-61) — live intakes
   are milestone-scoped under `.planning/milestones/v0.15.0-phases/`; the top-level
   file's old scheme is stale.
4. **P119 SC-1/SC-2 stale-criteria** (deferred "live deletions") — filed MEDIUM, a
   P121+ candidate.
5. **GTH-V15-62** — `strip_url_userinfo` reposix-URL passthrough footgun (§4).
6. **GTH-V15-67** — `cred-hygiene.sh` inline test-fixture allow-marker mechanism (§4).

## 6. Precise next steps (successor seat #61 runbook)

1. **CI gate (FIRST action, before anything else).** Re-check
   `gh run view 29601596722 --json status,conclusion,headSha` — confirm `headSha`
   still matches `859ba0e3` (i.e. no one force-pushed over it) and `status`. If
   `in_progress`, watch it: `gh run watch 29601596722 --exit-status --interval 20` in
   a `run_in_background` Bash call, then read the captured log for the final
   `final=`/exit result — **the wrapper Bash call's own exit code is NOT the run
   result**, read the actual watch output. Do not proceed to step 2 on a red or
   still-unresolved run; investigate first (P120 touched CLI/helper error paths, so a
   red result could be a real regression, not just infra flake — unlike the docs-only
   pushes earlier this milestone).
2. **On CI green — wake the P120 successor C1**, committed handover at `8d7832ca`
   (that C1's own agent-session context — confirm it is still addressable via
   SendMessage before assuming it's idle-and-safe to wake; if unreachable/expired,
   see step 2a). Direct it to run its verifier wave: `gsd-verifier` grades SC1–SC3
   (incl. catalog-first ordering + the 3 credential-leak fixes, §4) → verdict at
   `quality/reports/verdicts/p120/VERDICT.md` → on GREEN, advance STATE.md
   (`completed_phases` 6→7, `percent` 40→47, cursor P120 CLOSED / next P121) → commit
   → push (rebase-if-diverged, §3 concurrency reality) → re-confirm
   `code/ci-green-on-main` post-push.
   - **2a. If that C1 relieves again before finishing verify+close** (it was near
     ~100k and may dispatch its own `relief-handover-writer` first) — read ITS
     handover (will land under
     `.planning/phases/120-cli-helper-error-hardening/`), then dispatch a **fresh**
     `phase-coordinator` (`model: opus`) C1 from that handover to finish verify+close.
     **Do not orphan the verify** — someone must own it through to a committed
     STATE.md advance.
   - Before re-dispatching ANY C1 into this lane, confirm via `git log`/`git status`
     that the prior writer's work is fully landed and the tree is quiescent (§3
     concurrency note — avoid a repeat of the P119 benign push race).
3. **After P120 CLOSED GREEN — open P121.** Ground its ROADMAP entry via a
   reader-digester first (no `PLAN.md` exists yet — the dispatched C1 authors it).
   Dispatch a fresh **opus** `phase-coordinator` C1 (no-fable top-level → explicit
   `model: opus`, ORCHESTRATION §11) with the full OD-3 ownership charter,
   push-before-verifier cadence, and the `code/ci-green-on-main` P0 probe. Pull the
   `coordinator-dispatch` skill for the exact charter before dispatching.
4. **HOLD the E1 animation owner-gate** (§3). Never self-authorize, never tag
   `[OWNER]` without genuine owner input. `animation-renders` staying NOT-VERIFIED is
   a pending gate, not an owner-accepted deferral.
5. Carry §5's debt items forward unchanged unless drained in an OP-8/OP-9 absorption
   slot — none are urgent enough to interrupt P120 verify/close or P121 dispatch.
6. **REPLACE this handover** (not append) at your own relief, re-verifying every claim
   live before carrying it forward — an uncommitted handover didn't happen.
