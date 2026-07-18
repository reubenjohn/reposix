# SESSION-HANDOVER.md — v0.15.0 Floor: P122 CLOSED GREEN, 9/15 (60%),
next = P123 — 2026-07-18

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly; re-run the
ground-truth block yourself first (STATE.md's own "last activity" line and this
handover's git snapshot can both drift under concurrent pushes).**

Written by **workhorse seat #61** (L0 ROUTER), relieving to successor **seat #62**
(fresh L0 ROUTER — `.planning/ORCHESTRATION.md` § "L0 is a ROUTER"). This file
**REPLACES** the prior `#60→#61` handover (last reachable at commit `efdb38e6`) — that
handover's runbook (P120 verify+close, then P121, then P122) is fully executed and
DONE; do not re-run it. Milestone **v0.15.0 "Floor"**. Router ROUTES ONLY — delegate
reads through a reader-digester, cap subagent reports.

**Read order:** this file → §1 ground truth (verify live) → §2 wave/phase state → §3
binding constraints → §4 litmus/gate/REOPEN state → §5 mid-execution decisions +
noticed-not-filed → §6 runbook (start at step 1, PRIORITY 1 is the in-flight CI check).

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain
git log --oneline -8
gh run view 29639213587 --json status,conclusion,headSha,workflowName
gh run list --branch main --limit 5 --json databaseId,status,conclusion,headSha,workflowName
```

**Live-verified by #61 immediately before writing this handover (2026-07-18):**

- `HEAD` = `origin/main` = `95bc7c5f` (full: `95bc7c5f45e60bf72e7286029184ad0c1f3e6fa1`).
  `git status --porcelain` → empty, tree clean.
- `git log --oneline -8` (newest first): `95bc7c5f` docs(v0.15.0): correct C2 handover
  liveness doctrine (L0 ruling) + GSD-quick doctrine-update owed / `47c1f9d3` docs
  (v0.15.0): C2 milestone continuation handover at P122/P123 boundary (9/15, 60%) /
  `a9e1f4c4` docs(122-close): advance STATE 8→9 (P122 CLOSED GREEN) / `00ab1579` docs
  (122-verdict): phase-close verdict GREEN / `985e7dc2` docs(122-review): commit
  gsd-code-reviewer REVIEW.md (SHIP-WITH-NITS) / `cb7b511b` fix(code): manual_let_else
  clippy lint / `f5974ebe` fix(122-close): correct RPX-0406 latch-1 corruption
  narrative / `bcdcc983` docs(122-04): W4 SUMMARY. All P122-close + C2-handover commits
  present, no gaps.
- **CI, `ci.yml` specifically (the critical open item):**
  - `a9e1f4c4` (P122 STATE-advance push) → run `29638768189`, `completed/success`
    (confirmed both directly via `gh run view` and via the C2 handover's own citation).
  - `95bc7c5f` (CURRENT HEAD, docs-only C2-handover-correction push) → run
    **`29639213587`, `status=in_progress`, conclusion empty** at the moment of this
    check — **NOT YET RESOLVED. Seat #62's first live-check obligation (§6 PRIORITY
    1).** `headSha` on that run confirmed to match `95bc7c5f`. Sibling `release-plz`
    run `29639213563` also `in_progress` on the same sha; `CodeQL` run `29639213371`
    already `completed/success` on `95bc7c5f`.
  - This is a **docs-only** push (handover files) — low regression risk, but the
    push-cadence rule is unconditional: never open P123 over an in-flight/red main.
- The P122 close was driven by a **C2 milestone coordinator** (coordinator-of-
  coordinators), not directly by this L0 seat — full P122 close detail, the liveness
  incident that made deterministic C2 control necessary, and the P123–P128 phase-list
  ground truth all live in **`.planning/milestones/v0.15.0-phases/C2-MILESTONE-
  HANDOVER.md`** @ `95bc7c5f` (26,889 bytes — route through a reader-digester, do not
  read raw). **That file is the authoritative next-steps doc for the C2 lane**; this
  L0 handover summarizes it but does not replace it.

## 2. Wave/cycle state

| Item | State | Evidence |
|---|---|---|
| P114–P118 | CLOSED GREEN | prior handovers; unchanged this rotation |
| P119 (docs/planning simplification, "P112 RAISE") | CLOSED GREEN, DP-4 pivot | `quality/reports/verdicts/p119/VERDICT.md`; unchanged this rotation |
| P120 (CLI+helper error hardening, UX-01) | CLOSED GREEN | `quality/reports/verdicts/p120/VERDICT.md`; SC1–SC3 PASS, 3 credential-leak fixes (WR-01/02/03) verified non-leaking |
| P121 (RPX error-code namespace + `reposix explain`, UX-02) | CLOSED GREEN | `quality/reports/verdicts/p121/VERDICT.md` (`80a37cea`); 6/6 SC + OP-2 PASS; 5 LOW GTHs filed (GTH-V15-73..77) |
| P122 (`reposix-remote` + `init` hardening, DRAIN-07/08/09) | **CLOSED GREEN** | `quality/reports/verdicts/p122/VERDICT.md` @ `00ab1579`; 3/3 SC PASS; close commit `a9e1f4c4`; CI run `29638768189` = success; GTH-V15-79 filed (on-demand-cadence decision) |
| **Milestone v0.15.0 "Floor"** | **9/15 phases complete (P114–P122), 60%. Next = P123.** | `.planning/STATE.md` frontmatter (`completed_phases: 9`, `percent: 60`) — matches live git |
| P123–P128 | NOT STARTED, scope already PINNED in top-level `.planning/ROADMAP.md` (only wave/plan breakdown is TBD) | See C2-MILESTONE-HANDOVER.md §2 table + "Phase-list ground truth" subsection |

**Named-incident post-mortem carried forward from the C2 handover (read before
dispatching P123's C1):** the P122 C1 (opus) ran plan→execute→review→push, then went
**dormant** after backgrounding its own CI watch, which never re-woke it. The C2 had to
take deterministic control (dispatch gsd-verifier + gsd-executor directly) rather than
wait on the C1's self-resume. This is the root of the LOAD-BEARING LIVENESS DOCTRINE in
§5 below — the single most important operational finding of this rotation.

## 3. Binding constraints (carry forward, unchanged)

- One tree-writer at a time; **ONE cargo invocation machine-wide** (prefer `-p`,
  jobs=2, **cargo is FOREGROUND-only — NEVER `run_in_background`/detached**, root-
  caused a machine-wide deadlock earlier this milestone that burned ~180k tokens); no
  `--no-verify`; targeted staging only (never `-A`/`.`); no tag push by any
  coordinator; no git surgery (reset/rebase/amend/reorder) on shared/pushed `main`.
- **Commit-before-stop**: an executor/coordinator that ends its turn without committing
  leaves orphaned work.
- Leaf isolation: `reposix`/sim/git test setup in a `/tmp` clone, `cd` in the SAME Bash
  invocation as the mutating command — never the shared repo. Mechanically enforced
  (`leaf-isolation-guard.sh`, exit 2) + pre-commit backstop; coverage boundary is
  Bash-tool-only (a subprocess/script write can still bypass it — HIGH, still open, see
  SURPRISES-INTAKE line ~545 from the P116-close incident).
- Push cadence: `git push origin main` BEFORE any verifier-subagent dispatch, then
  `python3 quality/runners/run.py --cadence post-push --persist` — the
  `code/ci-green-on-main` (P0) probe must pass on main's NEWEST `ci.yml` run. Never
  open the next phase/wave over a red or in-flight main.
- **GAUGE NOTE:** relieve at ~100k soft / ~150k hard ABSOLUTE own-context (not % of
  window), at a wave/phase boundary, with a committed handover. **Manager-set delta
  this rotation:** ~18% gauge soft / ~22% hard (NOT the naive 10–13% read), fresh-seat
  baseline ~6% overhead — AND model quality also degrades past ~150k tokens regardless
  of budget, so relieving near that ceiling is doubly justified.
- **Concurrency reality — OTHER sessions push to `origin/main` concurrently.** PR #77
  (`docs(hooks): GOOD-TO-HAVES-17 hook no-op noise + .claude/hooks/CLAUDE.md authoring
  guide`) merged 2026-07-17T14:14:32Z mid-milestone from another session/PR (confirmed
  `gh pr view 77` → `MERGED`). Fetch-rebase-before-every-push is mandatory; expect
  divergence. Never re-wake a dormant C1/C2 while it may still have live children —
  confirm via `git log`/`git status` that the prior writer's work landed and the tree
  is quiescent before re-dispatching.
- **THE OPEN OWNER GATE (carry forward, unresolved — see also §5 escalation list):**
  launch-animation E1 publish remains **MANAGER-DEFERRED under standing doctrine
  (outward publishing = owner-only), OWNER APPROVAL STILL PENDING.** Ledger:
  `.planning/CONSULT-DECISIONS.md` `## 2026-07-17 [MANAGER] launch-animation publish
  held (117-07 second half)`, tracked **GTH-V15-37**. Never self-authorize; never tag
  `[OWNER]` without genuine owner input.

## 4. Litmus / gate / REOPEN state

- **`code/ci-green-on-main` (P0):** green through `a9e1f4c4` (run `29638768189`);
  **NOT yet confirmed for `95bc7c5f`** — run `29639213587`, `in_progress` at write time.
  Seat #62's PRIORITY 1 obligation (§1, §6).
- **P122 verdict:** GREEN, `quality/reports/verdicts/p122/VERDICT.md`, verdict commit
  `00ab1579`. 3/3 SC PASS (GTH-V15-04 RBF-LR-03 modern-git ref-lock, GTH-V15-05
  `resolve_import_parent` loud-fail RPX-0508, GTH-V15-06 init self-safety refusal
  RPX-0406). `persist_downgrade: NONE` (569 catalog rows byte-identical before/after
  `--persist`) — confirmed no silent corruption on this close.
- **Open-waiver expiry clocks (all still ticking, none newly created this rotation):**
  - `structure/file-size-limits` OVER-BUDGET-tier `--warn-only` waiver on
    `GOOD-TO-HAVES.md`/`SURPRISES-INTAKE.md`/etc — **expires 2026-08-08T00:00:00Z**
    (`quality/catalogs/freshness-invariants.json` L666). See §5 carried-debt lane.
  - Hero-number doc-alignment waivers (8 rows, BENCH-01-fed) — **expire 2026-08-15**;
    already re-measured by P115, waivers themselves still need to be lifted (check
    `115-UNWAIVE-PATH.md` if not already done — flagged as a re-ground item, not
    independently re-verified this rotation).
  - GTH-V15-78 `rebase-recovery-reconciles.sh` ~42k-char over-budget tier — same
    2026-08-08 umbrella.
- **No open REOPEN state.** P122 is CLOSED GREEN with no outstanding gate failures.
- **`docs-build/animation-renders`:** still `NOT-VERIFIED`, `blast_radius: P2`,
  intentionally absent pending the §3/§5 owner gate.

## 5. Mid-execution decisions + noticed-not-filed

### LOAD-BEARING LIVENESS DOCTRINE (this session's most important finding)

- A subagent's OWN background `gh run watch` does **NOT** reliably re-invoke that
  subagent — every time a coordinator "ended its turn on the watcher," the
  notification bubbled up to L0 with "no live background children," so the
  coordinator was **NOT** self-re-invoked. Only the **top-level (L0)** gets reliable
  background-task re-invocation. Direct child-AGENT completion notifications DO
  re-invoke the parent (that path works); bare background-bash watchers do not.
- **The `phase-coordinator` agent type has NO `SendMessage` tool** (only
  Agent/Bash/Read/Grep/Glob) — so a C2 CANNOT poke a dormant C1 to wake it. The
  "dormant-C1-after-push" stall is real and already happened once (P122's C1 stalled
  post-push; the C2 had to take deterministic control of the close instead of waiting).
- **Therefore: L0 owns CI-watching and pokes coordinators via `SendMessage` on
  green.** When a C1/C2 pushes and CI is in-flight, L0 must run `gh run watch <id>
  --exit-status` (background, reliable), and on green `SendMessage` the coordinator to
  resume the close. This keeps L0 in the per-phase post-push loop — the C2 does **NOT**
  fully absorb that step; budget L0 context accordingly.
- This is almost certainly the mechanism behind the manager's earlier "~4.5h relay
  gap." Mitigations in place: the manager's external watchdog (coarse backstop) + this
  L0-owns-watch pattern (fine-grained).
- **Doctrine-update owed (GSD-gated, not yet done):** fold this corrected doctrine into
  `.planning/ORCHESTRATION.md` §3/§11 (liveness) + the `coordinator-dispatch` skill.
  This MUST go through a GSD command (`/gsd-quick`), never an out-of-band edit — flagged
  in the C2 handover as an early `/gsd-quick` candidate for the successor C2, not yet
  executed as of this handover.

### Escalation list — NEVER self-authorize (report to owner/manager and WAIT)

- **E1 launch-animation publish (GTH-V15-37):** owner-PENDING, outward publishing =
  owner-only. P117 W5 incomplete on this one sub-task only. Never tag `[OWNER]`
  without genuine owner input.
- **No self-cut release:** any git tag `v*` or crates.io publish is outward →
  escalate; do NOT cut at milestone close.
- **Milestone archive** gated on OP-9 RETROSPECTIVE distillation + the non-skippable
  9th `pre-release-real-backend` probe + report-to-L0-before-archive.
- Any user-visible breaking change; any real-backend mutation beyond the sanctioned
  targets (Confluence TokenWorld / GitHub `reubenjohn/reposix` issues / JIRA `TEST`); a
  roadmap item that no longer seems right given new info.

### Carried debt / roadmap facts (from C2 handover — do not re-derive)

- **GOOD-TO-HAVES.md bloat:** 139,235 bytes / ~7× the 20k file-size ceiling, waiver
  expires **2026-08-08**. The waiver umbrella also covers the 26,889-byte C2 handover
  + the 95,772-byte SURPRISES-INTAKE.md — fold all into ONE split/archive lane, run at
  a quiet point when NO C1 is writing intake (every C1 writes to GOOD-TO-HAVES.md at
  its own phase close). Successor C2 owns exact timing; C2 handover recommends running
  it EARLY (after P123 or P124 closes), not gambling it lands naturally by P128.
- **Open GTHs for absorption slots (P127 OP-8 / P128 OP-9):** GTH-V15-73..79 (confirmed
  present in `GOOD-TO-HAVES.md`: 73 RPX-0402 URL interpolation, 74 ADR-009 flag-surface
  lock, 75 doc-alignment over-capture, 76 exit-codes.md clap layer, 77 pre-push timing,
  78 rebase-recovery-reconciles.sh oversize, 79 P122 on-demand-cadence decision).
- **Roadmap:** top-level `.planning/ROADMAP.md` HAS concrete P123–P128 Goal/
  Requirements/SC (only wave breakdown TBD, `**Plans**: TBD` on all six); the
  milestone-local `.planning/milestones/v0.15.0-phases/ROADMAP.md` is a stale
  "PLANNING / Phase TBD" stub superseded by the top-level file (GTH-V15-27, LOW, OPEN
  — do not plan phases from the milestone-local file). **P123 = Quality-runner &
  catalog integrity hardening** (DRAIN-01/03/04/05/06/10) — SC1 `run.py` self-sources
  `.env` (closes false-green-preflight gap), SC2 `--persist` refuses silent downgrade
  without `--allow-downgrade`, SC3 concurrent `--persist` race-safety, SC4
  `structure/verifier-script-exists.sh` gate, SC5 `code/ci-green-on-main` watches a
  required-workflow LIST + t4 gate surfaces real oid-drift stderr. **This directly
  absorbs the two open HIGH SURPRISES-INTAKE rows** (`.env` self-sourcing gap,
  `--persist` silent downgrade) — they are P123's reason for being, not separate
  follow-up. The Confluence oid-drift HIGH SURPRISES row is **stale bookkeeping**
  (fixed+verified in P114, `114-VERIFICATION.md`) — mark RESOLVED during P123 or P127
  intake sweep, NOT a fix-first item; a narrower pre-ADF storage-fallback residual is
  already covered by P123 SC5's scope, no new phase needed.
- **ROADMAP progress-table fix owed:** `.planning/ROADMAP.md`'s "## Progress" table
  still shows Phase 121 as "0/1 Not started" and Phase 122 stale, despite both CLOSED
  GREEN (phase-index checkboxes above the table DO show `[x]` for both — only the
  progress table itself is stale). Fold `roadmap.update-plan-progress 121` + `122`
  into the P123 C1's first grounding/planning-touch step.

### Manager deltas from this session (all resolved)

- Hermetic-runner flake debt row confirmed present in v0.15.0 GOOD-TO-HAVES (nothing
  new filed this rotation — pre-existing). PR #77 merged (manager,
  `2026-07-17T14:14:32Z`, confirmed via `gh pr view 77`). The manager's earlier
  ~4.5h relay gap → mitigated by the external watchdog (coarse backstop) + the
  L0-owns-watch doctrine above (fine-grained).
- Relief gauge: manager set ~18% soft / ~22% hard (fresh-seat baseline ~6%); note model
  quality also degrades past ~150k tokens regardless of budget — this seat (#61)
  relieved at that boundary for that reason, at a clean phase boundary
  (P122 CLOSED / P123 not yet dispatched).

**Noticed-not-filed:** none new from this L0 seat beyond what's already ledgered above
and in the C2 handover (GTH-V15-27, GTH-V15-37, GTH-V15-73..79, the ROADMAP
progress-table staleness, the stale oid-drift SURPRISES row) — this rotation's L0 work
was routing (dispatch P121's C1, dispatch the C2 for P122, relay the C2's liveness-
doctrine correction push) rather than direct code-touching, so no independent noticing
surface was generated at this seat.

## 6. Precise next steps (successor seat #62 runbook)

**PRIORITY 1 — confirm in-flight CI, then dispatch successor C2. Do this before
anything else.**

1. **CI gate (FIRST action).** Re-check `gh run view 29639213587 --json
   status,conclusion,headSha` — confirm `headSha` still matches `95bc7c5f` (no one
   force-pushed over it) and read `status`/`conclusion`. If `in_progress`, watch it
   YOURSELF (you are L0 — per §5's liveness doctrine, only L0 gets reliable
   background-task re-invocation): `gh run watch 29639213587 --exit-status --interval
   20` in a `run_in_background` Bash call, then read the captured log for the actual
   watch result (the wrapper Bash call's own exit code is NOT the run result). This is
   a docs-only push (handover files) — lower regression risk than a code push, but the
   rule is unconditional: do not open P123 over a red/in-flight main.
2. **Re-read STATE.md + `git log` fresh** before dispatching anything — the standard
   startup re-verify pattern. This handover's numbers were live-verified at write time
   but may have drifted if another session pushed since.
3. **On CI green — dispatch a FRESH opus `phase-coordinator` C2** for the v0.15.0
   remainder (P123→P128 + milestone-close), pointed at
   `.planning/milestones/v0.15.0-phases/C2-MILESTONE-HANDOVER.md` @ `95bc7c5f` as its
   required first read (that file is the authoritative next-steps doc for the C2 lane
   — it has the full P123–P128 phase-list ground truth, binding constraints, the OD-3
   ownership charter, the C2 operating doctrine, and its own numbered runbook; do not
   re-derive any of that here). Pull the `coordinator-dispatch` skill for the exact
   charter shape before dispatching.
4. **Own the CI-watch loop for every C1/C2 push this seat routes** (§5 liveness
   doctrine) — background `gh run watch`, `SendMessage` the coordinator on green. Do
   not assume a dispatched C1/C2 will self-resume after backgrounding its own watch.
5. **Fold the doctrine-update-owed item into an early GSD quick** (§5) — either
   dispatch it yourself via `/gsd-quick` early in this rotation, or explicitly hand it
   to the successor C2 as an owned early action item. Do not let it rot un-actioned
   across another full rotation.
6. **HOLD the E1 animation owner-gate** (§3/§5). Never self-authorize, never tag
   `[OWNER]` without genuine owner input. `animation-renders` staying NOT-VERIFIED is a
   pending gate, not an owner-accepted deferral.
7. Carry §5's carried-debt items forward unchanged unless drained in an OP-8/OP-9
   absorption slot (P127/P128) — none are urgent enough to interrupt the P123 dispatch.
8. **REPLACE this handover** (not append) at your own relief, re-verifying every claim
   live before carrying it forward — an uncommitted handover didn't happen.
