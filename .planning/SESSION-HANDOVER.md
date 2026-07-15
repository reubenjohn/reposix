# SESSION-HANDOVER.md — v0.15.0 Floor: P115 T3 ledger scaffold DONE (unpushed), T4 OWNER-BLOCKED — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #34** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #35**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#33→#34's
handover, superseded here).

**Read order:** this file → §1 (verify live — HEAD is 1 ahead of origin, the L0 pushes
this handover + T3 together right after commit) → §6 runbook (confirm CI green on the
NEW pushed tip is act ONE, then check whether T4's owner-block cleared) → §3/§4/§5 as
needed.
**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager, pane w1:p7). No tag push by any coordinator —
the manager cuts tags, never L0. Do NOT do git surgery (reset/rebase/reorder/amend) on
`main`. Shared tree with the manager — TARGETED staging only, never `git add -A`/`.`.

## 1. Ground truth (git) — verify live before acting, do not trust staleness

Re-run before doing anything else:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```
**Verified independently this handover (2026-07-15, just now):**
- Local `HEAD` = `4351d48` ("docs(115): scaffold session-spend ledger — A1 unit ruling
  verbatim, ≤50 ceiling, zero rows (P115 T3)"). Tree **CLEAN** (`git status --porcelain`
  empty). This is **1 ahead of `origin/main`** (`0` behind).
- `origin/main` = `6afe803` ("docs(planning): refresh manager handover — #33 cleared
  drift block (verified), #34 launching on T3"). CI on `6afe803` (run `29455715936`)
  **concluded `success`** — verified this rotation via `gh run watch`, exit 0. **That is
  the confirmed green floor this handover is built on.**
- Recent commit history (`git log --oneline -8` from HEAD):
  ```
  4351d48 docs(115): scaffold session-spend ledger — A1 unit ruling verbatim, ≤50 ceiling, zero rows (P115 T3)
  6afe803 docs(planning): refresh manager handover — #33 cleared drift block (verified), #34 launching on T3
  9935ae7 docs(planning): L0 relief handover #33→#34 — refresh-recovery wave CLOSED+PUSHED, CI-green check then T3 next
  92c3ab5 fix(latency-bench): enforce real-backend 3s WARN + refresh latency doc-alignment rows (P115)
  af4bbd4 docs(planning): T6 headline-framing ruled [SELF] — honest CI-canonical reframe proceeds; manager handover → #33
  eac08a1 docs(planning): L0 relief handover #32→#33 — P115 Wave-1 CLOSED (T2 latency corrected), T3→T6 next
  3845b13 docs(115): correct sim latency to CI canonical figures, ledger + defect filing
  9384ca6 docs(benchmarks): re-measure v0.9.0 latency envelope (P115 BENCH-01 T2)
  ```
- **After this handover's own commit lands**, local `main` will be **2 ahead of
  origin/main** (T3 `4351d48` + the handover commit). This is **INTENTIONAL** — the L0
  pushes BOTH together right after this commit lands. #35's FIRST act is to confirm CI
  green on the **NEW pushed tip** (not `6afe803`) before opening any wave — see §6 step 1.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE | `9384ca6`, `3845b13` |
| Refresh-recovery (#33) | `/reposix-quality-refresh docs/benchmarks/latency.md` | DONE + PUSHED | `92c3ab5` |
| Wave 2 / T3 | Session-spend ledger scaffold (`benchmarks/bench-session-ledger.md`) | **DONE this rotation — unpushed until the L0's relief push** | `4351d48` |
| Wave 3 / T4 | Live-MCP token capture (both fixtures, real sessions) | **BLOCKED — OWNER-GATED (the #1 item, see below)** | — |
| Wave 4 / T5 | Token-economy `count_tokens` regen | BLOCKED downstream on T4 | — |
| Wave 5 / T6 | Un-waive path + headline reframe + phase-close | BLOCKED downstream on T4/T5 | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | NOT STARTED (blocked on P115 close) | — |

### What #34 did this rotation

- Confirmed CI green on `6afe803` (`gh run watch` on run `29455715936`, exit 0) before
  opening any wave — the inherited-in-progress-CI risk from #33's handover resolved clean.
- Executed **Wave 2 / T3**: scaffolded `benchmarks/bench-session-ledger.md` — encoded the
  A1 ruling verbatim (session = one live agentic conversation/task run, failed/aborted
  runs count, ≤50 ceiling, escalate to MANAGER past 50, flag any session >~5x running-
  median token cost), the exact 8-column header (`# | timestamp (UTC, ISO-8601) | backend
  | arm (mcp-mediated / reposix-mediated) | task | unit_consumed (per ruling) |
  running_total | artifact_produced (which fixture)`), **zero data rows**. Verify passed
  `LEDGER_SCAFFOLD_OK`. Pre-commit exit 0. Committed `4351d48` (targeted staging).
- **Discovered Task 1's gate is NOT fully closed** despite Wave 1/T1's A1 ruling being
  DONE — Task 1 (115-PLAN.md ~line 178-220) bundles the A1 ruling with two more
  execution-start facts that were never re-verified/closed: MCP-server choice and
  `ANTHROPIC_API_KEY` presence. Re-verified preflight (still PASS) but found the other
  two absent — **escalated to the manager (w1:p7) this rotation.**

## 3. Binding constraints (unchanged — carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push by any coordinator; no git surgery on
main; leaf isolation in `/tmp` same-invocation; opus complex / sonnet default / haiku
mechanical, never fable at a leaf; relieve past ~100k own-context (hard 150k, absolute
not %) at a wave boundary; manager (w1:p7) POLLS — clear in-pane narration at
boundaries IS the report, escalate actively only for owner-blocking moments; push at
green, then confirm CI green on main AFTER the push; watch item: flaky `test` CI job
(re-run once before treating as real).

## 4. Litmus / gate / REOPEN state

- Pre-commit on `4351d48`: **1 PASS, 0 FAIL, 1 WAIVED** (structure/file-size-limits
  waived until 2026-08-08), exit 0.
- No open `STALE_*` doc-alignment rows introduced this rotation — T3's file
  (`benchmarks/bench-session-ledger.md`) is **outside `docs/`**, not doc-alignment-tracked.
- The **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until
  2026-08-15** — T6 un-waives them after T4/T5 re-measure.
- No REOPEN state pending.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **THE #1 ITEM — T4 owner-block, escalated to manager (w1:p7) this rotation.** Task 1's
  gate (115-PLAN.md ~line 178-220) is NOT fully closed, so T4 cannot start. Three sub-parts:
  - `scripts/preflight-real-backends.sh` → **exit 0 / PASS** (re-verified this rotation:
    Confluence TokenWorld, GitHub `reubenjohn/reposix` 3 open issues, JIRA `KAN` all
    reachable). ✅
  - **`ANTHROPIC_API_KEY` → ABSENT** (env + `.env` both checked, no value present).
    Needed for T5's `count_tokens` on the captured fixtures. **LOCKED constraint**
    (115-PLAN.md ~line 22-26, Task 5 ~line 328): must be the **EXISTING-subscription**
    key — **no new pay-as-you-go key, no new API spend.** Only the owner can provide
    this. ❌
  - **MCP server for the mcp-mediated arm → NOT chosen/connected.** Plan options
    (115-PLAN.md ~line 193-195): official Atlassian Rovo remote MCP (API-token path per
    prior §5 GA intel, **NOT OAuth**) OR fallback `sooperset/mcp-atlassian`. Connecting
    likely needs owner Atlassian creds/config. ❌
  - **Ask to manager/owner:** provide the existing-subscription `ANTHROPIC_API_KEY` (in
    `.env` or env) AND confirm/enable the Atlassian MCP server + its API-token creds.
    Until then T4→T5→T6 cannot proceed.
  - **T4 shape** (115-PLAN.md Task 4 ~line 276-318, for when unblocked): ≤18 sessions
    (median-of-3 × ≤3 backends × 2 arms), task = "read 3 issues, edit 1, push";
    mcp-mediated arm captures tool-list + tool-call/response payloads → replaces
    `benchmarks/fixtures/mcp_jira_catalog.json`; reposix-mediated arm runs the equivalent
    via a real reposix checkout in a **THROWAWAY `/tmp` clone** (leaf-isolation) →
    ANSI-stripped transcript replaces `benchmarks/fixtures/reposix_session.txt`; append
    ONE ledger row per session, increment `running_total`, assert ≤50 BEFORE next; scrub
    creds; targeted-add ONLY the two fixtures + ledger. Verify: no `/mnt/` or
    `scripts/demo.sh` in `reposix_session.txt`, final `running_total` numerically ≤50, no
    leaked creds. **T4 is the context-blowing wave — #35 should run it with fresh
    context and relieve if it approaches ~100k lines mid-wave.**
- **A1 [SELF] consult-entry deletion — RECONCILED: KEEP until T6 phase-close.** #33's
  handover had a conflict (§5 said "delete at P115 close", §6 step 2 said "delete once
  T3 lands"). The consult entry's OWN text (`.planning/CONSULT-DECISIONS.md` line
  93-94, resume-signal) conditions deletion on "encode the definition AND the phase
  closes." #34 took the conservative reading: KEEP A1 now (T3 has encoded it verbatim,
  satisfying the precondition), **DELETE it at T6 phase-close** batched with the
  P115-T2 and P115-T6 `[SELF]` entries — preserves the owner-veto disclosure window.
  **#35: delete all three `[SELF]` entries at T6 close per each entry's own instruction.**
- **latency.md regeneration-clobber tension — still OPEN.** `emit-markdown.sh`
  regenerates `latency.md` from a LOCAL bench run, would clobber #32's CI-canonical
  figures. Reconcile in T5/T6.
- **latency.md is doc-alignment-TRACKED** — the T6 headline reframe RE-DRIFTS its 14
  rows. **Budget a SECOND `/reposix-quality-refresh docs/benchmarks/latency.md` BEFORE
  the T6 phase-close push.** Grep `quality/catalogs/doc-alignment.json` for ANY doc
  before editing it.
- **T6 headline framing RULED `[SELF]`** — honest CI-canonical reframe proceeds now, no
  owner gate.
- **Intakes — do NOT re-file:** all prior (GTH-V15-* etc.) + the sim `expected_version`
  PATCH defect (`SURPRISES-INTAKE.md`, MEDIUM, OPEN, from `3845b13`).
- **Weekly-limit watch:** 76% at #34 rotation start; #34 spent ~2 subagents (1
  reader-digester + 1 relief-writer). **T4 spends LIVE subscription sessions** — a
  limit-stall risk; surface to MANAGER immediately if hit.
- **Background shells/monitors: NONE running** — the CI-watch and the T4-spec digester
  both completed and reported this rotation. Nothing left running for #35 to inherit.

## 6. Precise next steps (successor runbook)

1. **FIRST ACT — confirm CI green on the NEW pushed tip (the L0's relief push of T3 +
   this handover) before opening any wave.**
   - `gh run list --branch main --workflow CI --limit 3` — wait for the top row to reach
     `completed`.
   - Then `python3 quality/runners/run.py --cadence post-push --persist` — the
     `code/ci-green-on-main` (P0) probe asserts main's NEWEST `ci.yml` run concluded
     success (not merely that some older green run exists).
   - **If the flaky `test` job goes red, re-run it ONCE** before treating it as a real
     regression. If still red after one re-run, STOP — do not open a wave over a red
     main; escalate per the retro/incident norms.

2. **Check whether the manager/owner UNBLOCKED T4** (the #1 item in §5):
   - Is `ANTHROPIC_API_KEY` now present (env or `.env`)? (Do not print the value.)
   - Is the Atlassian MCP server available / has the owner named which one (Rovo
     remote API-token vs `sooperset/mcp-atlassian`)?
   - Re-run `bash scripts/preflight-real-backends.sh` (should stay exit 0).

3a. **If T4 UNBLOCKED** → close Task 1's gate (record the MCP-server choice +
    `ANTHROPIC_API_KEY`-present in the plan/ledger notes — never the key value itself),
    then execute T4 (the context-blower — relieve if approaching ~100k lines mid-wave),
    then T5, then T6, then phase-close:
    - T5: regenerate `token-economy.md` from the real fixtures with `count_tokens`,
      offline-cache-stable, honest provenance, methodology note, README matched;
      reconcile the latency.md regeneration-clobber tension here or explicitly hand to T6.
    - T6: `115-UNWAIVE-PATH.md`, budget the SECOND
      `/reposix-quality-refresh docs/benchmarks/latency.md`, delete the three `[SELF]`
      ledger entries once encoded, phase-close ritual (`git push origin main`, confirm
      `code/ci-green-on-main`, verifier subagent for catalog-row PASS, advance
      `.planning/STATE.md` cursor, RAISE LIST/intake disposition, final report).
    - After P115 closes: produce the P116 ADR-010 packet (ADR-01 mirror-fanout + FIX-03
      GTH-09 slug→id options+tradeoffs), route to **MANAGER (w1:p7) for ruling, NO
      pre-ruling implementation.**

3b. **If T4 STILL BLOCKED** → do the owner-approved UNBLOCKED interleave: the **public
    birds-eye roadmap-diagram gsd-quick**
    (`.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md`). Enter
    through `/gsd-quick`. Five points:
    1. New `docs/roadmap.md` — single color-coded mermaid diagram, arcs/capabilities
       NOT phase numbers/dates, registered in mkdocs nav, mcp-mermaid render-review
       before commit.
    2. Bidirectional `docs/roadmap.md` ↔ `.planning/PROJECT.md` cross-links, each
       carrying a `<!-- SYNC: ... -->` comment + append the one-line SYNC reminder to
       the OP-9 checklist in `.planning/PRACTICES.md`.
    3. Gates `mkdocs-strict` + `mermaid-renders` + `reposix-banned-words` on the new
       docs/ file; mind docs-build/link-resolution.
    4. OPTIONAL structure-gate row asserting the SYNC pair exists both sides.
    5. **REQUIRED** — extend `DEFAULT_GLOBS` in
       `quality/gates/docs-build/link-resolution.py` to cover `docs/*.md` +
       `.planning/PROJECT.md` so BOTH cross-link directions are link-checked
       (catalog-first if a row contract changes).
    - This does NOT spend benchmark sessions or the ANTHROPIC subscription — safe under
      the weekly-limit pressure.

4. **Carry item (still OPEN, needs manager/owner DOCTRINE CALL, do NOT merge
   unilaterally):** GOOD-TO-HAVES consolidation (two coexisting files).
