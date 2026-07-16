# SESSION-HANDOVER.md — v0.15.0 Floor: T5 CLOSED (live-capture token-economy headline SHIPPED, CI green) — T6 READY — 2026-07-16

Written by **workhorse #40** (L0 orchestrator), relieving to successor **#41**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#39→#40's handover,
superseded here). #40 relieved at ~115k own-context, at the T5/T6 wave boundary — T5 done,
pushed, CI green — per the standing "relieve past ~100k soft / 150k hard, absolute not %"
rule.

**Read order:** this file → §0 ground truth (verify live) → §1 headline (T5 CLOSED) → §4
Wave B/T6 charter → §6 findings #41 must respect → §7 runbook.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate document,
separate owner — the manager, pane w1:p7). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a leaf.

**MODEL NOTE (unchanged from #39→#40, load-bearing for dispatch):** the session model is
**Fable 5**. If #41 runs on fable at top level, delegate per fable-top-level doctrine —
**fable coordinators only**, explicit model overrides at leaves (opus complex / sonnet
default / haiku mechanical), **NEVER fable at a leaf**.

## 0. Ground truth (git)

**Verify live before acting:**
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```

**As of #40's handover commit:**
- `HEAD` == `origin/main` == **`b460008`** (before this handover commit lands; this
  handover is the next commit atop it and L0 pushes it — #41's first-act re-verifies 0/0
  after the push).
- Chain atop `28a9c50` (#39's relief handover, last known-clean sha), all on `main`, all CI
  green:
  1. `5366d29` — `perf(115-05): rewrite token-economy-bench GREEN contract to JSONL-usage
     methodology`. Catalog-first rewrite (GREEN contract lands before the implementation,
     per project convention).
  2. `1cdb381` — `feat(115-05): JSONL session-usage headline path for token-economy bench`.
     The headline-computation path in `quality/gates/perf/bench_token_economy.py`.
  3. `fd098c7` — `docs(115-05): regen token-economy.md from live GitHub captures + honest
     provenance`. `docs/benchmarks/token-economy.md` regenerated + `PROGRESS.md` refresh.
  4. `211f794` — `docs(115-05): file T5 close-out noticings + minted_at fix-it-twice line`.
  5. `63cb505` — `chore(115-05): persist post-push code/ci-green-on-main grade`.
  6. `2103d0c` — `fix(115-05): restore literal content_hash term in fixtures README` (a
     `code/fixtures-valid` regression fix — the fixtures README had lost a literal
     `content_hash` term the gate asserts on).
  7. `b460008` — `chore(115-05): close T5 wave — re-mint post-push P0 grade at CI-green`.
     The T5 wave-close boundary commit — P0 grade re-minted PASS at the CI-green
     conclusion + `PROGRESS.md` wave-close note.
- **CI:** run **`29487716448`** on `b460008` — **completed / success** (verified live by
  #40 via `gh run view --json status,conclusion,headSha`). P0 `code/ci-green-on-main`
  PASS (minted in `b460008`).
- **This handover commit will be pushed BY L0** (not by #40 — L0 pushes). #41's §0
  first-act re-verifies the tip independently.
- Milestone **v0.15.0 "Floor"**, phase **P115 executing** (`Execution mode: top-level`).
  P114 CLOSED GREEN (`dc26302`).
- Working tree clean at handover time; **no background shells, monitors, or live
  subagents** left running for #41 to inherit.
- **#41's FIRST ACT (before anything else):**
  ```
  git rev-list --left-right --count HEAD...origin/main   # expect 0/0
  gh run list --branch main --workflow CI --limit 3      # expect top row completed/success
                                                           # (will be in-flight right after
                                                           # L0's push — watch it bounded,
                                                           # e.g. `gh run watch`, ≥300s timeout)
  python3 quality/runners/run.py --cadence post-push --persist   # P0 ci-green-on-main
  ```
  If the flaky `test` job is red, re-run it ONCE before treating it as real. If still red
  after one retry, STOP — do NOT open T6 execution over a red main; escalate per §8.

## 1. THE HEADLINE: T5 is CLOSED (live-capture token-economy headline SHIPPED + CI GREEN)

- **`docs/benchmarks/token-economy.md` regenerated** from the T4 live GitHub captures
  (`benchmarks/captures/*.json`, 6 files) via a deterministic, offline, no-`ANTHROPIC_API_KEY`
  JSONL session-usage parse — **NOT** `count_tokens`. Catalog-first: the GREEN contract
  (`5366d29`) landed before the implementation (`1cdb381`), per project convention.
- **RECOMPUTED headline (four axes, from committed captures):**

  | Axis | reposix advantage |
  |---|---|
  | Output tokens | ~94.3% fewer |
  | Cache-create tokens | ~66.0% fewer |
  | Total input-context | ~55.6% fewer |
  | Cost / session | ~74.9% cheaper |

  Top-line framing: **~94.3% fewer output tokens, ~74.9% cheaper per session.**
- **Synthetic 89.1%/85.5% figures RETIRED** with an explicit provenance note (old
  `count_tokens`-on-fixtures methodology). False `scripts/demo.sh` / "modeled on Forge"
  claims killed.
- **Honesty caveats added** (from the T4 findings, carried into the doc): (1)
  read-only-write-back — reposix's GitHub connector cannot push writes in this cut, the
  token comparison is unaffected; (2) MCP-lossy-reads — `github-probe issue_read`
  HTML-escapes/drops raw markdown, reposix round-trips bytes faithfully.
- **`reposix_session.txt.tokens.json` stale sidecar DELETED** — GTH-V15-26 RESOLVED (the
  JSONL-usage methodology made the sidecar obsolete rather than requiring a live-key regen).
- `bench_token_economy_io.py` re-export surface + the module-level
  `FIXTURES`/`BENCH_DIR`/`RESULTS` monkeypatch contract **preserved**.
  `scripts/bench_token_economy.py` remains a shim (not a symlink).
- **Close-out noticings filed** at `211f794` (includes a `minted_at` fix-it-twice line —
  the P0 persist script's minted-timestamp handling was tightened in the same commit that
  noticed it, per OP-3-adjacent "fix it twice" discipline).
- **`code/fixtures-valid` regression caught + fixed same-wave** (`2103d0c`) — a literal
  `content_hash` term the gate asserts on had been dropped from the fixtures README during
  the regen; restored before the wave-close commit.

## 2. What #40 did this rotation

1. First-act verify inherited from #39: rev-list 0/0, CI green (`29477874579`), P0 PASS.
2. Confirmed GTH-V15-25 step-1 fixture (`benchmarks/fixtures/reposix_trajectory.json`)
   already committed at `4db6b64` — no salvage needed. Reconciled the owner's "four
   `[SELF]` entries" instruction against the actual ledger — **FIVE** exist (verified
   live: `grep -n '\[SELF\]' .planning/CONSULT-DECISIONS.md` → lines 71, 96, 114, 123,
   153); the fifth (T4 GitHub-pivot, line 153) self-documents "delete when T4 captures
   land" — T6 deletes all five, not four.
3. Dispatched T5 to a fable phase-coordinator (opus executor at the leaf). Ruled a
   mid-wave scope question: the plan wins — `benchmarks/fixtures/README.md` regen IS in
   T5 scope; L0's earlier "token-economy.md only" framing was a regen-clobber guard
   against overwriting CI-canonical sections, not a scope cut.
4. **MANAGER HEALTH-CHECK RECOVERY (named incident):** the C1→executor chain died
   silently post-push, mid-`--persist` (symptom: `TaskOutput: no task found`; the
   notification chain broke). L0 recovered per manager instruction: the dying executor's
   last persist had honestly graded **NOT-VERIFIED** (written 3s after CI run
   `29482425246` STARTED, before it could conclude). L0 re-minted PASS only after
   independently confirming the run concluded success, then landed the wave-close
   boundary commit `b460008` and watched CI to green with a bounded `gh run watch`.
5. Relieved at ~115k own-context at the T5/T6 wave boundary per the standing ~100k soft
   rule (the manager had said "proceed to Wave B" pre-context-check; the absolute-context
   rule governs regardless — Wave B is chartered below for #41 to dispatch fresh instead).

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
- This contract is part of the SESSION-HANDOVER replacement obligation — #41 and every
  successor MUST carry it forward in their own handover, verbatim if unchanged.
- **This rotation:** `PROGRESS.md` is **current as of `b460008`** (verified live by #40 —
  the T5 SHIPPED row carries the `1cdb381` landing SHA with the wave-close annotation, NOW
  reads as **T6** with the full un-waive/reframe/retire scope, NEXT is P115-close →
  P116 → P117-128). #40 made **no edits** to `PROGRESS.md` — it was already fresh from the
  T5 executor's `fd098c7`/`b460008` commits. #41: verify freshness at first-act; edit only
  if stale.

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
| Wave 3 / T4 | Live-MCP token capture, GitHub arm | DONE + PUSHED + CI GREEN — 6/6 valid, ledger 7/50, CAPTURE_OK | `4db6b64`, `40613f8`, `bf43c2c` |
| **Wave 4 / T5** | **Token-economy JSONL-usage regen + live headline** | **CLOSED + PUSHED + CI GREEN — headline shipped, `code/fixtures-valid` regression caught+fixed same-wave** | `5366d29`, `1cdb381`, `fd098c7`, `211f794`, `63cb505`, `2103d0c`, `b460008` |
| **Wave 5 / T6** | **Un-waive + headline reframe + phase-close (delete FIVE `[SELF]` entries)** | **READY TO DISPATCH — no upstream blocker, full charter in §5 below** | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | blocked on P115 close | — |

## 5. Wave B (T6) — READY TO DISPATCH, full charter for #41

Owner directives in force: finish P115 end-to-end; reset-gating RETIRED (cap-hit →
commit+push, refresh handover+PROGRESS, clean end). PROGRESS.md refresh discipline (§3)
applies to every T6 boundary commit.

T6 work items (source: `PROGRESS.md` NOW section + owner charter + T5 close-out RAISE list):

1. **Headline-framing reframe of hero surfaces** (README, `docs/index`, `docs/why`):
   re-anchor to the live GitHub numbers (~94.3% output-token / ~74.9% cost) OR keep 89.1%
   with an explicit GitHub-regime caveat. NOTE: `token-economy.md` ALREADY retired the
   89.1%/85.5% figures with a provenance note, so re-anchoring is the
   consistency-preserving move; executing this decision also executes the T6-headline
   `[SELF]` entry (line 96 of `CONSULT-DECISIONS.md`). MUST fold in **finding 1** (no
   claim that reposix→GitHub push write-back works — read-only in this cut) and
   **finding 2** (MCP `issue_read` lossy vs reposix byte-fidelity — a positioning
   talking point), both in §6 below.
2. Write `115-UNWAIVE-PATH.md` (per `PROGRESS.md` NOW).
3. **Retire+rebind the 6 `token-economy.md` doc-alignment rows** T5 left
   `WAIVED-STALE_DOCS_DRIFT` until 2026-08-15 (76.4% / 85.5% / 4883 / 531 / 89.1% /
   jira-real-adapter). **CONSTRAINT: confirm-retire is human-only** — if the
   reposix-quality binary blocks the retire step, escalate to the **MANAGER (pane w1:p7)**
   rather than working around it.
4. **SECOND** `/reposix-quality-refresh docs/benchmarks/latency.md` **AFTER** the reframe
   edits land (the reframe re-drifts its 14 doc-alignment rows — grep
   `quality/catalogs/doc-alignment.json` BEFORE editing docs). Top-level-only skill — run
   from L0, not inside a coordinator.
5. Regen-clobber guard: `emit-markdown.sh` must NOT overwrite CI-canonical latency
   sections.
6. **Un-waive the 8 hero-number rows** (`WAIVED-MISSING_TEST` until 2026-08-15); un-waive
   `perf/token-economy-bench` by adding the ~94% headline assertion; **WRITE the absent**
   `perf/headline-numbers-cross-check` catalog row (grep `index.md` + README heroes
   against fixtures) — confirmed absent by #40 (`grep -c perf/headline-numbers-cross-check
   quality/catalogs/*.json` → 0 everywhere), T6's to write.
7. **DELETE all FIVE `[SELF]` entries** in `.planning/CONSULT-DECISIONS.md` (verified live
   by #40 at their actual line numbers: A1 line 71; T6-headline line 96;
   T2-latency-canonical line 114; T5-JSONL-methodology line 123; T4-GitHub-pivot line 153).
8. **Phase-close cadence:** `git push origin main` BEFORE verifier dispatch →
   `python3 quality/runners/run.py --cadence post-push --persist` P0 → gsd-verifier
   subagent grades catalog rows (RED loops back) → advance `.planning/STATE.md` cursor →
   `PROGRESS.md` refresh in the close push → never open the next phase over a red main.

**Post-close:** P116 ADR-010 packet (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id
durable-create; options + tradeoffs) → route to **MANAGER w1:p7 for ruling — NO
pre-ruling implementation.**

## 6. Litmus / gate / REOPEN state

- **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until
  2026-08-15** — T6 un-waives per §5 item 6.
- **6 `token-economy.md` doc-alignment rows** (76.4% / 85.5% / 4883 / 531 / 89.1% /
  jira-real-adapter) left **WAIVED-STALE_DOCS_DRIFT until 2026-08-15** by T5 — T6
  retires+rebinds per §5 item 3 (human-only confirm-retire).
- **`perf/headline-numbers-cross-check` catalog row is ABSENT** (verified live — 0
  matches across all `quality/catalogs/*.json`) — T6 writes it per §5 item 6.
- **File-size soft-ceiling WARNs** (waived until 2026-08-08, class `GTH-V15-21`):
  `115-PLAN.md` ~32.6kB, `GOOD-TO-HAVES.md` and `SURPRISES-INTAKE.md` both still over
  soft-ceiling — waiver class covers them. Progressive-disclosure split eventually, not
  urgent.
- **Pre-push budget WARN re-baseline is a LIVE regression, not just filed**: this
  rotation's pushes measured **~121s** (dominated by kcov shell-coverage + full-workspace
  clippy/mkdocs), up from the earlier ~55→75s trajectory — root CLAUDE.md's Push cadence
  section now documents `pre-push ≈55s` as stale; the actual figure has moved further.
  Still **FILED not APPLIED** (`SURPRISES-INTAKE.md` 2026-07-15 17:18 entry) — apply
  during the OP-8 drain. **Operational consequence for #41: always push with a Bash
  timeout ≥300s** (see §6 findings, item 2).
- **No REOPEN state pending.**
- **GTH-V15-26 RESOLVED** this rotation (stale sidecar deleted, methodology moved off the
  fixture entirely — see §1).

## 7. Operational findings #41 must respect (new this rotation)

1. **Bound every wait (manager directive, standing):** never end a turn relying solely on
   a child notification — the T5 chain died silently and only a manager health-check
   caught it (§2 item 4). Use bounded backstops: `gh run watch` foreground with ≥300s
   Bash timeout for CI; git-poll monitors for child lanes; TaskOutput non-blocking checks.
2. **Default Bash 120s timeout KILLS `git push` mid-hook** (pre-push is now ≈121s) —
   **always push with timeout ≥300s.** The pre-push duration WARN re-baseline (~55→121s)
   remains FILED not APPLIED (`SURPRISES-INTAKE.md` 2026-07-15 17:18) — apply during the
   OP-8 drain.
3. **Post-push `--persist` re-mint convention:** persisting after a push flips only
   `last_verified` (timestamp churn); the established pattern is the persist rides the
   NEXT boundary commit — do not chase an eternal persist→push→CI→persist regression.
4. **Fable-top-level dispatch rules in force** (session model = Fable 5): delegate only to
   fable coordinators; explicit model overrides at leaves (opus complex / sonnet default /
   haiku mechanical); NEVER fable at a leaf.
5. **File-size WARNs** (waived to 2026-08-08, `GTH-V15-21`) unchanged.
6. **PROGRESS.md is current as of `b460008`** (T5 SHIPPED with wave-close annotation,
   NOW=T6, NEXT=P115-close→P116→P117-128) — verify freshness at first-act, edit only if
   stale.

**MCP state (outside the git tree, load-bearing):** `github-probe` registered +
connected — **leave it** (evidence surface for the T5/T6 headline claims). `atlassian-rovo`
still registered — safe to `claude mcp remove atlassian-rovo` after P115 closes,
discretionary. `mcp-mermaid` still **DOWN** — re-check before any diagram task.

## 8. Precise next steps (successor #41 runbook)

1. **FIRST ACT — the §0 verify block** (rev-list 0/0, `gh run list` top row completed/
   success — watch bounded if in-flight, post-push `--persist` P0 probe). Flaky `test`
   job → re-run ONCE; still red → STOP, never open T6 over a red main.
2. **T6 — dispatch to a fresh-context executor/coordinator** per the full §5 charter
   (8 numbered items). Order matters: reframe (item 1) before the second
   `/reposix-quality-refresh` (item 4), since the reframe is what re-drifts the
   latency-doc rows the refresh fixes.
3. **Item 3's human-only constraint is binding**: if the reposix-quality binary blocks
   the doc-alignment confirm-retire step, escalate to the MANAGER (w1:p7) — do not work
   around it.
4. **Item 7 — delete all FIVE `[SELF]` entries** in `CONSULT-DECISIONS.md` (line numbers
   verified live by #40: 71, 96, 114, 123, 153 — re-verify with a fresh grep before
   deleting, since line numbers shift as the file is edited).
5. **Phase-close cadence (§5 item 8):** `git push origin main` BEFORE verifier dispatch →
   post-push `--persist` P0 (`ci-green-on-main`) → gsd-verifier subagent for catalog-row
   PASS → advance `.planning/STATE.md` cursor → refresh `PROGRESS.md` in the close push →
   never open the next phase over a red main.
6. **P116** (after P115 closes): produce the ADR-010 packet (ADR-01 mirror-fanout +
   FIX-03 GTH-09 slug→id durable-create, options + tradeoffs) and route it to the
   **MANAGER (w1:p7) for ruling — NO pre-ruling implementation.**
7. **Every push in this rotation: use a Bash timeout ≥300s** — the 120s default kills
   `git push` mid pre-push-hook (§7 finding 2).
8. **If the weekly subscription cap hits mid-work:** commit+push whatever landed, REPLACE
   this handover, refresh `PROGRESS.md`, end cleanly. Reset-gating is RETIRED (owner
   ruling, `c7cea90`) — never defer or schedule work AROUND a reset; only react to a cap
   that hits.

## 9. Binding constraints (carry verbatim, unchanged)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at a
leaf** (and if #41 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE); relieve past ~100k own-context (hard 150k, absolute not %) at a wave boundary;
push at green, then confirm `code/ci-green-on-main` P0 AFTER push (with a Bash timeout
≥300s, per §7 finding 2); never open the next phase over a red main; reset-gating RETIRED —
never defer or schedule work for a weekly reset, only react to a cap that actually hits (if
it hits: commit+push, refresh this handover + `PROGRESS.md`, end cleanly).
