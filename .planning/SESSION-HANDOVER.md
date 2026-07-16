# SESSION-HANDOVER.md — v0.15.0 Floor: T6 item 1 LANDED + PUSHED (CI verifying) — items 2/3/5/6/7 remaining — 2026-07-16

Written by **workhorse #41** (L0 orchestrator), relieving to successor **#42**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#40→#41's handover,
superseded here). #41 relieved at ~146k own-context, at the T6 wave boundary — item 1
(headline reframe) landed and pushed, five items remain — per the standing "relieve past
~100k soft / 150k hard, absolute not %" rule.

**Read order:** this file → §0 ground truth (verify live, FIRST — CI was in-flight at
handover) → §1 headline → §5 successor charter (T6 items 2/3/5/6/7) → §6 findings #42 must
respect (one is a correction to #40's own charter — read it before touching item 6) → §8
runbook.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate document,
separate owner — the manager, pane w1:p7). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**. If
#42 runs on fable at top level, delegate per fable-top-level doctrine — **fable coordinators
only**, explicit model overrides at leaves (opus complex / sonnet default / haiku
mechanical), **NEVER fable at a leaf**.

## 0. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```

**As of #41's handover commit:**

- `HEAD` == `origin/main` == **`c9c2aee`** (before this handover commit lands; this
  handover is the next commit atop it and **L0 pushes it** — #42's first-act re-verifies
  0/0 after the push).
- Chain atop `0953ebd` (#40's relief handover commit, last known-clean sha), 4 commits, all
  on `main`, pushed ~2026-07-16 10:41 UTC:
  1. `d2fd85c` — `feat: T6 item 1 hero-surface reframe` — `README.md`, `docs/index.md`,
     `docs/concepts/reposix-vs-mcp-and-sdks.md` re-anchored from the retired synthetic
     89.1% to the live GitHub-capture figures (~94.3% output / ~74.9% cost, ~1.2k vs ~21k
     output tokens), folding in the two honesty findings (GitHub write-back read-only this
     cut; MCP `issue_read` lossy vs reposix byte-fidelity). Committed LOCAL first — its own
     push attempt BLOCKED on 4 newly-`STALE_DOCS_DRIFT` doc-alignment rows (no waiver yet).
  2. `fc232ee` — `docs(115-06): refresh PROGRESS for T6 headline-reframe (item 1)` — filed
     while still push-blocked, so its wording ("push BLOCKS as designed... not landed
     yet ⏳") predates the eventual landing — **this handover's PROGRESS.md companion
     commit corrects that framing** (see this rotation's PROGRESS refresh, done alongside
     this handover).
  3. `9a2b6f1` — `refresh(doc-alignment): re-grade 6 stale rows in docs/index.md +
     docs/concepts/reposix-vs-mcp-and-sdks.md` — top-level-only `/reposix-quality-refresh`
     run on BOTH drifted docs (NOT `latency.md` — confirmed untouched by the reframe, see
     §6 finding 1). 6 rows re-graded: `conflict-semantics-native-git` re-bound **GREEN**;
     `docs/index/mcp-loop-4883-tokens` + `docs/index/reposix-loop-531-tokens` re-cited to
     live figures (~21k / ~1.2k) → **MISSING_TEST**; `latency-cached-read-8ms` →
     **MISSING_TEST**; `token-baseline-mcp-4883` + `token-baseline-reposix-531` →
     **RETIRE_PROPOSED** (genuine supersession, human-only confirm-retire pending).
  4. `c9c2aee` — `chore(115-06): time-box waive 5 post-refresh rows pending T6 bind/retire`
     — walk still blocked (walker treats `MISSING_TEST`/`RETIRE_PROPOSED` as blocking
     states). Used the binary's sanctioned `waive` verb (T5 precedent): all 5 rows from
     step 3 waived `until: 2026-08-15T00:00:00Z`, `tracked_in: "P115 T6
     (115-UNWAIVE-PATH.md)"`. `doc-alignment.json` catalog summary `claims_waived` moved
     14→19 (the pre-existing 8 hero + 6 token-economy.md waivers, +5 new). Walk exit 0,
     push landed, pre-push **61 PASS / 0 FAIL**.
- **CI:** run `29488471287` on `0953ebd` (before T6 dispatch) — completed/success, P0 PASS,
  verified live by #41 as first-act. Run **`29491742214`** on `c9c2aee` was **IN FLIGHT at
  handover time** (`in_progress`, started `2026-07-16T10:41:52Z`, ~2m43s elapsed at last
  check) — **NOT yet confirmed green**.
- **This handover commit will be pushed BY L0** (not by #41 — L0 pushes). #42's §0 first-act
  re-verifies the tip AND the CI conclusion independently — do not assume green.
- Milestone **v0.15.0 "Floor"**, phase **P115 executing** (`Execution mode: top-level`).
- Working tree clean at handover time; **no background shells, monitors, or live
  subagents** left running for #42 to inherit.
- **#42's FIRST ACT (before anything else):**
  ```
  git rev-list --left-right --count HEAD...origin/main   # expect 0/0
  gh run list --branch main --workflow CI --limit 3      # confirm c9c2aee's run (29491742214)
                                                           # concluded — watch bounded if still
                                                           # in flight (`gh run watch 29491742214`,
                                                           # Bash timeout ≥360s)
  python3 quality/runners/run.py --cadence post-push --persist   # P0 ci-green-on-main
  ```
  If the flaky `test` job is red, re-run it ONCE before treating it as real. If still red
  after one retry, **STOP** — do NOT open further T6 work over a red main; escalate per §9.

## 1. THE HEADLINE: T6 item 1 (headline reframe) LANDED + PUSHED; CI verifying; 5 items remain

- Hero surfaces (`README.md`, `docs/index.md`,
  `docs/concepts/reposix-vs-mcp-and-sdks.md`) re-anchored from the retired synthetic
  **89.1%** to the live GitHub-capture headline **~94% fewer output tokens, ~75% cheaper
  per session** (output ~94.3% / cache-create ~66.0% / total input-context ~55.6% / cost
  ~74.9%), matching `docs/benchmarks/token-economy.md`'s existing provenance framing —
  hero copy and the benchmark page now tell one consistent story.
- Both honesty findings from T4/T5 folded into the reframe: (1) reposix's GitHub connector
  cannot push writes in this cut — read-only, comparison unaffected; (2) MCP
  `issue_read` HTML-escapes/drops raw markdown — reposix round-trips bytes faithfully.
- The reframe's own push attempt BLOCKED as designed on 4 newly-drifted doc-alignment rows
  (no waiver existed yet for post-reframe drift). #41 cleared the block via the sanctioned
  path: top-level `/reposix-quality-refresh` (re-graded 6 rows, 1 GREEN + 3 MISSING_TEST +
  2 RETIRE_PROPOSED) then a time-boxed `waive` on the still-blocking 5 (until 2026-08-15,
  tracked to `115-UNWAIVE-PATH.md`) — see §0 for the exact commits.
- **What's still open (5 remaining T6 items + phase-close):** `115-UNWAIVE-PATH.md` does
  not exist yet; the 6 `token-economy.md` rows and 8 hero-number rows T5 waived remain
  waived (un-waive/retire is T6's job, not done); the `perf/headline-numbers-cross-check`
  regen-clobber guard is unwritten; the 5 `[SELF]` `CONSULT-DECISIONS.md` entries are
  undeleted. **None of this blocks main** — the waivers hold until 2026-08-15 — but P115
  cannot phase-close until all of it lands.

## 2. What #41 did this rotation

1. First-act verify inherited from #40: rev-list 0/0, CI green (`29488471287`), P0 PASS.
2. Dispatched T6 to a fable phase-coordinator (per #40's §5 charter). It completed item 1
   (reframe, `d2fd85c` local + `fc232ee` PROGRESS) then **checkpointed exactly as
   chartered** when the push BLOCKed on docs-alignment/walk P0 (`STALE_DOCS_DRIFT`) — this
   is the documented "orchestration-shaped work resolves at top-level" pattern
   (`.planning/CLAUDE.md`), not a coordinator failure.
3. Ran the top-level-only `/reposix-quality-refresh` flow on BOTH drifted docs
   (`docs/index.md` + `docs/concepts/reposix-vs-mcp-and-sdks.md`; **NOT** `latency.md` —
   verified pre-edit that its 14 rows did not re-drift, see §6 finding 1). 6 stale rows
   re-graded by parallel graders per the refresh playbook — see §0 step 3 for the verdict
   breakdown. Committed `9a2b6f1`.
4. Walk still blocked (walker treats `MISSING_TEST` + `RETIRE_PROPOSED` as blocking states,
   `walk.sh:71-73`). Used the binary's sanctioned `waive` verb (loud+tracked, T5 precedent):
   5 rows waived until 2026-08-15, `tracked_in: "P115 T6 (115-UNWAIVE-PATH.md)"` (`c9c2aee`).
   Walk exit 0, push landed, pre-push 61 PASS / 0 FAIL.
5. Relieved at ~146k own-context at this wave boundary.

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
- This contract is part of the SESSION-HANDOVER replacement obligation — #42 and every
  successor MUST carry it forward in their own handover, verbatim if unchanged.
- **This rotation:** `PROGRESS.md` WAS stale (the mid-wave `fc232ee` refresh predates the
  eventual push landing — it still framed item 1 as blocked/⏳ and its NOW list used
  different item numbering than the T6 charter). #41 refreshed it in the SAME commit as
  this handover: SHIPPED's item-1 entry corrected to reflect the landed push (still
  carrying `d2fd85c` as the primary SHA per instruction, since that SHA was already
  present and not absent), NOW rewritten to name T6 items 2/3/5/6/7 (item 4's second
  `latency.md` refresh dropped — not needed, see §6 finding 1) using the charter's own
  item numbers so the two documents stay addressable against each other. #42: verify
  freshness at first-act; edit only if stale.

## 4. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session-definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE + PUSHED | `9384ca6`, `3845b13` |
| Wave 2 / T3 | Session-spend ledger scaffold | DONE + PUSHED | `4351d48` |
| Wave 3 / T4 | Live-MCP token capture, GitHub arm | DONE + PUSHED + CI GREEN | `4db6b64`, `40613f8`, `bf43c2c` |
| Wave 4 / T5 | Token-economy JSONL-usage regen + live headline | CLOSED + PUSHED + CI GREEN | `5366d29`, `1cdb381`, `fd098c7`, `211f794`, `63cb505`, `2103d0c`, `b460008` |
| **Wave 5 / T6** | **Un-waive + headline reframe + phase-close (delete FIVE `[SELF]` entries)** | **item 1 (reframe) LANDED + PUSHED, CI verifying; items 2/3/5/6/7 REMAIN; item 4 NOT NEEDED (see §6 finding 1); item 8 (phase-close) blocked on 2–7** | `d2fd85c`, `fc232ee`, `9a2b6f1`, `c9c2aee` |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | blocked on P115 close | — |

(Earlier pre-Wave-1 rows and the #33/#35/#36/#37/#38 pre-work rows from prior rotations are
compressed out of this table — see `git log` / #40's handover for that history if needed.)

## 5. Successor #42 charter — T6 items 2, 3, 5, 6, 7 then phase-close then P116

Re-dispatch a **FRESH fable phase-coordinator** for items 2, 3, 5, 6, 7 (item 1 is done,
item 4 is dropped per §6 finding 1). Charter, unchanged in substance from #40's original
8-item list except as corrected below:

1. ~~Headline reframe~~ — **DONE** (`d2fd85c`, `9a2b6f1`, `c9c2aee`).
2. **Write `115-UNWAIVE-PATH.md`** in the P115 phase dir
   (`.planning/phases/115-live-mcp-benchmark-re-measurement/`) — does not exist yet
   (verified live by #41). It MUST name **all 19** currently-waived doc-alignment rows
   that reference it via `tracked_in`, not just the original 14 — i.e. the 8 hero-number
   rows + 6 `token-economy.md` rows (T5) **AND** the 5 new rows from `c9c2aee` (3
   MISSING_TEST: `docs/index/mcp-loop-4883-tokens`, `docs/index/reposix-loop-531-tokens`,
   `latency-cached-read-8ms`; 2 RETIRE_PROPOSED: `token-baseline-mcp-4883`,
   `token-baseline-reposix-531`), plus the `perf/token-economy-bench` un-waive path and the
   pre-existing `perf/headline-numbers-cross-check` dangling-verifier waiver (see item 6
   correction below — that row is older than P115 and also tracked to a launch-readiness
   slot in its own waiver, but item 6 now closes it, so name it here too).
3. **Retire+rebind the 6 `token-economy.md` doc-alignment rows** (76.4% / 85.5% / 4883 /
   531 / 89.1% / jira-real-adapter) — **human-only confirm-retire constraint is binding**:
   if the reposix-quality binary blocks the retire step, escalate to the **MANAGER (pane
   w1:p7)** rather than working around it. The 2 concepts-page rows from `9a2b6f1`
   (`token-baseline-mcp-4883`, `token-baseline-reposix-531`, currently `RETIRE_PROPOSED` +
   waived) need the SAME human confirm-retire — batch both asks into ONE manager message
   (see §5 escalation note below).
4. ~~Second `/reposix-quality-refresh docs/benchmarks/latency.md`~~ — **NOT NEEDED**, see
   §6 finding 1 (latency.md did not re-drift; only run it if a future edit actually drifts
   its rows).
5. **Regen-clobber guard**: `emit-markdown.sh` must NOT overwrite CI-canonical
   `latency.md` sections.
6. **Un-waive the 8 hero-number rows** (`WAIVED-MISSING_TEST` until 2026-08-15); un-waive
   `perf/token-economy-bench` by adding the ~94% headline assertion; **ALSO un-waive the 3
   newly-waived MISSING_TEST rows** from `c9c2aee` (`docs/index/mcp-loop-4883-tokens`,
   `docs/index/reposix-loop-531-tokens`, `latency-cached-read-8ms`).
   **CORRECTION to #40's original framing — read before starting this item:** the
   `perf/headline-numbers-cross-check` catalog row is **NOT absent**. #41 verified it
   already EXISTS in `quality/catalogs/perf-targets.json` (added at P90, 2026-07-04,
   currently `WAIVED until 2026-09-15`, flagged as a **dangling verifier** — its
   `verifier.script`, `quality/gates/perf/headline-numbers-cross-check.py`, is confirmed
   absent from `quality/gates/perf/`, per that row's own `owner_hint`). #40's grep
   (`grep -c perf/headline-numbers-cross-check quality/catalogs/*.json` → reported 0
   everywhere) was **stale or mistaken** — re-running it live today returns non-zero
   matches in `perf-targets.json` (the row itself, id/command/artifact/owner_hint text)
   AND in `docs-reproducible.json` (2 cross-references) AND in `doc-alignment.json` (2
   fresh mentions from the `c9c2aee` waiver reasons). **Item 6's real task is: WRITE the
   missing verifier SCRIPT (`quality/gates/perf/headline-numbers-cross-check.py`) and
   un-waive the EXISTING row** — not create a new catalog row. Also check/update that
   row's `sources` line refs (`docs/index.md:13-14`, `README.md:21,23`) since `d2fd85c`
   likely shifted line numbers in both files. Also fold in **§6 finding 2** (8ms-hero vs
   6/7ms-canonical latency mismatch) — the cross-check WILL fail if hero prose and
   canonical `latency.md` disagree, so reconcile the prose (or the gate) as part of this
   item, not after.
7. **Delete ALL FIVE `[SELF]` entries** in `.planning/CONSULT-DECISIONS.md` — re-grep line
   numbers first (verified live by #41 today, unchanged from #40's count: lines 71 (A1),
   96 (T6-headline), 114 (T2-latency-canonical), 123 (T5-JSONL-methodology), 153
   (T4-GitHub-pivot) — note line 159 has a companion note "fold into T6 [SELF]-deletion
   sweep" that should also be cleaned up as part of deleting line 153's entry).
8. **Phase-close cadence** (unchanged): `git push origin main` BEFORE verifier dispatch →
   `python3 quality/runners/run.py --cadence post-push --persist` P0 → gsd-verifier
   subagent grades catalog rows (RED loops back) → advance `.planning/STATE.md` cursor →
   `PROGRESS.md` refresh in the close push → never open the next phase over a red main.

**MANAGER ESCALATION PENDING (batch into ONE message to w1:p7):** human-only confirm-retire
needed for (a) 2 concepts-page rows (`token-baseline-mcp-4883`, `token-baseline-reposix-531`,
`RETIRE_PROPOSED` at `9a2b6f1`, currently waived at `c9c2aee`) + (b) the 6 T5-waived
`token-economy.md` rows item 3 retires. `confirm-retire` is `$CLAUDE_AGENT_CONTEXT`-guarded
— likely owner-relay via manager. This escalation was ALREADY PENDING from #40's handover
and remains pending — #42 should confirm whether it has been actioned before re-raising it.

**Post-close:** P116 ADR-010 packet (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id
durable-create; options + tradeoffs) → route to **MANAGER w1:p7 for ruling — NO pre-ruling
implementation.**

## 6. Findings (new this rotation, #42 must respect)

1. **§5-item-4's predicted second `/reposix-quality-refresh` of `docs/benchmarks/
   latency.md` is NOT needed**: `latency.md` was untouched by the reframe, its 14 rows did
   NOT re-drift (coordinator verified pre-edit; #41 confirmed this by reading the file
   list touched by `d2fd85c` — `latency.md` is absent from it). Only run it if a future
   edit actually drifts them.
2. **LOAD-BEARING grader noticing, still live**: canonical `docs/benchmarks/latency.md`
   measures "Get one record = 6 ms / List records = 7 ms" (verified live by #41,
   `latency.md` lines 41–42), so the "8 ms" hero figure (`docs/index.md:18,:87`,
   `README.md:23,25`, `docs/concepts/reposix-vs-mcp-and-sdks.md:14` — verified all four
   locations live, plus carried by 3 catalog rows incl. canonical
   `docs/index/latency-8ms-read`) no longer matches its cited source. Item 6's cross-check
   gate (once its script exists) must reconcile — fix prose to canonical values or the
   gate will (correctly) fail. `latency-bench.sh` applies NO assertion to read latencies
   (soft WARN >500ms only; cross-check deferred since v0.12.1 MIGRATE-03,
   `docs/index/latency-8ms-read`'s own `rationale` field says this explicitly).
3. **CORRECTION to #40's charter (load-bearing, see §5 item 6 for the full writeup):** the
   `perf/headline-numbers-cross-check` catalog row is NOT absent — it already exists in
   `quality/catalogs/perf-targets.json` (P90-era, WAIVED until 2026-09-15, dangling
   verifier). #40's "confirmed absent... 0 everywhere" claim does not hold against a live
   grep today. Do not write a duplicate row; write the missing verifier script and
   un-waive the existing one.
4. **Walker semantics** (unchanged from #40's handover, reconfirmed): `MISSING_TEST` and
   `RETIRE_PROPOSED` are BLOCKING pre-push states (`walk.sh` header); the `waive`/`unwaive`
   verbs are the sanctioned time-box (≤90 days, loud+tracked, `last_extracted_by:
   "waive-call"` in the JSON). A refresh that lands honest non-GREEN states does NOT
   unblock a push by itself — the waive step is a separate, deliberate action.
5. **zsh trap** (unchanged): `$?` after a pipeline reports the LAST pipe stage (e.g.
   `tail`). Use `${pipestatus[1]}` for the real exit. Also `===` as a bare word triggers
   zsh equals-expansion — quote it.
6. **Pre-push duration** measured ~128s this rotation (WARN, stated budget ~60s) — every
   push needs Bash timeout ≥300s. Re-baseline still FILED not APPLIED
   (`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`, 2026-07-15 17:18 entry,
   verified live still present) — apply during the OP-8 drain, not now.
7. **Liveness discipline worked**: bounded backstops caught nothing stalled this rotation
   (no repeat of #40's silent-chain incident). Keep arming backstops when yielding on
   children; health-check quiet children ≤30min.
8. **MCP state unchanged**: `github-probe` connected (leave it — evidence surface for the
   T5/T6 headline claims). `atlassian-rovo` removable after P115 close, discretionary.
   `mcp-mermaid` DOWN — re-check before any diagram task.

## 7. Litmus / gate / REOPEN state

- **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until
  2026-08-15** — T6 un-waives per §5 item 6.
- **6 `token-economy.md` doc-alignment rows** remain **WAIVED-STALE_DOCS_DRIFT until
  2026-08-15** — T6 retires+rebinds per §5 item 3 (human-only confirm-retire).
- **5 NEW rows waived this rotation** (`c9c2aee`, until 2026-08-15, `tracked_in: "P115 T6
  (115-UNWAIVE-PATH.md)"`): 3 `MISSING_TEST` (`docs/index/mcp-loop-4883-tokens`,
  `docs/index/reposix-loop-531-tokens`, `latency-cached-read-8ms`) + 2 `RETIRE_PROPOSED`
  (`token-baseline-mcp-4883`, `token-baseline-reposix-531`). `doc-alignment.json` catalog
  summary `claims_waived` now **19** (was 14).
- **`perf/headline-numbers-cross-check` catalog row EXISTS** (correction — see §6 finding
  3), WAIVED until 2026-09-15, dangling verifier — T6 writes the script + un-waives per
  §5 item 6.
- **File-size soft-ceiling WARNs** (waived until 2026-08-08, class `GTH-V15-21`):
  unchanged, still over soft-ceiling. Waiver class covers them, not urgent.
- **Pre-push budget WARN re-baseline** is a live, growing regression (~121s → ~128s this
  rotation) — still **FILED not APPLIED**. **Every push in this rotation needs a Bash
  timeout ≥300s.**
- **No REOPEN state pending.**

## 8. Precise next steps (successor #42 runbook)

1. **FIRST ACT — the §0 verify block**: rev-list 0/0, confirm `c9c2aee`'s CI run
   (`29491742214`) concluded success — watch bounded if still in flight (`gh run watch
   29491742214`, Bash timeout ≥360s) — then run the post-push `--persist` P0 probe. Flaky
   `test` job → re-run ONCE; still red → STOP, escalate, never proceed over red main.
2. **T6 — dispatch a FRESH fable phase-coordinator** for items 2, 3, 5, 6, 7 per the full
   §5 charter. Read §6 finding 3 BEFORE starting item 6 — the catalog row already exists,
   only the verifier script and the un-waive are outstanding.
3. **Item 3's human-only constraint is binding**: if the reposix-quality binary blocks the
   doc-alignment confirm-retire step, escalate to the MANAGER (w1:p7) — do not work
   around it. Batch BOTH pending confirm-retire asks (the 2 concepts rows + the 6
   token-economy.md rows) into ONE manager message; check first whether #40's original
   escalation was already actioned.
4. **Item 7 — delete all FIVE `[SELF]` entries** in `CONSULT-DECISIONS.md` (line numbers
   verified live by #41 today: 71, 96, 114, 123, 153 — re-verify with a fresh grep before
   deleting, since line numbers shift as the file is edited; also clean the companion note
   at line 159).
5. **Phase-close cadence (§5 item 8):** `git push origin main` BEFORE verifier dispatch →
   post-push `--persist` P0 (`ci-green-on-main`) → gsd-verifier subagent for catalog-row
   PASS → advance `.planning/STATE.md` cursor → refresh `PROGRESS.md` in the close push →
   never open the next phase over a red main.
6. **P116** (after P115 closes): produce the ADR-010 packet (ADR-01 mirror-fanout +
   FIX-03 GTH-09 slug→id durable-create, options + tradeoffs) and route it to the
   **MANAGER (w1:p7) for ruling — NO pre-ruling implementation.**
7. **Every push in this rotation: use a Bash timeout ≥300s** — the 120s default kills
   `git push` mid pre-push-hook (§6 finding 6).
8. **If the weekly subscription cap hits mid-work:** commit+push whatever landed, REPLACE
   this handover, refresh `PROGRESS.md`, end cleanly. Reset-gating is RETIRED (owner
   ruling) — never defer or schedule work AROUND a reset; only react to a cap that hits.

## 9. Binding constraints (carry verbatim, unchanged)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at a
leaf** (and if #42 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE); relieve past ~100k own-context (hard 150k, absolute not %) at a wave boundary;
push at green, then confirm `code/ci-green-on-main` P0 AFTER push (with a Bash timeout
≥300s); never open the next phase over a red main; reset-gating RETIRED — never defer or
schedule work for a weekly reset, only react to a cap that actually hits (if it hits:
commit+push, refresh this handover + `PROGRESS.md`, end cleanly).
