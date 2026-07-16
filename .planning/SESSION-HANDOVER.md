# SESSION-HANDOVER.md — v0.15.0 Floor: manager hero-number intake filed at a clean
green boundary; two heavy passes (refresh lanes + P116 planning) queued for #48 — 2026-07-16

Written by **workhorse #47** (L0 orchestrator), relieving to successor **#48**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#46→#47's handover,
commit `69f0814`, superseded here). #47 relieves at a **clean, fully-committed, CI-green
wave boundary** — NOT mid-workflow — before starting either of the two remaining heavy
top-level passes (P116 planning; the 3 doc-alignment refresh lanes). Rationale below (§5
item 5): at ~88k own-context, entering either ~60k-cost skill would land ~150k mid-pass,
the exact anti-pattern #46 relieved to avoid; #46's item-5 doctrine says enter these
"well under ~50k own-context." So #47 completed the manager's session-start intake
directive + full verification, then relieved cleanly rather than gamble a blind
heavy-skill entry.

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state →
§3 binding constraints (unchanged, carry verbatim) → §4 litmus/gate/REOPEN state (P115
human gate — still open at 11, do NOT close on inference) → §5 mid-execution decisions +
noticed-not-filed → §6 runbook (verify block → human-gate re-check → then TWO heavy
passes sequenced by fresh budget: manager-priority refresh lanes + P116 planning).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a
leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**.
If #48 runs on fable at top level, delegate per fable-top-level doctrine — **fable
coordinators only**, explicit model overrides at leaves (opus complex / sonnet default /
haiku mechanical), **NEVER fable at a leaf**.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain --untracked-files=all && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --limit 8 --json databaseId,headSha,conclusion,name,status
```

**Verified live by #47 at relief (this is the state as this handover commits + pushes):**

- **`HEAD` = `029bde7`** (`docs(planning): file uncatalogued-hero-number surfaces intake
  (manager finding → #47)`), sitting on top of `69f0814` (#46→#47's handover commit).
  `git status --porcelain` returned **empty** — clean tree. Before this handover commit,
  `git rev-list --left-right --count HEAD...origin/main` → **`0  0`** — `029bde7` is
  **already pushed and synced**. This handover commit sits directly on top and is pushed
  with it (§6 execution). Unlike the #46→#47 handover (which left `31ac414` unpushed for
  #47 to land), **#47 leaves nothing unpushed except this handover commit itself** — #48's
  §6 step 1 confirms this handover's own push + CI, no backlog to land.
- **The one substantive commit since the last handover's tip (`69f0814`):**
  - `029bde7` — touches ONLY
    `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (appends one MEDIUM,
    HIGH-visibility intake row: the three uncatalogued hero-number surfaces — see §5 item
    1). Pre-commit hook warned the file is 54,959 chars (over the 20k ceiling) — this is
    the EXPECTED `GTH-V15-21` warn-only waiver (until 2026-08-08), not a block.
- **CI: GREEN on `029bde7`, re-verified live by #47** via `gh run list`: `CI`
  `29527730723`-successor for this sha **success**, `Docs` **success**, `release-plz`
  **success**, `Push on main` (CodeQL) **success** — all four `headSha = 029bde7`, all
  concluded (polled to completion, not inferred). Main is green on #47's tip; #48 starts
  clean. (Manager also certified `69f0814`'s CI green — run `29527730723` — before #47's
  intake commit; that green is now superseded by `029bde7`'s own green.)
- **Human gate re-verified live by #47** at session start: `grep -c '"last_verdict":
  "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` → **`11`**, unchanged. P115
  stays CHECKPOINTED at the human-only confirm-retire gate; `STATE.md`'s cursor is
  deliberately NOT advanced. **Do not close P115 on inference — only a real drop in this
  count closes it.** Row-ID list + copy-paste commands:
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md`
  §"FINAL consolidated confirm-retire batch". **MANAGER STANDING NOTE (from #47's brief):
  the owner NOW HAS the 11 confirm-retire commands in hand and may run them at ANY
  moment — re-check this count at EVERY boundary; when it drops below 11, advance
  `STATE.md` past P115 and close the checkpoint per §6 step 2.**
- **No deviation this turn** — clean tree, no stray edits, no unattributed reversions.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Waves 1–5 / T1–T6 | Benchmark ratification → latency re-measure → live-MCP capture → headline reframe → un-waive prep | DONE + PUSHED + CI GREEN (compressed; full list in prior handovers / `git log` / `PROGRESS.md` SHIPPED) | — |
| P115 phase-close | Verifier dispatch (catalog-row PASS grading) | **DONE — GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) | `ce4d3b7` |
| P115 phase-close | Human-only confirm-retire gate (11 rows, `WAIVED-RETIRE_PROPOSED`) | **OPEN — re-verified live by #47 (11/11).** Owner has the commands in hand; may land any moment. Sole remaining P115 action. | — |
| P115 phase-close | `.planning/STATE.md` cursor advance past P115 | **NOT DONE — deliberately deferred** until the human batch lands (checkpoint semantics) | — |
| Manager finding | Uncatalogued hero-number surfaces (3 docs) | **FILED to `SURPRISES-INTAKE.md`** by #47 (MEDIUM, HIGH-visibility). Remedy = refresh lanes below. | `029bde7` |
| P116 | ADR-010 rulings + GSD entry + `116-CONTEXT.md` authored | **DONE — locked contract** (`31ac414`); do not re-run discuss-phase, do not rewrite CONTEXT | `8212373`, `31ac414` |
| **Refresh lanes** | Mint doc-alignment bindings for the 3 uncatalogued hero surfaces | **NOT STARTED — MANAGER PRIORITY, deadline: not past P117.** Heavy top-level `/reposix-quality-refresh` pass (§6 step 3a). | — |
| P116 | Planning: research → Nyquist `VALIDATION.md` → pattern mapper `PATTERNS.md` → plan(s) → plan checker → coverage gates | **NOT STARTED** — resumes at the research step; heavy top-level pass (§6 step 3b) | — |
| P116 | Execution (doc-truth rewrites, ADR-010 §2/§3 amendments, litmus-non-idempotency intake retirement) | **NOT STARTED** — strictly after planning; ROADMAP marks `Execution mode: top-level` (top-level coordinator IS the executor, never `/gsd-execute-phase`) | — |

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at
a leaf** (and if #48 runs on fable at top level, delegate fable-coordinators-only per the
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
  live by #47 = **11**. Owner HAS the commands in hand (manager note) — re-check at every
  boundary. Authoritative row-ID list + copy-paste `confirm-retire --row-id <ID>`
  commands: `115-UNWAIVE-PATH.md` §"FINAL consolidated confirm-retire batch."
- Verb is human-only: `reposix-quality doc-alignment confirm-retire --row-id <ROW_ID>`
  from a real TTY. Refuses `$CLAUDE_AGENT_CONTEXT`. `--i-am-human` is an audited escape
  hatch for HUMANS, NOT agents — agents must never pass it.
- **P115 verifier verdict: GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) —
  checkpoint semantics: phase is NOT held open idle-waiting on the human step.
- **CI GREEN on `029bde7`**, re-verified live this turn (all four workflows success). No
  REOPEN state pending.
- **`GTH-V15-35`** — DONE (unchanged).
- **File-size soft-ceiling waiver `GTH-V15-21`** — still masking the OVER-BUDGET tier as
  `--warn-only` until **2026-08-08** (`quality/catalogs/freshness-invariants.json:666`).
  #47 saw it fire (warn-only) on the `SURPRISES-INTAKE.md` intake commit — working as
  designed. Ledger-split decision it depends on still needs an owner call before lapse.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **NEW (#47) — FILED at `029bde7`, do NOT re-file.** Manager-verified finding: three
   hero surfaces present the NEW ~94.3%/~74.9% four-axis token-economy figures with NO
   doc-alignment catalog binding — `docs/index.md:17` (restated L42/L156/L168; only the
   mermaid sub-numbers L31-39 are bound), `README.md:27` (the only README token-economy
   row is the RETIRING `README-md/token-89-percent`), and
   `docs/concepts/reposix-vs-mcp-and-sdks.md:29-31`. Once the 11-row retire batch lands
   these become entirely uncatalogued (no gate watches them). **Remedy = the refresh
   lanes in §6 step 3a.** Full row: `SURPRISES-INTAKE.md` (2026-07-16, discovered-by
   manager → #47).
2. **CLOSED/ABSORBED — P116 fully RULED, encoded as the locked contract in
   `116-CONTEXT.md` (`31ac414`).** Both manager rulings (ADR-01 mirror fan-out = Option
   B+A folded in; FIX-03 slug→id = Option A this milestone, design-only). **Treat
   `116-CONTEXT.md` as the authoritative, locked contract** — do not re-run
   `discuss-phase`, do not rewrite CONTEXT to relitigate. Original provenance: `8212373`.
3. **Carried from #46, still live for the planner — ROADMAP P116 criterion 1** says the
   packet exists "alongside `docs/decisions/010-l2-l3-cache-coherence.md`" but it
   physically lives in the P115 phase dir
   (`.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`).
   `116-CONTEXT.md` already scopes closing this gap (its lines 18–20 / 82–83, planner's
   discretion). **The planner MUST cover it or the verifier will flag it** — don't let it
   silently drop from the plan.
4. **Carried from #46 — `auto_advance: true`** in the P116 init-query config means a bare
   `/gsd-plan-phase` run auto-chains into `/gsd-execute-phase` at workflow step 15. #48
   **MUST NOT** take that chain — ROADMAP marks P116 `Execution mode: top-level` (the
   top-level coordinator IS the executor). Actively check for and clear
   `workflow._auto_chain_active` if the resumed workflow set it.
5. **NEW (#47) — context-budget datapoint reinforcing #46's item 5.** #47 entered the
   session, ran full first-act verification (CI + human gate), and filed the manager
   intake — reaching **~88k own-context** BEFORE any heavy skill entry (much of it
   unavoidable session-start reads: the ~240-line #46 handover, the ~405-line
   `SURPRISES-INTAKE.md`, auto-loaded CLAUDE.md's, the giant tool/agent system reminder).
   With TWO ~60k heavy passes remaining, #47 relieved rather than start one — corroborating
   #46's rule ("enter these skills well under ~50k, or split across reliefs"). **Practical
   consequence for #48: you likely have budget for at most ONE of the two heavy passes
   this rotation — pick per §6 step 3, do it, relieve at that boundary for the next.**
   Whether this warrants a `GOOD-TO-HAVES.md` doctrine row on GSD/quality-skill context
   budgeting is still an open call (carried from #46 item 5, now with a second datapoint).
6. **Carried from #46, unfiled, LOW** — `docs/index.md` carries a near-duplicate bootstrap
   sequence (the `<details>` copy is doc-alignment-bound; the visible-prose copy added by
   `260716-fmt` is NOT) — two copies to keep in sync by hand. Natural home: P117/P119
   under `GTH-V15-36`. Decide whether it warrants a `GOOD-TO-HAVES.md` row.
7. **Carried from #44 via #45/#46, still unfiled** — two dangling docs-repro/benchmark
   rows (`benchmark-claim-8ms-cached-read`,
   `benchmark-claim-89.1-percent-token-reduction`, minted P106) point at claim text P115
   moved/retired — P2, worth a `SURPRISES-INTAKE.md` row if not already present (verify
   against the file before filing — it's grown large).
8. **Carried from #43, unreproduced** — intermittent Read/Edit harness-failure noticing.
   File only on live repro; do not fabricate one.
9. **Carried — `GTH-V15-21`** file-size waiver masks the OVER-BUDGET tier as `--warn-only`
   until 2026-08-08; ledger-split decision needs an owner call before lapse. (The
   `SURPRISES-INTAKE.md` file is itself now 54,959 chars — the drain-phase split scope,
   already noted in the file's own 2026-07-14 entry's 2026-07-16 update.)
10. **Already FILED and durable — do NOT re-file:** the MEDIUM `SURPRISES-INTAKE.md` row
    on `test_main_offline_regenerates_doc_from_captures` missing a byte-compare against
    the committed doc (the `260716-f6o` gap class), filed during `260716-fmt`.

## 6. Precise next steps (successor #48 runbook)

1. **Standard first-act verify block.** Run: `git rev-parse HEAD` (expect this handover
   commit on top of `029bde7`), `git status --porcelain` (expect clean), `git rev-list
   --left-right --count HEAD...origin/main`. If this handover commit is not yet pushed,
   `git push origin main` (**Bash timeout ≥300s**), then `gh run list --branch main
   --limit 5` (timeout ≥300s) — confirm the NEW tip's CI concluded `success`. (#47 pushed
   `029bde7` + confirmed its CI green already; only this handover commit's own push/CI
   remains to confirm.) Flaky `test` → re-run ONCE; still red → STOP, escalate, never
   proceed over a red main.
2. **Human gate re-check (do this at EVERY boundary — owner has the commands in hand).**
   `git fetch origin && grep -c '"last_verdict": "RETIRE_PROPOSED"'
   quality/catalogs/doc-alignment.json` — `11` = still open (do nothing further on the
   P115 close ritual). If it reads lower/`0`, the batch landed: advance
   `.planning/STATE.md`'s cursor past P115, close the checkpoint, note it in the next
   `PROGRESS.md` refresh.
3. **TWO heavy top-level passes remain — sequence by your fresh budget (§5 item 5: likely
   only ONE fits per rotation).** Recommended order = manager-priority refresh FIRST (it
   has a hard deadline and is more bounded), then P116 planning:
   - **(3a) MANAGER PRIORITY — the 3 doc-alignment refresh lanes.** Run
     `/reposix-quality-refresh` per doc (TOP-LEVEL — the skill is top-level-only; depth-2
     fan-out is unreachable from inside another skill) for `docs/index.md` (the L17 hero
     bullet), `README.md` (L27), and `docs/concepts/reposix-vs-mcp-and-sdks.md` (L29-31).
     Bind each live figure to `bench_token_economy.py` tests / `headline-numbers-cross-check.py`
     — **the same test targets the existing `output-reduction-94-percent` rows bind to**
     (extend an established pattern; ground it by reading those rows first).
     Commit per-doc so any relief point is clean. **Manager deadline: do NOT let this slip
     past P117.** NOTE: these are UNBOUND (net-new) claims, not drifted existing rows — if
     `/reposix-quality-refresh` (built for STALE_DOCS_DRIFT re-bind) doesn't mint net-new
     rows cleanly, the backfill flow (`/reposix-quality-backfill` / the
     `reposix-quality-doc-alignment` backfill playbook) is the alternative mint path; the
     manager specified the refresh flow, so try it first.
   - **(3b) P116 planning.** Re-enter `/gsd-plan-phase 116`. The workflow finds
     `116-CONTEXT.md` on disk — **do NOT re-run `discuss-phase`, do NOT rewrite CONTEXT**
     (§5 item 2). Continue from where it left off: research (sonnet; if prompted
     research-vs-skip, choose research) → Nyquist `VALIDATION.md` → pattern mapper
     `PATTERNS.md` → planner (opus) → plan checker (sonnet) → coverage gates →
     `state.planned-phase` → ROADMAP annotation → commit. Planner MUST address the
     criterion-1 "packet lives alongside the ADR" gap (§5 item 3). **Do NOT auto-chain
     into execution at workflow step 15** (§5 item 4) — clear `workflow._auto_chain_active`
     if set.
4. **P116 execution AFTER planning completes**, per the ROADMAP `Execution mode:
   top-level` marker: the top-level coordinator dispatches leaves per the completed
   PLAN.md's waves (opus complex / sonnet default / haiku mechanical, **NEVER fable at a
   leaf**); phase-close push cadence applies (push `origin main` BEFORE verifier dispatch,
   then `quality/runners/run.py --cadence post-push --persist`; `code/ci-green-on-main` is
   P0).
5. **Every push Bash timeout ≥300s** — pre-push wall time has crept (well above the
   ~55–60s documented budget); re-baseline is FILED not APPLIED — apply at OP-8 drain.
6. **Refresh `PROGRESS.md`'s `## NOW` at every boundary push** — do not let it go stale.
7. **REPLACE this handover** (not append) at #48's own relief, following this same
   ORCHESTRATION.md §3 template, with live-verified ground truth — re-check every claim
   live before carrying it forward.
