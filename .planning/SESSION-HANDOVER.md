# SESSION-HANDOVER.md — RED-main BLOCKER b773c04 RESOLVED via Manager Ruling #5/Option A; CI verification IN FLIGHT @ 05aa23c; successor #16 resumes the post-tag queue — 2026-07-13

Map, not territory — detail lives in git + the linked committed artifacts, not restated
here. **HEAD = live state; verify live before trusting anything in this file** — it is a
snapshot, not a subscription. This REPLACES (does not append to) the prior
`SESSION-HANDOVER.md` (written under Ruling #4/Option B for the now-shipped v0.14.0 tag
push; fully superseded — the tag landed, then a NEW RED-main incident happened and was
fixed this arc). Resume an agent via SendMessage, never fork (ORCHESTRATION §11).

**STATUS: v0.14.0 SHIPPED (tag @ `bcdee07`-era chain, crates.io 0.14.0, GH release
2026-07-14T01:23:03Z, "Latest"). A RED-main blocker surfaced AFTER shipping (owner ruling
`b773c04`); it is now RESOLVED via an honest bounded fix under Manager Ruling #5 (Option
A). The fix is pushed to `origin/main`; a CI verification run was IN FLIGHT (not yet
concluded) at handoff. Successor #16's FIRST job is to confirm that run went GREEN before
touching anything else.**

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run view 29301412750
```
Verified this session: HEAD = `05aa23c`, tree **clean**, `0 0` vs `origin/main` (this
commit is already pushed — 05aa23c itself, NOT this handover commit on top of it, which
per this session's charter is intentionally **left unpushed** for the successor to push
alongside item-0's cursor update, to avoid a second CI trigger mid-verification).

Commit chain this arc, oldest → newest (baseline: prior handover's HEAD `d68fa8a`, which
was RED, not green — see deviation note below):
- `03e7a6f` — **fix(quality): honest bounded F-K4b fix.** Rewords `container-rehearse.sh`
  example-05 asserts #2/#3 to what `run.sh` actually proves on exit 0 (pre-emptive
  sparse-checkout pattern + guard-constant grep — NOT a runtime blob-limit error
  observation, which the fast-import fetch path never triggers). Examples 01/02/04 keep
  the exit-0-emits-`expected.asserts` behavior as-is (verified genuinely fail-loud).
  F-K4b verifier logic untouched; no waivers added.
- `3775075` — v0.15.0 intake filed:
  `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (ONE MEDIUM row, two
  sub-items: F-K4b container-class tautology redesign + example-05 real-runtime-error
  deeper fix).
- `05aa23c` (HEAD) — Ruling #5 recorded in `CONSULT-DECISIONS.md` + `STATE.md` ledger row
  updated with the honest-rework framing and SHAs. No push in this commit (documented as
  "orchestrator owns push + post-release re-trigger per Ruling #5 rider 4" — that push
  already happened; HEAD and `origin/main` agree at `05aa23c`).

**Numbered deviations the successor MUST know:**
1. **Root cause was P106 (`804eedc` + `c4f1261`, 2026-07-12), a HARNESS regression, not a
   binary defect** — `container-rehearse.sh` hand-minted a `status:PASS` the container
   example rows can't honestly earn under the runner's F-K4b per-assert congruence check.
   Named suspects `cb8ad11`/`970d466` (surfaced earlier as candidate causes) were **red
   herrings** — do not re-investigate them.
2. Symptom-fix commit `0f2b7c5` is **orphaned** (not reachable from any branch — it was
   un-stacked via `git reset --soft d68fa8a` when the arc pivoted from a symptom patch to
   the honest rework). Do not try to cherry-pick or reference it; the honest fix
   (`03e7a6f`) fully supersedes it.
3. **Ground-truth catch this session — the "post-tag queue Item 1: v0.13.0 tag
   sequence, drive to READY-TO-TAG" framing carried into this handover is STALE.**
   `git tag -l v0.13.0` / `gh release list` show **v0.13.0 was already tagged and
   released on 2026-07-07** (`chore: release v0.13.0 (#68)`, commit `3423b18`), and
   v0.13.1 shipped 2026-07-08 — both predate v0.14.0 (2026-07-14, current "Latest").
   `.planning/STATE.md`'s `workstream_a` block (`blocks_tag: true`, `next_phase: P98
   # ...awaiting owner pre-tag actions + L0 tag push`) was never updated after that tag
   actually landed — it is exactly the kind of staleness item-0's cursor refresh exists
   to fix. **Do not re-run a v0.13.0 tag-prep sequence.** The genuinely-still-queued tag
   is `workstream_b` = **v0.13.2** (`status: queued`, `blocks_tag: false` per STATE.md) —
   verify with the successor which milestone "Item 1" was actually meant to name before
   spending any effort on it.

## 2. Wave/cycle state

| Step | Artifact | State | Commit |
|---|---|---|---|
| RED-main root-cause diagnosis | Ruling #5 entry, `CONSULT-DECISIONS.md` | DONE | `05aa23c` |
| Honest bounded F-K4b fix (quick 260713-q0e) | `container-rehearse.sh` + example-05 asserts | DONE, pushed | `03e7a6f` |
| v0.15.0 intake (F-K4b redesign + ex-05 deeper fix) | `v0.15.0-phases/SURPRISES-INTAKE.md` | DONE | `3775075` |
| STATE.md ledger row + Ruling #5 doc | `STATE.md`, `CONSULT-DECISIONS.md` | DONE | `05aa23c` |
| Rider-5 cleanup (260713-pnv scaffold + docs-repro report droppings) | working tree | DONE | (no residue found this session) |
| CI verification of the fix | `quality-post-release` run `29301412750` | **IN FLIGHT at handoff** — not concluded when last checked; superseded prior RED run `29298424648` (sha `d68fa8a`) | — |
| Post-tag queue items 0–5 (below) | — | **NOT STARTED** — blocked on the above run's conclusion | — |

No named-incident post-mortem beyond §1 item 1 above (P106 root cause) — read the Ruling
#5 `CONSULT-DECISIONS.md` entry + quick `260713-q0e` SUMMARY/PLAN before dispatching any
further F-K4b-adjacent work.

## 3. Binding constraints (unchanged)

- Reality-check arc is **NOT owner-ratified** — no defect-fixing lanes beyond
  tag-blockers; the (now potentially 1, was 8) OPEN intakes route to v0.15.0, do NOT
  drain them now.
- ONE cargo invocation machine-wide (prefer `-p <crate>`). Leaf isolation: `/tmp` clones,
  `cd` in the SAME Bash call, never the shared tree.
- Uncommitted = didn't happen. Push per phase cadence → then the post-push cadence
  (`code/ci-green-on-main`, P0) → **never proceed over a red main.**
- You **route, don't work**: delegate to opus (complex/security), sonnet (default),
  haiku (mechanical). Relieve past ~100k own-context tokens (hard stop ~150k) at a wave
  boundary — write+commit a handover first. Report to the manager (w1:p7) at each
  queue-item boundary or when blocked.
- No `--no-verify`, no tag push by any coordinator (manager's action alone, when a tag
  arc is live).

## 4. Litmus / gate / REOPEN state

- **`quality-post-release` run `29301412750`** — triggered `workflow_dispatch` on sha
  `05aa23c`; status at last check was **queued/in-progress, NOT yet concluded**. This is
  the run that must be confirmed GREEN before anything else proceeds (§6 step 1).
- **Prior run `29298424648`** (sha `d68fa8a`, the pre-fix HEAD) was **RED** — this is the
  run the fix targets; it is now superseded, do not re-litigate it once `29301412750`
  concludes.
- **F-K4b verifier logic itself is untouched** by the fix — no waiver was added anywhere;
  the fix changed only what example-05's asserts CLAIM (to match what the script
  provably does), not the congruence check that catches false claims.
- No open REOPEN-gate clock beyond confirming the in-flight run's conclusion. No P0 row
  carries a waiver.

## 5. Mid-execution decisions + noticed-not-filed

- **Manager Ruling #5 (E2/E3 valve, CLOSED, Option A executed):** honest bounded fix —
  KEEP `container-rehearse.sh`'s exit-0-emits-`expected.asserts` behavior for examples
  01/02/04 (genuinely fail-loud), REWORD example-05's asserts #2/#3 to the truth rather
  than either (B) reversing the shipped P106 behavior or (C) deferring the whole fix.
  The deeper redesign (per-step-earned emission, or a fail-loud meta-check for the
  container class generally) and example-05's still-missing real-runtime-error coverage
  are both FILED to v0.15.0, not fixed now. Full binding text:
  `.planning/CONSULT-DECISIONS.md` `2026-07-13 [MANAGER] Ruling #5`. Do not re-litigate.
- **Red herrings, do not re-open:** `cb8ad11` and `970d466` were investigated as possible
  RED-main causes and cleared — the actual root cause was P106's harness-side
  `status:PASS` minting, not either of those commits.
- **Noticed, filed this session** (`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`,
  one MEDIUM row): (a) F-K4b container-class tautology — any no-op exit-0 script would
  currently pass the congruence check for a `kind:container` row, because honesty rests
  entirely on each script being fail-loud rather than on a structural guarantee; (b)
  example-05 only exercises the pre-emptive sparse-checkout pattern, not a genuine
  observe-runtime-error → sparse-checkout → retry cycle (that cycle is currently covered
  only by `dark-factory.sh`).
- **Noticed, not filed as a fresh intake (carried forward, low-urgency housekeeping)**:
  - `.planning/CONSULT-DECISIONS.md` is ~53.8k chars vs. the ~20k progressive-disclosure
    guideline (pre-commit WARN, non-blocking). Per decision-procedures the ledger should
    stay bounded to LIVE decisions (git is the archive) but it still carries the full
    closed manager-rulings archive #2–#5. Flag to the manager before pruning — it's the
    manager's own ruling-archive pattern, so deletion needs the manager's sign-off, not a
    unilateral coordinator trim.
  - `quality/gates/docs-repro/container-rehearse.sh`'s header comment still claims
    "<=150 lines" — the file is actually **195 lines** (confirmed this session via
    `wc -l`). Stale comment, GOOD-TO-HAVE-sized, not touched this arc.
  - `.playwright-mcp/audit-03..08*.png` droppings are confirmed present on disk
    (gitignored, ~6 files from 2026-07-12) — this is queue item 3 below, still pending.
  - **New this session (§1 item 3 above):** `.planning/STATE.md`'s `workstream_a`
    (v0.13.0) block is stale by ~a week — it still reads "awaiting owner pre-tag actions
    + L0 tag push" for a milestone that was tagged and released 2026-07-07. This should
    be folded into item-0's cursor refresh, and the post-tag-queue's "Item 1" identity
    (v0.13.0 vs. the actually-still-queued v0.13.2) needs reconciling, not blind
    execution.

## 6. Precise next steps (successor runbook)

1. **Confirm run `29301412750` concluded SUCCESS**: `gh run view 29301412750` (or `gh run
   watch 29301412750` if still in progress). 
   - **GREEN** → report "BLOCKER `b773c04` CLOSED, main green @ `05aa23c`" to the manager,
     then proceed to step 2.
   - **RED** → main was ALREADY red going into this fix (not a fresh regression from the
     fix itself, unless the failure signature is new) — re-classify honestly and report
     to the manager BEFORE proceeding to any queue item. Do not open queue work over a
     red main.
2. **Resume the post-tag queue** (was manager-preempted pending the main-green
   precondition, now met if step 1 is GREEN):
   - **Item 0 — GSD cursor refresh.** Update `STATE.md` + `PROJECT.md` (+ reconcile this
     `SESSION-HANDOVER.md`) to reflect: v0.14.0 SHIPPED; main green @ `05aa23c` (NOT
     `d68fa8a` — that sha was RED; an earlier item-0 lane was killed after being handed
     the false "green @ `d68fa8a`" claim, per the prior coordinator's notes); AND the
     `workstream_a` (v0.13.0) stale-tag-pending framing from §1 item 3 / §5 above. Run at
     `/gsd-quick` scale. Push it (bundle this handover's commit in the same push per this
     session's push-deferral instruction).
   - **Item 1 — reconcile identity, then act.** Before driving anything to
     READY-TO-TAG, confirm whether "Item 1" means v0.13.0 (**already tagged/released
     2026-07-07 — nothing to do**) or v0.13.2 (Cross-link fidelity, genuinely
     `status: queued` per STATE.md workstream_b). If it's v0.13.2: drive to
     READY-TO-TAG (manager cuts the tag, never the coordinator); the READY-TO-TAG report
     MUST include a tag-script guards DRY-RUN result. **Carry-forward escalation trigger
     (verify before any READY-TO-TAG report, regardless of which milestone):** a prior
     digest concluded `gh release create` without `--latest` would NOT steal "latest"
     from v0.14.0 on this `gh` CLI version, but that was never verified against real `gh`
     CLI behavior — **VERIFY the actual `make_latest` behavior first; if a new tag could
     steal "latest"/404 installer URLs, STOP and report to the manager** instead of
     executing.
   - **Item 2 — Q1c interim hero qualifiers.** README "Three measured numbers" +
     `docs/index.md:17` synthetic-baseline caveat. Cold-reader pass via
     `/doc-clarity-review` on touched pages before calling it done.
   - **Item 3 — `.playwright-mcp/audit-03..08*` droppings sweep.** Confirmed present
     this session (gitignored, ~6 files, 2026-07-12). Simple `rm`, verify nothing else in
     that dir depends on them first.
   - **Item 4 — `/gsd-cleanup` archival cascade.** The v0.14.0 tag unblocks this; run it.
   - **Item 5 — `ORCHESTRATION.md` size split.** Currently over its progressive-disclosure
     budget; apply the same layering rules the doc itself prescribes.
3. **Do not drain the reality-check-arc intakes** (v0.15.0 or otherwise) beyond
   tag-blockers — that arc is not owner-ratified for defect-fixing lanes yet.
4. **At each queue-item boundary, or if blocked, report to the manager (w1:p7)** — do not
   silently continue past a boundary without a checkpoint.
