# SESSION-HANDOVER.md — v0.15.0 Floor: Rovo-auth blocker REFUTED, pre-push spike ROOT-CAUSED, T4 still HARD-STOPPED until 2026-07-16 02:00 PT — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #36** (L0
orchestrator), relieving to **successor #37**. This file **REPLACES** (does not append
to) the prior `SESSION-HANDOVER.md` (#35→#36's handover, superseded here).

**Read order:** this file → §1 (verify live — 2 local commits are UNPUSHED, no CI run
exists on them yet; confirm push + green after the L0 pushes) → §6 runbook (act 1 =
confirm CI on the post-push tip, act 2 = check the 2026-07-16 02:00 PT clock, act 3 =
either pre-work or T4→T6→phase-close) → §2/§3/§5 as needed.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate
document, separate owner — the manager, pane w1:p7). No tag push by any coordinator —
the manager cuts tags, never L0. Do NOT do git surgery (reset/rebase/reorder/amend) on
`main`. Shared tree with the manager — TARGETED staging only, never `git add -A`/`.`.
**T4 (live-MCP capture) is HARD-STOPPED until 2026-07-16 02:00 PT (weekly subscription
reset) — do not start ANY live-MCP capture session before that time, regardless of what
else this file says, even though the auth blocker that used to gate it is now
REFUTED (see §2).**

## 1. Ground truth (git) — verify live before acting, do not trust staleness

Re-run before doing anything else:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 5
```
**Verified independently this handover (2026-07-15, ~17:20 PT):**
- Local `HEAD` = `fcddf90` ("docs(planning): file root-cause of pre-push over-budget
  WARN — variance + kcov corpus creep (P115, OD-3)"). Tree **CLEAN**
  (`git status --porcelain` empty). **2 ahead / 0 behind `origin/main`** — `origin/main`
  is still at `1b20c15`; the two commits `5374fe0` and `fcddf90` made this rotation are
  **NOT YET PUSHED** (this handover's own commit will be a third unpushed commit; the
  L0, not this writer, pushes all three together — see the note below).
- **CI on `1b20c15` (run `29460132017`) is CONFIRMED `completed`/`success`** (verified
  live via `gh run list` at write time, 5m18s, matches the manager's pre-verification
  named in the launch charter). **No CI run exists yet on `5374fe0` or `fcddf90`** —
  they are unpushed, so nothing has triggered against them. #37's mandatory FIRST verify
  (§6 step 1) is against whatever tip exists AFTER the L0 pushes this handover commit
  plus the two prior ones, NOT against `1b20c15` alone.
- Commit history this rotation (`git log --oneline -6` from `HEAD`):
  ```
  fcddf90 docs(planning): file root-cause of pre-push over-budget WARN — variance + kcov corpus creep (P115, OD-3)
  5374fe0 docs(115): READ-ONLY Rovo MCP auth check — #34 'API-token-endpoint blocker' REFUTED (P115 T4 pre-work)
  1b20c15 docs(planning): L0 relief handover #35→#36 — roadmap-diagram quick SHIPPED+PUSHED, T5 JSONL-usage methodology ENCODED, T4 HARD-STOPPED until 2026-07-16 02:00 PT
  4b38e62 docs(planning): file noticing — docs/development/roadmap.md stale internal snapshot (P115 roadmap lane, OD-3)
  9be5439 docs(115): amend plan — JSONL-usage token-economy methodology adopted [SELF]; ANTHROPIC_API_KEY gate dropped
  fa58ad6 docs(quick-260715-mk5): public birds-eye roadmap diagram — PLAN + SUMMARY + STATE
  ```
- **This is 2 new commits (`5374fe0`, `fcddf90`) this rotation, both read-only-charter
  deliverables, neither pushed yet.** No cargo/code was touched — both are
  `.planning/`-only doc commits (a new `115-ROVO-AUTH-CHECK.md` file + a
  `SURPRISES-INTAKE.md` append). Pre-commit ran clean on both (see §4).
- **After this handover's own commit lands, local `main` will be 3 ahead of
  `origin/main`** (the two rotation commits + this handover commit — the L0 pushes all
  three, not this writer). #37's FIRST act is to confirm CI green on the tip that
  exists AFTER that push, not on `1b20c15` alone.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE | `9384ca6`, `3845b13` |
| Refresh-recovery (#33) | `/reposix-quality-refresh docs/benchmarks/latency.md` | DONE + PUSHED | `92c3ab5` |
| Wave 2 / T3 | Session-spend ledger scaffold (`benchmarks/bench-session-ledger.md`) | DONE + PUSHED | `4351d48` |
| Interleave (#35) | Public birds-eye roadmap diagram gsd-quick (owner-approved unblocked interleave, all 5 points) | DONE + PUSHED | `1db48e4`, `16fb356`, `fa58ad6` |
| Interleave / methodology (#35) | T5 JSONL-usage token-economy methodology [SELF] + 115-PLAN.md amendment | DONE + PUSHED (ruling only, not yet executed) | `9be5439` |
| Interleave / noticing (#35) | File `docs/development/roadmap.md` stale-snapshot noticing | DONE + PUSHED | `4b38e62` |
| Pre-work (#36) | READ-ONLY Rovo MCP auth check | **DONE, UNPUSHED** — VERDICT: #34's blocker REFUTED (HIGH confidence) | `5374fe0` |
| Pre-work (#36) | Pre-push over-budget spike diagnosis (read-only) | **DONE, FILED, UNPUSHED** — root cause = variance + kcov corpus creep; recommendation filed, NOT applied | `fcddf90` |
| Wave 3 / T4 | Live-MCP token capture (both fixtures, real sessions) | **HARD-STOPPED until 2026-07-16 02:00 PT** (subscription reset) — auth pre-req now RESOLVED (see below); only the formal MCP-server choice + the clock remain | — |
| Wave 4 / T5 | Token-economy JSONL-usage regen (`bench_token_economy.py` new path) | METHODOLOGY RULED, implementation BLOCKED downstream on T4 | — |
| Wave 5 / T6 | Un-waive path + headline reframe + phase-close (delete 4 `[SELF]` entries) | BLOCKED downstream on T4/T5 | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | NOT STARTED (blocked on P115 close) | — |

### What #36 did this rotation

- Confirmed ground truth at rotation start: `main == origin/main == 1b20c15`, tree
  clean. **Skipped a fresh CI re-verify** per the launch charter — the manager had
  already pre-verified CI run `29460132017` on `1b20c15` = SUCCESS (this writer
  independently re-confirmed the same run/result live while assembling this handover —
  see §1).
- **Charter item 1 — READ-ONLY Rovo MCP auth check: DONE.** Committed `5374fe0`
  (`.planning/phases/115-live-mcp-benchmark-re-measurement/115-ROVO-AUTH-CHECK.md`).
  **VERDICT: #34's "API-token-endpoint blocker" is REFUTED (HIGH confidence).** The
  existing `ATLASSIAN_API_KEY` (+ `ATLASSIAN_EMAIL`) authenticates the OFFICIAL
  Atlassian remote MCP endpoint `https://mcp.atlassian.com/v1/mcp`
  (`atlassian-mcp-server` v1.0.0) via BOTH Basic (`email:token`) and Bearer — the
  `initialize` handshake returned HTTP 200 + `mcp-session-id` under both forms,
  bracketed by a no-auth 401 control and a 200 REST token-validity baseline. Official
  docs confirm the API-token path ("if enabled by your org admin"); enablement
  empirically confirmed for this tenant. Read-only: NO live MCP connection wired, NO
  capture session spent, NO backend write, NO `tools/call` — stopped at `initialize`.
  **This removes the T4 mcp-arm auth uncertainty.** Recommendation (recommendation, NOT
  a ratified choice — formal server pick stays T4-executor/manager's call): **official
  Rovo remote MCP via API token** (no OAuth browser flow, no self-hosted `sooperset`
  fallback needed).
- **Charter item 2 — pre-push over-budget spike diagnosis (read-only): DONE + FILED.**
  Committed `fcddf90` — appended a root-cause entry to
  `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (dated `2026-07-15 17:18`,
  cross-referencing and enriching the existing `2026-07-15 06:35` pre-push-timing entry
  — NOT a duplicate). Finding: the WARN is **mostly environment variance** (a fresh
  re-run on identical state measured **64s**, not 109s) layered on a modest kcov-corpus
  creep — `code/shell-coverage` grew 29s→~37s because two shell harnesses (`fbb7782`,
  `fe8febb`, both 2026-07-12) landed AFTER the ~55s budget was documented. **No new
  gate.** Budget is STALE, not a stable regression. Recommendation FILED not applied
  (charter = change nothing): re-baseline budget to ~75s + raise WARN 90s→100s. The
  drain phase (OP-8 Slot 1) can apply it.
- **Charter item 3 — T4 HARD-STOP: honored.** Started ZERO capture sessions. As of this
  handover the 2026-07-16 02:00 PT reset **has NOT passed** (PT was `2026-07-15 17:20
  PDT` when this handover was written — confirm freshly via
  `TZ='America/Los_Angeles' date '+%Y-%m-%d %H:%M %Z'` before acting).
- Did **NOT** push either of this rotation's commits — that is the L0's job after this
  handover lands (see §1).

## 3. Binding constraints (unchanged — carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push by any coordinator; no git surgery on
main; leaf isolation in `/tmp` same-invocation; opus complex / sonnet default / haiku
mechanical, never fable at a leaf; relieve past ~100k own-context (hard 150k, absolute
not %) at a wave boundary; push at green, then confirm CI green on main AFTER the push
(`code/ci-green-on-main` P0 post-push probe); **T4 HARD-STOPPED until 2026-07-16 02:00
PT (weekly subscription reset)** — this is an ABSOLUTE gate, not a soft preference, and
supersedes any apparent unblock signal short of that clock passing (including the
auth-blocker REFUTED finding this rotation — that resolves a PRE-REQ, it does not
move the clock).

**Superseded from the prior (#35→#36) handover:** the "Rovo-auth check still
unverified" framing is **GONE** — `5374fe0` resolves it (REFUTED, HIGH confidence). Do
not re-run the read-only auth probe; treat it as closed pre-work. What remains before
T4 opens is (1) the formal MCP-server choice (recommended: official Rovo via API
token) and (2) the clock.

## 4. Litmus / gate / REOPEN state

- Pre-commit across #36's 2 commits: PASS, 0 FAIL (file-size-limits waiver still
  active, unchanged from prior rotations, expires 2026-08-08).
- Pre-push **not yet re-run this rotation** on the current tip (`fcddf90`) — the two
  commits this rotation have not been pushed. #37's runbook step 1 covers this
  (confirm CI after the L0's push).
- **Pre-push timing WARN (109s vs ~55-60s documented budget) is now ROOT-CAUSED, not
  just observed** (see §2 charter item 2 / §5). Recommendation to re-baseline the
  budget doc to ~75s + raise the WARN threshold to ~100s is FILED
  (`SURPRISES-INTAKE.md`, 2026-07-15 17:18 entry) but **NOT applied** — this rotation's
  charter was read-only diagnosis, not a fix. Next agent doing OP-8 drain work (or
  anyone touching `quality/CLAUDE.md` § Cadences) should apply it then.
- The **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until
  2026-08-15** — T6 un-waives them after T4/T5 re-measure. Unchanged.
- No REOPEN state pending.
- **CI on the exact current tip (`fcddf90`) does not exist yet** — it is unpushed. Last
  confirmed-green run is `29460132017` on `1b20c15` (verified live by this writer, see
  §1). §6 step 1 is the mandatory first re-check once the L0 has pushed.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **T4 gating has CHANGED SHAPE again this rotation — re-read even if you skimmed the
  prior handover.** What remains before T4 can execute:
  1. ~~Verify (READ-ONLY) whether the existing `ATLASSIAN_API_KEY` authenticates the
     Rovo MCP endpoint~~ — **DONE this rotation, REFUTED the blocker** (`5374fe0`,
     `115-ROVO-AUTH-CHECK.md`). Do not re-run.
  2. **Choose the MCP server for the mcp-mediated arm** (formal ratification, still
     open): official Atlassian Rovo remote MCP via API token (RECOMMENDED per
     `115-ROVO-AUTH-CHECK.md`'s findings — auth proven end-to-end, no OAuth browser
     flow needed) vs fallback `sooperset/mcp-atlassian` (self-hosted, always
     API-token-only). This is the T4-executor's / manager's call to formally make, not
     a rubber stamp of the recommendation.
  3. Once the server choice is recorded AND the 2026-07-16 02:00 PT clock has passed,
     T4 can open: ≤18 sessions (median-of-3 × ≤3 backends × 2 arms), task = "read 3
     issues, edit 1, push"; mcp-mediated arm captures tool-list + tool-call/
     response payloads → replaces `benchmarks/fixtures/mcp_jira_catalog.json`;
     reposix-mediated arm runs the equivalent via a real reposix checkout in a
     **THROWAWAY `/tmp` clone** (leaf-isolation) → ANSI-stripped transcript
     replaces `benchmarks/fixtures/reposix_session.txt`; append ONE ledger row per
     session, increment `running_total`, assert ≤50 BEFORE next; scrub creds;
     targeted-add ONLY the two fixtures + ledger. **T4 is the context-blowing
     wave — run with fresh context, relieve if approaching ~100k own-context
     mid-wave.**
- **Pre-push over-budget WARN root-caused this rotation, recommendation FILED not
  APPLIED** (see §2/§4): re-baseline `quality/CLAUDE.md` § Cadences to ~75s + raise
  WARN threshold 90s→100s. Not this rotation's charter to apply (read-only diagnosis
  only) — leave for OP-8 drain or whoever next touches that doc.
- **File-size soft-ceiling WARNs** (waived until 2026-08-08, known bloat class
  `GTH-V15-21`): `115-PLAN.md` 32633B, `SURPRISES-INTAKE.md` now **31061B** (grew
  ~2.9kB this rotation from the new root-cause entry — still under the active waiver,
  carried bloat, not a new item), `GOOD-TO-HAVES.md` ~30.6kB, `260715-mk5-PLAN.md`
  22.5kB. A progressive-disclosure split is needed eventually — not this rotation's
  job, just carried forward.
- **`docs/development/roadmap.md` stale internal snapshot — ALREADY FILED (2026-07-15,
  #35's rotation).** Do not re-file.
- **`link-resolution` reads `docs/index.md` twice** (cosmetic double-count) —
  documented inline in the script's code comment, harmless, not filed as a defect
  (too trivial). Do not re-file.
- **latency.md regeneration-clobber tension — still OPEN, unchanged.**
  `emit-markdown.sh` regenerates `latency.md` from a LOCAL bench run, would clobber
  the CI-canonical figures corrected in #32/#33. Reconcile in T5/T6.
- **latency.md is doc-alignment-TRACKED** — the eventual T6 headline reframe
  RE-DRIFTS its 14 rows. **Budget a SECOND
  `/reposix-quality-refresh docs/benchmarks/latency.md` BEFORE the T6 phase-close
  push.** Grep `quality/catalogs/doc-alignment.json` for ANY doc before editing it.
- **FOUR `[SELF]` ledger entries pending deletion at T6 phase-close** (each entry's own
  text conditions its deletion on "the phase closes"): the A1 definition entry, the
  P115-T2 latency-canonical-source entry, the P115-T6 headline-framing entry, and the
  P115-T5 JSONL-usage-methodology entry. Delete all four, not three, at T6.
- **Carry item (needs owner/manager DOCTRINE CALL, do NOT merge unilaterally):**
  GOOD-TO-HAVES consolidation (two coexisting files) — pending todo at
  `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`.
- **Weekly subscription-limit watch:** T4 spends LIVE subscription sessions once it
  opens post-reset — a limit-stall risk; surface to MANAGER immediately if hit.
- **mcp-mermaid MCP server was UNREACHABLE in #35's rotation** — re-check reachability
  before assuming it's still down next time a mermaid-diagram task needs it; #36 did
  not touch mermaid tooling this rotation so has nothing new to report here.
- **Background shells/monitors: NONE running** — nothing left open for #37 to inherit.

## 6. Precise next steps (successor runbook)

1. **FIRST ACT — confirm CI green on the tip that exists AFTER the L0 pushes this
   handover commit (plus the two prior unpushed commits `5374fe0`, `fcddf90`).**
   - `git rev-list --left-right --count HEAD...origin/main` — confirm 0 ahead / 0
     behind (i.e. the push landed).
   - `gh run list --branch main --workflow CI --limit 3` — wait for the top row to
     reach `completed`.
   - Then `python3 quality/runners/run.py --cadence post-push --persist` — the
     `code/ci-green-on-main` (P0) probe asserts main's NEWEST `ci.yml` run
     concluded success (not merely that some older green run exists).
   - If the flaky `test` CI job goes red, re-run it ONCE before treating it as a
     real regression. If still red after one re-run, STOP — do not open a wave
     over a red main; escalate per the retro/incident norms.
   - Note: all three commits (`5374fe0`, `fcddf90`, and this handover commit) are
     `.planning/`-only — if `ci.yml`'s path filters skip pure-`.planning/` diffs, no
     new run will trigger at all, and the last-confirmed-green run stays
     `29460132017` on `1b20c15`. Verify which case applies (check the workflow's
     `paths`/`paths-ignore` filters or just observe whether a new run appears);
     do not assume either way.

2. **Check the 2026-07-16 02:00 PT clock** (`TZ='America/Los_Angeles' date '+%Y-%m-%d
   %H:%M %Z'`). T4 (and ONLY T4 — any live-MCP capture session) is HARD-STOPPED until
   then, no exceptions, REGARDLESS of the auth-blocker having been refuted this
   rotation.
   - If it hasn't passed yet: do NOT start T4. Check
     `.planning/todos/pending/` for any new owner-queued interleave work (none known
     to exist right now). Otherwise use the time for pre-work only: formally record
     the MCP-server choice (§5 item 2 — recommended official Rovo via API token, but
     this still needs an explicit ratifying note, not just a rubber stamp), re-read
     `115-PLAN.md` Task 4's exact shape, or review this handover with the manager.
   - If it has passed: proceed to step 3.

3. **Once the clock has passed AND the MCP-server choice is formally recorded** →
   confirm `ATLASSIAN_API_KEY` (or equivalent) works per `115-ROVO-AUTH-CHECK.md`
   (never print the key value itself), then execute T4 (the context-blower — relieve
   if approaching ~100k own-context mid-wave), then T5, then T6, then phase-close:
   - **T5**: implement the JSONL-usage path in
     `quality/gates/perf/bench_token_economy.py` (headline source = session-analyzer
     parse of captured JSONL records per the ruling in §2/§5 of the prior handover and
     `115-PLAN.md`'s `<amendment id="jsonl-usage-methodology">`), demote
     `count_tokens` to optional enrichment, regenerate `token-economy.md` + methodology
     note, update the Task-5 `<automated>` check, offline-cache-stable, honest
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

4. **Optional, non-blocking, take only if convenient before T4 opens:** apply the
   filed-not-applied pre-push-budget recommendation (§5) — re-baseline
   `quality/CLAUDE.md` § Cadences pre-push budget to ~75s, raise WARN 90s→100s. Small,
   low-risk, but not required before T4; do not let it delay the clock-check in step 2.

5. **Carry item (still OPEN, needs manager/owner DOCTRINE CALL, do NOT merge
   unilaterally):** GOOD-TO-HAVES consolidation (two coexisting files).
