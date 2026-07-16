# SESSION-HANDOVER.md — v0.15.0 Floor: P115 CHECKPOINTED GREEN at the human gate,
P116 rulings encoded, execution unblocked — 2026-07-16

Written by **workhorse #44** (L0 orchestrator), relieving to successor **#45**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#43→#44's handover,
commit `f5652b2`, superseded here). #44 relieves at a clean wave boundary: the P115
verifier ran and returned **GREEN-CHECKPOINT** (phase goal achieved, only a human-only
step remains open), the P116 ADR-010 decision packet was rescued from `/tmp` and both
manager rulings were encoded verbatim, and a cold-reader pass on the two hero surfaces
completed clean. Relief is triggered by this checkpoint being a deliberate, complete
stopping point, not by a specific token count.

**Read order:** this file → §1 ground truth (verify live FIRST, including the unstaged
working-tree oddity flagged below) → §2 wave/cycle state → §4 litmus/gate state (the
human gate — still open, do not close it on inference) → §5 mid-execution decisions +
noticed-not-filed → §6 runbook (starts with the standard verify block, then the human
gate re-check, then GTH-V15-35, then P116 execution).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager, pane w1:p7). No tag push by any coordinator. No
git surgery (reset/rebase/reorder/amend) on `main`. Shared tree with the manager —
TARGETED staging only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf
isolation in `/tmp` same-Bash-invocation. opus complex / sonnet default / haiku
mechanical, never fable at a leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**.
If #45 runs on fable at top level, delegate per fable-top-level doctrine — **fable
coordinators only**, explicit model overrides at leaves (opus complex / sonnet default /
haiku mechanical), **NEVER fable at a leaf**.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain --untracked-files=all && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --limit 5
```

**Verified live by #44 as of ~2026-07-16 17:20 UTC (immediately before writing this
file):**

- **Local `HEAD` = `ce4d3b7`** (`docs(115): phase-close verification — GREEN-CHECKPOINT`).
  **`origin/main` = `2f96e69`** (manager's rotation #10→#11 handover refresh). Local is
  **1 commit ahead** of origin at the moment this handover is written; this handover
  commit stacks on `ce4d3b7` and #44 pushes both together immediately after writing this
  file — do not treat `ce4d3b7` as landed on `origin/main` until the push is re-verified.
- **Per-commit one-liners since the last known-clean pushed sha (`2f96e69`):**
  - `da41d7d` — rescued the P116 ADR-010 decision packet out of `/tmp` and committed it
    (`P116-ADR-010-DECISION-PACKET.md`) before it could be lost to a crash; filed the
    owner's mp4-export path as an addendum on `GTH-V15-37`.
  - `8212373` — encoded BOTH P116 ADR-010 manager rulings verbatim as `[MANAGER]` entries
    in `CONSULT-DECISIONS.md`; fixed a dead link at `README.md:133`; filed 2 new MEDIUM
    `SURPRISES-INTAKE.md` rows; filed `GTH-V15-38` (Option C pull-forward trigger); added
    two addenda onto the existing `GTH-V15-35` row.
  - `ce4d3b7` — `115-VERIFICATION.md`: the phase-close verifier's GREEN-CHECKPOINT verdict
    (7/7 goal-backward truths, catalog 17 PASS / 0 FAIL / 2 pre-existing unrelated WAIVED
    / 2 NOT-VERIFIED-on-weekly-cadence).
  - (unpushed at write time; #44 pushes `ce4d3b7` + this handover commit together right
    after this file lands, per the "commit BEFORE push" ordering §3 requires.)
- **CI: green on every checked sha.** `da41d7d`'s push CI run (`29517341234`, per #44's
  direct check during that push) concluded success; `2f96e69`'s CI run
  (`29518587955`, re-confirmed live by this writer via `gh run list --branch main
  --limit 5`) shows `completed success 5m58s`, alongside a green downstream `Docs`
  workflow_run (`29519010482`) and a green `release-plz` run on the same push. Post-push
  P0 probe (`code/ci-green-on-main`) PASSED after `da41d7d`; **#44 has NOT yet re-run it
  after `ce4d3b7` / this handover's push** — that is the first item in #45's runbook (§6).
- **Human gate independently re-verified this turn:**
  `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` →
  **`11`** — unchanged, all 11 rows still open. This is the sole remaining action on
  P115 (§4).
- **DEVIATION — flag before touching the tree further:** `git status` shows ONE unstaged
  working-tree modification, **`docs/benchmarks/token-economy.md`**, that this writer did
  NOT make and does not attribute to any commit in the log above. The diff **re-adds**
  the exact "## What retired the old 89.1% / 85.5% figures" section that `5a5dd29`
  (owner ruling, 2026-07-16, "strip retirement-history narrative from user-facing docs")
  explicitly and deliberately removed 12 lines earlier this same day. In other words: the
  working tree right now silently contains an uncommitted reversion of a shipped owner
  ruling. **Do not stage or commit it** (targeted-staging discipline already excludes it
  from this handover's own commit). Do not `git checkout --` it either without
  understanding provenance first — it could be (a) a stray artifact from an unrelated
  tool/editor action, (b) an in-progress manual edit by the manager or owner reverting
  their own ruling, or (c) leftover scratch from a diagnostic. **#45: investigate
  provenance (check for any other session/pane touching this file) before deciding
  discard vs. escalate** — see §5 and §6 item 2.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Waves 1–5 / T1–T6 | Benchmark ratification → latency re-measure → live-MCP capture → headline reframe → un-waive prep (all items) | DONE + PUSHED + CI GREEN (compressed; full list in #40–#43's handovers / `git log`) | — |
| Post-T6 / pre-close | Retirement-narrative strip + FINAL 11-row consolidated confirm-retire batch + 3 new owner directives (`GTH-V15-35/36/37`) | SHIPPED, pushed, CI green | `5a5dd29`, `a1f2494`, `484ca52`, `187809f` |
| Post-close prep | P116 ADR-010 decision packet rescued from `/tmp` + committed; mp4 path filed on `GTH-V15-37` | **SHIPPED**, pushed, CI green | `da41d7d` |
| Post-close prep | Both P116 manager rulings (ADR-01 mirror fan-out; FIX-03 slug→id) encoded verbatim; README dead-link fix; 2 intake rows; `GTH-V15-38`; `GTH-V15-35` addenda | **SHIPPED**, pushed, CI green | `8212373` |
| P115 phase-close | Cold-reader pass on `docs/index.md` + `README.md` | **DONE** — hero numbers all clean, no retired/stale figures found on either surface | (folded into `8212373`'s README fix + verified clean by the verifier) |
| P115 phase-close | Verifier dispatch (catalog-row PASS grading) | **DONE — GREEN-CHECKPOINT.** 7/7 goal-backward truths; catalog 17 PASS / 0 FAIL / 2 pre-existing unrelated WAIVED / 2 NOT-VERIFIED (weekly cadence, pre-existing, not a P115 regression) | `ce4d3b7` (`115-VERIFICATION.md`) — **NOT YET PUSHED** as of this writing, pushed by #44 immediately after this handover lands |
| P115 phase-close | Human-only confirm-retire gate (11 rows, `WAIVED-RETIRE_PROPOSED`) | **OPEN, re-verified live this turn (11/11 still `RETIRE_PROPOSED`).** Sole remaining action; the phase is CHECKPOINTED at this gate, not held open idle. `STATE.md` cursor stays put until it lands. | — |
| P115 phase-close | `.planning/STATE.md` cursor advance past P115 | **NOT DONE — deliberately deferred** until the human batch lands (checkpoint semantics, not a gap) | — |
| P116 | ADR-010 rulings (ADR-01 mirror fan-out; FIX-03 slug→id) | **RULED** by the manager (decide-and-disclose, owner veto window open); encoded verbatim | `8212373` |
| P116 | Execution (doc-truth rewrites, ADR-010 §2/§3 amendments, litmus-non-idempotency intake-row retirement) | **NOT STARTED** — sequenced after the P115 checkpoint per the ruling; this is #45's primary work item | — |

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at
a leaf** (and if #45 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE above); relieve past ~100k own-context (hard 150k, absolute not %) at a wave
boundary; push at green, then confirm `code/ci-green-on-main` P0 AFTER push (Bash timeout
≥300s — pre-push wall time has crept across multiple corroborating datapoints this
milestone, a SIXTH datapoint at 124s landed this session, well above the ~55–60s
documented budget; re-baseline is FILED, not yet APPLIED); never open the next phase over
a red main; reset-gating RETIRED — never defer or schedule work for a weekly reset, only
react to a cap that actually hits (if it hits: commit+push, refresh this handover +
`PROGRESS.md`, end cleanly).

## 4. Litmus / gate / REOPEN state

- **11 rows at `WAIVED-RETIRE_PROPOSED`** remain the ONLY open human-only gate —
  re-verified live in THIS turn (not inherited): `grep -c '"last_verdict":
  "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` → **11**. Authoritative row-ID
  list + copy-paste `confirm-retire --row-id <ID>` commands live in
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md` §"FINAL
  consolidated confirm-retire batch."
- Verb confirmed human-only via `--help`: `reposix-quality doc-alignment confirm-retire
  --row-id <ROW_ID>` from a real TTY. Refuses `$CLAUDE_AGENT_CONTEXT`. `--i-am-human` is
  an audited escape hatch for humans, NOT agents — agents must never pass it.
- **P115 verifier verdict: GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) — the
  phase goal (live MCP benchmark re-measurement + honest headline re-anchoring) is
  achieved in the codebase; the checkpoint semantics mean the phase is NOT held open
  idle-waiting on the human step, per the manager's standing instruction.
- **CI green on `main`'s tip, re-verified live this turn** (`2f96e69`, run `29518587955`,
  `completed success`). `da41d7d`'s push CI (`29517341234`) also confirmed green per
  #44's direct check at push time. No REOPEN state pending on either.
- **`ce4d3b7` itself is NOT YET pushed / NOT YET probed** as of this handover's write
  time — first act of #45's runbook (§6 item 1).
- **File-size soft-ceiling waiver `GTH-V15-21`** — unchanged, still masking the
  OVER-BUDGET tier as `--warn-only` until **2026-08-08**
  (`quality/catalogs/freshness-invariants.json:666`). Ledger-split decision it depends on
  (`SURPRISES-INTAKE.md`, `GOOD-TO-HAVES.md`, top-level `ROADMAP.md` all over the 20k
  soft ceiling) still needs an owner call before the lapse date.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **P116 fully RULED, execution now unblocked** — both manager rulings are encoded
   verbatim as `[MANAGER]` entries dated 2026-07-16 in `CONSULT-DECISIONS.md` (`8212373`):
   Decision 1 (ADR-01 mirror fan-out) = Option B with A folded in — doc-truth rewrite of
   the conflated "mirror" docs (`dvcs-topology.md`, root `CLAUDE.md`, the part-02 row's
   false "`sync --reconcile` heals the external mirror" claim), webhook + 30-min-cron
   BLESSED as authoritative external-mirror convergence,
   `scripts/refresh-tokenworld-mirror.sh` = manual op-recovery only, Option D REJECTED
   (keep the `files_touched>0` gate), Option C not sanctioned (filed `GTH-V15-38` with
   verbatim pull-forward trigger). Also per the ruling: retire the litmus-non-idempotency
   intake row DURING P116 execution. Decision 2 (FIX-03 slug→id) = Option A this
   milestone (design-only); Option B (durable slug→id map alongside `oid_map`) recorded
   as SANCTIONED TARGET DESIGN in the ADR-010 §3 amendment; §3 waiver stays, now
   qualified; explicitly NO v0.15 build; propose a dedicated design+build phase at the
   next milestone boundary; Option D = incident-only stopgap. **This is real, un-executed
   work — #45's primary work item after the checkpoint housekeeping** (§6 item 4).
2. **`GTH-V15-35` now carries TWO addenda** (both filed `8212373`, neither yet executed):
   (a) surface the sim-bootstrap lines outside the collapsed source-build block in
   `docs/index.md`, (b) the cold-reader-found stale Phase-36 claim at `docs/index.md:93`
   MUST be fixed in the SAME `/gsd-quick` — and line 93 is a two-claim line hash-bound by
   catalog row `docs/index/soft-threshold-24ms`, so the rebind must happen in the same
   wave as the edit (eager-fix leaf must fail closed on this, not silently skip the
   rebind).
3. **NEW — unattributed working-tree reversion of a shipped owner ruling** (§1 deviation):
   `docs/benchmarks/token-economy.md` has an unstaged, uncommitted diff re-adding the
   exact section `5a5dd29` deliberately removed earlier today. Not filed yet — this
   writer could not establish provenance (no commit, no session artifact explaining it)
   in the time available. **#45: this needs active triage, not silent discard** — check
   whether the manager or owner is mid-edit in a parallel pane before assuming it's stray
   and safe to `git checkout --` away.
4. **Two pre-existing docs-repro/benchmark-claim rows NOT-VERIFIED** (verifier noticing,
   `115-VERIFICATION.md`): `benchmark-claim-{8ms-cached-read,89.1-percent-token-reduction}`
   (minted P106) point at claim text P115 moved/retired — dangling citations, not a P115
   regression, P2. **Not yet filed to `SURPRISES-INTAKE.md` — worth a row if not already
   present**, carried forward from the prior handover, still unfiled.
5. **#43's intermittent Read/Edit harness-failure noticing** — still unreproduced, still
   unfiled. File only on a live repro; do not fabricate one.
6. **Pre-push wall-time creep — SIXTH corroborating datapoint** landed this session (124s,
   flagged by the hook itself), joining the five prior (91.7s/94–95s/98–99s/128s/141s),
   all above the ~55–60s documented budget. Re-baseline is FILED (an existing
   `SURPRISES-INTAKE.md` entry proposes ~75s), still not APPLIED — apply during the OP-8
   drain phase, not mid-phase.
7. **Filed this session, already durable, do not re-file:** two MEDIUM
   `SURPRISES-INTAKE.md` rows — `README.md:109-110` project-status 4-versions stale
   (natural home is P117), and the user-global `doc-clarity-review` skill silently
   attaching NO files via `claude -p f1 f2` (leaf-confirmed by a diagnostic; the P115
   cold-reader pass used a manual isolated fallback instead).

## 6. Precise next steps (successor #45 runbook)

1. **Standard first-act verify block.** `git rev-parse HEAD`, `git status --porcelain`,
   `git rev-list --left-right --count HEAD...origin/main` (expect `0/0` once this
   handover's push lands — confirm both `ce4d3b7` and this handover commit are on
   `origin/main`), `gh run list --branch main --limit 5` (confirm the tip's CI run
   concluded success; Bash timeout ≥300s). Then `python3 quality/runners/run.py
   --cadence post-push --persist` for the fresh `code/ci-green-on-main` P0 check — this
   has NOT yet been run against `ce4d3b7` / this handover's push (§4). Flaky `test` job →
   re-run ONCE; still red → STOP, escalate, never proceed over a red main.
2. **Triage the `token-economy.md` working-tree deviation (§1, §5 item 3) BEFORE any
   further tree writes.** Check whether a parallel session/pane (the manager at w1:p7,
   or the owner directly) is mid-edit; if genuinely stray/unattributed and confirmed safe,
   discard with `git checkout -- docs/benchmarks/token-economy.md` and note the discard
   in the next progress refresh. Do not silently commit it — it directly contradicts a
   shipped, ruled owner decision (`5a5dd29`).
3. **Human gate re-check.** `git fetch origin && grep -c '"last_verdict":
   "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` — `11` means still open (do
   nothing further on the phase-close ritual, the checkpoint stands); if it reads lower
   or `0`, the batch landed: advance `.planning/STATE.md`'s cursor past P115 and close the
   checkpoint (note the closure in the next `PROGRESS.md` refresh).
4. **`GTH-V15-35` `/gsd-quick`** — execute the install-section nesting eager-fix with BOTH
   addenda folded in (§5 item 2): surface sim-bootstrap lines outside the collapsed
   source-build block, fix the stale Phase-36 claim at `docs/index.md:93`, and rebind
   `docs/index/soft-threshold-24ms` in the SAME wave (two-claim line, rebind is
   mandatory, not optional).
5. **P116 execution through GSD**, sequenced after the checkpoint housekeeping above, per
   both rulings (§5 item 1): doc-truth rewrites (`dvcs-topology.md`, root `CLAUDE.md`, the
   part-02 row), webhook/cron-authoritative framing, ADR-010 §2/§3 amendments (record
   Option B as sanctioned target design for FIX-03, qualify the §3 waiver), and retire
   the litmus-non-idempotency intake row. This is the primary substantive work item for
   #45 — enter via a tracked GSD phase, not ad hoc edits (root `CLAUDE.md` § GSD
   workflow: never edit planning artifacts outside a GSD-tracked phase or quick).
6. **Every push Bash timeout ≥300s** — the 120s default kills `git push` mid pre-push
   hook (§3, §5 item 6).
7. **Refresh `PROGRESS.md`'s `## NOW` section at every boundary push** — do not let it go
   stale relative to the checkpoint/close state.
8. **REPLACE this handover** (not append) at #45's own relief, following this same §3
   template, with live-verified ground truth — do not carry forward any claim in this
   file without re-checking it live first.
