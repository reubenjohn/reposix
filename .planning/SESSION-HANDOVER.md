# SESSION-HANDOVER.md — v0.15.0 Floor milestone re-anchor COMPLETE, phase-execution opens next — 2026-07-14

Written by the **relief-handover-writer** on behalf of **workhorse #24** (L0
orchestrator, pane w1:p5, herded by the manager in w1:p7), relieving to **successor
#25**. This file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md`
(#23→#24's handover, committed at `c1f4f21`).

**Read order:** this file → §1 (verify live, do not trust timestamps) → §6 (runbook) →
dip into §2/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED
staging only, never `git add -A`/`.`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3 --json headSha,status,conclusion
```
**Verified independently this handover (2026-07-14, just now):**
- Local `main` HEAD = `baa3583`, tree **clean** (`git status --porcelain` empty),
  **EVEN** with `origin/main` (`0  0`).
- Newest `ci.yml` run on `main` = headSha `baa35830d068d33905e0803f4dde4e3573fecff4`,
  `status: completed`, `conclusion: success`. The two prior rows (`825c449`, `50b9e39`)
  are also `completed`/`success` — three-deep green streak.
- This handover's own commit will land on top of `baa3583` and get pushed immediately
  after (by the orchestrator, not by this writer — see hard constraints). **By the time
  #25 reads this, HEAD will be that NEW sha and should again read EVEN** — re-poll the
  block above, do not trust this file's timestamp. If still ahead, wait for the push to
  land; if CI is `in_progress`, re-poll; if `failure`, stop and diagnose — never proceed
  over a red or pending main.
- **Commit lineage this rotation** (all pushed, all pre-push-gate-clean):
  `baa3583` (freshness historical-H2 gate robust fix) ← `54a804a` (strip stale
  v0.13.0/v0.13.2 H2 blocks from live root ROADMAP) ← `6cdd283` (create milestone
  v0.15.0 Floor roadmap, 15 phases P114–P128) ← `1c33d91` (restore 3 dropped v0.15.0
  REQUIREMENTS scope gaps) ← `bb12601` (define milestone v0.15.0 Floor requirements) ←
  `825c449` (start milestone v0.15.0 Floor — re-anchor PROJECT.md + STATE.md) ←
  `50b9e39` (manager log, rotations #22–#24) ← `c1f4f21` (#23→#24 handover).

## 2. Wave/cycle state

**`/gsd-new-milestone v0.15.0 Floor` is COMPLETE — both Item 1 and Item 2 from #24's
inherited runbook are DONE.**

| Wave | Artifact | State | Commit(s) |
|---|---|---|---|
| Wave 1 — re-anchor | `.planning/PROJECT.md` + `STATE.md` | **DONE, pushed, CI green.** PROJECT.md: v0.13.0/v0.13.1/v0.14.0 all marked SHIPPED, Arc D RATIFIED (`6aa734a`), Current Milestone = v0.15.0 "Floor". STATE.md: milestone-switched via `gsd-sdk` (`milestone: v0.15.0`, `name: Floor`, `status: planning`), `phases.clear` ran = `cleared: 0` (confirmed safe no-op per #23's de-risking recon). | `825c449` |
| Wave 2 — requirements | `.planning/REQUIREMENTS.md` | **DONE.** 41 REQ-IDs defined: FIX ×3, DOCS ×9, UX ×2, BENCH ×1, ADR ×1, DRAIN ×25. | `bb12601` + amendment `1c33d91` |
| Wave 3 — roadmap | `.planning/ROADMAP.md` | **DONE.** 15 phases (P114–P128), all 41/41 REQ-IDs mapped (verified programmatically, no orphans either direction), OP-8 +2 absorption slots reserved (P127 = SURPRISES drain, P128 = GOOD-TO-HAVES drain + milestone-close). | `6cdd283` |
| Wave 4 — freshness cleanup + gate hardening | root `.planning/ROADMAP.md` + `quality/gates/structure/freshness/structure_misc.py` | **DONE.** Stripped stale v0.13.0/v0.13.2 H2 blocks from the live root ROADMAP (manually, per #24's inherited critical noticing); then went further and **robustly fixed** the freshness historical-H2 gate itself — replaced the hardcoded version regex with a version-tuple comparison against the active milestone read from `STATE.md` (fails closed), plus a 9-test pytest suite. This fully discharges the manager's amendment-2 ask; the gap is no longer a manual-strip liability going forward. | `54a804a` + `baa3583` |
| Push batch | — | All of `825c449..baa3583` pushed; every pre-push gate green (61 PASS / 0 FAIL) across the batch; post-push `code/ci-green-on-main` P0 PASS confirmed live (§1). | — |

**No named-incident / diagnostic pending.** No litmus reopened this rotation.
**Milestone status has moved from "planning" to ready-for-execution** — #25 opens with
`/gsd-plan-phase 114`, not more milestone-definition work.

## 3. Binding constraints (unchanged — carried forward)

- **ONE cargo invocation machine-wide** (prefer `-p <crate>`). Leaf isolation: `/tmp`
  clones, `cd` in the SAME Bash invocation, never the shared tree.
- **Uncommitted = didn't happen.** Push per phase → confirm `code/ci-green-on-main` (P0)
  green → **never open next work over a red or pending main.**
- You **route, don't work**: delegate opus (complex/security), sonnet (default), haiku
  (mechanical); never fable at a leaf. Report to the manager (w1:p7) at each boundary or
  when blocked. Relieve past ~100k own-context tokens (hard stop ~150k) at a clean wave
  boundary — write+commit a handover first (token-absolute, not %-of-window).
- **No `--no-verify`. No tag push by any coordinator** — the MANAGER cuts tags. No git
  surgery on `main`.
- **Shared repo with the manager (w1:p7)** — both commit to the SAME working tree. Use
  TARGETED staging (`git add <explicit path>`, NEVER `git add -A`/`.`) so you never sweep
  the manager's uncommitted `MANAGER-HANDOVER.md` edits. **Do NOT touch
  `.planning/MANAGER-HANDOVER.md`** (separate owner).
- **Owner-only stays owner-only:** interactive sudo, new creds/scopes/spend beyond the
  50-session benchmark ceiling, outward publishing.
- **Arc D is RATIFIED** (`6aa734a`, under owner delegation) — normal GSD gates apply, no
  pipeline pause in effect.

## 4. Litmus / gate / REOPEN state (carried forward, with updates)

- CI green on `baa3583` (§1), three-deep streak. No gate re-run this rotation beyond the
  normal push-cadence pre-push/post-push batch (§2).
- **t4 row (`agent-ux/t4-conflict-rebase-ancestry-real-backend`, P0) = genuine PRODUCT
  DEFECT, is now ROADMAPPED as P114** (FIX-01, `crates/reposix-cache/src/builder.rs`
  `read_blob` oid drift on Confluence). **Do NOT re-run this row to "retire a
  caveat"** — it's a scheduled fix, not an open caveat; P114 is the fix-first opening
  phase of the milestone.
- `milestone-close-vision-litmus` FAIL under mirror non-idempotency is KNOWN — needs
  `scripts/refresh-tokenworld-mirror.sh` run FIRST, before the cadence, or it
  false-negatives on mirror lag every time.
- **UPDATE — the freshness historical-H2 regex gap flagged by #23/#24 is now FIXED
  (robust, version-tuple comparison, `baa3583`).** It is no longer a manual-strip
  liability for future milestone starts; no further action needed here.
- **Open waiver clocks (unchanged):**
  - 8 hero-number doc-alignment rows expire **2026-08-15** = HARD DEADLINE for the
    funded Q1 live MCP re-measurement. **Now addressed by P115 (BENCH-01)**, scheduled
    early in the v0.15 roadmap specifically to beat this deadline.
  - `structure/file-size-limits` waiver expires **2026-08-08** — covers oversized
    `SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md`; the progressive-disclosure split is
    **v0.17 bloat remediation**, not v0.15. Do NOT split it early out of turn.
  - `perf-targets` self-WAIVED until **2026-07-26**.

## 5. Mid-execution decisions + noticings (KEY)

**Scope-gap triage this rotation — 3 items the parent triaged back IN after a
verification pass over what Wave 2/3 initially captured:**

1. **GTH-09 → REQ FIX-03.** Verified owner-carried-forward v0.15.0 scope: tagged
   `TAG v0.15.0`, explicitly deferred FROM the v0.14.0 close INTO v0.15 — "not a silent
   slip," a real prior commitment. It's a MEDIUM-HIGH data-integrity hazard: the
   ADR-010 convergence contract is FALSE for CREATEs on id-assigning real backends (an
   interrupted create can duplicate on retry, because the slug→id binding isn't durable
   across the failure window). It was sitting in ROOT `.planning/GOOD-TO-HAVES.md`
   rather than a phases-scoped intake file — that's why the first Wave-2 pass missed
   it. Now co-located with ADR-01 in **P116**.
2. **4 homeless `SURPRISES-INTAKE.md` rows → DRAIN-22..25** (OP-8 drain scope, folded
   into the existing DRAIN REQ-ID family).
3. **Tag-cut prose correction → DOCS-09.** All three tags (v0.13.0, v0.13.1, v0.14.0)
   are already cut and public — this REQ is a PROSE fix confirming the archival cascade
   ran, **NEVER** a literal tag-cut action. (Carried forward from #23/#24's earlier
   "LAUNCH-BLOCKER #7 = OBSOLETE" finding — do not re-litigate, just don't misread the
   REQ-ID as an action item.)

**ADR-010 now has TWO in-scope concerns, both routed into P116** (a top-level,
decision-only phase — no implementation):
- **ADR-01** — the mirror-fanout decision packet the manager is waiting to rule on
  (produce options + tradeoffs, do not pre-decide).
- **FIX-03** — the GTH-09 slug→id durable-create hazard above.

P116's job is to PRODUCE the options+tradeoffs packet for BOTH and route it to the
**MANAGER (w1:p7) for ruling** — **do NOT implement any chosen ADR-010 option ahead of
that ruling.** The manager has already been told (this rotation) that both concerns are
co-located in the same phase, so it's not new information to relay — just don't skip
producing the packet.

**Carry-forward noticings to FILE (not yet filed by #24 — inherited debt for #25):**

a. **`gsd-sdk` `commit --message` footgun.** The commit message argument is
   **POSITIONAL**, not a `--message` flag; passing `--message "..."` silently commits a
   garbage/empty message instead of erroring. Fix-twice obligation: (i) file a
   SURPRISES/infra intake row, (ii) update the coordinator-dispatch skill /
   `ORCHESTRATION.md` commit example to the correct form —
   `gsd-sdk query commit "<msg>" --files <path>`. **This handover itself was written
   using the correct positional form** (see §-writer's own commit below) — the fix is
   about the DOCUMENTED example other agents copy from, not this specific commit.
b. **Stale catalog example text.** `quality/catalogs/freshness-invariants.json`
   (~L227–229), the `structure/top-level-requirements-roadmap-scope` row's
   `expected.asserts` text still hardcodes a stale `"v0.12.0"` example. Doc-only,
   non-blocking, cosmetic — file for a catalog refresh; fits naturally inside P119
   (a DOCS-lane phase) rather than a standalone fix.
c. **`PROJECT.md` "Context" section is FUSE-era.** Still carries the old
   `/mnt/jira/PROJ-123.md` FUSE-mount example and a disputed "~150k→~2k (98.7%)" token
   headline figure. **Deliberately left alone** — this is exactly what P115 (BENCH-01
   re-measure) and P117/P118 (DOCS truth-purge lanes) exist to fix with real
   measurement, not a half-fix now. Do not touch ahead of those phases.

## 6. Precise next steps (successor #25 runbook)

1. **Re-verify §1 ground truth live first.** Confirm `main` is EVEN with
   `origin/main` and the newest CI run on the current HEAD sha is `success` before
   opening any new work. If still ahead, wait for the push to land; if CI is
   `in_progress`, re-poll; if `failure`, stop and diagnose.
2. **Milestone re-anchor + definition + planning are DONE — begin phase EXECUTION.**
   Opening move: `/gsd-plan-phase 114` (t4 Confluence oid-drift fix-first, REQs FIX-01 +
   FIX-02). Then proceed per normal GSD cadence: discuss → plan → execute → verify per
   phase, push-per-phase, verifier subagent grades catalog rows before phase-close.
3. **Schedule P115 (BENCH-01) early.** Hero-number waiver HARD DEADLINE is
   **2026-08-15**. Spend ceiling ≤50 benchmark sessions on the existing subscription
   (owner-confirmed) — escalate to the manager only past 50, never exceed without a
   manager GO.
4. **P116 (ADR-010 packet)** — produce the options+tradeoffs packet for BOTH ADR-01
   (mirror-fanout) and FIX-03 (slug→id durable create); route to the MANAGER for
   ruling; **no pre-ruling implementation** of either concern.
5. **Directive 2 (GSD-quick scale, low urgency, still NOT started):** scratch-repo
   `reposix-scope-test-DELETEME` KEEP-policy doc into
   `docs/reference/testing-targets.md` (reset via force-push, never delete; currently
   archived, unarchive via API on first reuse).
6. **File the 3 carry-forward noticings from §5 early** (the `gsd-sdk` positional-arg
   footgun, the stale catalog example text, and confirm the PROJECT.md FUSE-era Context
   section stays untouched until its scheduled phases) so they aren't lost to another
   rotation.
7. **Push per cadence** + `quality/runners/run.py --cadence post-push --persist` →
   confirm `code/ci-green-on-main` (P0) green; never open next phase over a
   red/pending main.
8. **Report to the manager (w1:p7)** at each phase boundary; **relieve past ~100k
   own-context tokens at the next clean wave boundary** — write+commit a fresh
   `.planning/SESSION-HANDOVER.md` (REPLACE, not append), naming successor **#26**,
   following this same §3-of-`ORCHESTRATION.md` template.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** (current milestone, execution now open) → **v0.17
meta-milestone** (5 gate shapes: pivot-vocabulary lint, nav-budget, hero-redundancy,
framing-claim rows, persona whole-journey rubric; + subjective-runner Task-dispatch fix
unfreezing 3 WAIVED meaning-gates; + waiver-escalation rule; + transcript retention; +
bloat remediation incl. the SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure
split) → **v0.19** truth purge + IA rebuild → **v0.21** benchmark honesty (re-fixture
live baseline, CI job, headline-cross-check verifier) → **v0.23** journey slices →
**v0.25** launch kit → Show-HN. **Q3 launch gate:** Show-HN gated on a walkable
REAL-BACKEND journey (GitHub minimum), not sim-first. **Deep-survey calibration:** ~10%
latent work per pass, ~10 passes to converge, recurring deep surveys are STANDING
practice. **Q9 ceiling:** keep v0.15→v0.25 ≈ 6-milestone scale.
