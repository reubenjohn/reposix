# SESSION-HANDOVER.md — v0.15.0 Floor: T4 live GitHub capture LANDED (6/6, CI green) — T5 token-economy regen READY — 2026-07-16

Written by **workhorse #39** (L0 orchestrator), relieving to successor **#40**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#38→#39's handover,
superseded here). #39 relieved at a clean wave boundary — T4 done, pushed, CI green —
per the standing "relieve past ~100k soft / 150k hard, absolute not %" rule.

**Read order:** this file → §0 ground truth (verify live) → §1 headline (T4 DONE) → §2
measured medians → §6 findings #40 must respect → §7 runbook.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate document,
separate owner — the manager, pane w1:p7). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a leaf.

**MODEL NOTE (new this rotation, load-bearing for dispatch):** the session model was
switched to **Fable 5** mid-#39 via `/model` (saved as default). If #40 runs on fable at
top level, delegate per fable-top-level doctrine — **fable coordinators only**, explicit
model overrides at leaves (opus complex / sonnet default / haiku mechanical), **NEVER fable
at a leaf**.

## 0. Ground truth (git)

**Verify live before acting:**
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```

**As of #39's handover commit:**
- `HEAD` == `origin/main` == **`bf43c2c`** (before this handover commit lands; this
  handover is the next commit atop it and #40's first-act re-verifies 0/0 after L0 pushes).
- Chain atop `db9b1dc` (#38's relief handover, last known-clean sha):
  1. `4db6b64` — `bench(115): T4 live GitHub token capture — 6 real sessions, median-of-3 ×
     2 arms`. **This is the T4 capture payload** (fixtures + captures + ledger rows).
  2. `40613f8` — `docs(planning): record T4 GitHub-capture outcome — server-choice,
     PROGRESS, intakes`. Rewrote PROGRESS NOW/NEXT to T5, updated
     `115-MCP-SERVER-CHOICE.md`, filed the two findings in the intakes.
  3. `bf43c2c` — `docs(planning): reconcile T4 reposix medians to authoritative captures`.
     Median reconciliation.
- **CI:** run **`29476979095`** on `bf43c2c` — **completed / success** (verified live by
  #39). P0 `code/ci-green-on-main` PASS both BEFORE and AFTER the T4 push (exit 0, `--persist`).
- **This handover commit + PROGRESS SHA-fill + one GTH row will be pushed BY L0** (not by
  #39 — L0 pushes). #40's §0 first-act re-verifies the tip independently.
- Milestone **v0.15.0 "Floor"**, phase **P115 executing** (`Execution mode: top-level`).
  P114 CLOSED GREEN (`dc26302`).
- Working tree clean at handover time; **no background shells or monitors** left running
  for #40 to inherit.
- **#40's FIRST ACT (before anything else):**
  ```
  git rev-list --left-right --count HEAD...origin/main   # expect 0/0
  gh run list --branch main --workflow CI --limit 3      # expect top row completed/success
  python3 quality/runners/run.py --cadence post-push --persist   # P0 ci-green-on-main
  ```
  If the flaky `test` job is red, re-run it ONCE before treating it as real. If still red
  after one retry, STOP — do NOT open T5 execution over a red main; escalate per §8.

## 1. THE HEADLINE: T4 is DONE + PUSHED + CI GREEN (live GitHub capture landed)

- **6/6 valid capture sessions** — median-of-3 × 2 arms, all `--model sonnet` nested
  `claude -p` subprocesses, identical task **"read 3 issues (#56, #57, #60), edit 1 (#60
  body marker), push"** against the sanctioned OP-6 target **`reubenjohn/reposix`**.
- Session ledger `benchmarks/bench-session-ledger.md` at **7/50** (smoke row 1 + 6 T4 rows,
  rows 2–7). CAPTURE_OK green — re-verified independently by L0 #39.
- **Secret scans clean** — L0 batch `grep` over staged files (`Bearer `, `token`, `api_key`,
  literal `GITHUB_TOKEN`) + pre-push gitleaks, both clean.
- **Honesty guards held:**
  - MCP arm JSONL shows real `mcp__github-probe__{issue_read,issue_write,search_issues}`
    calls in all 3 runs.
  - reposix arm ran `--strict-mcp-config` with NO `--mcp-config` → **zero MCP servers
    loaded, zero `mcp__*` calls** in the JSONL (pure git/POSIX). This was a **[SELF]-level
    methodological hardening** this rotation (recorded in `115-MCP-SERVER-CHOICE.md`) —
    strengthens validity, changed no scope.
- **Backend left clean** — only issue #60 was touched; restored byte-for-byte to its
  pristine original after the runs.

## 2. Measured medians (reposix vs MCP — REAL numbers, from committed captures)

Medians of 3, from `benchmarks/captures/*.json` (6 files):

| Axis | reposix arm | MCP arm | reposix advantage |
|---|---|---|---|
| Output tokens | ~1.2k | ~21.2k | **~94% fewer** |
| Cache-create tokens | ~19.1k | ~56.1k | ~66% fewer |
| Total input-context | ~245k | ~550k | ~56% fewer |
| Cost / session | ~$0.21 | ~$0.83 | ~75% cheaper |

**No single live metric equals the published 85.5%.** The exact headline "% fewer tokens"
figure is **T5's to define** from the committed captures (each file = `session_id` + token
counts + turns + cost + tool-call names; offline-CI-stable, no body content). The
per-backend 85.5% GitHub figure in `docs/benchmarks/token-economy.md` came from the OLD
synthetic methodology — T5 must pick and justify an honest headline from the medians above
(output-token reduction ~94% vs input-context ~56% vs cost ~75%).

**GTH-V15-25 step 1 DONE** — `benchmarks/fixtures/reposix_trajectory.json` committed. The
CI byte-tripwire (the REST of GTH-V15-25) remains **NOT implemented by design** (post-T4).
**`mcp_github_catalog.json`** (44 tools, real surface) committed; **`mcp_jira_catalog.json`**
retained as the honest 3-tool atlassian-rovo record; **`reposix_session.txt`** replaced with
the real GitHub transcript (8041 bytes, ANSI-stripped, no `/mnt/` or `scripts/demo.sh`).

## 3. PROGRESS.md refresh contract (owner directive — carry into EVERY future handover)

- `.planning/PROGRESS.md` is the **owner's live-watch surface**: an ordered **SHIPPED → NOW
  → NEXT** pipeline the owner watches items move through. It is a middle-altitude view
  (outsider-recognizable deliverables), **not** a task tracker.
- **REFRESH DISCIPLINE (load-bearing):** EVERY boundary commit that closes a
  task/wave/capture-batch updates `PROGRESS.md` **in the SAME push** — a shipped item moves
  NEXT→SHIPPED with its landing SHA, the NOW line is rewritten to the current focus, NEXT is
  trimmed to what's actually queued next. **Every relief handover refreshes it.** A stale
  `PROGRESS.md` is worse than no `PROGRESS.md` — it actively misleads the owner. Route
  `PROGRESS.md` edits through `/gsd-quick` or a delegated executor; it's a planning
  artifact, not a hand-edit target.
- This contract is part of the SESSION-HANDOVER replacement obligation — #40 and every
  successor MUST carry it forward in their own handover, verbatim if unchanged.
- **This rotation:** the T4 SHIPPED row's landing SHA is filled to `4db6b64` and NOW reads
  as **T5 (token-economy regen)** (the T4 executor rewrote NOW/NEXT at `40613f8`).

## 4. Wave/cycle state

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
| Pre-work (#37) | T4 MCP-wiring mechanism viability probe | DONE — RESOLVED: MCP arm IS wireable headlessly | — |
| Pre-work (#38) | T4 Jira/atlassian-rovo feasibility probe | DONE — REFUTED: no CRUD tool, token authz-denied, KAN empty. Path DEAD | `ece072f` |
| Pivot (#38) | T4 backend pivot to GitHub [SELF] | DECIDED — recipe grounded, mechanism proven | — |
| Owner directive (#38) | PROGRESS.md owner-watch pipeline stood up | DONE + PUSHED | `de06e00` |
| **Wave 3 / T4** | **Live-MCP token capture, GitHub arm** | **DONE + PUSHED + CI GREEN — 6/6 valid, ledger 7/50, CAPTURE_OK** | `4db6b64`, `40613f8`, `bf43c2c` |
| **Wave 4 / T5** | **Token-economy JSONL-usage regen** | **READY TO EXECUTE — captures committed, methodology RULED (`9be5439`), no upstream blocker** | — |
| **Wave 5 / T6** | Un-waive + headline reframe + phase-close (delete FIVE `[SELF]` entries) | blocked on T5 only | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | blocked on P115 close | — |

### What #39 did this rotation
- Verified ground truth inherited from #38 (`db9b1dc`, CI green, P0 PASS).
- Dispatched the T4 capture to a fresh-context executor (opus). Landed `4db6b64` (6 real
  sessions), `40613f8` (server-choice note, PROGRESS, intakes), and — after reconciling the
  reposix-arm medians to the authoritative captures — `bf43c2c`. Pushed; confirmed CI green
  (`29476979095`) and P0 PASS after the push.
- Applied a [SELF] methodological hardening: reposix arm run under `--strict-mcp-config`
  with zero MCP config, so its JSONL provably carries zero `mcp__*` calls (recorded in
  `115-MCP-SERVER-CHOICE.md`).
- Verified CAPTURE_OK, secret scans (batch grep + gitleaks), and backend cleanliness (#60
  restored byte-for-byte) independently.
- Writing + committing this handover, the PROGRESS SHA-fill (`<pending>`→`4db6b64`), and the
  GTH-V15-27 row (filing the #38-noticed ROADMAP-stub drift) — for L0 to push.

## 5. Litmus / gate / REOPEN state

- **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until
  2026-08-15** — T6 un-waives after the T5 re-measure lands (now GitHub-sourced).
- **File-size soft-ceiling WARNs** (waived until 2026-08-08, class `GTH-V15-21`):
  `115-PLAN.md` ~32.6kB, `GOOD-TO-HAVES.md` now **~35.5kB**, `SURPRISES-INTAKE.md` now
  **~36.6kB** — both intakes grew this rotation (T4 finding appends); the pre-existing
  waiver class covers them. Progressive-disclosure split eventually, not urgent.
- **Pre-push budget WARN** re-baseline (~55s→~75s, WARN 90s→100s) still **FILED not
  APPLIED** (#36 diagnosis, `SURPRISES-INTAKE.md` 2026-07-15 17:18 entry). Apply during the
  OP-8 drain.
- **`reposix_session.txt.tokens.json` sidecar STALE** — the fixture was replaced (real
  transcript) but no `ANTHROPIC_API_KEY` was available to regen the sidecar honestly.
  Non-blocking: the consuming bench (`bench_token_economy.py --offline`) is NOT in `ci.yml`,
  its catalog row `perf/token-economy-bench` is WAIVED until 2026-09-15. Filed under
  **GTH-V15-26** (T5 makes it obsolete by moving to the captures methodology).
- **No REOPEN state pending.**

## 6. Findings #40 must respect (from the T4 executor, filed in the intakes at `40613f8`)

1. **reposix→GitHub write-back is READ-ONLY in this cut.** `crates/reposix-github/src/lib.rs`
   `create/update/delete_record` return not-supported (`:654/666/677`); documented in
   `crates/reposix-cli/src/doctor.rs:1467` ("github: read=yes, create/update/delete=— (read-
   only in this cut)"). The reposix arm did read+edit+local-commit+push-ATTEMPT; the push was
   **correctly rejected** with the documented read-only error. The token comparison is
   UNAFFECTED (it measures agent context size, not write capability). **MEDIUM
   SURPRISES-INTAKE entry** (2026-07-16 06:05) filed — **T6 honesty framing must NOT claim
   reposix push write-back works on GitHub**; the route decision (implement writes vs doc
   audit) is OPEN, for L0 to route.
2. **github-probe `issue_read` is LOSSY on reads.** It HTML-escapes body content
   (`>=` → `&gt;=`) and drops literal angle-bracket content (incl. HTML comments);
   `issue_write` stores raw fine — so the MCP arm's read-modify-write **corrupted #60
   mid-run** (executor restored it). Strong live evidence FOR reposix bytes-in-bytes-out
   fidelity — a **T6 positioning talking point**, filed under GTH-V15-26.
3. **Sidecar staleness** — see §5 (GTH-V15-26).

Also: `115-MCP-SERVER-CHOICE.md` now records `github-probe` as the RESOLVED live-capture
choice, with `atlassian-rovo` retained as the infeasible-attempt evidence trail (not deleted).

## 7. Precise next steps (successor #40 runbook)

1. **FIRST ACT — the §0 verify block** (rev-list 0/0, `gh run list` top row green, post-push
   `--persist` P0 probe). Flaky `test` job → re-run ONCE; still red → STOP, never open T5
   over a red main.
2. **T5 — dispatch a fresh-context executor** (explicit model override, e.g. opus): implement
   the JSONL-usage path in `quality/gates/perf/bench_token_economy.py` per the RULED
   methodology (`9be5439`) — headline via a session-analyzer-style parse of the committed
   `benchmarks/captures/*.json`, **NOT** `count_tokens`, **no `ANTHROPIC_API_KEY`**. Preserve
   the `bench_token_economy_io.py` re-export surface + the module-level
   `FIXTURES`/`BENCH_DIR`/`RESULTS` monkeypatch contract. Regen
   `docs/benchmarks/token-economy.md` offline-CI-stable from committed captures; match README;
   catalog-first if a perf-row contract changes. `scripts/bench_token_economy.py` is a shim
   (**not** a symlink). **T5 MUST define the honest headline metric** from §2's medians
   (output-token ~94% vs input-context ~56% vs cost ~75% — pick + justify; the published
   per-backend 85.5% GitHub figure came from the OLD synthetic methodology).
3. **T6** (`115-UNWAIVE-PATH.md`) — the headline-framing decision (re-anchor the hero to the
   live GitHub number, OR keep 89.1% with an explicit GitHub-regime caveat) MUST fold in
   **finding 1** (no GitHub write-back claims) + **finding 2** (fidelity positioning). Then:
   a SECOND `/reposix-quality-refresh docs/benchmarks/latency.md` (the headline reframe
   re-drifts its 14 doc-alignment rows — grep `quality/catalogs/doc-alignment.json` BEFORE
   editing docs), the regen-clobber guard (`emit-markdown.sh` must NOT overwrite CI-canonical
   latency sections), un-waive the 8 hero rows, and **DELETE the FIVE `[SELF]` ledger entries**
   in `CONSULT-DECISIONS.md` (A1 definition, T2 latency-canonical, T6 headline-framing, T5
   JSONL-methodology, **and** T4 GitHub-pivot — captures have landed, so the pivot entry is
   deletable at this sweep).
4. **Phase-close cadence:** `git push origin main` BEFORE verifier dispatch → post-push
   `--persist` P0 (`ci-green-on-main`) → verifier subagent for catalog-row PASS → advance
   `.planning/STATE.md` cursor → refresh `PROGRESS.md` in the close push → never open the
   next phase over a red main.
5. **P116** (after P115 closes): produce the ADR-010 packet (ADR-01 mirror-fanout + FIX-03
   GTH-09 slug→id durable-create, options + tradeoffs) and route it to the **MANAGER (w1:p7)
   for ruling — NO pre-ruling implementation.**
6. **If the weekly subscription cap hits mid-work:** commit+push whatever landed, REPLACE
   this handover, refresh `PROGRESS.md`, end cleanly. Reset-gating is RETIRED (owner ruling,
   `c7cea90`) — never defer or schedule work AROUND a reset; only react to a cap that hits.

**MCP state (outside the git tree, load-bearing):** `github-probe` registered + connected —
**leave it** (evidence surface; T5 does not need it live). `atlassian-rovo` still registered
— safe to `claude mcp remove atlassian-rovo` after P115 closes, discretionary.
`mcp-mermaid` still **DOWN** — re-check before any diagram task.

## 8. Binding constraints (carry verbatim, unchanged)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at a
leaf** (and if #40 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE); relieve past ~100k own-context (hard 150k, absolute not %) at a wave boundary;
push at green, then confirm `code/ci-green-on-main` P0 AFTER push; never open the next phase
over a red main; reset-gating RETIRED — never defer or schedule work for a weekly reset, only
react to a cap that actually hits (if it hits: commit+push, refresh this handover +
`PROGRESS.md`, end cleanly).
