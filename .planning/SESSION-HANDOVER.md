# SESSION-HANDOVER.md — v0.15.0 Floor: manager-priority hero-number refresh lanes
SHIPPED (pushed, CI-green); P116 planning is the ONE remaining heavy pass — 2026-07-16

Written by **workhorse #48** (L0 orchestrator), relieving to successor **#49**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#47→#48's handover,
commit `b4044ad`, superseded here). #48 relieves at a **clean, fully-committed, CI-green
wave boundary** — NOT mid-workflow — having spent its full context budget on ONE of the
two heavy passes #47 queued (the manager-priority refresh lanes), corroborating #46/#47's
doctrine that a rotation fits roughly ONE ~60k-cost top-level skill pass, not two.

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state →
§3 binding constraints (unchanged, carry verbatim) → §4 litmus/gate/REOPEN state (P115
human gate — still open at 11, do NOT close on inference) → §5 mid-execution decisions +
noticed-not-filed → §6 runbook (verify block → human-gate re-check → P116 planning, the
ONE remaining heavy pass → P116 execution after).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a
leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**.
If #49 runs on fable at top level, delegate per fable-top-level doctrine — **fable
coordinators only**, explicit model overrides at leaves (opus complex / sonnet default /
haiku mechanical), **NEVER fable at a leaf**.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain --untracked-files=all && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --limit 8 --json databaseId,headSha,conclusion,name,status
```

**Verified live by #48 at relief (this is the state as this handover commits):**

- **HEAD at the time this handover was authored = `e185e6e`**
  (`docs(planning): mark hero-number intake RESOLVED + file concepts four-axis coverage
  gap`) — the substantive tip of #48's refresh-lane work. This handover's own commit (and
  the small bookkeeping edits to `CONSULT-DECISIONS.md` / `SURPRISES-INTAKE.md` /
  `PROGRESS.md` bundled with it — see the commit message for the exact SHA) sits directly
  ON TOP of `e185e6e`. **`e185e6e` itself is ALREADY PUSHED and CI-GREEN, verified live by
  #48** (see below) — only THIS handover's own commit needs its push+CI confirmed by #49's
  first act; there is no substantive backlog to land.
- **The chain of commits since the last handover's tip (`b4044ad`), all landed by #48:**
  - `a679d03` — DP-3 inversion ledger entry (`[SELF]`, recorded BEFORE implementing, per
    the escalation-valve bar — mint via `bind`, not the manager-named backfill fallback).
  - `c35f993` — bind `docs/index/hero-token-economy-94-75` (`docs/index.md:17`).
  - `7553c36` — bind `README/hero-token-economy-94-75` (`README.md:27`).
  - `aa75e96` — bind `docs/concepts/reposix-vs-mcp-and-sdks/token-economy-output-cost`
    (`docs/concepts/reposix-vs-mcp-and-sdks.md:29-31`, deliberately narrowed to the
    output+cost axes only).
  - `e185e6e` — mark the manager's `SURPRISES-INTAKE.md` intake row RESOLVED + file the
    concepts four-axis coverage gap as a new LOW row.
  - (This handover's own commit, on top — SHA reported at the end of this rotation's
    report; ALSO touches `.planning/CONSULT-DECISIONS.md` [deletes the now-closed DP-3
    ledger entry] and `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` [files a
    new LOW row for the `bind` `::fn` help/validator discrepancy] and `.planning/PROGRESS.md`
    [`## NOW` refresh].)
- **CI: GREEN on `e185e6e`, re-verified live by #48** via `gh run list --branch main
  --limit 6 --json databaseId,headSha,conclusion,name,status`: `Docs` **success**, `CI`
  **success**, `release-plz` **success**, `Push on main` (CodeQL) **success** — all four
  `headSha = e185e6e44f0f3056ef7e8f473dfd52ea0082f86b`, all `status: completed`, polled to
  conclusion (not inferred; the prior tip `b4044ad` was also confirmed all-green
  beforehand). **#49's first act only needs to confirm the NEW tip (this handover's own
  commit) lands its push + CI** — the substantive mint work's green is already nailed
  down.
- **Human gate re-verified live by #48** at both session start and end: `grep -c
  '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` → **`11`**,
  unchanged throughout the rotation. P115 stays CHECKPOINTED at the human-only
  confirm-retire gate; `STATE.md`'s cursor is deliberately NOT advanced. **Do not close
  P115 on inference — only a real drop in this count closes it.** Row-ID list +
  copy-paste commands: `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md`
  §"FINAL consolidated confirm-retire batch". **The owner NOW HAS the 11 confirm-retire
  commands in hand and may run them at ANY moment — re-check this count at EVERY
  boundary; when it drops below 11, advance `STATE.md` past P115 and close the
  checkpoint per §6 step 2.**
- **No deviation this turn** — clean tree at every commit boundary, no stray edits, no
  unattributed reversions. Pre-push on the `b4044ad..e185e6e` push: 61 PASS / 1 WAIVED
  (the expected `GTH-V15-21` file-size warn-only waiver), secret-scan clean.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Waves 1–5 / T1–T6 | Benchmark ratification → latency re-measure → live-MCP capture → headline reframe → un-waive prep | DONE + PUSHED + CI GREEN (compressed; full list in prior handovers / `git log` / `PROGRESS.md` SHIPPED) | — |
| P115 phase-close | Verifier dispatch (catalog-row PASS grading) | **DONE — GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) | `ce4d3b7` |
| P115 phase-close | Human-only confirm-retire gate (11 rows, `WAIVED-RETIRE_PROPOSED`) | **OPEN — re-verified live by #48 (11/11), twice this rotation.** Owner has the commands in hand; may land any moment. Sole remaining P115 action. | — |
| P115 phase-close | `.planning/STATE.md` cursor advance past P115 | **NOT DONE — deliberately deferred** until the human batch lands (checkpoint semantics) | — |
| **Refresh lanes** | Mint doc-alignment bindings for the 3 uncatalogued hero-number surfaces | **DONE + PUSHED + CI GREEN.** 3 rows BOUND (`c35f993`/`7553c36`/`aa75e96`); manager intake marked RESOLVED (`e185e6e`); `walk` exit 0. | `c35f993`, `7553c36`, `aa75e96`, `e185e6e` |
| P116 | ADR-010 rulings + GSD entry + `116-CONTEXT.md` authored | **DONE — locked contract** (`31ac414`); do not re-run discuss-phase, do not rewrite CONTEXT | `8212373`, `31ac414` |
| **P116 planning** | Research → Nyquist `VALIDATION.md` → pattern mapper `PATTERNS.md` → plan(s) → plan checker → coverage gates | **NOT STARTED — the ONE remaining heavy top-level pass, #49's primary work.** Resumes at the research step (§6 step 3). | — |
| P116 | Execution (doc-truth rewrites, ADR-010 §2/§3 amendments, litmus-non-idempotency intake retirement) | **NOT STARTED** — strictly after planning; ROADMAP marks `Execution mode: top-level` (top-level coordinator IS the executor, never `/gsd-execute-phase`) | — |

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at
a leaf** (and if #49 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE above); relieve past ~100k own-context (hard 150k, absolute not %) at a wave
boundary; push at green, then confirm `code/ci-green-on-main` P0 AFTER push (**Bash
timeout ≥300s** — pre-push wall time has crept well above the documented ~55–60s budget,
multiple corroborating datapoints; re-baseline is FILED not APPLIED, apply at OP-8 drain,
not mid-phase); never open the next phase over a red main; reset-gating RETIRED — react
only to a cap that actually hits (if it hits: commit+push, refresh this handover +
`PROGRESS.md`, end cleanly). **LIVENESS (manager standing note):** bounded backstop
≤20min on EVERY child wait; health-check self-paused children ≤30min.

## 4. Litmus / gate / REOPEN state

- **11 rows at `WAIVED-RETIRE_PROPOSED`** — the ONLY open human-only gate; re-verified
  live by #48 = **11** (checked twice this rotation, start and end — unchanged). Owner HAS
  the commands in hand — re-check at every boundary. Authoritative row-ID list + copy-paste
  `confirm-retire --row-id <ID>` commands: `115-UNWAIVE-PATH.md` §"FINAL consolidated
  confirm-retire batch."
- Verb is human-only: `reposix-quality doc-alignment confirm-retire --row-id <ROW_ID>`
  from a real TTY. Refuses `$CLAUDE_AGENT_CONTEXT`. `--i-am-human` is an audited escape
  hatch for HUMANS, NOT agents — agents must never pass it.
- **P115 verifier verdict: GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) —
  checkpoint semantics: phase is NOT held open idle-waiting on the human step.
- **CI GREEN on `e185e6e`**, re-verified live this turn (all four workflows success). No
  REOPEN state pending.
- **Refresh lanes gate: GREEN.** `headline-numbers-cross-check.py` exit 0;
  `bash quality/gates/docs-alignment/walk.sh` (the sanctioned wrapper, NOT raw
  `doc-alignment walk`) → `WALK_EXIT=0`, zero BLOCK on the 3 newly-minted rows.
- **`GTH-V15-35`** — DONE (unchanged).
- **File-size soft-ceiling waiver `GTH-V15-21`** — still masking the OVER-BUDGET tier as
  `--warn-only` until **2026-08-08** (`quality/catalogs/freshness-invariants.json:666`).
  #48 saw it fire (warn-only) on the `SURPRISES-INTAKE.md` intake commits — working as
  designed; the file is now larger still after this handover's task-2 append. Ledger-split
  decision it depends on still needs an owner call before lapse.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **CLOSED/ABSORBED — P116 fully RULED, encoded as the locked contract in
   `116-CONTEXT.md` (`31ac414`).** Both manager rulings (ADR-01 mirror fan-out = Option
   B+A folded in; FIX-03 slug→id = Option A this milestone, design-only). **Treat
   `116-CONTEXT.md` as the authoritative, locked contract** — do not re-run
   `discuss-phase`, do not rewrite CONTEXT to relitigate. Original provenance: `8212373`.
2. **Carried, still live for the planner — ROADMAP P116 criterion 1** says the packet
   exists "alongside `docs/decisions/010-l2-l3-cache-coherence.md`" but it physically
   lives in the P115 phase dir
   (`.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`).
   `116-CONTEXT.md` already scopes closing this gap (its lines 18–20 / 82–83, planner's
   discretion). **The planner MUST cover it or the verifier will flag it** — don't let it
   silently drop from the plan.
3. **Carried — `auto_advance: true`** in the P116 init-query config means a bare
   `/gsd-plan-phase` run auto-chains into `/gsd-execute-phase` at workflow step 15. #49
   **MUST NOT** take that chain — ROADMAP marks P116 `Execution mode: top-level` (the
   top-level coordinator IS the executor). Actively check for and clear
   `workflow._auto_chain_active` if the resumed workflow set it.
4. **Context-budget datapoint, now with a THIRD corroboration.** #46 first proposed
   "enter heavy skills well under ~50k own-context, or split across reliefs"; #47
   corroborated by relieving at ~88k rather than gamble a blind heavy-skill entry with TWO
   ~60k passes queued; **#48 now corroborates a third time**: entering the refresh-lane
   pass consumed #48's full rotation budget end-to-end (empirical DP-3 discovery +
   escalation-valve write-up + dispatching the mint executor + verifying + filing the
   coverage-gap intake + this handover), leaving no room to also start P116 planning this
   rotation. **Practical consequence for #49: budget for roughly ONE heavy top-level pass
   per rotation — P116 planning is that ONE pass this time (§6 step 3).** Whether this
   warrants a `GOOD-TO-HAVES.md` doctrine row on GSD/quality-skill context budgeting is
   still an open call (now three datapoints deep).
5. **NEW (#48) — FILED, do NOT re-file.** The concepts-page hero row
   (`docs/concepts/reposix-vs-mcp-and-sdks/token-economy-output-cost`, `aa75e96`) was
   deliberately narrowed to the two test-pinned axes (output 94.3% / cost 74.9%); the
   page's stated four-axis claim also asserts cache-creation (66.0%) and input-context
   (55.6%) reductions that NO test currently pins to the real committed captures (only a
   synthetic-seed test proves the reduction FORMULA is applied per-axis, not the real
   values). Filed by the mint executor as a LOW `SURPRISES-INTAKE.md` row at `e185e6e`
   (sketched resolution: extend `TOKEN_CLAIMS`/`parse_token_canonical` in
   `headline-numbers-cross-check.py`, or add a `bench_token_economy.py` band-assertion for
   the two extra axes, then widen the bound row via refresh). Correct-but-unwatched, same
   class as the parent row, one layer finer.
6. **NEW (#48) — FILED by this handover, do NOT re-file.** `reposix-quality doc-alignment
   bind --help` advertises `--test <file>::<fn>` generically, but the validator
   (`parse_test` in `crates/reposix-quality/src/commands/doc_alignment.rs`) only resolves
   the `::fn` split into a hashable function when the file ends in `.rs`; for `.py` (and
   any non-`.rs` extension) the WHOLE `<file>::<fn>` string is checked as a literal file
   path and rejected ("test file `…py::run_cross_check` does not exist" — confirmed live).
   Every existing Python-bound row (including the 3 minted this rotation) already stores
   the BARE path, matching the form the validator actually accepts — LOW severity, a
   workaround/established pattern exists, not a live blocker. Filed as a new LOW
   `SURPRISES-INTAKE.md` row by this handover (task 2); sketched fix = reword the
   `--help` string to state `::fn` is Rust-only (cheaper) or teach the validator to hash a
   named Python function (larger).
7. **Carried, unfiled, LOW** — `docs/index.md` carries a near-duplicate bootstrap
   sequence (the `<details>` copy is doc-alignment-bound; the visible-prose copy added by
   `260716-fmt` is NOT) — two copies to keep in sync by hand. Natural home: P117/P119
   under `GTH-V15-36`. Decide whether it warrants a `GOOD-TO-HAVES.md` row.
8. **Carried, still unfiled** — two dangling docs-repro/benchmark rows
   (`benchmark-claim-8ms-cached-read`, `benchmark-claim-89.1-percent-token-reduction`,
   minted P106) point at claim text P115 moved/retired — P2, worth a
   `SURPRISES-INTAKE.md` row if not already present (verify against the file before
   filing — it's grown large, now ~60KB).
9. **Carried, unreproduced** — intermittent Read/Edit harness-failure noticing. File only
   on live repro; do not fabricate one.
10. **Carried — `GTH-V15-21`** file-size waiver masks the OVER-BUDGET tier as `--warn-only`
    until 2026-08-08; ledger-split decision needs an owner call before lapse. (The
    `SURPRISES-INTAKE.md` file is itself now larger after this rotation's two new rows —
    the drain-phase split scope, already noted in the file's own 2026-07-14 entry.)
11. **Already FILED and durable — do NOT re-file:** the MEDIUM `SURPRISES-INTAKE.md` row
    on `test_main_offline_regenerates_doc_from_captures` missing a byte-compare against
    the committed doc (the `260716-f6o` gap class), filed during `260716-fmt`.

## 6. Precise next steps (successor #49 runbook)

1. **Standard first-act verify block.** Run: `git rev-parse HEAD` (expect this handover
   commit on top of `e185e6e`), `git status --porcelain` (expect clean), `git rev-list
   --left-right --count HEAD...origin/main`. If this handover commit is not yet pushed,
   `git push origin main` (**Bash timeout ≥300s**), then `gh run list --branch main
   --limit 5` (timeout ≥300s) — confirm the NEW tip's CI concluded `success`. (#48 already
   confirmed `e185e6e`'s push + CI green live; only this handover commit's own push/CI
   remains to confirm.) Flaky `test` → re-run ONCE; still red → STOP, escalate, never
   proceed over a red main.
2. **Human gate re-check (do this at EVERY boundary — owner has the commands in hand).**
   `git fetch origin && grep -c '"last_verdict": "RETIRE_PROPOSED"'
   quality/catalogs/doc-alignment.json` — `11` = still open (do nothing further on the
   P115 close ritual). If it reads lower/`0`, the batch landed: advance
   `.planning/STATE.md`'s cursor past P115, close the checkpoint, note it in the next
   `PROGRESS.md` refresh.
3. **P116 planning — the ONE remaining heavy top-level pass, your primary work this
   rotation.** Re-enter `/gsd-plan-phase 116`. The workflow finds `116-CONTEXT.md` on
   disk — **do NOT re-run `discuss-phase`, do NOT rewrite CONTEXT** (§5 item 1). Continue
   from where it left off: research (sonnet; if prompted research-vs-skip, choose
   research) → Nyquist `VALIDATION.md` → pattern mapper `PATTERNS.md` → planner (opus) →
   plan checker (sonnet) → coverage gates → `state.planned-phase` → ROADMAP annotation →
   commit. Planner MUST address the criterion-1 "packet lives alongside the ADR" gap (§5
   item 2). **Do NOT auto-chain into execution at workflow step 15** (§5 item 3) — clear
   `workflow._auto_chain_active` if set.
4. **P116 execution AFTER planning completes**, per the ROADMAP `Execution mode:
   top-level` marker: the top-level coordinator dispatches leaves per the completed
   PLAN.md's waves (opus complex / sonnet default / haiku mechanical, **NEVER fable at a
   leaf**); phase-close push cadence applies (push `origin main` BEFORE verifier dispatch,
   then `quality/runners/run.py --cadence post-push --persist`; `code/ci-green-on-main` is
   P0).
5. **Every push Bash timeout ≥300s** — pre-push wall time has crept (well above the
   ~55–60s documented budget); re-baseline is FILED not APPLIED — apply at OP-8 drain.
6. **Refresh `PROGRESS.md`'s `## NOW` at every boundary push** — do not let it go stale.
7. **REPLACE this handover** (not append) at #49's own relief, following this same
   ORCHESTRATION.md §3 template, with live-verified ground truth — re-check every claim
   live before carrying it forward.
