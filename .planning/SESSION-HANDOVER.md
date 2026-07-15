# SESSION-HANDOVER.md — v0.15.0 Floor: roadmap-diagram gsd-quick SHIPPED+PUSHED, T5 JSONL-usage methodology ENCODED, T4 HARD-STOPPED until 2026-07-16 2am PT — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #35** (L0
orchestrator), relieving to **successor #36**. This file **REPLACES** (does not append
to) the prior `SESSION-HANDOVER.md` (#34→#35's handover, superseded here).

**Read order:** this file → §1 (verify live — CI on the pushed tip was still
`in_progress` at write time, confirm it landed green) → §6 runbook (act 1 = confirm CI,
act 2 = the READ-ONLY Rovo auth check, act 3 = T4 gating check against the 2am PT
2026-07-16 reset) → §2/§3/§5 as needed.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager, pane w1:p7). No tag push by any coordinator —
the manager cuts tags, never L0. Do NOT do git surgery (reset/rebase/reorder/amend) on
`main`. Shared tree with the manager — TARGETED staging only, never `git add -A`/`.`.
**T4 (live-MCP capture) is HARD-STOPPED until 2026-07-16 02:00 PT (weekly subscription
reset) — do not start ANY live-MCP capture session before that time, regardless of what
else this file says.**

## 1. Ground truth (git) — verify live before acting, do not trust staleness

Re-run before doing anything else:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```
**Verified independently this handover (2026-07-15, just now):**
- Local `HEAD` = `4b38e62` ("docs(planning): file noticing — docs/development/roadmap.md
  stale internal snapshot (P115 roadmap lane, OD-3)"). Tree **CLEAN**
  (`git status --porcelain` empty). **0 ahead / 0 behind `origin/main`** — this tip is
  already pushed.
- **CI on `4b38e62` (run `29459800289`) was `in_progress` at last check (3m5s elapsed),
  NOT yet concluded.** Do not assume green — this is the successor's mandatory FIRST
  verify (§6 step 1), not a formality; the prior two runs on this branch (`29457486754`,
  `29456631954`) both concluded `success`, so the trend is good but unconfirmed on the
  actual tip.
- Commit history this rotation (`git log --oneline -8` from `HEAD`):
  ```
  4b38e62 docs(planning): file noticing — docs/development/roadmap.md stale internal snapshot (P115 roadmap lane, OD-3)
  9be5439 docs(115): amend plan — JSONL-usage token-economy methodology adopted [SELF]; ANTHROPIC_API_KEY gate dropped
  fa58ad6 docs(quick-260715-mk5): public birds-eye roadmap diagram — PLAN + SUMMARY + STATE
  16fb356 docs(roadmap): add public birds-eye roadmap diagram + bi-directional SYNC cross-links
  1db48e4 docs(quality): extend link-resolution catalog contract to name docs/*.md + .planning/PROJECT.md (catalog-first)
  25bd6a3 docs(planning): refresh manager handover — rotation #9→#10 (JSONL-usage methodology adopted, T4 deferred to post-reset, #35 launch charter)
  ade5e50 docs(planning): L0 relief handover #34→#35 — P115 T3 CLOSED, T4→T6 OWNER-BLOCKED on ANTHROPIC_API_KEY + MCP setup
  4351d48 docs(115): scaffold session-spend ledger — A1 unit ruling verbatim, ≤50 ceiling, zero rows (P115 T3)
  ```
- **This is 5 new commits (`25bd6a3`..`4b38e62`) all pushed and landed** — no unpushed
  local work remains from #35's rotation. Pre-push this rotation ran 61 PASS / 0 FAIL,
  secret-scan clean (per #35's own pre-wrap check; re-verify if in doubt via
  `python3 quality/runners/run.py --cadence post-push --persist`).
- **After this handover's own commit lands, local `main` will be 1 ahead of
  `origin/main`** (this handover commit only — the L0, not this writer, pushes it). #36's
  FIRST act is to confirm CI green on the tip that exists AFTER that push, not on
  `4b38e62` alone.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE | `9384ca6`, `3845b13` |
| Refresh-recovery (#33) | `/reposix-quality-refresh docs/benchmarks/latency.md` | DONE + PUSHED | `92c3ab5` |
| Wave 2 / T3 | Session-spend ledger scaffold (`benchmarks/bench-session-ledger.md`) | DONE + PUSHED | `4351d48` |
| Interleave (#35) | Public birds-eye roadmap diagram gsd-quick (owner-approved unblocked interleave, all 5 points) | **DONE + PUSHED** | `1db48e4`, `16fb356`, `fa58ad6` |
| Interleave / methodology (#35) | T5 JSONL-usage token-economy methodology [SELF] + 115-PLAN.md amendment | **DONE + PUSHED (ruling only, not yet executed)** | `9be5439` |
| Interleave / noticing (#35) | File `docs/development/roadmap.md` stale-snapshot noticing | **DONE + PUSHED** | `4b38e62` |
| Wave 3 / T4 | Live-MCP token capture (both fixtures, real sessions) | **HARD-STOPPED until 2026-07-16 02:00 PT** (subscription reset) — the `ANTHROPIC_API_KEY` block is GONE (see §3), remaining pre-work is the Rovo-auth check below | — |
| Wave 4 / T5 | Token-economy JSONL-usage regen (`bench_token_economy.py` new path) | METHODOLOGY RULED, implementation BLOCKED downstream on T4 | — |
| Wave 5 / T6 | Un-waive path + headline reframe + phase-close (delete 4 `[SELF]` entries) | BLOCKED downstream on T4/T5 | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | NOT STARTED (blocked on P115 close) | — |

### What #35 did this rotation

- Confirmed CI green on `25bd6a3` (run `29457486754`, exit 0) before opening any wave.
- **Roadmap-diagram gsd-quick: shipped all 5 points**, entered through `/gsd-quick`
  (`.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md`, now
  consumed): new `docs/roadmap.md` (color-coded mermaid, arcs/capabilities not
  phase-numbers, in mkdocs nav), bidirectional `<!-- SYNC: -->`-commented cross-links
  with `docs/roadmap.md` ↔ `.planning/PROJECT.md`, `mkdocs-strict` +
  `mermaid-renders` + `reposix-banned-words` all green, and point 5 (extend
  `DEFAULT_GLOBS` in `quality/gates/docs-build/link-resolution.py` to cover
  `docs/*.md` + `.planning/PROJECT.md`) landed as a **catalog-first** commit
  (`1db48e4`) before the diagram commit, per the project's catalog-first rule. Point 4
  (optional SYNC-marker-pair structure gate) was **NOT built** — filed instead as
  `GTH-V15-24` (LOW, OPEN, `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md`
  line 152) because a real structure gate is a genuine multi-file add, beyond a
  quick's budget.
  - **mcp-mermaid MCP server was UNREACHABLE this session.** The diagram was instead
    rendered locally via `mmdc` and verified against the REAL mkdocs page via
    playwright (DOM eval + screenshot, 0 console errors); render-proof artifact
    committed at `.planning/verifications/playwright/roadmap.json`. **Flag to
    owner: is mcp-mermaid configured/reachable? Next session hitting the same tool
    should re-check before assuming it's still down.**
- **Encoded the T5 JSONL-usage methodology ruling.** Added a `[SELF]` entry to
  `.planning/CONSULT-DECISIONS.md` ("2026-07-15 [SELF] P115-T5 token-economy
  methodology") plus a 6-edit amendment block
  (`<amendment id="jsonl-usage-methodology">`, `.planning/phases/
  115-live-mcp-benchmark-re-measurement/115-PLAN.md` line 179+) formalizing the
  manager's rotation-#10 adoption: T5 headline numbers derive from captured Claude
  Code **session JSONL usage records** (session-analyzer skill), NOT `count_tokens`.
  **This DROPS the `ANTHROPIC_API_KEY` requirement entirely, first run included** —
  `count_tokens` becomes an OPTIONAL later per-artifact enrichment only (free
  endpoint, subscription OAuth could auth it, no new pay-as-you-go key ever). This
  is a ruling, not yet an implementation — `bench_token_economy.py`'s new JSONL path
  does not exist yet; it is T5's job (see §6).
- **Filed 1 noticing item** (OD-3 obligation, discovered incidentally while working
  the roadmap lane): `docs/development/roadmap.md` is a stale internal snapshot
  (still claims "v0.11.0 Polish & Reproducibility — PLANNING" as active milestone,
  shipped-table stops at v0.10.0) — filed to `SURPRISES-INTAKE.md` (LOW, OPEN,
  2026-07-15). Not in mkdocs nav so it doesn't surface to readers, but it's a
  committed artifact a contributor could still trust.
- Did **NOT** start any T4 live-MCP capture session (hard-stop honored).
- Did **NOT** do the Rovo-MCP read-only auth check (deferred — context exhausted this
  rotation). This is carried forward as #36's first substantive (non-CI-check) act —
  see §6.

## 3. Binding constraints (unchanged — carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push by any coordinator; no git surgery on
main; leaf isolation in `/tmp` same-invocation; opus complex / sonnet default / haiku
mechanical, never fable at a leaf; relieve past ~100k own-context (hard 150k, absolute
not %) at a wave boundary; push at green, then confirm CI green on main AFTER the push
(`code/ci-green-on-main` P0 post-push probe); **T4 HARD-STOPPED until 2026-07-16 02:00
PT (weekly subscription reset)** — this is an ABSOLUTE gate, not a soft preference, and
supersedes any apparent unblock signal short of that clock passing.

**Superseded from the prior (#34→#35) handover:** the "T4 owner-blocked on
`ANTHROPIC_API_KEY`" framing is **GONE** — the JSONL-usage methodology ruling (§2)
removes that key requirement entirely. Do not re-ask the owner for
`ANTHROPIC_API_KEY`; it is no longer on T4/T5's critical path.

## 4. Litmus / gate / REOPEN state

- Pre-commit across #35's 5 commits: consistently PASS, 0 FAIL (file-size-limits
  waiver still active, unchanged from prior rotations, expires 2026-08-08).
- Pre-push this rotation (per #35's own pre-wrap run): **61 PASS / 0 FAIL**, secret-scan
  clean. **Took 109s — WARN against the ~60s budget.** Flagged as a noticing item
  (§5) — not yet triaged into a root cause, just observed; do not treat as a
  regression to fix reflexively, investigate first.
- New catalog-first row this rotation: `docs-build/link-resolution` contract extended
  to name `docs/*.md` + `.planning/PROJECT.md` (both cross-link directions now
  checked) — landed BEFORE the diagram commit per the project's catalog-first
  convention. `quality/catalogs/docs-build.json` line 88 carries the row text.
- The **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until
  2026-08-15** — T6 un-waives them after T4/T5 re-measure. Unchanged.
- No REOPEN state pending.
- **CI on the exact current tip (`4b38e62`) was unconfirmed (`in_progress`) at the
  time this handover was written** — see §1, this is the mandatory first re-check.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **T4 gating has CHANGED SHAPE — re-read this even if you skimmed the prior
  handover.** What remains before T4 can execute is NO LONGER "owner must supply
  `ANTHROPIC_API_KEY`" (that block is gone, see §3). What remains:
  1. **Choose the MCP server for the mcp-mediated arm**: official Atlassian Rovo
     remote MCP (API-token path, per prior GA intel — NOT OAuth) vs fallback
     `sooperset/mcp-atlassian`.
  2. **Verify (READ-ONLY) whether the existing `ATLASSIAN_API_KEY` authenticates the
     Rovo MCP endpoint.** The manager's own rotation-#10 refresh (`.planning/
     MANAGER-HANDOVER.md` line 105-107) explicitly names this: "#34's noted
     'API-token-endpoint blocker' needs verification first." **This check is
     STILL UNVERIFIED** — #35 did not do it (context exhausted). **#36 should do
     this READ-ONLY check when convenient, pre-reset**: confirm or refute whether
     the existing key authenticates against the Rovo MCP endpoint. Report only —
     do NOT set up the live connection, do NOT spend a capture session, do NOT
     make any write against a real backend as part of this check.
  3. Once both are resolved AND the 2026-07-16 02:00 PT clock has passed, T4 can
     open: ≤18 sessions (median-of-3 × ≤3 backends × 2 arms), task = "read 3
     issues, edit 1, push"; mcp-mediated arm captures tool-list + tool-call/
     response payloads → replaces `benchmarks/fixtures/mcp_jira_catalog.json`;
     reposix-mediated arm runs the equivalent via a real reposix checkout in a
     **THROWAWAY `/tmp` clone** (leaf-isolation) → ANSI-stripped transcript
     replaces `benchmarks/fixtures/reposix_session.txt`; append ONE ledger row per
     session, increment `running_total`, assert ≤50 BEFORE next; scrub creds;
     targeted-add ONLY the two fixtures + ledger. **T4 is the context-blowing
     wave — run with fresh context, relieve if approaching ~100k own-context
     mid-wave.**
- **pre-push took 109s this rotation (WARN, budget ~60s).** Noticed, not yet
  triaged — possible new whole-repo gate added recently, or environment variance.
  Investigate before assuming either; non-blocking for now.
- **File-size soft-ceiling WARNs** (waived until 2026-08-08, known bloat class
  `GTH-V15-21`): `115-PLAN.md` 32633B, `SURPRISES-INTAKE.md` 28183B,
  `GOOD-TO-HAVES.md` ~30.6kB, `260715-mk5-PLAN.md` 22.5kB. A progressive-disclosure
  split is needed eventually — not this rotation's job, just carried forward.
- **`docs/development/roadmap.md` stale internal snapshot — ALREADY FILED this
  rotation** to `SURPRISES-INTAKE.md` (LOW, OPEN, 2026-07-15). **Do not re-file.**
- **`link-resolution` now reads `docs/index.md` twice** (cosmetic double-count, the
  extended glob overlaps the pre-existing explicit entry) — documented inline in
  the script's code comment, harmless, not filed as a defect (too trivial).
- **latency.md regeneration-clobber tension — still OPEN, unchanged.**
  `emit-markdown.sh` regenerates `latency.md` from a LOCAL bench run, would clobber
  the CI-canonical figures corrected in #32/#33. Reconcile in T5/T6.
- **latency.md is doc-alignment-TRACKED** — the eventual T6 headline reframe
  RE-DRIFTS its 14 rows. **Budget a SECOND
  `/reposix-quality-refresh docs/benchmarks/latency.md` BEFORE the T6 phase-close
  push.** Grep `quality/catalogs/doc-alignment.json` for ANY doc before editing it.
- **FOUR `[SELF]` ledger entries now pending deletion at T6 phase-close** (each
  entry's own text conditions its deletion on "the phase closes"): the A1
  definition entry, the P115-T2 latency-canonical-source entry, the P115-T6
  headline-framing entry, and the NEW P115-T5 JSONL-usage-methodology entry added
  this rotation. **#36/whoever runs T6: delete all four, not three** — the prior
  (#34) handover only knew about three; this handover updates that count.
- **Carry item (needs owner/manager DOCTRINE CALL, do NOT merge unilaterally):**
  GOOD-TO-HAVES consolidation (two coexisting files) — pending todo at
  `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`.
- **Weekly subscription-limit watch:** T4 spends LIVE subscription sessions once it
  opens post-reset — a limit-stall risk; surface to MANAGER immediately if hit
  again.
- **Background shells/monitors: NONE running** — nothing left open for #36 to
  inherit.

## 6. Precise next steps (successor runbook)

1. **FIRST ACT — confirm CI green on the tip that exists AFTER this handover
   commit is pushed by the L0 (not necessarily `4b38e62` alone — see §1).**
   - `gh run list --branch main --workflow CI --limit 3` — wait for the top row to
     reach `completed`.
   - Then `python3 quality/runners/run.py --cadence post-push --persist` — the
     `code/ci-green-on-main` (P0) probe asserts main's NEWEST `ci.yml` run
     concluded success (not merely that some older green run exists).
   - If the flaky `test` CI job goes red, re-run it ONCE before treating it as a
     real regression. If still red after one re-run, STOP — do not open a wave
     over a red main; escalate per the retro/incident norms.

2. **Do the READ-ONLY Rovo-auth check** (§5 item, still unverified from #34/#35):
   confirm or refute whether the existing `ATLASSIAN_API_KEY` authenticates
   against the official Atlassian Rovo remote MCP endpoint. Report-only — do NOT
   wire the live connection, do NOT spend a capture session, do NOT write to any
   real backend. This can be done ANY TIME before the reset (it's pre-work, not
   gated on the clock).

3. **Check the 2026-07-16 02:00 PT clock.** T4 (and ONLY T4 — any live-MCP
   capture session) is HARD-STOPPED until then, no exceptions. If it hasn't
   passed yet:
   - Do NOT start T4.
   - If there is remaining owner-approved unblocked interleave work (check
     `.planning/todos/pending/` for anything new the owner queued since this
     handover was written; none is known to exist right now beyond the
     already-consumed roadmap-diagram quick), do that instead.
   - Otherwise, use the time for pre-work only: the Rovo-auth check (step 2
     above), re-reading `115-PLAN.md` Task 4's exact shape, or reviewing this
     handover with the manager.

4. **Once the clock has passed AND the Rovo-server choice is resolved** → record
   the MCP-server choice + confirm `ATLASSIAN_API_KEY` (or equivalent) works in
   the plan/ledger notes (never print the key value itself), then execute T4 (the
   context-blower — relieve if approaching ~100k own-context mid-wave), then T5,
   then T6, then phase-close:
   - **T5**: implement the JSONL-usage path in
     `quality/gates/perf/bench_token_economy.py` (headline source = session-analyzer
     parse of captured JSONL records per the ruling in §2/§5), demote `count_tokens`
     to optional enrichment, regenerate `token-economy.md` + methodology note,
     update the Task-5 `<automated>` check, offline-cache-stable, honest
     provenance, README matched; reconcile the latency.md regeneration-clobber
     tension here or explicitly hand to T6; catalog-first if a perf-row contract
     changes.
   - **T6**: `115-UNWAIVE-PATH.md`, budget the SECOND
     `/reposix-quality-refresh docs/benchmarks/latency.md`, **delete all FOUR
     `[SELF]` ledger entries** (A1, P115-T2 latency-canonical, P115-T6 headline-
     framing, P115-T5 JSONL-usage-methodology) once each is encoded per its own
     precondition, phase-close ritual (`git push origin main`, confirm
     `code/ci-green-on-main`, verifier subagent for catalog-row PASS, advance
     `.planning/STATE.md` cursor, RAISE LIST/intake disposition, final report).
   - After P115 closes: produce the P116 ADR-010 packet (ADR-01 mirror-fanout +
     FIX-03 GTH-09 slug→id options+tradeoffs), route to **MANAGER (w1:p7) for
     ruling, NO pre-ruling implementation.**

5. **Carry item (still OPEN, needs manager/owner DOCTRINE CALL, do NOT merge
   unilaterally):** GOOD-TO-HAVES consolidation (two coexisting files).
