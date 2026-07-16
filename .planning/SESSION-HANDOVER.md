# SESSION-HANDOVER.md — v0.15.0 Floor: P116 research + validation skeleton SHIPPED
(pushed, CI-green); P116 planning TAIL (pattern-mapper → planner → checker) is next — 2026-07-16

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the § 1
verify block yourself first.**

Written by **workhorse #49** (L0 orchestrator), relieving to successor **#50**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#48→#49's handover,
commit `b325caf`, superseded here). #49 relieves at a **clean, fully-committed, CI-green
wave boundary** — NOT mid-workflow — having onboarded (long #48 handover + a 25k-capped
`/gsd-plan-phase` skill read) and hit ~93k own-context BEFORE reaching the primary work,
the exact scenario #46/#47/#48 warned of. Rather than gamble blowing the 150k hard stop
mid-`/gsd-plan-phase`, #49 applied #46's "split across reliefs" doctrine: it advanced only
the ONE foundational sub-step that runs in a SUBAGENT (context not charged to L0) — the
`gsd-phase-researcher` dispatch — plus hand-authored the `116-VALIDATION.md` skeleton
needed so the successor's re-entry sails past the step-7.5 Nyquist gate, then relieved
clean. This is now a **fourth corroboration** that a rotation fits roughly ONE heavy
top-level skill pass, and the first datapoint showing ONBOARDING ALONE can consume ~90k
before primary work starts.

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state →
§3 binding constraints (unchanged, carry verbatim) → §4 litmus/gate/REOPEN state (P115
human gate — still open at 11, do NOT close on inference) → §5 mid-execution decisions +
noticed-not-filed (research corrected CONTEXT's framing — reconcile, do NOT rewrite
CONTEXT) → §6 runbook (verify block → human-gate re-check → P116 planning TAIL, the
ONE remaining heavy pass → P116 execution after).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a
leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**.
If #50 runs on fable at top level, delegate per fable-top-level doctrine — **fable
coordinators only**, explicit model overrides at leaves (opus complex / sonnet default /
haiku mechanical), **NEVER fable at a leaf**.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain --untracked-files=all && \
  git rev-list --left-right --count HEAD...origin/main && \
  git fetch origin && \
  grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json && \
  gh run list --branch main --limit 6 --json databaseId,headSha,conclusion,name,status
```

**Verified live by #49 immediately before writing this handover:**

- **HEAD = `05085fe`** (`docs(116): research + validation skeleton — ADR-010 mirror
  doc-truth + slug→id design encoding`) — **pushed to origin/main** (`git rev-list
  --left-right --count HEAD...origin/main` → `0  0`, no divergence). Working tree
  **clean** (`git status --porcelain --untracked-files=all` → empty).
- **CI GREEN on `05085fe`**, confirmed live via `gh run list`: `Docs` **success**, `CI`
  **success** (run `29534583633`, watched to conclusion, `WATCH_RC=0`), `Push on main`
  (CodeQL) **success**. `release-plz` shows `status: in_progress` / `conclusion: ""` at
  verify-time — this is a non-blocking, separate release-automation workflow (not part of
  the `code/ci-green-on-main` P0 gate contract); do not treat it as a red signal. The
  prior tip `b325caf` was ALSO independently confirmed CI-green (`Docs` + `CI` both
  `success`) — #49 verified it at rotation start before doing any phase work.
- **Chain of commits since the last handover's tip (`b325caf`), landed by #49:**
  - `05085fe` — `gsd-phase-researcher` (sonnet) output `116-RESEARCH.md` (52,340 bytes,
    HIGH confidence) + hand-authored `116-VALIDATION.md` skeleton (frontmatter filled,
    body explicitly marked planner-owned).
- **Human gate re-verified live by #49, unchanged all rotation:** `grep -c
  '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` → **`11`**.
  P115 stays CHECKPOINTED at the human-only confirm-retire gate; `STATE.md`'s cursor is
  deliberately NOT advanced. **Do not close P115 on inference — only a real drop in this
  count closes it.** Row-ID list + copy-paste commands:
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md`
  §"FINAL consolidated confirm-retire batch". **Re-check this count at EVERY boundary;
  when it drops below 11, advance `STATE.md` past P115 and close the checkpoint.**
- **No deviation this turn** — clean tree at every commit boundary, targeted staging
  only, no stray edits.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Waves 1–5 / T1–T6 | Benchmark ratification → latency re-measure → live-MCP capture → headline reframe → un-waive prep | DONE + PUSHED + CI GREEN (compressed; full list in prior handovers / `git log` / `PROGRESS.md` SHIPPED) | — |
| Refresh lanes | Mint doc-alignment bindings for the 3 uncatalogued hero-number surfaces | DONE + PUSHED + CI GREEN | `c35f993`, `7553c36`, `aa75e96`, `e185e6e` |
| P115 phase-close | Verifier dispatch (catalog-row PASS grading) | **DONE — GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) | `ce4d3b7` |
| P115 phase-close | Human-only confirm-retire gate (11 rows, `WAIVED-RETIRE_PROPOSED`) | **OPEN — re-verified live by #49 (11/11).** Owner has the commands in hand; may land any moment. Sole remaining P115 action. | — |
| P115 phase-close | `.planning/STATE.md` cursor advance past P115 | **NOT DONE — deliberately deferred** until the human batch lands (checkpoint semantics) | — |
| P116 | ADR-010 rulings + GSD entry + `116-CONTEXT.md` authored | **DONE — locked contract** (`31ac414`); do not re-run discuss-phase, do not rewrite CONTEXT | `8212373`, `31ac414` |
| P116 planning — research | `gsd-phase-researcher` dispatch → `116-RESEARCH.md` (HIGH confidence, 52KB) | **DONE + PUSHED + CI GREEN** | `05085fe` |
| P116 planning — validation skeleton | `116-VALIDATION.md` frontmatter + skeleton (so step-7.5 Nyquist gate passes on re-entry) | **DONE + PUSHED + CI GREEN** (body explicitly planner-owned, not yet populated) | `05085fe` |
| **P116 planning — TAIL** | `gsd-pattern-mapper` → `PATTERNS.md`; `gsd-planner` (opus) → `PLAN.md`(s); `gsd-plan-checker`; coverage gates; `state.planned-phase`; ROADMAP annotation | **NOT STARTED — the ONE remaining heavy top-level pass, #50's primary work.** Re-enter `/gsd-plan-phase 116` (§6 step 3). | — |
| P116 | Execution (doc-truth rewrites, ADR-010 §2/§3 amendments, litmus-non-idempotency intake retirement) | **NOT STARTED** — strictly after planning; ROADMAP marks `Execution mode: top-level` (top-level coordinator IS the executor, never `/gsd-execute-phase`) | — |

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate owner); no tag push by any coordinator; no git
surgery (reset/rebase/amend/reorder) on main; leaf isolation in `/tmp` same-Bash-invocation;
opus complex / sonnet default / haiku mechanical, **never fable at a leaf** (session model
is Fable 5 — if #50 runs fable at top level, delegate fable-coordinators-only, explicit
model overrides at leaves); relieve past ~100k own-context (hard 150k, absolute not %) at
a wave boundary; **every push Bash timeout ≥300s** — pre-push wall time was ~92s on this
rotation's push (per #49's own measurement), WELL above the documented ~55–60s budget,
corroborated again; re-baseline is FILED not APPLIED (apply at OP-8 drain, not mid-phase);
refresh `PROGRESS.md`'s `## NOW` at every boundary push; never open the next phase over a
red main. **LIVENESS (manager standing note):** bounded backstop ≤20min on EVERY child
wait; health-check self-paused children ≤30min.

## 4. Litmus / gate / REOPEN state

- **11 rows at `WAIVED-RETIRE_PROPOSED`** — the ONLY open human-only gate; re-verified
  live by #49 = **11**, unchanged. Owner HAS the commands in hand — re-check at every
  boundary. Authoritative row-ID list + copy-paste `confirm-retire --row-id <ID>`
  commands: `115-UNWAIVE-PATH.md` §"FINAL consolidated confirm-retire batch."
- Verb is human-only: `reposix-quality doc-alignment confirm-retire --row-id <ROW_ID>`
  from a real TTY. Refuses `$CLAUDE_AGENT_CONTEXT`. `--i-am-human` is an audited escape
  hatch for HUMANS, NOT agents — agents must never pass it.
- **P115 verifier verdict: GREEN-CHECKPOINT** (`115-VERIFICATION.md`, `ce4d3b7`) —
  checkpoint semantics: phase is NOT held open idle-waiting on the human step.
- **CI GREEN on `05085fe`**, re-verified live this turn (`Docs`/`CI`/`Push on main` all
  `success`; `release-plz` in_progress, non-blocking — see §1). No REOPEN state pending.
- **`116-RESEARCH.md` is 52,340 bytes** — over the 20KB file-size `structure/
  file-size-limits` warn floor, but non-blocking under the pre-existing `GTH-V15-21`
  waiver (expires 2026-08-08). Single-consumer research artifact; splitting not warranted
  now (researcher's own noticing).
- **File-size soft-ceiling waiver `GTH-V15-21`** — still masking the OVER-BUDGET tier as
  `--warn-only` until **2026-08-08** (`quality/catalogs/freshness-invariants.json:666`).
  Ledger-split decision still needs an owner call before lapse.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **CLOSED/ABSORBED — P116 fully RULED, encoded as the locked contract in
   `116-CONTEXT.md` (`31ac414`).** Both manager rulings (ADR-01 mirror fan-out = Option
   B+A folded in; FIX-03 slug→id = Option A this milestone, design-only). Treat
   `116-CONTEXT.md` as the authoritative, locked contract — do not re-run
   `discuss-phase`, do not rewrite CONTEXT to relitigate.
2. **NEW (#49) — the research CORRECTED the CONTEXT's mechanical framing; the planner
   must RECONCILE both, not rewrite CONTEXT.** The rulings are unchanged (still locked);
   only mechanical location details are corrected in `116-RESEARCH.md` (verified against
   reality this rotation):
   - The false claim "`sync --reconcile` heals the external mirror" is **NOT** in
     `quality/catalogs/doc-alignment.json` — jq/grep-verified live by #49: **zero**
     doc-alignment rows bind ANY file literally named `CLAUDE.md` (root or otherwise). The
     false claim lives ONLY in an ARCHIVED v0.14.0 SURPRISES-INTAKE entry
     (`.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md:299-329`, STATUS
     DEFERRED, discovered 2026-07-13 by B1). The LIVE docs (root `CLAUDE.md` §
     "Mirror-head refresh promise", `docs/concepts/dvcs-topology.md`) ALREADY scope
     `reconcile` correctly — the real gap is a MISSING explicit blessing of webhook+cron
     as authoritative, not false prose to delete.
   - The row to ACTUALLY retire is the LIVE `.planning/milestones/v0.15.0-phases/
     SURPRISES-INTAKE.md:108-116` entry (`## 2026-07-14 20:42 | ... litmus
     non-idempotency ...`, STATUS: OPEN) — confirmed live by #49 at exactly those line
     numbers (header line 108, `**STATUS:** OPEN` line 116) — this is a DISTINCT row
     from the archived v0.14.0 twin above; do not conflate the two or retire the wrong
     one.
   - Packet co-location recommendation from research: **cross-link only** (one backtick
     path citation added to ADR-010's existing "## References" section, matching the
     file's own established convention for citing `.planning/` provenance) — NOT a file
     move. Avoids a new mkdocs nav entry, the `structure/no-orphan-docs` gate, and
     further bloating ADR-010 (already 138% over its file-size budget per research).
   - Doc-alignment rebind risk from the P116 edit set is LOW — research jq-verified every
     catalog row bound to the 4 touched files anchors at/above the line ranges this phase
     will edit.
3. **Carried — ROADMAP P116 criterion 1** says the packet exists "alongside
   `docs/decisions/010-l2-l3-cache-coherence.md`" but it physically lives in the P115
   phase dir (`.planning/phases/115-live-mcp-benchmark-re-measurement/
   P116-ADR-010-DECISION-PACKET.md`). The planner MUST cover it (via the cross-link
   recommendation in item 2 above) or the verifier will flag it.
4. **Carried — `auto_advance: true`** in the P116 init-query config means a bare
   `/gsd-plan-phase` re-run could auto-chain into `/gsd-execute-phase` at workflow step 15.
   #50 **MUST NOT** take that chain — ROADMAP marks P116 `Execution mode: top-level` (the
   top-level coordinator IS the executor). Actively check for and clear
   `workflow._auto_chain_active` if the resumed workflow set it.
5. **Context-budget datapoint, now with a FOURTH corroboration.** #46/#47/#48 each
   confirmed a rotation fits roughly ONE heavy top-level skill pass; **#49 now
   corroborates a fourth time, and adds a new wrinkle**: ONBOARDING ALONE (reading the
   long #48 handover + the 25k-capped `/gsd-plan-phase` workflow read) consumed ~90k of
   #49's own context BEFORE any primary work started. #49's mitigation: dispatch the ONE
   foundational sub-step that runs in a subagent (research, context not charged to L0),
   hand-author the lightweight `VALIDATION.md` skeleton, then relieve clean rather than
   gamble entering the heavier pattern-mapper/planner/checker chain near the hard stop.
   **Whether this warrants a `GOOD-TO-HAVES.md` doctrine row on GSD/quality-skill context
   budgeting is an OPEN call, now four datapoints deep — #50 may want to file it.**
6. **MANAGER-ROUTED, MUST FILE (LOW, → P126 doc-alignment polish) — NOT YET FILED, do not
   drop.** Catalog row-id prefixes are inconsistent for the same doc — legacy rows use
   `README-md/` (e.g. `README-md/token-89-percent`, confirmed live at
   `quality/catalogs/doc-alignment.json:4450`) while the new hero row minted this
   milestone is `README/hero-token-economy-94-75` (confirmed live at line 9581). This cost
   the manager a false-negative grep. Fix = one naming convention or a linter check. #49
   did NOT file this (was at relief) — **#50 MUST file it into `GOOD-TO-HAVES.md`
   (v0.15.0-phases, tagged P126) so it survives past this session's launch context.**
7. **Researcher noticings, filed inside `116-RESEARCH.md` § "Noticed" (informational,
   already durable in the committed artifact, no further filing needed):** (a)
   `116-CONTEXT.md` mislabels the false-claim location [reconciled in item 2 above]; (b)
   stray copy-paste bleed at `GOOD-TO-HAVES.md:303-304` adjacent to `GTH-V15-38`; (c)
   `docs/concepts/dvcs-topology.md` at 90.9% of its file-size budget; (d) ROADMAP Phase
   116's `Execution mode: top-level` annotation + success-criteria text are STALE vs the
   now-locked CONTEXT (the "gather a ruling" framing is done — the ruling already
   happened, only the packet + doc-truth work remains).
8. **Carry-forward from #48 (still live, do NOT re-file):** the concepts-page four-axis
   hero coverage gap (LOW, `SURPRISES-INTAKE.md`, minted `e185e6e`); the `bind --help
   ::fn` Rust-only validator discrepancy (LOW, filed by #48's handover commit); `docs/
   index.md` near-duplicate bootstrap sequence (LOW, natural home P117/P119 under
   `GTH-V15-36`); two dangling P106 docs-repro/benchmark rows
   (`benchmark-claim-8ms-cached-read`, `benchmark-claim-89.1-percent-token-reduction`)
   pointing at claim text P115 moved — verify against the file before filing; the MEDIUM
   `test_main_offline_regenerates_doc_from_captures` byte-compare gap (already durable, do
   NOT re-file). Full detail: `git log` + the handover history at `b325caf`'s own §5.

## 6. Precise next steps (successor #50 runbook)

1. **Standard first-act verify block.** Run the § 1 command block yourself. Confirm HEAD
   is `05085fe` (or this handover's own commit, on top of it), tree clean, `0  0`
   ahead/behind origin/main, CI concluded `success` on that sha (Docs + CI + Push on main;
   `release-plz` is non-blocking). Flaky `test` job → re-run ONCE; still red → STOP,
   escalate, never proceed over a red main.
2. **Human gate re-check (do this at EVERY boundary — owner has the commands in hand).**
   `git fetch origin && grep -c '"last_verdict": "RETIRE_PROPOSED"'
   quality/catalogs/doc-alignment.json` — `11` = still open (do nothing further on the
   P115 close ritual). If it reads lower/`0`, the batch landed: advance
   `.planning/STATE.md`'s cursor past P115, close the checkpoint, note it in the next
   `PROGRESS.md` refresh.
3. **File the row-id-prefix noticing (§5 item 6) FIRST**, before anything else — it is a
   small, cheap, `<1h` fix-sketch write to `GOOD-TO-HAVES.md` (tag P126) and must not be
   dropped again across a session boundary.
4. **P116 planning TAIL — the remaining pass, your primary work this rotation.**
   Re-enter `/gsd-plan-phase 116`. The workflow finds `116-CONTEXT.md` +
   `116-RESEARCH.md` + `116-VALIDATION.md` already on disk → skips research
   (`has_research=true`) → step 7.5 passes (VALIDATION.md exists) → continues to:
   `gsd-pattern-mapper` (sonnet) → `PATTERNS.md`; `gsd-planner` (opus) → `PLAN.md`(s);
   `gsd-plan-checker` (sonnet); coverage gates → `state.planned-phase` → ROADMAP
   annotation → commit.
   - **Budget check: this tail is lighter than a full pass but STILL a heavy skill —
     enter it well under ~50k own-context (four datapoints now, §5 item 5). If you
     onboard past ~50k, consider dispatching the pattern-mapper as a standalone subagent
     and relieving again rather than pushing through.**
   - Planner MUST: (a) cover ROADMAP criterion-1 "packet lives alongside ADR-010" gap via
     the cross-link recommendation (§5 items 2–3); (b) reconcile `116-RESEARCH.md`'s
     corrections vs `116-CONTEXT.md`'s mislabels — retire the LIVE
     `SURPRISES-INTAKE.md:108-116` row, NOT the archived v0.14.0 twin (§5 item 2); (c)
     address BOTH req IDs FIX-03 + ADR-01; (d) FIX-03 is DESIGN-ONLY (ADR-010 §3
     amendment, waiver stays qualified, NO v0.15 build).
   - **Do NOT auto-chain into execution at workflow step 15** (§5 item 4) — actively
     check for and clear `workflow._auto_chain_active` if the resumed workflow set it.
     ROADMAP marks P116 `Execution mode: top-level` (the top-level coordinator IS the
     executor).
5. **P116 execution AFTER planning completes**, per the ROADMAP `Execution mode:
   top-level` marker: the top-level coordinator dispatches leaves per the completed
   PLAN.md's waves (opus complex / sonnet default / haiku mechanical, **NEVER fable at a
   leaf**); phase-close push cadence applies (push `origin main` BEFORE verifier dispatch,
   then `quality/runners/run.py --cadence post-push --persist`; `code/ci-green-on-main` is
   P0). Never open the next phase over a red main.
6. **Every push Bash timeout ≥300s** — pre-push wall time was ~92s this rotation, well
   above the ~55–60s documented budget; re-baseline is FILED not APPLIED — apply at OP-8
   drain, not mid-phase.
7. **Refresh `PROGRESS.md`'s `## NOW` at every boundary push** — do not let it go stale.
8. **REPLACE this handover** (not append) at #50's own relief, following this same
   ORCHESTRATION.md §3 template, with live-verified ground truth — re-check every claim
   live before carrying it forward.
