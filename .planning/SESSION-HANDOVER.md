# SESSION-HANDOVER.md — v0.15.0 Floor: T6 COMPLETE (all 7 items agent-side) — P115 phase-close + human confirm-retire gate remain — 2026-07-16

Written by **workhorse #42** (L0 orchestrator), relieving to successor **#43**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#41→#42's handover,
commit `70cbf76`, superseded here). #42 relieves at a clean wave/phase boundary — T6 (all
7 items) landed, pushed, CI green, tree clean — rather than at a specific token count;
**this rotation's own context spend was not itemized in the brief this writer received, so
it is not fabricated here** (see §2). Relief is triggered by wave-boundary completeness
per the standing "relieve at wave boundaries, absolute not %" doctrine, not by a hard cap
this time.

**Read order:** this file → §0 ground truth (verify live FIRST) → §1 headline (the ONE
open item: human confirm-retire) → §5 successor charter (cold-reader pass → check human
gate → phase-close → P116 packet) → §6 findings #43 must respect → §8 runbook.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate document,
separate owner — the manager, pane w1:p7). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**. If
#43 runs on fable at top level, delegate per fable-top-level doctrine — **fable coordinators
only**, explicit model overrides at leaves (opus complex / sonnet default / haiku
mechanical), **NEVER fable at a leaf**.

## 0. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```

**Verified live by #42 as of ~2026-07-16 13:45 UTC (immediately before writing this file):**

- `HEAD` == `origin/main` == **`776ca85`** (before this handover commit lands; rev-list
  `0/0`, confirmed live). This handover is the next commit atop it and **L0 pushes it**
  (this writer does NOT push) — #43's first-act re-verifies `0/0` independently after
  that push lands.
- **CI: run `29501752893`** on `776ca85` — **`completed` / `success`**, confirmed live via
  `gh run view 29501752893`. All 15 jobs green (test, clippy, rustfmt, runner unit tests,
  quality gates pre-pr, shell-coverage, gitleaks, dark-factory sim, 4× real-backend
  integration contracts + 2× v09 variants, coverage, bench-latency-v09). The two
  `ANNOTATIONS` lines (`ENOENT ... opendir '.../target/tests/target'`) are benign
  artifact-upload noise from the `test` job's post-step, NOT a job failure — the `test`
  job itself shows `✓` / success.
- **Post-push P0 probe, run live by #42:** `python3 quality/runners/run.py --cadence
  post-push --persist` → `code/ci-green-on-main` **PASS** (P0, 0.87s), `1 PASS, 0 FAIL,
  0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0`. Tree clean immediately after
  (`git status --porcelain` empty).
- Milestone **v0.15.0 "Floor"**, phase **P115** (`Execution mode: top-level` for the T6
  orchestration-shaped cleanup lane). **T6 (Wave 5) is now COMPLETE — all 7 items landed
  agent-side.** Phase-close cadence (item 8: verifier dispatch + `STATE.md` cursor
  advance) has **NOT** run yet — that is #43's job, gated on the human confirm-retire
  action described in §1.
- Working tree clean at handover time; **no background shells, monitors, or live
  subagents** left running for #43 to inherit — #42 stopped its own last backstop before
  ending this turn.
- **#43's FIRST ACT (before anything else):**
  ```
  git rev-parse HEAD && git status --porcelain && \
    git rev-list --left-right --count HEAD...origin/main   # expect 0/0 after L0's push
  gh run list --branch main --workflow CI --limit 3        # confirm the tip's run concluded;
                                                              # watch bounded if in flight
                                                              # (Bash timeout ≥360s)
  python3 quality/runners/run.py --cadence post-push --persist   # P0 ci-green-on-main
  ```
  If the flaky `test` job is red, re-run it ONCE before treating it as real. If still red
  after one retry, **STOP** — do NOT open further work over a red main; escalate per §9.

## 1. THE HEADLINE: T6 COMPLETE — sole remaining action is a HUMAN-ONLY confirm-retire gate

- **All T6 items (1 through 7) are DONE, agent-side, on `main`, CI green.** Nothing in T6
  is blocked by code, tests, or agent-executable work. The **only** thing standing between
  P115 and phase-close is a human action this writer (and no agent) can perform:
  **`reposix-quality doc-alignment confirm-retire <id>`, run from a real TTY** (the binary
  refuses when `$CLAUDE_AGENT_CONTEXT` is set — fail-closed by design, not a bug to route
  around, per `.planning/ORCHESTRATION.md` §9).
- **The batch grew from 8 to ELEVEN rows during item 6b** (3 more 89.1%-era rows went
  `RETIRE_PROPOSED`; a duplicate pair was folded into one true dup, no distinct claim
  lost). **Authoritative row list + exact statuses:**
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md` — verified
  live: exactly 11 rows at `WAIVED-RETIRE_PROPOSED` today (grep'd both catalogs), **all**
  waiting on the same single human action. **No other T6-owned row remains waived** —
  `perf/token-economy-bench` and `perf/headline-numbers-cross-check` are both un-waived
  and `PASS` today (verified live in `quality/catalogs/perf-targets.json`).
- **Manager (pane w1:p7) has been pinged TWICE by #42** relaying this: (1) the initial
  8-row confirm-retire ask (ACKed by the manager with an explicit instruction), then (2)
  the 11-row correction once item 6b grew the batch, pointing the owner directly at
  `115-UNWAIVE-PATH.md` for the authoritative list. The manager relays to the owner via
  push notification. **The manager's standing instruction: do NOT idle-wait on the
  owner** — bring the phase to close readiness and **CHECKPOINT at the gate**
  (commit+push+handover naming the sole human action) if the confirm-retire pass hasn't
  landed by the time #43 is ready to close. #43 inherits this instruction unchanged.

## 2. What #42 did this rotation

1. First-act verify inherited from #41: rev-list `0/0`, CI conclusion on `c9c2aee`'s run
   confirmed, post-push P0 PASS.
2. **Item 3 agent-side (Wave 1), `d7da383`:** retired the 6 synthetic
   `count_tokens`-over-fixture `token-economy.md` rows (propose-retire only, human-only
   confirm-retire respected as a hard limit — not worked around) and bound 3 replacement
   rows to the live four-axis GitHub-capture figures (AND-drift-watch bindings against
   both the regenerator and its offline test suite).
3. **Wave 2, items 2/5/7/6, in commit order:**
   - `c2af48b` + `567dce8` — wrote `115-UNWAIVE-PATH.md` (full 19+2-row waived-row
     inventory, later grown to 21 rows tracked / 11 remaining waived) and filed a third
     corroborating pre-push wall-time-creep `SURPRISES-INTAKE.md` entry (item 2).
   - `2eb5836` — regen-clobber guard: `emit-markdown.sh` now refuses to overwrite
     `latency.md`'s CI-canonical sections; teaching error + `regen-guard.selftest.sh`
     (item 5).
   - `e7a1fd2` — deleted all FIVE `[SELF]` entries from `CONSULT-DECISIONS.md` (item 7,
     verified live: only the format-definition line + the unrelated live `RBF-LR-03`
     owner-decision entry remain).
   - `3eacb53` — **CI hotfix, concurrent lane:** the item-5 regen-clobber guard had broken
     `bench-latency-v09` on main; fixed via a `ci.yml` scratch-path carve-out, plus a
     latent cron-path break eager-fixed in the same commit, plus a new selftest case.
   - `63fdd8d` + `cd125eb` — wrote the missing
     `quality/gates/perf/headline-numbers-cross-check.py` verifier + tests, reconciled the
     "8 ms" hero prose to the canonical "6 ms get / 7 ms list" across all three hero
     surfaces, un-waived + minted the pre-existing P90-era
     `perf/headline-numbers-cross-check` catalog row (dangling-verifier fixed, not
     duplicated), rebound the two `8ms`-claim rows (item 6a).
   - `776ca85` — cold-init hero reconciled 27 ms → canonical **278 ms**; extended the
     cross-check gate with a cold-init axis + absolute loop-figure checks; un-waived +
     bound the cold-init rows, the two loop-token rows (`~21k` / `~1.2k`), and
     `README-md/latency-8ms`; un-waived + minted `perf/token-economy-bench` (now asserts
     ~94.3% ±1.0 pp); propose-retired + re-attributed 3 more superseded 89.1%-era rows
     (folding a true duplicate pair); persisted two benign validate-only status flips
     (item 6b — **T6 items 1-7 all complete**).
4. Confirmed CI green independently on each wave-boundary push; final confirmation on
   `776ca85` (run `29501752893`, all jobs success) done live moments before writing this
   handover, plus the post-push `--persist` P0 probe (PASS, exit 0).
5. Sent the manager (w1:p7) the corrected 11-row confirm-retire ask (see §1), per the
   manager's own "checkpoint, don't idle-wait" instruction.
6. Wrote and is committing this relief handover + the `PROGRESS.md` refresh in the same
   commit, then ends the turn — no live subagents, background shells, or monitors left
   running.

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
- This contract is part of the SESSION-HANDOVER replacement obligation — #43 and every
  successor MUST carry it forward in their own handover, verbatim if unchanged.
- **This rotation:** `PROGRESS.md` was already substantially refreshed by the T6 lanes
  themselves as each item landed (SHIPPED entries for items 1/3/5 already present; the NOW
  section already carried a detailed items-2/3/5/6a/6b/7 narrative). #42 completed the
  refresh in the SAME commit as this handover: added the missing SHIPPED entries for items
  2, 7, 6a, and 6b with their landing SHAs, collapsed the NOW section to the single
  remaining focus (P115 phase-close cadence: verifier dispatch + the human confirm-retire
  gate), and reordered NEXT so the P116 ADR-010 packet leads (P115-closed is no longer a
  future NEXT item — it's the literal NOW). #43: verify freshness at first-act; edit only
  if stale.

## 4. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session-definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE + PUSHED | `9384ca6`, `3845b13` |
| Wave 2 / T3 | Session-spend ledger scaffold | DONE + PUSHED | `4351d48` |
| Wave 3 / T4 | Live-MCP token capture, GitHub arm | DONE + PUSHED + CI GREEN | `4db6b64`, `40613f8`, `bf43c2c` |
| Wave 4 / T5 | Token-economy JSONL-usage regen + live headline | CLOSED + PUSHED + CI GREEN | `5366d29`, `1cdb381`, `fd098c7`, `211f794`, `63cb505`, `2103d0c`, `b460008` |
| **Wave 5 / T6** | **Un-waive + headline reframe + phase-close prep (delete FIVE `[SELF]` entries)** | **ALL 7 ITEMS COMPLETE (agent-side), pushed, CI green. Item 8 (phase-close cadence: verifier + `STATE.md` cursor) NEXT, gated on the human confirm-retire batch (§1) — NOT an agent-executable blocker.** | `d2fd85c`, `fc232ee`, `9a2b6f1`, `c9c2aee`, `d7da383`, `c2af48b`, `567dce8`, `2eb5836`, `e7a1fd2`, `3eacb53`, `63fdd8d`, `cd125eb`, `776ca85` |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | blocked on P115 close | — |

(Earlier pre-Wave-1 rows and the #33/#35/#36/#37/#38 pre-work rows from prior rotations are
compressed out of this table — see `git log` / earlier handovers for that history if needed.)

## 5. Successor #43 charter

Read in this order — do not skip the cold-reader pass, it is a root-`CLAUDE.md`
requirement this rotation did not discharge:

1. **Cold-reader pass owed BEFORE declaring the hero surfaces shipped** (root `CLAUDE.md`
   § "Cold-reader pass on user-facing surfaces"): T6's lanes made minimal, mechanical
   number swaps (8ms→6/7ms, 27ms→278ms, 89.1%→94.3%/74.9%) across `docs/index.md` and
   `README.md` — the surrounding prose/framing was NOT re-reviewed for coherence.
   Dispatch `/doc-clarity-review` on `docs/index.md` + `README.md`. Also: `GTH-V15-33`
   (verified live, `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:223`) already
   files a related mental-model-page inconsistency — `mental-model-in-60-seconds.md:21`
   still reads "24 ms" while the same page's line 69 now reads the canonical 278 ms
   (item 6b deliberately left L21 alone to avoid flipping 3 non-blocking rows to
   blocking) — read it before the cold-reader dispatch so the two don't duplicate work.
2. **Check whether the owner's confirm-retire landed.** `grep RETIRE_PROPOSED
   quality/catalogs/doc-alignment.json` — a catalog commit may have been pushed by the
   owner directly; **pull first** (`git fetch && git log origin/main` ahead-check) before
   assuming #42's state is still current.
3. **Phase-close cadence:** push already landed + post-push P0 already done for
   `776ca85` (§0). If new commits land (e.g. the confirm-retire, or the cold-reader
   follow-ups), re-push + re-run the post-push `--persist` P0 probe. Then dispatch a
   `gsd-verifier` subagent per `quality/PROTOCOL.md` + `quality/CLAUDE.md` (catalog-row
   grading, RED loops back). **Expect the 11 `WAIVED-RETIRE_PROPOSED` walk lines** —
   the verifier should grade these as the documented, tracked human gate per
   `115-UNWAIVE-PATH.md`, NOT as a silent failure or an unexplained non-GREEN state.
4. **If the human gate is still open when the phase reaches close-readiness:**
   checkpoint per the manager's standing instruction (§1) — commit+push+handover naming
   confirm-retire as the sole remaining action. **Do NOT hold the phase open
   idle-waiting** on the owner.
5. **After P115 closes:** produce the **P116 ADR-010 packet** — ADR-01 mirror-fanout +
   FIX-03 `GTH-09` slug→id durable-create; options + tradeoffs, **NO implementation**.
   Route it to the **MANAGER (w1:p7) for ruling** → **END TURN and await the ruling.**

## 6. Findings (this rotation, #43 must respect)

1. **T6 self-reported orchestration violation, noticed-not-yet-formalized:** the T6
   coordinator course-corrected a lane via a **FORK** (spun up a fable-tier leaf while a
   momentary second tree-writer briefly existed) because `SendMessage` is disabled inside
   subagents and it needed to deliver a mid-task correction to a running lane. It executed
   cleanly — no damage, no lost work, no corrupted tree — but it is a real deviation from
   the single-tree-writer doctrine, done under pressure because the documented mitigation
   (a `SendMessage` capability, or an explicit fallback pattern) doesn't fully cover this
   case. **This corroborates, but is NOT identical to,** the already-filed
   `SURPRISES-INTAKE.md` entry (`2026-07-16 07:50`, discovered during T5) about
   `phase-coordinator` lacking `SendMessage` entirely — that entry documents the general
   capability gap; this rotation's FORK is a NEW concrete instance of working around it.
   **Not yet filed as its own intake row or `ORCHESTRATION.md` doctrine-note** — a
   fix-twice candidate for #43 or the close-drain: either (a) file a fresh corroborating
   `SURPRISES-INTAKE.md` row naming this specific FORK incident, or (b) if judged the same
   underlying gap, add a cross-reference to the existing 07:50 entry rather than
   duplicating it. Either way, do not let it drop silently.
2. **Pre-push wall-time creep, further corroborated:** this rotation's pushes measured
   ~141s / ~128s (filed, `SURPRISES-INTAKE.md` `2026-07-16 12:00` entry, third
   corroborating data point after the `2026-07-15 06:35` and `17:18` entries) plus two
   more late-rotation pushes in the ~98-99s range (not yet individually filed as separate
   rows — same ongoing creep, not a new phenomenon) — all well above the documented
   ~55-60s budget in root `CLAUDE.md` § GSD workflow, and above even the ~75s re-baseline
   the `17:18` entry already proposed. **Every push in this rotation and #43's needs a
   Bash timeout ≥300s.** Re-baseline is FILED, not yet APPLIED — apply during the OP-8
   drain, not mid-phase.
3. **sim `"unknown field expected_version"` JSON errors** observed in bench logs during
   this rotation's live-capture-adjacent runs — already filed (see prior rotations'
   intake); not re-filed here, just re-noted as still present.
4. **Wave-2 noticing — two corrected framings, both already resolved in this rotation's
   own work (documented here so #43 doesn't rediscover them from scratch):** (a) the
   pre-existing "8 uniform hero rows" framing (carried in #40/#41's handovers) was wrong —
   item 2's `115-UNWAIVE-PATH.md` inventory pass corrected it (the batch is heterogeneous:
   `WAIVED-MISSING_TEST` hero rows + `RETIRE_PROPOSED` token-economy rows + perf rows, not
   one uniform class); (b) item 6b **refuted item 6a's `bind --test` diagnosis** — the
   real gap is NOT a missing test binding but **fn-resolution being unenforced** in the
   `doc-alignment bind --test` verb (verified live: `GTH-V15-29`,
   `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:203` — "advertises `<file>::<fn>`
   but the fn half is optional + unvalidated"). Full writeup:
   `.planning/phases/115-live-mcp-benchmark-re-measurement/115-T6-CLOSEOUT.md`.
5. **Evidence home:**
   `.planning/phases/115-live-mcp-benchmark-re-measurement/115-T6-CLOSEOUT.md` is the
   authoritative "what landed" record for all of T6 — read it, not chat history, for any
   item's exact verification detail.
6. **Unchanged mechanics:** zsh `${pipestatus[1]}` trap after a pipeline (`$?` reports the
   LAST pipe stage only); leaf-isolation `/tmp` same-Bash-invocation rule for any
   reposix/sim/git test setup.
7. **Liveness discipline in force, unchanged:** ≤20min bounded backstops when yielding on
   children; health-check quiet children ≤30min; **never end a final turn with a
   background shell running** (verified none left running at this handover).
8. **Reset-gating is RETIRED (owner ruling), unchanged:** on any cap-hit, commit+push
   progress, update this handover + `PROGRESS.md`, end the turn cleanly — never defer or
   schedule work around a weekly reset, only react to a cap that actually hits.

## 7. Litmus / gate / REOPEN state

- **11 rows at `WAIVED-RETIRE_PROPOSED`** are the ONLY waivers left in T6's scope — the
  full authoritative list (row IDs, retired figures, human-confirm-retire command) is
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md`. Nothing
  else T6-owned is waived. **No REOPEN state pending.**
- **File-size soft-ceiling WARNs** — unchanged, still over the soft ceiling, masked by the
  `structure/file-size-limits` **`--warn-only` waiver until 2026-08-08**, class
  **`GTH-V15-21`** (verified live: `quality/catalogs/freshness-invariants.json:666`,
  `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:133`). Not urgent; the OVER-BUDGET
  tier reactivates `exit 1` on 2026-08-08 if the waiver isn't renewed or the drain doesn't
  land first.
- **`perf/headline-numbers-cross-check` catalog row is now UN-WAIVED and GREEN**
  (verified live: `quality/catalogs/perf-targets.json` — `status: PASS`, `waiver: null`
  equivalent state, verifier script `quality/gates/perf/headline-numbers-cross-check.py`
  confirmed present and passing). This is a genuine state change from the prior
  dangling-verifier WAIVED row — item 6a/6b's work, not a re-waive.
- **`perf/token-economy-bench` is also un-waived and minted PASS** (asserts ~94.3% ±1.0pp
  headline reduction now; was previously WAIVED pending this assertion).
- **Pre-push wall-time budget re-baseline is FILED, not APPLIED** — see §6 finding 2.
  Every push needs Bash timeout ≥300s until the drain lands.

## 8. Precise next steps (successor #43 runbook)

1. **FIRST ACT — the §0 verify block:** `git rev-parse HEAD`, `git status --porcelain`,
   `git rev-list --left-right --count HEAD...origin/main` (expect `0/0` once L0's push of
   this handover lands), confirm the tip's CI run concluded success (watch bounded,
   `gh run watch <id>`, Bash timeout ≥360s, if still in flight), then
   `python3 quality/runners/run.py --cadence post-push --persist` (P0
   `code/ci-green-on-main`). Flaky `test` job → re-run ONCE; still red → STOP, escalate,
   never proceed over a red main.
2. **Cold-reader pass** — dispatch `/doc-clarity-review` on `docs/index.md` + `README.md`
   before treating T6's hero-surface edits as fully shipped (§5 item 1). Cross-check
   against `GTH-V15-33` first so the two don't duplicate.
3. **Check the human confirm-retire gate** — `git fetch`, then `grep RETIRE_PROPOSED
   quality/catalogs/doc-alignment.json`; if the owner has actioned it (directly or via the
   manager), the 11-row batch will show fewer/zero `RETIRE_PROPOSED` rows. Pull any new
   commits before proceeding.
4. **Phase-close cadence:** re-push + re-probe if new commits landed; dispatch
   `gsd-verifier` for catalog-row PASS grading (RED loops back; expect the 11 waived rows
   to read as the documented human gate, not a silent failure) → advance
   `.planning/STATE.md` cursor → refresh `PROGRESS.md` in the close push.
5. **If the human gate is still open at close-readiness:** checkpoint per the manager's
   standing instruction — commit+push+handover naming confirm-retire as the sole
   remaining action. Do NOT hold the phase open idle-waiting for the owner.
6. **File or absorb the FORK noticing** (§6 finding 1) during this rotation or the
   close-drain — do not let it disappear unfiled.
7. **After P115 closes:** produce the **P116 ADR-010 packet** (ADR-01 mirror-fanout +
   FIX-03 GTH-09 slug→id durable-create; options + tradeoffs, **NO implementation**) and
   route it to the **MANAGER (w1:p7) for ruling** → **END TURN and await the ruling** —
   do not begin implementation pre-ruling.
8. **Every push needs a Bash timeout ≥300s** — the 120s default kills `git push` mid
   pre-push-hook (§6 finding 2).
9. **If the weekly subscription cap hits mid-work:** commit+push whatever landed, REPLACE
   this handover, refresh `PROGRESS.md`, end cleanly. Reset-gating is RETIRED (owner
   ruling) — never defer or schedule work AROUND a reset; only react to a cap that hits.

## 9. Binding constraints (carry verbatim, unchanged)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at a
leaf** (and if #43 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE); relieve past ~100k own-context (hard 150k, absolute not %) at a wave boundary;
push at green, then confirm `code/ci-green-on-main` P0 AFTER push (with a Bash timeout
≥300s); never open the next phase over a red main; reset-gating RETIRED — never defer or
schedule work for a weekly reset, only react to a cap that actually hits (if it hits:
commit+push, refresh this handover + `PROGRESS.md`, end cleanly).
