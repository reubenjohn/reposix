# SESSION-HANDOVER.md — v0.15.0 Floor: pre-close owner-directive lane SHIPPED, 3 new owner
directives filed — P115 phase-close + human confirm-retire gate remain — 2026-07-16

Written by **workhorse #43** (L0 orchestrator), relieving to successor **#44**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#42→#43's handover,
commit `df27d87`, superseded here). #43 relieves at a clean stopping point — the pre-close
owner-directive lane (retirement-narrative strip) shipped, the FINAL 11-row confirm-retire
batch consolidated, three NEW owner directives filed durably, the manager pinged, and a
P116 ADR-010 decision-packet draft produced — rather than at a specific token count; **this
rotation's own context spend was not itemized in the brief this writer received, so it is
not fabricated here**, matching #42's precedent. Relief is triggered by task/wave-boundary
completeness, not a hard cap this time.

**Read order:** this file → §1 ground truth (verify live FIRST — includes TWO discrepancies
this writer found between the inherited briefing and live reality, read them before acting)
→ §2 wave/cycle state → §4 litmus/gate state (the human gate) → §5 mid-execution decisions
+ noticed-not-filed (the P116 packet is sitting in `/tmp` and does NOT survive a crash) →
§6 runbook (resolve the manager-pane discrepancy BEFORE assuming the human gate's state).

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate document,
separate owner — the manager, pane w1:p7). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a leaf.

**MODEL NOTE (unchanged, load-bearing for dispatch):** the session model is **Fable 5**. If
#44 runs on fable at top level, delegate per fable-top-level doctrine — **fable coordinators
only**, explicit model overrides at leaves (opus complex / sonnet default / haiku
mechanical), **NEVER fable at a leaf**.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git status --porcelain --untracked-files=all && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --limit 3
```

**Verified live by #43 as of ~2026-07-16 16:25 UTC (immediately before writing this file):**

- `HEAD` == `origin/main` == **`187809f`** (before this handover commit lands; rev-list
  `0/0`, confirmed via live `git fetch origin` — the only remote-side change was a forced
  update of an unrelated `release-plz-2026-07-14T00-41-16Z` branch, irrelevant to `main`).
  This handover is the next commit atop it and this writer pushes it directly (see §6 for
  the push/CI/probe sequence run as part of this same turn).
- **Per-commit one-liners since the last known-clean sha (`df27d87`, #42→#43's handover):**
  - `5a5dd29` — `docs(115)`: strip retirement-history narrative from user-facing docs
    (owner ruling 2026-07-16) — 4 files (`token-economy.md`, `reposix-vs-mcp-and-sdks.md`,
    `docs/index.md`, `latency.md`); zero new doc-alignment rows propose-retired.
  - `a1f2494` — `docs(planning)`: owner ruling ledger entry; FINAL 11-row consolidated
    confirm-retire batch landed in `115-UNWAIVE-PATH.md`; `GTH-V15-34` filed
    (`confirm-retire --batch` mode); FORK noticing cross-referenced onto the 07:50
    `SURPRISES-INTAKE.md` entry; 2 new MEDIUM intake rows (docs/social stale-89.1% figures;
    34-row silent `STALE_TEST_DRIFT` bit-rot); `quality/CLAUDE.md` fix-twice note; PROGRESS
    refresh.
  - `484ca52` — `docs(planning)`: broadened the 2026-07-14 file-size intake entry
    (`SURPRISES-INTAKE.md` itself now oversized + a `GOOD-TO-HAVES.md` growth datapoint).
  - `187809f` — `docs(planning)`: filed THREE new owner directives durably — `GTH-V15-35`
    (post-close index.md install-section eager-fix), `GTH-V15-36` (furnished-product
    quality bar for P117/P119), `GTH-V15-37` (P117 animation-embed lane); annotated inline
    on the **top-level** `.planning/ROADMAP.md` Phase 117 (~L122-139) and Phase 119
    (~L151-167) sections — verified live, both annotations present, both cite the correct
    GTH IDs. CONSULT-DECISIONS.md carries the matching `[OWNER]` ledger entry (L88-109).
- **CI: three runs verified live, all `completed`/`success`:**
  - `29512948144` on `a1f2494` — success (`gh run view --json conclusion,status,headSha`).
  - `29513705075` on `484ca52` — success (same verification).
  - `29514714269` on `187809f` — success, 5m45s, confirmed via `gh run list --branch main
    --limit 5` (also shows a green downstream `Docs` workflow_run `29515115871` and a green
    `CodeQL` dynamic run `29514713655` off the same push).
  - Post-push P0 probe (`quality/runners/run.py --cadence post-push --persist`) was run and
    PASSED after each of these three pushes per the prior rotation's own record; **#43 did
    not re-run it independently in this turn before writing this file** — #44's first act
    (below) re-confirms it fresh regardless.
- **DISCREPANCY #1 (verified live, corrects the inherited briefing):** the briefing this
  writer received claimed "tree clean except 3 untracked root PNGs (`anim-t2s*.png`)."
  **Live reality is different:** `git status --porcelain --untracked-files=all` returns
  **completely empty** and `ls /home/reuben/workspace/reposix/*.png` matches **zero files**.
  No `anim-t2s*.png` files exist at the repo root right now. Do not assume they are still
  there, and do not fabricate a "never touch them" ritual over files that aren't present —
  #44 should just re-verify at its own first act rather than trust this snapshot to persist
  (they may reappear if the owner drops new ones for the P117 animation-embed lane,
  `GTH-V15-37` — if so, treat them as owner-local scratch, never stage them).
- **DISCREPANCY #2 (verified live, CRITICAL, corrects the inherited briefing — read before
  touching the human gate):** the briefing described the open item as "verify #43's ping to
  the manager submitted; if stuck, press Enter again." Live inspection
  (`herdr agent read w1:p7 --lines 40` + `herdr agent explain w1:p7 --json`) shows **that
  specific ping DID submit and WAS answered** — the manager's pane transcript shows #43's
  ping ("P115 pre-close owner-directive lane SHIPPED...") followed by an explicit manager
  response ("All confirmed... nothing has changed about your action..."). **That part of
  the briefing is stale/resolved — no action needed on it.**

  **However, a NEW and more consequential item was found live in that same pane read: the
  manager's prompt box currently holds an UNSUBMITTED line reading exactly**
  `❯ done, ran the 11 commands and pushed` **— sitting after a "Replace monitor with polling
  model" separator, with no `●` (manager) response following it.** `herdr agent explain`
  matches this against rule `live_prompt_box` / state `idle`, i.e. the box content is
  sitting there, not yet acted on by the manager. Read literally, this looks like the
  **owner** reporting they already ran the 11-command confirm-retire batch and pushed.
  **This is NOT corroborated by independently-verified ground truth:**
  - `git fetch origin` shows `main` unchanged at `187809f` — no new commit landed.
  - `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json`
    (run live, in this tree) returns **11** — unchanged from the pre-existing count, i.e.
    the local working tree's catalog still shows all 11 rows un-retired.

  **This is a live open question #44 MUST resolve before treating the human gate as
  either open or closed** — three possibilities, none yet distinguished: (a) the message
  is genuine but was typed/run in a *different* checkout that hasn't pushed yet (or the
  push silently failed); (b) the message is premature/aspirational (the owner intends to
  run it but hasn't actually done so); (c) the message is stale UI content unrelated to
  this specific ask. **Do not assume any of the three — verify by re-reading the pane,
  re-fetching `main`, and re-grepping the catalog before acting.** See §6 runbook item 1.

## 2. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Waves 1–4 / T1–T5 | Benchmark session ratification → latency re-measure → ledger scaffold → live-MCP GitHub capture → JSONL-usage regen | DONE + PUSHED + CI GREEN (compressed; full commit list in #40–#42's handovers / `git log`) | — |
| **Wave 5 / T6** | Un-waive + headline reframe + phase-close prep (all 7 items) | **ALL 7 ITEMS COMPLETE, pushed, CI green** (unchanged since #42's handover) | `d7da383`, `c2af48b`, `567dce8`, `2eb5836`, `e7a1fd2`, `3eacb53`, `63fdd8d`, `cd125eb`, `776ca85` |
| Post-T6 / pre-close | Owner-directive lane: strip retirement-history narrative from `token-economy.md`, `reposix-vs-mcp-and-sdks.md`, `docs/index.md`, `latency.md` | **SHIPPED, pushed, CI green** — zero new doc-alignment rows propose-retired | `5a5dd29` |
| Post-T6 / pre-close | Ledger entry + FINAL 11-row consolidated confirm-retire batch + `GTH-V15-34` + FORK cross-ref + 2 new intake rows + `quality/CLAUDE.md` fix-twice note + PROGRESS refresh | **SHIPPED, pushed, CI green** | `a1f2494` |
| Post-T6 / pre-close | Broaden 2026-07-14 file-size intake scope | **SHIPPED, pushed, CI green** | `484ca52` |
| Post-T6 / pre-close | File 3 NEW owner directives (`GTH-V15-35/36/37`) + top-level `ROADMAP.md` P117/P119 annotations | **SHIPPED, pushed, CI green** | `187809f` |
| Post-P115 (blocked on P115 close) | P116 ADR-010 decision packet (ADR-01 mirror-fanout + FIX-03 `GTH-09` slug→id durable-create; options + tradeoffs only) | **DRAFTED** by an opus researcher (~141 lines, 2 decisions each with ~4 options + a recommendation) — **NOT YET COMMITTED.** Lives only at `/tmp/claude-1000/-home-reuben-workspace-reposix/050a77c0-114c-4223-b110-ff1337196c0f/scratchpad/P116-ADR-010-packet.md`. `/tmp` does not survive a crash — **at risk, rescue-commit or regenerate ASAP** (§6 item 3). | — (uncommitted) |
| P115 phase-close | Human-only confirm-retire gate (11 rows, `WAIVED-RETIRE_PROPOSED`) | **OPEN per independently-verified live state** (catalog still shows 11/11 `RETIRE_PROPOSED`; `main` unchanged). An unsubmitted, uncorroborated message in the manager's pane claims completion — see §1 Discrepancy #2. **Do not mark this closed on the strength of that message alone.** | — |
| P115 phase-close | Cold-reader pass on hero surfaces (`docs/index.md` + `README.md`) | **NOT YET DISPATCHED** — owed per root `CLAUDE.md` § "Cold-reader pass on user-facing surfaces"; carried forward unstarted from #42's handover through this rotation too | — |
| P115 phase-close | Verifier dispatch (catalog-row grading) + `STATE.md` cursor advance | **NOT STARTED** — gated on the above two items | — |

## 3. Binding constraints (unchanged, carry verbatim)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, **never fable at
a leaf** (and if #44 runs on fable at top level, delegate fable-coordinators-only per the
MODEL NOTE above); relieve past ~100k own-context (hard 150k, absolute not %) at a wave
boundary; push at green, then confirm `code/ci-green-on-main` P0 AFTER push (Bash timeout
≥300s — pre-push wall time has crept to 94–141s across multiple corroborating datapoints
this milestone, well above the ~55–60s documented budget; re-baseline is FILED, not yet
APPLIED); never open the next phase over a red main; reset-gating RETIRED — never defer or
schedule work for a weekly reset, only react to a cap that actually hits (if it hits:
commit+push, refresh this handover + `PROGRESS.md`, end cleanly).

## 4. Litmus / gate / REOPEN state

- **11 rows at `WAIVED-RETIRE_PROPOSED`** remain the ONLY waivers in T6's scope — verified
  live in this turn: `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json`
  → **11**, matching the FINAL consolidated batch enumerated in
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md` §"FINAL
  consolidated confirm-retire batch" (the single authoritative row-ID list + copy-paste
  `confirm-retire --row-id <ID>` commands — supersedes scattered "11 rows" mentions
  elsewhere). **Per §1 Discrepancy #2, an unsubmitted/uncorroborated manager-pane message
  claims this batch is already done — treat the gate as OPEN until independently
  re-verified**, not as closed on the strength of that message.
- Verb confirmed via `--help` (never executed against a row, human-only,
  `$CLAUDE_AGENT_CONTEXT` env-guard, `--i-am-human` is an audited escape hatch for humans
  running inside a Claude Code session, NOT for agents — agents must never pass it):
  `reposix-quality doc-alignment confirm-retire --row-id <ROW_ID>` from a real TTY.
- **`perf/token-economy-bench` and `perf/headline-numbers-cross-check` are both un-waived
  and `PASS`** — verified live in this turn (`quality/catalogs/perf-targets.json`:
  `"status": "PASS"`, `"waiver": null` for both rows, `last_verified` 2026-07-16). No
  other T6-owned row remains waived.
- **File-size soft-ceiling waiver `GTH-V15-21`** — unchanged, still masking the OVER-BUDGET
  tier as `--warn-only` until **2026-08-08**
  (`quality/catalogs/freshness-invariants.json:666`). Not urgent yet, but the ledger-split
  decision it depends on (SURPRISES-INTAKE.md, GOOD-TO-HAVES.md, and top-level ROADMAP.md
  are all now over the 20k soft ceiling and still growing) needs an owner call before the
  lapse date.
- **CI green on `main`'s tip verified three times independently this rotation** (§1) — no
  REOPEN state pending on any of those three pushes.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **THREE new owner directives filed durably (`187809f`), all with fix-sketches ready for
   a future planner, none yet acted on:**
   - `GTH-V15-35` — post-P115-close eager-fix: nest "Build from source (advanced)" under
     "30-second install" in `docs/index.md`, via a tracked `/gsd-quick`. Two hard cautions
     in the row: keep `structure/install-leads-with-pkg-mgr-docs-index` green, and refresh
     any doc-alignment rows bound to `docs/index.md` in the SAME wave as the edit.
   - `GTH-V15-36` — owner mandate (verbatim quote in the ledger: *"Its good, but we can do
     so much better!"*): the docs site must read as a FURNISHED PRODUCT, not merely
     factually correct. Explicit quality-bar input for **P117** and **P119** planners
     (information architecture, progressive disclosure, visual polish, a cold-reader
     rubric pass on every landing surface) — annotated inline on the top-level
     `ROADMAP.md`'s Phase 117 and Phase 119 sections, NOT the (stale, per `GTH-V15-27`)
     milestone-scoped `v0.15.0-phases/ROADMAP.md` stub.
   - `GTH-V15-37` — owner-approved P117 scope addition: embed the owner's ~80s launch
     animation (source at `~/workspace/reposix-animation-pitch`, a React/JSX 7-scene
     animation) on the mkdocs home page. A 5-point productionization checklist is filed in
     the row: offline JSX pre-compile (removes the unpkg CDN dependency + a 2.8s blank
     load), self-host Space Grotesk + JetBrains Mono fonts, an embed mode
     (`motionEditor=false`, neutralize the `animstage:t` localStorage playhead, poster +
     click-to-play), ship the ~7MB mp4 as a GitHub release attachment (NOT committed to the
     repo), assets under `docs/assets/animation/` with `Zone.Identifier` files stripped +
     `mkdocs-strict`/playwright-walk coverage.
2. **P116 ADR-010 packet is DRAFTED but UNCOMMITTED** (§2 table) — an opus researcher wrote
   ~141 lines covering ADR-01 (mirror-fanout coherence, 4 options + a recommended
   doc-truth+webhook-bless design) and FIX-03/`GTH-09` (slug→id durable-create, 4 options +
   a recommended design-only durable-map). This is real, substantive work sitting in
   `/tmp` — a crash loses it. **Not yet routed to the manager for ruling** (per the P116
   charter: options only, zero implementation pre-ruling).
3. **The FORK noticing (T6's mid-task tiering deviation) is now RESOLVED as filed** — it
   was cross-referenced (not duplicated) onto the existing 07:50 `SURPRISES-INTAKE.md`
   entry about `phase-coordinator` lacking `SendMessage`, landed in `a1f2494`. No longer an
   open loose end from #42's handover.
4. **Intermittent Read/Edit harness tool failures** ("No tools needed for summary" or
   similar non-response) reportedly hit THREE separate dispatched leaves during this
   rotation, per the inherited briefing — this writer did not independently reproduce or
   verify this claim live (no artifact/transcript was available to inspect in this turn).
   **Not yet filed as its own `SURPRISES-INTAKE.md` row.** #44: if seen again, file it
   fresh; if judged the same class as an existing 07:50-adjacent tooling entry, cross-
   reference rather than duplicate (same pattern as finding 3 above).
5. **Docs/social stale-89.1%-figure intake row** (`docs/social/twitter.md:18`,
   `docs/social/linkedin.md:21`) — verified live present in `SURPRISES-INTAKE.md`, STATUS
   OPEN, awaiting an owner call (refresh to current ~94.3%/~75% figures / freeze as a dated
   snapshot / retire the two catalog rows).
6. **34-row silent `STALE_TEST_DRIFT` bit-rot** on `latency-bench.sh`-sourced rows —
   verified live present in `SURPRISES-INTAKE.md`, STATUS OPEN, a hash-refresh (not a
   content fix) needed; also flags that `walk.sh` does not block on this drift class at
   all today (only `STALE_DOCS_DRIFT`/`MISSING_TEST`/`STALE_TEST_GONE`/`TEST_MISALIGNED`/
   `RETIRE_PROPOSED` block).
7. **Ledger-split decision needed** before the 2026-08-08 file-size waiver lapse:
   `SURPRISES-INTAKE.md`, `GOOD-TO-HAVES.md`, and top-level `ROADMAP.md` are all over the
   20k soft ceiling and still growing — an owner call on splitting/archiving is not yet
   made.
8. **Pre-push wall-time creep** — now FIVE-plus corroborating datapoints across
   `SURPRISES-INTAKE.md` entries (91.7s / 94-95s / 98-99s / 128s / 141s), all above the
   documented ~55-60s budget and above the ~75s re-baseline already proposed in an earlier
   entry. Re-baseline is FILED, not APPLIED — apply during the OP-8 drain, not mid-phase.
   **Every push, by #43 or #44, needs a Bash timeout ≥300s.**

## 6. Precise next steps (successor #44 runbook)

1. **FIRST — resolve the §1 Discrepancy #2 manager-pane question before touching the human
   gate at all.** Re-run `herdr agent read w1:p7 --lines 40`: if the box still shows the
   unsubmitted `done, ran the 11 commands and pushed` line with no manager response, it is
   still stuck — do NOT assume it means the batch ran. Independently re-verify via
   `git fetch origin && git log origin/main -3` (expect still `187809f` unless a new
   commit landed) AND `grep -c '"last_verdict": "RETIRE_PROPOSED"'
   quality/catalogs/doc-alignment.json` (expect `11` if truly not yet run). Only if BOTH
   independent checks show the batch landed should the gate be treated as closed. If the
   message is genuinely stuck unsubmitted and appears to be a real owner reply, submitting
   it via `herdr pane send-keys w1:p7 Enter` is a reasonable unstick — but confirm via the
   git/catalog checks FIRST so a stale or premature message isn't mistaken for ground truth.
2. **Standard first-act verify block:** `git rev-parse HEAD`, `git status --porcelain`,
   `git rev-list --left-right --count HEAD...origin/main` (expect `0/0` once this
   handover's push lands), confirm the tip's CI run concluded success (`gh run list
   --branch main --limit 3`, watch bounded if in flight, Bash timeout ≥360s), then
   `python3 quality/runners/run.py --cadence post-push --persist` (P0
   `code/ci-green-on-main`). Flaky `test` job → re-run ONCE; still red → STOP, escalate,
   never proceed over a red main.
3. **Rescue the P116 ADR-010 packet** — it is real work sitting only in a session-scoped
   `/tmp` scratchpad (§5 item 2) and does not survive a crash. Commit it into
   `.planning/phases/115-live-mcp-benchmark-re-measurement/` early, as cheap insurance
   (regenerate from scratch via a fresh opus researcher dispatch if it's already gone).
4. **Cold-reader pass owed on hero surfaces** (root `CLAUDE.md` requirement, still
   undischarged across #42 AND #43): dispatch `/doc-clarity-review` on `docs/index.md` +
   `README.md` post-strip. Read `GTH-V15-33` first
   (`mental-model-in-60-seconds.md:21` still reads a stale "24 ms") to avoid duplicating an
   already-filed finding.
5. **Re-check the human gate** after step 1's resolution: `git fetch`, then
   `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` — `11`
   means still open, lower/zero means the owner (or a delayed push) landed it.
6. **Phase-close cadence:** re-push + re-probe if new commits landed (including this
   handover's own push). Dispatch a `gsd-verifier` subagent (sonnet) for catalog-row PASS
   grading per `quality/PROTOCOL.md` — expect the 11 `WAIVED-RETIRE_PROPOSED` rows (or
   fewer, if the gate resolved) to grade as the documented human gate per
   `115-UNWAIVE-PATH.md`, NOT as a silent failure. Advance `.planning/STATE.md`'s cursor
   past P115. Refresh `PROGRESS.md` in the same close push (verified fresh as of this
   handover — NOW reads the P115 phase-close focus, NEXT was refreshed in `187809f` with
   the three new GTH items ordered ahead of the remaining milestone phase list; re-verify
   it's still accurate before the close push, don't assume it stays static).
7. **If the human gate is still open at close-readiness:** CHECKPOINT per the manager's
   standing instruction — commit+push+handover naming `confirm-retire` as the sole
   remaining action. Do NOT hold the phase open idle-waiting on the owner.
8. **Immediately post-P115-close:** execute `GTH-V15-35` (the `docs/index.md` install-
   section nesting eager-fix) as a tracked `/gsd-quick`, respecting its two cautions (keep
   `structure/install-leads-with-pkg-mgr-docs-index` green; refresh bound doc-alignment
   rows in the same wave).
9. **Route the (now-committed, per step 3) P116 packet to the MANAGER (w1:p7) for ruling**
   → **END TURN and await the ruling.** Do NOT begin implementation pre-ruling — the P116
   charter is options-only.
10. **File-or-absorb the intermittent tool-failure noticing** (§5 item — 3 leaves hit
    "No tools needed for summary"-style failures per the inherited briefing, not
    independently reproduced by this writer): if seen again this rotation, file a fresh
    `SURPRISES-INTAKE.md` row; if judged the same underlying gap as an existing entry,
    cross-reference rather than duplicate.
11. **Every push needs a Bash timeout ≥300s** — the 120s default kills `git push` mid
    pre-push-hook (§3, §5 item 8).
12. **If the weekly subscription cap hits mid-work:** commit+push whatever landed, REPLACE
    this handover, refresh `PROGRESS.md`, end cleanly. Reset-gating is RETIRED — never defer
    or schedule work AROUND a reset; only react to a cap that actually hits.
