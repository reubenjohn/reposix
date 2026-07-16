# SESSION-HANDOVER.md — v0.15.0 Floor: P115 still CHECKPOINTED at the human gate,
two quicks shipped green, relief at a clean wave boundary — 2026-07-16

Written by **workhorse #45** (L0 orchestrator), relieving to successor **#46**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#44→#45's handover,
commit `3b78292`, superseded here). #45 relieves at a clean wave boundary: both
checkpoint-housekeeping quicks (`260716-f6o`, `260716-fmt`) shipped green, the P115 human
gate was re-checked twice (start + end, unchanged), and #45's own context has crossed the
~100k soft-relief line with the next queued item (P116 execution) being a full GSD phase
that must not be started this deep into a session.

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state →
§3 binding constraints (unchanged, carry verbatim) → §4 litmus/gate/REOPEN state (the
human gate — still open, do not close it on inference) → §5 mid-execution decisions +
noticed-not-filed → §6 runbook (starts with the standard verify block, then the human
gate re-check, then P116 execution).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a
leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**.
If #46 runs on fable at top level, delegate per fable-top-level doctrine — **fable
coordinators only**, explicit model overrides at leaves (opus complex / sonnet default /
haiku mechanical), **NEVER fable at a leaf**.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain --untracked-files=all && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --limit 5
```

**Verified live by #45 immediately before writing this file:**

- **`HEAD` = `2398b34`** (`docs(planning): record quick task 260716-fmt in STATE.md Quick
  Tasks Completed`). `git status --porcelain` returned **empty** (clean tree — no
  unstaged/untracked deviation this turn, unlike the prior handover's flagged
  `token-economy.md` reversion, which is now fully resolved, see §5 item 1).
  `git rev-list --left-right --count HEAD...origin/main` → **`0	0`** — local and
  `origin/main` are identical.
- **Per-commit one-liners since the last handover's tip (`3b78292`):**
  - `19f9ae2` — quick `260716-f6o`: stripped the retired "## What retired the old 89.1% /
    85.5% figures" section from the token-economy GENERATOR template
    (`bench_token_economy_captures.py::render_token_economy_markdown`); offline regen now
    byte-for-byte matches committed HEAD (sha256 `5620699b...364fcf`, re-verified live
    this turn — matches).
  - `ac9e717` — records `260716-f6o` in `STATE.md`'s Quick Tasks Completed table.
  - `97fad0d` — quick `260716-fmt` / `GTH-V15-35` (both addenda): nested "Build from
    source (advanced)" under "30-second install"; surfaced bootstrap lines in visible
    prose; split + destaled the two-claim `docs/index.md:93` line; mechanically rebound
    all 11 shifted doc-alignment rows; filed one MEDIUM SURPRISES-INTAKE row.
  - `2398b34` — records `260716-fmt` in `STATE.md`'s Quick Tasks Completed table
    (backfills the pushed commit hash into the quick's SUMMARY.md).
- **CI: green on the tip, re-verified live this turn** via `gh run list --branch main
  --limit 10 --json databaseId,headSha,conclusion,name,event`:
  - `97fad0d` — `CI` run `29524678534` **success**; `release-plz` `29524678602` success;
    `Push on main` (CodeQL) `29524676622` success; `Docs` `29525065700` success.
  - `2398b34` (current tip) — `CI` run `29525256773` **success**; `release-plz`
    `29525256844` success; `Push on main` (CodeQL) `29525256287` success; `Docs`
    `29525635632` success.
  No REOPEN state pending on either sha.
- **Human gate re-verified live, TWICE this session** (start-of-session and again
  immediately before writing this handover): `grep -c '"last_verdict":
  "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` → **`11`**, unchanged both
  times. P115 stays CHECKPOINTED at the human-only confirm-retire gate; `STATE.md`'s
  cursor is deliberately NOT advanced. **Do not close P115 on inference — only a real
  drop in this count closes it.** Row-ID list + copy-paste commands:
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md`
  §"FINAL consolidated confirm-retire batch".
- **No open deviation this turn** — the prior handover's flagged
  `docs/benchmarks/token-economy.md` unattributed working-tree reversion is fully
  RESOLVED (see §5 item 1); `git status --porcelain` confirms a clean tree.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Waves 1–5 / T1–T6 | Benchmark ratification → latency re-measure → live-MCP capture → headline reframe → un-waive prep (all items) | DONE + PUSHED + CI GREEN (compressed; full list in #40–#44's handovers / `git log`) | — |
| Post-T6 / pre-close | Retirement-narrative strip + FINAL 11-row consolidated confirm-retire batch + 3 new owner directives (`GTH-V15-35/36/37`) | SHIPPED, pushed, CI green | `5a5dd29`, `a1f2494`, `484ca52`, `187809f` |
| Post-close prep | P116 ADR-010 decision packet rescued + committed; both manager rulings encoded verbatim; README dead-link fix; 2 intake rows; `GTH-V15-38`; `GTH-V15-35` addenda | SHIPPED, pushed, CI green | `da41d7d`, `8212373` |
| P115 phase-close | Verifier dispatch (catalog-row PASS grading) | **DONE — GREEN-CHECKPOINT** (unchanged since #44; `115-VERIFICATION.md`, `ce4d3b7`) | `ce4d3b7` |
| P115 phase-close | Human-only confirm-retire gate (11 rows, `WAIVED-RETIRE_PROPOSED`) | **OPEN — re-verified live TWICE this session (11/11 both times).** Sole remaining action; checkpoint stands, not held open idle. | — |
| P115 phase-close | `.planning/STATE.md` cursor advance past P115 | **NOT DONE — deliberately deferred** until the human batch lands (checkpoint semantics, not a gap) | — |
| Checkpoint housekeeping | Quick `260716-f6o` — fix-it-twice (owner ruling `5a5dd29`): strip retired-narrative section from the perf-gate GENERATOR, not just the doc | **SHIPPED**, pushed, CI green; provenance established by the manager (accidental regression vector, not deliberate); §1/§5-item-3 deviation from #44's handover fully RESOLVED | `19f9ae2`, `ac9e717` |
| Checkpoint housekeeping | Quick `260716-fmt` — `GTH-V15-35` docs/index.md install-IA fix, BOTH addenda folded in | **SHIPPED**, pushed, CI green; `GTH-V15-35` STATUS → **DONE**; all 11 shifted doc-alignment rows mechanically rebound (walk exit 0) | `97fad0d`, `2398b34` |
| P116 | ADR-010 rulings (ADR-01 mirror fan-out; FIX-03 slug→id) | **RULED** by the manager (decide-and-disclose, owner veto window open); encoded verbatim | `8212373` (unchanged since #44) |
| P116 | Execution (doc-truth rewrites, ADR-010 §2/§3 amendments, litmus-non-idempotency intake-row retirement) | **NOT STARTED** — sequenced after checkpoint housekeeping, which is now complete; this is #46's primary work item | — |

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at
a leaf** (and if #46 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE above); relieve past ~100k own-context (hard 150k, absolute not %) at a wave
boundary; push at green, then confirm `code/ci-green-on-main` P0 AFTER push (Bash timeout
≥300s — pre-push wall time has crept across multiple corroborating datapoints this
milestone, well above the ~55–60s documented budget; re-baseline is FILED, not yet
APPLIED — apply at OP-8 drain, not mid-phase); never open the next phase over a red main;
reset-gating RETIRED — never defer or schedule work for a weekly reset, only react to a
cap that actually hits (if it hits: commit+push, refresh this handover + `PROGRESS.md`,
end cleanly).

## 4. Litmus / gate / REOPEN state

- **11 rows at `WAIVED-RETIRE_PROPOSED`** remain the ONLY open human-only gate —
  re-verified live TWICE this session (not inherited, not stale): `grep -c
  '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` → **11** both
  times. Authoritative row-ID list + copy-paste `confirm-retire --row-id <ID>` commands
  live in `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md`
  §"FINAL consolidated confirm-retire batch."
- Verb confirmed human-only via `--help`: `reposix-quality doc-alignment confirm-retire
  --row-id <ROW_ID>` from a real TTY. Refuses `$CLAUDE_AGENT_CONTEXT`. `--i-am-human` is
  an audited escape hatch for humans, NOT agents — agents must never pass it.
- **P115 verifier verdict: GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`,
  unchanged since #44) — the phase goal (live MCP benchmark re-measurement + honest
  headline re-anchoring) is achieved in the codebase; checkpoint semantics mean the phase
  is NOT held open idle-waiting on the human step.
- **CI green on `main`'s tip, re-verified live this turn** (`2398b34`, run
  `29525256773`, `completed success`; `97fad0d`'s own push CI `29524678534` also
  confirmed `completed success`). No REOPEN state pending.
- **`GTH-V15-35` flipped DONE this session** (quick `260716-fmt`) — no longer an open
  item; both addenda executed together as the row required.
- **File-size soft-ceiling waiver `GTH-V15-21`** — unchanged, still masking the
  OVER-BUDGET tier as `--warn-only` until **2026-08-08**
  (`quality/catalogs/freshness-invariants.json:666`, re-verified live this turn). Ledger-
  split decision it depends on still needs an owner call before the lapse date.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **RESOLVED — the `token-economy.md` unattributed working-tree reversion flagged by
   #44 is now fully closed out.** Manager established provenance: the P115 phase-close
   gate-run regen re-added the retired-narrative section because the GENERATOR template
   (`bench_token_economy_captures.py::render_token_economy_markdown`) still templated it
   — an accidental regression vector, NOT a deliberate override of owner ruling
   `5a5dd29`. Quick `260716-f6o` stripped the section from the generator; the stray
   working-tree diff was discarded (never committed); offline regen re-verified
   byte-for-byte identical to the committed doc (sha256 `5620699b...364fcf`, re-checked
   live this turn — matches, `git status` clean). Doc-alignment rows unaffected (doc
   bytes unchanged, catalog untouched). Nothing further to do on this item.
2. **P116 fully RULED, execution still unblocked and now #46's PRIMARY work item** —
   both manager rulings are encoded verbatim as `[MANAGER]` entries dated 2026-07-16 in
   `CONSULT-DECISIONS.md` (`8212373`, unchanged since #44):
   Decision 1 (ADR-01 mirror fan-out) = Option B with A folded in — doc-truth rewrite of
   the conflated "mirror" docs (`docs/concepts/dvcs-topology.md`, root `CLAUDE.md`, the
   part-02 row's false "`sync --reconcile` heals the external mirror" claim); webhook +
   30-min-cron BLESSED as authoritative external-mirror convergence;
   `scripts/refresh-tokenworld-mirror.sh` = manual op-recovery only; Option D REJECTED
   (keep the `files_touched>0` gate); Option C not sanctioned (`GTH-V15-38` holds the
   verbatim pull-forward trigger); retire the litmus-non-idempotency intake row DURING
   P116 execution. Decision 2 (FIX-03 slug→id) = Option A this milestone (design-only);
   Option B recorded as SANCTIONED TARGET DESIGN in the ADR-010 §3 amendment; §3 waiver
   stays, qualified; explicitly NO v0.15 build; propose a dedicated design+build phase at
   the next milestone boundary; Option D = incident-only stopgap. Decision packet:
   `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`.
3. **NEW (#45, `GTH-V15-35` executor noticing, not filed as its own row) — `docs/index.md`
   now carries a near-duplicate bootstrap sequence.** The copy inside the collapsed
   "Build from source (advanced)" `<details>` block (`reposix sim &` / `reposix init
   sim::demo` / `cd /tmp/reposix-demo && git checkout -B main ...`) is doc-alignment-bound
   (line-anchored catalog rows); the new visible-prose copy added under "After — one
   commit" per the addendum is NOT bound — two copies to keep in sync by hand going
   forward. LOW severity; natural home is P117/P119 under the `GTH-V15-36`
   furnished-product mandate. Not filed as its own `GOOD-TO-HAVES.md` row this session —
   #46 or a later phase should decide whether it warrants one.
4. **Carried from #44, still unfiled** — two dangling docs-repro/benchmark-claim rows
   (`benchmark-claim-8ms-cached-read`, `benchmark-claim-89.1-percent-token-reduction`,
   minted P106) point at claim text P115 moved/retired — P2, worth a `SURPRISES-INTAKE.md`
   row if not already present.
5. **Carried from #43** — intermittent Read/Edit harness-failure noticing, still
   unreproduced. File only on live repro; do not fabricate one.
6. **Carried from #44, unchanged** — `GTH-V15-21` file-size waiver masks the OVER-BUDGET
   tier as `--warn-only` until 2026-08-08
   (`quality/catalogs/freshness-invariants.json:666`); ledger-split decision needs an
   owner call before lapse.
7. **Filed this session, already durable, do not re-file:** one MEDIUM
   `SURPRISES-INTAKE.md` row (`test_main_offline_regenerates_doc_from_captures` in the
   token-economy perf-gate test suite never byte-compares regen output against the real
   committed doc — the exact gap class behind the `260716-f6o` regression), filed at
   `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` during the `260716-fmt`
   quick.

## 6. Precise next steps (successor #46 runbook)

1. **Standard first-act verify block.** `git rev-parse HEAD`, `git status --porcelain`,
   `git rev-list --left-right --count HEAD...origin/main` (expect `0/0`),
   `gh run list --branch main --limit 5` (confirm the tip's CI run concluded success;
   Bash timeout ≥300s). Flaky `test` job → re-run ONCE; still red → STOP, escalate, never
   proceed over a red main.
2. **Human gate re-check.** `git fetch origin && grep -c '"last_verdict":
   "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` — `11` means still open (do
   nothing further on the phase-close ritual, the checkpoint stands); if it reads lower
   or `0`, the batch landed: advance `.planning/STATE.md`'s cursor past P115, close the
   checkpoint, and note the closure in the next `PROGRESS.md` refresh.
3. **PRIMARY WORK ITEM — P116 execution through GSD**, per both manager rulings encoded
   verbatim in `.planning/CONSULT-DECISIONS.md` (dated 2026-07-16, commit `8212373`):
   Decision 1 (ADR-01 mirror fan-out) = Option B with A folded in — doc-truth rewrite of
   the conflated "mirror" docs (`docs/concepts/dvcs-topology.md`, root `CLAUDE.md`, the
   part-02 row's false "`sync --reconcile` heals the external mirror" claim); webhook +
   30-min-cron BLESSED as authoritative external-mirror convergence;
   `scripts/refresh-tokenworld-mirror.sh` = manual op-recovery only; Option D REJECTED
   (keep `files_touched>0` gate); Option C not sanctioned (`GTH-V15-38` holds the
   verbatim pull-forward trigger); retire the litmus-non-idempotency intake row DURING
   P116 execution. Decision 2 (FIX-03 slug→id) = Option A this milestone (design-only);
   Option B recorded as SANCTIONED TARGET DESIGN in the ADR-010 §3 amendment; §3 waiver
   stays, qualified; NO v0.15 build; propose dedicated design+build phase at next
   milestone boundary; Option D = incident-only stopgap. Enter via tracked GSD
   (`/gsd-plan-phase 116` then `/gsd-execute-phase 116`, or per ROADMAP execution-mode
   marker) — never ad hoc edits. Decision packet:
   `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`.
4. **Every push Bash timeout ≥300s** — pre-push wall time has crept (multiple
   corroborating datapoints across sessions, well above the ~55–60s documented budget);
   re-baseline is FILED not APPLIED — apply at OP-8 drain, not mid-phase.
5. **Refresh `PROGRESS.md`'s `## NOW` section at every boundary push** — do not let it go
   stale relative to the checkpoint/close state.
6. **REPLACE this handover** (not append) at #46's own relief, following this same §3
   template, with live-verified ground truth — do not carry forward any claim in this
   file without re-checking it live first.
