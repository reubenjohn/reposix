# SESSION-HANDOVER.md — RED-main blocker `b773c04` CLOSED (main GREEN @ `8e2aae5`); successor #17 resumes post-tag queue items 0–5 — 2026-07-13

Map, not territory — detail lives in git + the linked committed artifacts, not restated
here. **HEAD = live state; verify live before trusting anything in this file.** This
REPLACES (does not append to) the prior `SESSION-HANDOVER.md` (successor #16's charter,
written while the fix-verification CI run was still in flight). That run concluded
GREEN — successor #16's entire session was closing out that one blocker; the post-tag
queue itself has **NOT been touched yet**. Resume an agent via SendMessage, never fork
(ORCHESTRATION §11).

**STATUS: v0.14.0 SHIPPED (crates.io 0.14.0, GH release "Latest", unchanged this
session). The post-shipping RED-main incident (owner ruling `b773c04`, diagnosed as a
docs-repro CI timeout budget, NOT a hang/guardrail conflict) is RESOLVED and CI-verified
GREEN. Successor #17's job is the post-tag queue (items 0–5), starting with item 0
(cursor refresh, which also carries the push).**

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main
```
Verified this session: HEAD = `8e2aae5`, tree **clean**, `0 0` vs `origin/main` (i.e.
`8e2aae5` is already pushed). **This handover's own commit will land ON TOP of
`8e2aae5`, unpushed** — per the queue's own item-0 instruction, push it together with
the item-0 cursor-refresh commit to avoid a second CI trigger.

Commit chain this arc, oldest → newest (baseline: prior handover's HEAD `05aa23c`):
- `8e2aae5` (HEAD) — **fix(quality): green example-04 via TIMEOUT-BUDGET fix (ruling
  `b773c04`)**, quick `260713-rug` (`.planning/quick/260713-rug-example04-timeout-budget/`
  has PLAN + SUMMARY). Root cause: `docs-repro/example-04-conflict-resolve` was FAILING
  at exactly 300.00s in `quality-post-release` run `29301412750` — reproduced twice in
  `/tmp`, VERDICT = TIMEOUT-BUDGET, not a hang. The example workflow itself is ~0.5s and
  passes all 3 asserts; the 300s cap was being consumed by per-container-row SETUP
  (`apt-get install build-essential pkg-config libssl-dev`), compile-time deps the
  container examples never exercise (they run the host-mounted pre-built
  `target/debug/reposix`). Two edits, no asserts/waivers/proofs touched: (1)
  `quality/gates/docs-repro/container-rehearse.sh` SETUP drops those 3 packages, keeps
  curl/ca-certificates/python3/git/sqlite3 (+ fix-it-twice comment); (2)
  `quality/catalogs/docs-reproducible.json` bumps `timeout_s` 300→600 on the 4
  `kind:container` rows (**confirmed this session, lines 27/64/137/174**);
  `tutorial-replay` (kind:mechanical, line 213) is untouched at 300.

**Numbered facts the successor MUST know:**
1. **Prove-before-fix reality-check (executed, not assumed):** all 4 container rows ran
   locally with rc=0 before the fix landed on top of the earlier honest rework — example-01
   16s, example-02 15s, example-04 16s (was the 300s SIGKILL), example-05 19s.
2. **CI confirmation (both GREEN, checked this session):**
   `gh run view 29302973970` → **SUCCESS**, 4m38s, docs-repro gate green (`quality-post-release`,
   workflow_dispatch on `8e2aae5`). `gh run view 29302967371` → **SUCCESS**, all 15 jobs
   including dark-factory + all 6 real-backend integration jobs (`ci.yml` push on `8e2aae5`).
3. **No `CONSULT-DECISIONS.md` entry was added for this fix** — #16 judged it a
   straightforward reproduced-and-proven fix (DP-2/DP-3 clear, all four escalation-valve
   checks E1–E4 negative) and recorded it `[SELF]` only in the quick-lane SUMMARY, not the
   ledger. If you want the ledger complete, add a one-line `[SELF]` entry — git (the
   commit + the quick SUMMARY) is already the durable record either way; this is a
   nice-to-have, not a gap that blocks anything.
4. **Ground-truth correction inherited from #16, still true:** `.planning/STATE.md`'s
   `workstream_a` (v0.13.0) block still reads `blocks_tag: true` /
   `next_phase: P98  # ...awaiting owner pre-tag actions + L0 tag push`. **v0.13.0 was
   already tagged and released 2026-07-07** (`chore: release v0.13.0 (#68)`, commit
   `3423b18`), and v0.13.1 shipped 2026-07-08 (`04640d5`) — both predate v0.14.0. STATE.md's
   status line (`status: v0.13.1-shipped-v0.14.0-fix-first-items-4-8-pending`) is also
   stale — fix-first is done. **Do not re-run a v0.13.0 tag-prep sequence.** The
   genuinely-still-queued tag is `workstream_b` = **v0.13.2** (STATE.md line 17:
   `milestone: v0.13.2`, `status: queued`, `blocks_tag: false`, per line 108 "QUEUED behind
   workstream C... and the not-yet-scoped launch-readiness milestone"). Confirm with
   whoever is dispatching which milestone "post-tag queue item 1" was actually meant to
   name before spending effort.

## 2. Wave/cycle state

| Step | Artifact | State | Commit |
|---|---|---|---|
| Diagnose `docs-repro/example-04` 300.00s failure (opus investigation leaf, /tmp repro x2) | quick `260713-rug` PLAN | DONE — VERDICT: TIMEOUT-BUDGET | — |
| Fix: drop compile-time apt deps from container SETUP + bump 4 container rows to 600s | `container-rehearse.sh`, `docs-reproducible.json` | DONE, pushed | `8e2aae5` |
| Prove-before-fix + prove-after-fix (4/4 container rows rc=0 locally) | quick SUMMARY | DONE | `8e2aae5` |
| CI verification (`quality-post-release` + `ci.yml` on `8e2aae5`) | run `29302973970` / `29302967371` | DONE — **both SUCCESS** | — |
| Report blocker CLOSED to manager (w1:p7) | — | DONE (this session) | — |
| **Post-tag queue items 0–5** (§6 below) | — | **NOT STARTED** | — |

No named-incident post-mortem beyond §1 above (TIMEOUT-BUDGET root cause) — read the
quick `260713-rug` PLAN/SUMMARY before touching `container-rehearse.sh` or
`docs-reproducible.json` again.

## 3. Binding constraints (unchanged)

- Reality-check arc is **NOT owner-ratified** — no defect-fixing lanes beyond
  tag-blockers; OPEN intakes (v0.15.0 or otherwise, see §5) route forward, do NOT drain
  them now.
- ONE cargo invocation machine-wide (prefer `-p <crate>`). Leaf isolation: `/tmp` clones,
  `cd` in the SAME Bash call, never the shared tree.
- Uncommitted = didn't happen. Push per phase cadence → then the post-push cadence
  (`code/ci-green-on-main`, P0) → **never proceed over a red main.**
- You **route, don't work**: delegate to opus (complex/security), sonnet (default),
  haiku (mechanical). Relieve past ~100k own-context tokens (hard stop ~150k) at a wave
  boundary — write+commit a handover first. Report to the manager (w1:p7) at each
  queue-item boundary or when blocked.
- No `--no-verify`. **No tag push by any coordinator** — the MANAGER cuts tags, never the
  coordinator, even at READY-TO-TAG.

## 4. Litmus / gate / REOPEN state

- **`quality-post-release` run `29302973970`** (workflow_dispatch, sha `8e2aae5`) —
  **SUCCESS**, 4m38s. This is the run that closes the `b773c04` blocker; do not
  re-litigate it.
- **`ci.yml` run `29302967371`** (push, sha `8e2aae5`) — **SUCCESS**, all 15 jobs
  including dark-factory regression and all 6 real-backend integration contract jobs.
- **F-K4b congruence-check verifier logic itself is untouched** — this fix touched only
  timeout budgets and SETUP package lists, no assert/waiver/proof surface. The still-open
  F-K4b container-class tautology issue (§5 below) is a SEPARATE, already-filed item —
  do not conflate it with this arc.
- No open REOPEN-gate clock. No P0 row carries a waiver from this fix.

## 5. Mid-execution decisions + noticed-not-filed

- **`[SELF]` decision (this session, not yet in `CONSULT-DECISIONS.md` — see §1 item
  3):** TIMEOUT-BUDGET fix judged self-decidable — reproduced twice, proven before AND
  after, both CI gates confirmed green, no escalation-valve trigger (E1–E4 all negative).
- **NEW noticings this arc — route to v0.15.0 intake, NOT yet filed, do not drop
  (global principle #2):**
  1. **Sim-leak-on-SIGKILL (robustness/flakiness, MEDIUM, SURPRISES-INTAKE
     candidate).** `container-rehearse.sh` backgrounds the sim (`&`) and cleans it via an
     EXIT trap; when `subprocess.run(timeout=...)` SIGKILLs the script (as it did in the
     original CI failure — `Terminate orphan process pid 15322`), the trap never fires and
     the sim orphans on port 7878. A subsequent container row could then bind-fail or
     silently curl a stale sim. Hardening sketch: wrap the docker run in an internal
     `timeout` shorter than the row's `timeout_s`, and/or start the sim in its own process
     group.
  2. **Harness rc(0) vs artifact exit_code(1) mismatch + a sim-readiness race between
     rapid sequential `container-rehearse.sh` runs** (GOOD-TO-HAVE) — executor observed
     example-02 flake once on back-to-back local runs, rc=0 on an isolated re-run.
     Reconcile the two success signals.
  3. **`.sim-*.log` files under `quality/reports/verifications/docs-repro/` are NOT
     gitignored** (confirmed this session — `.gitignore` has the `*.json` and
     `*.cobertura.xml` patterns for that tree but no `.sim-*.log` pattern; the sibling
     `.json` reports ARE covered). One-line `.gitignore` fix, GOOD-TO-HAVE. (#16 manually
     removed local residue for a clean tree; the gap in `.gitignore` itself remains.)
  4. **Post-release-CI binary provenance (verify, don't assume).** `quality-post-release.yml`
     has no explicit `cargo build -p reposix-cli` step visible, yet `container-rehearse.sh`
     needs `target/debug/reposix` host-mounted. Run `29302973970` executed the rows
     successfully (so the binary WAS present), but the provenance (cache restore? prior
     job artifact? a build step not obviously named?) is unconfirmed — worth a 10-minute
     look so rows don't silently degrade to NOT-VERIFIED on a clean/cold runner.
- **Carried-forward noticings (still live, low-urgency, unchanged from #16's handover):**
  - `.planning/CONSULT-DECISIONS.md` is large relative to the ~20k progressive-disclosure
    guideline (400 lines / historically flagged ~54k chars) — item 5's rider (below)
    addresses this with manager-pre-approved pruning of entries older than the current
    milestone; the file is NOT strictly chronological (Ruling #5 sits near the top at
    line 13; older resolved sagas — e.g. the TokenWorld mirror-drift/DIAGNOSTIC-lane
    thread — run to the bottom), so prune by content/date, not by position.
  - `quality/gates/docs-repro/container-rehearse.sh`'s header comment still claims
    "<=150 lines" — the file is actually ~195 lines. Stale comment, GOOD-TO-HAVE, not
    touched this arc (the fix only edited the SETUP block, not the header).
  - `.playwright-mcp/audit-03..08*.png` droppings **confirmed still present on disk this
    session** (gitignored, 6 files, all dated 2026-07-12): `audit-03-landing-narrow-viewport.png`,
    `audit-04-confluence-reference.png`, `audit-05-first-run.png`,
    `audit-06-token-economy.png`, `audit-07-git-layer.png`, `audit-08-filesystem-narrow.png`.
    This is queue item 3 below.

## 6. Precise next steps (successor #17 runbook)

Manager herds from w1:p7; report at each numbered boundary or if blocked.

1. **Item 0 — GSD cursor refresh (`/gsd-quick` scale; PUSH rides here).**
   - Update `STATE.md` + `PROJECT.md` to: v0.14.0 SHIPPED (unchanged); RED-main arc
     `b773c04` CLOSED, main green @ `8e2aae5` (both CI gates SUCCESS, §4); AND fix the
     stale `workstream_a` block (§1 item 4) — it still says `blocks_tag: true` /
     `next_phase: P98 # awaiting owner pre-tag actions + L0 tag push` for a milestone
     (v0.13.0) tagged/released 2026-07-07. Also fix the stale top-line `status:` field.
   - Fold this file's §5 noticings into the appropriate intake file(s)
     (`SURPRISES-INTAKE.md` for items 1/4, `GOOD-TO-HAVES.md` for items 2/3) — v0.15.0
     scope per the reality-check-arc constraint (§3).
   - **Push this commit + the SESSION-HANDOVER commit together** (one CI trigger), then
     run `python3 quality/runners/run.py --cadence post-push --persist` and confirm
     `code/ci-green-on-main` (P0) is green before opening item 1.
2. **Item 1 — reconcile identity FIRST, then act.** "Item 1 = v0.13.0 tag sequence" is
   STALE (nothing to do, already shipped — §1 item 4). The genuinely-queued tag is
   **v0.13.2** (workstream_b, Cross-link fidelity). Confirm which was meant before
   spending effort.
   - **HARD PRECONDITION before ANY READY-TO-TAG report, regardless of milestone:**
     resolve the `make_latest` hazard. v0.13.0's `release.yml` calls `gh release create`
     with no `--latest` flag; the GitHub API default (`make_latest=true`) could steal
     `releases/latest` from v0.14.0 and 404 the installer URLs on a back-tag. A prior
     digest GUESSED this is safe but it was NEVER verified against real `gh`/API
     behavior. **VERIFY actual `make_latest` behavior first** (read the `gh release
     create` docs/source, or test against a disposable repo) — **if a back-tag could
     steal "latest", STOP and report to the manager**, do not proceed.
   - A READY-TO-TAG report MUST include a tag-script guards **DRY-RUN** result.
   - **STOP at READY-TO-TAG — the MANAGER (w1:p7) cuts tags, never the coordinator.**
3. **Item 2 — Q1c interim hero qualifiers.** README "Three measured numbers" +
   `docs/index.md:17` synthetic-baseline caveat. Run a cold-reader pass via
   `/doc-clarity-review` on the touched pages before calling it done.
4. **Item 3 — `.playwright-mcp/audit-03..08*` droppings sweep.** 6 files confirmed
   present this session (§5, gitignored, from 2026-07-12). Verify nothing in that dir
   depends on them (they look like a one-off doc-alignment walk's screenshots), then
   `rm`.
5. **Item 4 — `/gsd-cleanup` archival cascade.**
6. **Item 5 — `ORCHESTRATION.md` progressive-disclosure size split**, plus the
   manager-pre-approved rider: archive `CONSULT-DECISIONS.md` entries older than the
   current milestone to `.planning/archive/` (keep recent rulings — currently #2–#5 and
   this session's `[SELF]` note if added — inline; older resolved sagas, e.g. the
   TokenWorld mirror-drift thread from lines ~195–400, are archive candidates).
7. **Do not drain the reality-check-arc intakes** (v0.15.0 or otherwise) beyond
   tag-blockers — that arc is not owner-ratified for defect-fixing lanes yet.
8. **At each queue-item boundary, or if blocked, report to the manager (w1:p7)** — do not
   silently continue past a boundary without a checkpoint.
