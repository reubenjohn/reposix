# SESSION-HANDOVER.md — v0.15.0 Floor: T4 MCP-wiring mechanism RESOLVED (arm is wireable), owner LIFTED the hard-stop, T4 ready to execute with fresh context — 2026-07-15

Written by **workhorse #37** (L0 orchestrator), relieving to successor **#38**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#36→#37's handover,
superseded here). #37 relieved at a clean wave boundary **before** T4 (the explicit
context-blower) at ~111k own-context, per the standing "relief at ~100k soft" rule and the
prior handover's own "run T4 with fresh context" mandate.

**Read order:** this file → §1 (verify live) → §5 (T4 execution recipe — the payload) →
§6 runbook.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate document,
separate owner — the manager, pane w1:p7). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a leaf.

## 0. THE OWNER DIRECTIVE THAT GOVERNS THIS AND YOUR SESSION (2026-07-15 ~20:40 PT)

Relayed by the manager: **start T4 NOW — do NOT wait for the 2am PT reset — and drive P115
to close (T4→T5→T6→phase-close→P116 packet).** This **LIFTS the T4 hard-stop** the prior
(#36→#37) handover documented (which was "HARD-STOPPED until 2026-07-16 02:00 PT"). The live
owner word supersedes that stale clock gate. Treat T4 as OPEN.

**CAP RULE (active, load-bearing):** we are on the **last ~20% of this week's subscription**.
Every nested `claude -p` capture session AND every subagent you spawn spends that budget. If
the weekly cap hits mid-work: **immediately commit+push all progress, REPLACE this session
handover, and end your turn cleanly** — your successor resumes after the reset. Be frugal:
prefer the MINIMUM viable capture set (see §5).

## 1. Ground truth (git) — verify live before acting

Re-run first:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```
**Verified by #37 at rotation start AND end (2026-07-15 ~21:25 PT):**
- `HEAD` = `065c0b4` (the #36→#37 manager-handover refresh commit). Tree CLEAN. **0 ahead /
  0 behind `origin/main`** — main == origin/main. #37 made ZERO commits before this
  handover (see §2); this handover commit is the FIRST of #37's rotation.
- **CI on `065c0b4` is CONFIRMED `completed`/`success`** (run `29462093167`, verified live).
  Pure-`.planning/` commits DO trigger CI here and go green — this handover commit will too.
- **P0 `code/ci-green-on-main` post-push probe: PASS** (exit 0, run by #37 via
  `python3 quality/runners/run.py --cadence post-push --persist`). Green-on-main persisted.
- After this handover commit lands + is pushed, #38's first act is to reconfirm CI green on
  the new tip (§6 step 1).

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session-definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE + PUSHED | `9384ca6`, `3845b13` |
| Refresh-recovery (#33) | `/reposix-quality-refresh docs/benchmarks/latency.md` | DONE + PUSHED | `92c3ab5` |
| Wave 2 / T3 | Session-spend ledger scaffold | DONE + PUSHED | `4351d48` |
| Interleave (#35) | Public roadmap diagram gsd-quick | DONE + PUSHED | `1db48e4`, `16fb356`, `fa58ad6` |
| Methodology (#35) | T5 JSONL-usage token-economy methodology [SELF] + 115-PLAN amendment | RULED (not executed) | `9be5439` |
| Pre-work (#36) | READ-ONLY Rovo MCP **auth** check | DONE + PUSHED — auth blocker REFUTED (HIGH) | `5374fe0` |
| Pre-work (#36) | Pre-push over-budget spike diagnosis | DONE + FILED (not applied) | `fcddf90` |
| **Pre-work (#37)** | **T4 MCP-WIRING mechanism viability probe** | **DONE — RESOLVED: MCP arm IS wireable headlessly (see §5). No commit (read-only + external config).** | — |
| Wave 3 / T4 | Live-MCP token capture (both arms) | **READY TO EXECUTE** — hard-stop LIFTED by owner; auth proven (#36); wiring proven (#37); `atlassian-rovo` MCP pre-registered | — |
| Wave 4 / T5 | Token-economy JSONL-usage regen | METHODOLOGY RULED, blocked downstream on T4 captures | — |
| Wave 5 / T6 | Un-waive + headline reframe + phase-close (delete 4 `[SELF]`) | blocked downstream on T4/T5 | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | NOT STARTED (blocked on P115 close) | — |

### What #37 did this rotation (all read-only / config — ZERO repo commits before this handover)
- Verified ground truth, CI green (`29462093167`), P0 post-push probe PASS.
- Verified real backends reachable: `bash scripts/preflight-real-backends.sh` → exit 0, PASS
  (Confluence/TokenWorld, GitHub `reubenjohn/reposix` open_issues=3, JIRA/KAN "My Kanban Space").
- Digested the exact T4 protocol + T5 generator state (see §5).
- **RESOLVED the T4 MCP-arm mechanism gap** — the blocker beyond auth. Found NO Atlassian MCP
  was wired anywhere (session or CLI). Proved the official Atlassian remote MCP wires into
  Claude Code HEADLESSLY via Bearer API-token and connects. **Registered `atlassian-rovo`** as
  the T4 prerequisite (left registered — see §5). No backend writes, no capture session spent.

## 3. Binding constraints (carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no `--no-verify`;
targeted staging (never `-A`/`.`); don't touch `.planning/MANAGER-HANDOVER.md`; no tag push;
no git surgery on main; leaf isolation in `/tmp` same-invocation; opus complex / sonnet default
/ haiku mechanical, never fable at a leaf; relieve past ~100k own-context (hard 150k, absolute
not %) at a wave boundary; push at green, then confirm `code/ci-green-on-main` P0 AFTER push;
never open the next phase over a red main. **CAP RULE (§0) is now the tightest active
constraint — watch subscription budget.**

## 4. Litmus / gate / REOPEN state
- CI on tip `065c0b4`: GREEN (`29462093167`). P0 post-push probe: PASS.
- **Pre-push timing WARN root-caused (#36), recommendation FILED not APPLIED**: re-baseline
  `quality/CLAUDE.md` § Cadences pre-push budget ~55s→~75s + raise WARN 90s→100s. Apply during
  OP-8 drain or whoever next touches that doc. `SURPRISES-INTAKE.md` 2026-07-15 17:18 entry.
- **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until 2026-08-15**
  — T6 un-waives after T4/T5 re-measure.
- File-size soft-ceiling WARNs (waived until 2026-08-08, class `GTH-V15-21`): `115-PLAN.md`
  ~32.6kB, `SURPRISES-INTAKE.md` ~31kB, `GOOD-TO-HAVES.md` ~30.6kB. Progressive-disclosure
  split eventually — not blocking.
- No REOPEN state pending.

## 5. T4 EXECUTION RECIPE — the payload (mechanism now RESOLVED)

### 5a. MCP-arm wiring — RESOLVED (proven by #37, cap-cheap probe)
- **`atlassian-rovo` is ALREADY REGISTERED** (local scope, project `/home/reuben/workspace/reposix`,
  in `~/.claude.json`) and `claude mcp list` shows `✔ Connected`. Exact command used (idempotent
  to re-run from the repo dir if it ever disappears):
  ```
  claude mcp add --transport http atlassian-rovo https://mcp.atlassian.com/v1/mcp \
    --header "Authorization: Bearer $ATLASSIAN_API_KEY"
  ```
  (Claude Code auto-redacts the header value. `ATLASSIAN_API_KEY` is in the repo `.env` — never
  echo its value.) Remove with `claude mcp remove atlassian-rovo` from that dir if needed.
- **No browser OAuth needed** — the Bearer API-token path proven at the `initialize` layer in
  `115-ROVO-AUTH-CHECK.md` works end-to-end through Claude Code's own MCP client.
- **CAVEAT for the capture harness:** a bare fast `claude -p` may return BEFORE `mcp__atlassian*`
  tools finish loading (print-mode startup race — the nested session DID inherit `atlassian-rovo`
  but answered mid-handshake). The capture task MUST let MCP servers reach ready before issuing
  the benchmark prompt — warm-up/retry, or use a longer agentic run. Do NOT trust a sub-few-second
  reply as "no tools."
- **Formal MCP-server choice** (plan Task-1's still-technically-open ratification): official
  Atlassian Rovo remote MCP via API-token Bearer. #37 RECOMMENDS + PROVED it. #38/T4-executor
  should record the formal ratifying note in the phase dir (grounded now, not a rubber-stamp).

### 5b. The measurement (from digested `115-PLAN.md` Task 4 + `115-RESEARCH.md`)
- **Adopted methodology = JSONL-usage** (H1-flavored: one session = one live agentic
  conversation; headline = captured Claude Code **session JSONL usage records** parsed by the
  `session-analyzer` skill; `count_tokens` demoted to optional enrichment). JSONL home:
  `~/.claude/projects/-home-reuben-workspace-reposix/`.
- **Task per session:** "read 3 issues, edit 1, push" against sanctioned targets: **KAN** (Jira),
  **TokenWorld** (Confluence), **`reubenjohn/reposix`** (GitHub, 3 open issues).
- **MCP-mediated arm:** nested `claude -p` capture session WITH `atlassian-rovo` wired (warm-up
  per §5a caveat) → capture real tool-list + tool-call/response payloads → replace
  `benchmarks/fixtures/mcp_jira_catalog.json`. JSONL usage = headline MCP cost.
- **reposix-mediated arm:** run the equivalent task via a real reposix checkout in a **THROWAWAY
  `/tmp` clone** — leaf-isolation, `cd "$(mktemp -d)" && reposix init <backend>::<project> . && …`
  in the SAME Bash invocation, never the shared repo. ANSI-strip the transcript → replace
  `benchmarks/fixtures/reposix_session.txt`. For JSONL symmetry, ALSO run this arm as a real
  agentic session so both arms have comparable JSONL headline numbers. COPY (not git-mutate) the
  scrubbed transcript into the shared tree before `git add`.
- **≤18 = median-of-3 × ≤3 backends × 2 arms.** GIVEN THE CAP RULE: **START MINIMAL** — 1 backend
  (Jira/KAN, matches the `mcp_jira_catalog` fixture) × median-of-3 × 2 arms = **6 sessions**;
  expand to more backends ONLY if budget clearly allows. Never exceed the ≤50 ledger ceiling.
- **Ledger** `benchmarks/bench-session-ledger.md` (empty scaffold, 0/50): append ONE row per
  session, in order, columns `# | timestamp (UTC ISO-8601) | backend | arm (mcp-mediated /
  reposix-mediated) | task | unit_consumed | running_total | artifact_produced`. Increment
  `running_total`; **verify ≤50 BEFORE starting the next session.** Flag any session >~5× the
  running median. Do NOT backfill.
- **Cred scrub** all transcripts (OAuth tokens / API keys) before commit. **Targeted-add ONLY:**
  `benchmarks/fixtures/mcp_jira_catalog.json`, `benchmarks/fixtures/reposix_session.txt`,
  `benchmarks/bench-session-ledger.md`.
- **Acceptance (`115-PLAN.md` ~347-350):**
  ```
  ! grep -qE '/mnt/|scripts/demo\.sh' benchmarks/fixtures/reposix_session.txt && \
    <ledger has ≥2 rows> && <tail row running_total ≤ 50> && echo CAPTURE_OK
  ```
- **Preflight** (already PASS this rotation): `bash scripts/preflight-real-backends.sh` exit 0.
- **T4 is the context-blower** — run with FRESH context; relieve if approaching ~100k mid-wave.

### 5c. FOLD-IN during T4 (owner directive #2, <1h byproduct)
While the JSONL data is fresh: extract the agent command list from a captured session JSONL into
a committed **trajectory fixture** — GTH-V15-25 **step 1 ONLY**. The REST of GTH-V15-25 (the
token-bloat CI tripwire, `e1c71c4`) stays a post-T4 lane — do NOT implement it now.

## 5.5 Downstream (T5 / T6 / close / P116) — owner directives #3-#6
- **T5** (`115-PLAN.md` `<amendment id="jsonl-usage-methodology">`, `9be5439`): implement the
  JSONL-usage path in `quality/gates/perf/bench_token_economy.py` (headline = `session-analyzer`
  parse of captured JSONL; demote `count_tokens` to optional enrichment). **Preserve the
  re-export surface** (`bench_token_economy_io.py`) + the module-level `FIXTURES`/`BENCH_DIR`/
  `RESULTS` test-monkeypatch contract. Regen `docs/benchmarks/token-economy.md` + methodology
  note; keep reruns offline-cache-stable on CI from committed fixtures; match README; catalog-first
  if a perf-row contract changes. `scripts/bench_token_economy.py` is a shim (differs from the
  quality one) — don't assume it's a symlink.
- **T6**: `115-UNWAIVE-PATH.md`; honest-headline reframe; **budget a SECOND
  `/reposix-quality-refresh docs/benchmarks/latency.md`** (the headline reframe RE-DRIFTS its 14
  doc-alignment rows — grep `quality/catalogs/doc-alignment.json` before editing any doc);
  un-waive the 8 hero-number rows; **delete ALL FOUR `[SELF]` ledger entries** (A1 definition,
  P115-T2 latency-canonical, P115-T6 headline-framing, P115-T5 JSONL-usage-methodology) once each
  is encoded per its own precondition.
- **latency.md regen-clobber tension (OPEN):** `emit-markdown.sh` regenerates `latency.md` from a
  LOCAL sim-only bench run and would CLOBBER the CI-canonical figures corrected in #32/#33. The
  local generator must NOT overwrite CI-canonical sections. Reconcile in T5 or explicitly defer to T6.
- **Phase-close cadence:** `git push origin main` BEFORE the verifier dispatch; then
  `python3 quality/runners/run.py --cadence post-push --persist` (`code/ci-green-on-main` P0);
  verifier subagent for catalog-row PASS; advance `.planning/STATE.md` cursor; RAISE-LIST/intake
  disposition; final report. Never open the next phase over a red main.
- **P116 (after P115 closes):** produce the ADR-010 packet (ADR-01 mirror-fanout + FIX-03 GTH-09
  slug→id durable-create options+tradeoffs) and route to the **MANAGER (w1:p7) for ruling — NO
  pre-ruling implementation.**

## 6. Precise next steps (successor #38 runbook)
1. **FIRST ACT — confirm CI green on the tip AFTER #37 pushed this handover commit.**
   `git rev-list --left-right --count HEAD...origin/main` (expect 0/0); `gh run list --branch
   main --workflow CI --limit 3` (top row `completed`/`success`); then
   `python3 quality/runners/run.py --cadence post-push --persist` (P0 asserts NEWEST `ci.yml`
   run = success). If the flaky `test` job goes red, re-run ONCE before treating as real; if
   still red, STOP — do not open T4 over a red main.
2. **Execute T4 per §5** (mechanism is fully resolved — `atlassian-rovo` already wired). Honor
   the CAP RULE: start minimal (6 sessions, Jira/KAN only), ledger-append one row at a time,
   assert ≤50 before each. Fold in GTH-V15-25 step 1 (§5c) while data is fresh. If the cap hits
   mid-capture: commit+push whatever landed, REPLACE this handover, end cleanly.
3. **Then T5 → T6 → phase-close → P116 packet** per §5.5.

## 7. Carry items / noticed (unchanged unless marked)
- **`atlassian-rovo` MCP left registered** in `~/.claude.json` (local scope, project reposix). This
  is OUTSIDE the repo tree (not a git change) and is the T4 prerequisite — leave it. Remove only
  post-P115 if desired (`claude mcp remove atlassian-rovo`).
- **`mcp-mermaid` MCP server is DOWN** — confirmed `✘ Failed to connect` this rotation. Re-check
  before any mermaid-diagram task; use a fallback if still down.
- **GOOD-TO-HAVES consolidation** (two coexisting files) — needs owner/manager DOCTRINE CALL, do
  NOT merge unilaterally. Todo: `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`.
- **Weekly subscription-limit watch:** T4 spends LIVE sessions — surface to MANAGER immediately if hit.
- **Background shells/monitors: NONE running** — nothing left open for #38 to inherit.
- Pre-work docs from prior rotations remain valid: `115-ROVO-AUTH-CHECK.md` (auth REFUTED),
  `SURPRISES-INTAKE.md` (pre-push budget root-cause). Do not re-run either diagnosis.
