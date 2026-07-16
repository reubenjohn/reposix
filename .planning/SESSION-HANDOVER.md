# SESSION-HANDOVER.md — v0.15.0 Floor: P116 EXECUTION BLOCKED on GitHub Actions 503
outage (tip CI env-fail, NOT code-red) — 2026-07-16

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the § 1
verify block yourself first.**

Written by **workhorse #52** (L0 orchestrator), relieving to successor **#53**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#51→#52's
handover, commit `79200b7`, superseded here). #52's entire rotation was consumed by the
CI-certification duty #51 handed off, plus bounded outage-recovery polling — the
GitHub Actions REST API outage first observed by #51 (~22:15 UTC) was STILL ONGOING
throughout #52's rotation. #52 made **zero tree-writes** before this handover: it
diagnosed the tip's CI failure down to root cause (environmental, not a code defect —
see §1/§4) and left P116 execution fully pre-digested and ready for #53 to run the
moment GitHub recovers. **No code was touched. No plan was executed. This is a clean,
zero-deviation relief at a wave boundary, forced by an external outage, not a stall.**

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state
→ §3 binding constraints (unchanged, carry verbatim) → §4 litmus/gate/REOPEN state (CI
diagnosis is the load-bearing new content here — read before touching CI status) → §5
mid-execution decisions + noticed-not-filed → §6 runbook (verify → CI check via
check-suites → human-gate re-check → P116 EXECUTION, the milestone's next primary
work).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a
leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**.
If #53 runs on fable at top level, delegate per fable-top-level doctrine — **fable
coordinators only**, explicit model overrides at leaves (opus complex / sonnet default /
haiku mechanical), **NEVER fable at a leaf**.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && \
  git status --porcelain --untracked-files=all && \
  git rev-list --left-right --count origin/main...HEAD && \
  grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json
```

**Verified live by #52 immediately before writing this handover:**

> **⚠️ POST-PUSH CORRECTION (#52, 2026-07-16 — added AFTER `4bb0596` landed on top of this
> handover's own commit, confirmed live. The bullets immediately below were written BEFORE
> the owner's confirm-retire batch and are SUPERSEDED by this block — every "11 / gate
> OPEN / re-check at every boundary" statement in §1/§2/§4 below is now STALE; do NOT act
> on those, the gate is closed):**
> - **P115 HUMAN GATE CLOSED.** The owner's `confirm-retire` batch landed post-handover-
>   write: all 11 rows flipped `RETIRE_PROPOSED` → `RETIRE_CONFIRMED`. Live-verified:
>   `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` →
>   **`0`** (was `11`); `RETIRE_CONFIRMED` count → **`68`**. #52 committed the batch as
>   **`4bb0596`** (`chore(quality): land owner confirm-retire batch — 11 rows retired,
>   P115 human gate closed`), owner-authorized + manager-verified.
> - **#53's FIRST runbook action (before P116 execution):** advance
>   `.planning/STATE.md`'s cursor past P115 and CLOSE the P115 GREEN-CHECKPOINT — the
>   confirm-retire gate was the only thing holding it open (per §6 runbook + checkpoint
>   semantics below). This is real GSD phase-close state work, deliberately left to #53
>   by the manager.
> - **CI note:** `4bb0596` + `4c069ba` CI certification remains DEFERRED on the ongoing
>   GitHub Actions API 503 outage (still down as of this correction). Read status via
>   the check-suites fallback documented in §1 CI DIAGNOSIS / §4; the tip's real-github +
>   release-plz failures remain ENVIRONMENTAL, not a code red.

- `git rev-parse HEAD` → **`16a71134e90ab7b1ce86408be4cade69e6b8b936`**
- `git rev-parse origin/main` → **`16a71134e90ab7b1ce86408be4cade69e6b8b936`** — same
  sha. **This handover's own commit (with the PROGRESS.md refresh) will land ON TOP of
  `16a7113` as the new tip; once pushed, HEAD == origin/main == THIS commit, `0  0`
  ahead/behind, tree clean.** State this explicitly to #53: by the time you read this
  file, the tip you're standing on IS #52's handover commit, not `16a7113`.
- `git status --porcelain --untracked-files=all` → **empty (clean tree)** before this
  write.
- `git rev-list --left-right --count origin/main...HEAD` → **`0  0`** — HEAD and
  origin/main are identical; #51's push (`cbd1ff0..79200b7`) plus #51's own
  post-push-correction commit (`79200b7..16a7113`) are BOTH already on `origin/main`.
  #52 made no further commits until this handover.
- `grep -c RETIRE_PROPOSED` → **`11`**, unchanged.
- Tip chain since the last CI-certified-green base `cbd1ff0`:
  - `011096b` `74fb907` `9dbb860` — #51's P116 planning-tail commits (planner PASS,
    `GTH-V15-40` filing, plan-checker PASS). Docs/planning-only.
  - `79200b7` — #51's own relief handover (#51→#52).
  - `16a7113` — #51's post-push correction commit (CI UNKNOWN-not-red note, certifying
    duty for #52). Docs/planning-only.
  - **This commit** — #52's relief handover (#52→#53) + PROGRESS.md refresh.

### CI DIAGNOSIS (LOAD-BEARING — read before touching CI status)

**The tip's CI is NOT green, but this is ENVIRONMENTAL (the ongoing GitHub Actions API
outage), NOT a code red. DO NOT fix-first — there is NO code defect to fix.** Evidence
#52 verified live this rotation:

- Tip `16a7113`'s CI = two failing GitHub-Actions check-suites: (1) `release-plz`
  (whole suite failed), (2) `ci.yml` — in which **13 of 15 jobs are GREEN**
  (`gitleaks`, `test`, `quality gates (pre-pr)`, `runner unit tests (hermetic)`,
  `shell-coverage`, `clippy`, `rustfmt`, `integration (contract, real confluence)`,
  `integration (contract, real jira v09)`, `dark-factory regression (sim)`,
  `integration (contract, real confluence v09)`, `bench-latency-v09`, `coverage`) and
  the ONLY two failures are `integration (contract, real github)` and `integration
  (contract, real github v09)`.
- `16a7113` and `79200b7` (and `011096b`/`74fb907`/`9dbb860` before them) are ALL
  **docs/planning-only** — `git diff --stat cbd1ff0 16a7113 -- crates/ .github/` →
  **0 lines changed**. A docs-only diff cannot regress GitHub-integration code.
- The real-github test code is **byte-identical** to the last CI-certified-green base
  `cbd1ff0`, where #52 live-verified BOTH real-github check-suites concluded
  `success` (4/4 GitHub-Actions check-suites green on `cbd1ff0`, via the check-suites
  API).
- All NON-github real-backend jobs on the tip (confluence, jira v09, confluence v09,
  sim dark-factory) concluded `success`. Only the GitHub-API-dependent jobs failed —
  consistent with "only GitHub's own API is degraded, not the code under test."
  `release-plz` also calls the GitHub API for its own operation → same root cause.
- **Remediation (once GitHub recovers), NOT a code fix:** either (a)
  `gh run rerun <ci.yml run id> --failed` on the tip + re-run release-plz, OR (b) just
  execute P116 (also docs-only) → push → the fresh CI run on the new tip will be green
  once the outage has cleared → certify via the post-push cadence. **Prefer (b)** —
  P116 execution is the next work anyway, and a fresh push gives a clean certification
  in one motion rather than two.
- **CRITICAL TOOL TIP for reading CI during this outage:** `gh run list` / `gh run
  view` / `commits/<sha>/check-runs` / `commits/<sha>/status` ALL return `503` (raw
  HTML "Unicorn" error page) while the outage persists. BUT the **check-SUITES API
  stays up**. Use:
  ```
  gh api repos/reubenjohn/reposix/commits/<sha>/check-suites \
    --jq '.check_suites[]|select(.app.name=="GitHub Actions")|{id,conclusion}'
  ```
  then drill into any suite that shows `failure`:
  ```
  gh api repos/reubenjohn/reposix/check-suites/<id>/check-runs \
    --jq '.check_runs[]|"\(.conclusion)\t\(.name)"'
  ```
  This is exactly how #52 read status this whole rotation, and produced the per-job
  breakdown above. **(Noticing for §5: a resilient `reposix doctor`/quality CI-status
  probe that falls back to the check-suites API when the runs API 503s is a genuine
  GOOD-TO-HAVES tooling candidate — #53 or a later rotation may file it; #52 did not
  file it this rotation to preserve context for the runbook.)**
- **CRITICAL FOR #53: your OWN relief-handover commit (this one) becomes the new tip.
  Its CI will show the SAME environmental real-github + release-plz failures while the
  outage persists — do NOT mistake that for a genuine regression introduced by this
  handover commit.** This commit touches only `.planning/SESSION-HANDOVER.md` and
  `.planning/PROGRESS.md` — it cannot possibly break GitHub integration tests. #53's
  first act: re-read status via the check-suites path above; if GitHub has recovered,
  re-run/re-push to get clean CI before proceeding to P116 phase-close.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Waves 1–5 / T1–T6 | Benchmark ratification → latency re-measure → live-MCP capture → headline reframe → un-waive prep | DONE + PUSHED + CI GREEN (compressed; full list in prior handovers / `git log` / `PROGRESS.md` SHIPPED) | — |
| P115 phase-close | Verifier dispatch (catalog-row PASS grading) | **DONE — GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) | `ce4d3b7` |
| P115 phase-close | Human-only confirm-retire gate (11 rows, `WAIVED-RETIRE_PROPOSED`) | **OPEN — re-verified live by #52 at rotation start, 11/11, unchanged.** Sole remaining P115 action. | — |
| P115 phase-close | `.planning/STATE.md` cursor advance past P115 | **NOT DONE — deliberately deferred** until the human batch lands (checkpoint semantics) | — |
| P116 planning (research/validation/patterns/planner/checker) | ADR-010 rulings, `116-CONTEXT.md`, research, patterns, 3 wave-1 plans, plan-checker VERDICT PASS | **DONE + COMMITTED + PUSHED** (`31ac414`, `05085fe`, `08e94a4`, `011096b`, `74fb907`, `9dbb860`, `79200b7`) — see prior handovers for full breakdown | — |
| P116 EXECUTION | 3 wave-1 parallel plans: 116-01 (mirror-convergence "authoritative" blessing + doc-alignment guard), 116-02 (ADR-010 §2/§3 amendments), 116-03 (LIVE ledger retirement + GOOD-TO-HAVES-09) | **NOT STARTED. Fully READY, pre-digested for #53 (see §6) — the block is the GitHub Actions outage, NOT missing planning.** `Execution mode: top-level` — top-level coordinator IS the executor, never `/gsd-execute-phase`. | — |
| #52 rotation | CI diagnosis of the outage-caused tip failure; bounded recovery polling; zero tree-writes until this handover | **DONE this rotation** — see §1 CI DIAGNOSIS. No named-incident post-mortem needed (external outage, not an agent error). | this commit |

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate owner); no tag push by any coordinator; no git
surgery (reset/rebase/amend/reorder) on main; leaf isolation in `/tmp` same-Bash-invocation;
opus complex / sonnet default / haiku mechanical, **never fable at a leaf** (session model
is Fable 5 — if #53 runs fable at top level, delegate fable-coordinators-only, explicit
model overrides at leaves); relieve past ~100k own-context (hard 150k, absolute not %) at
a wave boundary; **every push Bash timeout ≥300s**; refresh `PROGRESS.md`'s `## NOW` at
every boundary push; never open the next phase over a red main. **FIX-03 execution is
DESIGN-ONLY — NO `crates/` edit** (Plan 116-02/116-03's phase-close gate should assert
`git diff --stat -- crates/ | wc -l` == 0 before declaring done). **LIVENESS (manager
standing note):** bounded backstop ≤20min on EVERY child wait; health-check self-paused
children ≤30min.

## 4. Litmus / gate / REOPEN state

- **11 rows at `WAIVED-RETIRE_PROPOSED`** — the ONLY open human-only gate; re-verified
  live by #52 at rotation start = **11**, unchanged. Owner HAS the commands in hand —
  re-check at every boundary. Authoritative row-ID list + copy-paste
  `confirm-retire --row-id <ID>` commands: `115-UNWAIVE-PATH.md` §"FINAL consolidated
  confirm-retire batch."
- Verb is human-only: `reposix-quality doc-alignment confirm-retire --row-id <ROW_ID>`
  from a real TTY. Refuses `$CLAUDE_AGENT_CONTEXT`. `--i-am-human` is an audited escape
  hatch for HUMANS, NOT agents — agents must never pass it.
- **P115 verifier verdict: GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) —
  checkpoint semantics: phase is NOT held open idle-waiting on the human step.
- **P116 planner→checker gate: VERDICT PASS** (`9dbb860`) — CLOSED for the planning
  tail. The next gate is P116's own phase-close (push → post-push cadence → verifier
  dispatch → catalog-row PASS grading) after execution lands.
- **CI on tip `16a7113` = environmental fail, NOT a code red.** See §1 CI DIAGNOSIS for
  the full evidence chain (13/15 `ci.yml` jobs green, only the two GitHub-API-dependent
  integration jobs + `release-plz` fail; zero code diff since the last certified-green
  base `cbd1ff0`; outage confirmed via the still-live check-suites API while the runs
  API 503s). **This state persists until GitHub Actions' API recovers — it is not
  something #53 can fix by editing code.** Do NOT treat this tip as a red main in the
  "STOP + escalate" sense; treat it as "certification blocked, pending an external
  recovery," and poll in bounded ≤10-20min legs per the LIVENESS constraint (§3).
- **`116-RESEARCH.md` is 52,340 bytes; `116-PATTERNS.md` is 22,259 bytes** — both over
  the 20KB file-size warn floor, non-blocking under `GTH-V15-21` (expires 2026-08-08).
- **File-size soft-ceiling waiver `GTH-V15-21`** — masking OVER-BUDGET as `--warn-only`
  until **2026-08-08** (RESEARCH 52KB, PATTERNS 22KB, ROADMAP ~33KB, GOOD-TO-HAVES
  ~60KB, ADR-010 138% of 20KB). Ledger-split decision still needs an owner call before
  lapse.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **CLOSED/ABSORBED — P116 fully RULED**, encoded as the locked contract in
   `116-CONTEXT.md` (`31ac414`); planning tail COMPLETE (planner PASS + checker PASS,
   `9dbb860`). Nothing further to relitigate before execution.
2. **RECURRING FALSE POSITIVE — `GTH-V15-38` copy-paste bleed.** STALE, fixed in
   `6d21cae`, lives in `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (NOT
   root). #53: do NOT chase this a fourth time if a subagent re-raises it.
3. **Carry-forward from #49/#50 §5 item 8 (still live, do NOT re-file):** concepts-page
   four-axis hero coverage gap; `bind --help ::fn` Rust-only validator discrepancy;
   `docs/index.md` near-duplicate bootstrap sequence; two dangling P106 benchmark rows
   (verify against the file before filing); the MEDIUM
   `test_main_offline_regenerates_doc_from_captures` byte-compare gap (durable, do NOT
   re-file).
4. **NEW noticing (#52, not filed this rotation — see §1 CI DIAGNOSIS tip block):** a
   resilient CI-status probe (in `reposix doctor` or a quality gate) that falls back to
   the check-suites API when the runs API 503s would have saved #52 real diagnostic
   effort and would help any future rotation caught in a similar outage. Candidate
   GOOD-TO-HAVES row for a later phase — #52 deliberately did NOT file it this rotation
   to conserve context for a complete, precise handover; #53 or a later rotation should
   file it (severity LOW-MEDIUM, tooling/resilience tag).
5. **CONTEXT-BUDGET datapoint — SEVENTH corroboration.** #52 spent its whole rotation
   on CI-certification diagnosis + bounded outage polling and relieved at a clean wave
   boundary WITHOUT executing P116 — the cause was the **outage** (an external block),
   NOT excessive context reads; #52's own digest-and-carry-forward discipline (this
   handover reuses #51's already-pre-digested §2/§6 P116 content nearly verbatim rather
   than re-reading the 3 plan files) kept the rotation light. Whether the seven-plus
   context-budget datapoints across #46–#52 warrant a standing `GOOD-TO-HAVES.md`
   doctrine row on GSD/quality-skill context budgeting remains an OPEN call — #53 may
   be the rotation to close it if a slot opens.

## 6. Precise next steps (successor #53 runbook)

1. **Standard first-act verify block (§1).** Run it yourself; confirm HEAD == this
   handover's own commit == origin/main, tree clean, `0  0`.
2. **Read CI status via the check-suites path (§1 CI DIAGNOSIS)** — the runs API
   (`gh run list`/`gh run view`) may still be 503ing; the check-suites API stays up.
   - If GitHub has recovered AND the tip shows a genuinely NEW class of failure (not
     just `release-plz` + the two real-github integration jobs) → treat as a real red,
     STOP, escalate per normal doctrine — do not assume "it's just the outage" blindly.
   - If the tip shows ONLY `release-plz` + `integration (contract, real github[, v09])`
     failing AND GitHub is still degraded → that is the documented environmental state
     from this handover, NOT a code red. Poll in bounded ≤10-20min legs (LIVENESS, §3)
     rather than looping tightly.
   - If GitHub has recovered and those jobs are now green (or a re-run turns them
     green) → the tip is certified; proceed to step 3.
3. **Human gate re-check (do this at EVERY boundary).**
   `git fetch origin && grep -c '"last_verdict": "RETIRE_PROPOSED"'
   quality/catalogs/doc-alignment.json` — `11` = still open, do nothing further on the
   P115 close ritual. If lower, the batch landed: advance `.planning/STATE.md`'s cursor
   past P115, close the checkpoint, note it in the next PROGRESS.md refresh.
4. **P116 EXECUTION — the primary work, once the tip is certified (or once you accept
   that the FIRST P116 push will itself be the fresh-CI certification post-outage).**
   ROADMAP marks P116 `Execution mode: top-level` — you (the top-level coordinator) ARE
   the executor, never `/gsd-execute-phase`. Dispatch the 3 wave-1 plans
   **SEQUENTIALLY, one at a time** (despite zero file-overlap, all three share ONE git
   index and would race on `git add`/`commit` if run concurrently — §3's
   one-tree-writer-at-a-time rule holds here) — verify each plan's guard before
   dispatching the next:
   - **116-01** (tier **sonnet**): files = `quality/gates/docs-alignment/mirror-convergence-blessed.sh`,
     `quality/catalogs/doc-alignment.json`, `docs/concepts/dvcs-topology.md`,
     `CLAUDE.md`. Blesses webhook+cron as the AUTHORITATIVE external-mirror convergence
     mechanism in both live docs + mints a doc-alignment regression guard. HARD RULES:
     guard keys on the string `"authoritative"` (0 occurrences today), NEVER on
     `"webhook"` (present 3×/11× → tautological always-green trap); mint the catalog
     row via `reposix-quality doc-alignment bind` CLI ONLY (NEVER hand-edit
     `doc-alignment.json`); catalog-first (guard row is the FIRST commit; the verifier
     script MUST exit non-zero BEFORE the doc edit and exit 0 AFTER — prove it's not
     tautological); EXTEND around `CLAUDE.md:92-100` and `dvcs-topology.md:174-182`,
     don't rewrite them; if `bind` CLI unavailable → skip the row, file a
     GOOD-TO-HAVES follow-up, grep verification suffices. **BINDING ADDITION: 116-01
     mutates the SAME `doc-alignment.json` the human RETIRE gate operates on — the
     executor MUST assert `grep -c '"last_verdict": "RETIRE_PROPOSED"'
     quality/catalogs/doc-alignment.json` stays `11` after its bind (do not disturb
     the 11 pending rows).**
   - **116-02** (tier **OPUS** — nuanced ratified-decision prose spanning two
     requirements): file = `docs/decisions/010-l2-l3-cache-coherence.md` ONLY. Three
     APPEND-only dated blockquotes: (T1) §2 RBF-LR-04 lever CLOSED; (T2) §3 FIX-03
     Option B "SANCTIONED TARGET DESIGN", design-only; (T3) References cross-link
     bullet to
     `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`
     (backtick path, NOT a markdown hyperlink — `.planning/` is not mkdocs-served).
     HARD RULES: FIX-03 is DESIGN-ONLY → `git diff --stat -- crates/ | wc -l` == 0
     gate; APPEND only, never rewrite the ratified `## Decision` prose; keep terse
     (ADR already 138% of 20KB); the L238-257 "WAIVED for v0.13.0" marker stays
     present; avoid the bare word "replace" (banned in docs/ — use
     "supersedes"/"amends"); T3 closes ROADMAP criterion 1 via CROSS-LINK ONLY (no
     file move — packet stays in the P115 dir).
   - **116-03** (tier **sonnet or haiku** — mechanical status-flips): files =
     `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` +
     `.planning/GOOD-TO-HAVES.md`. (T1) Flip the LIVE litmus-non-idempotency SURPRISES
     row `**STATUS:** OPEN`→`RESOLVED` at ~L116 (`## 2026-07-14 20:42 | ... litmus
     non-idempotency ...`), cite ruling + commit `8212373` + `GTH-V15-38`; keep the
     entry intact — NEVER delete it — it becomes the LIVE ledger's FIRST
     terminal-status row (no in-file precedent; shape copied from the archived
     v0.14.0 analog). (T2) `.planning/GOOD-TO-HAVES.md` GOOD-TO-HAVES-09 (L44-78) →
     STATUS "SANCTIONED TARGET DESIGN", TAG rewritten to boundary-relative phrasing
     (e.g. "propose at the then-current milestone boundary"), NOT hardcoded v0.16.0;
     preserve body. HARD RULES: retire the **LIVE v0.15.0** row, **NOT** the archived
     v0.14.0 twin (`.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md:299-329`
     — leave untouched, incl. its false "--reconcile heals the mirror" prose); do NOT
     touch `GTH-V15-38` or its copy-paste-bled duplicate (separate hygiene defect, out
     of scope). Verify archived twin untouched: `git diff --stat --
     .planning/milestones/v0.14.0-phases/ | wc -l` == 0.
5. **Phase-close cadence once all 3 land.** Push `origin main` BEFORE dispatching the
   verifier subagent; then run `python3 quality/runners/run.py --cadence post-push
   --persist` (`code/ci-green-on-main` is P0 — asserts main's NEWEST run concluded
   success; this REQUIRES GitHub to have recovered, else it correctly fails on the
   still-ongoing outage — do not treat that as a new code defect). Then dispatch
   `gsd-verifier` for catalog-row PASS grading. Never open the next phase over a red
   main.
6. **Every push Bash timeout ≥300s.**
7. **Refresh `PROGRESS.md`'s `## NOW` at every boundary push** — do not let it go
   stale.
8. **REPLACE this handover** (not append) at #53's own relief, following this same
   ORCHESTRATION.md §3 template, with live-verified ground truth — re-check every claim
   live before carrying it forward.
