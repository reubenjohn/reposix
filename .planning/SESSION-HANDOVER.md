# SESSION-HANDOVER.md — t4 = v0.15 fix-first PRODUCT DEFECT (not a caveat); 6 intake rows filed — 2026-07-14

Written by the **relief-handover-writer** on behalf of **workhorse #22** (L0 orchestrator,
pane w1:p5, herded by the manager in w1:p7), relieving to **successor #23**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#21→#22's handover).

**Read order:** this file → §1 (verify live, do not trust timestamps) → §6 (runbook) →
dip into §2/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3 --json headSha,status,conclusion
```
**Verified independently this handover (2026-07-14, just now):**
- Local `main` HEAD (before this handover's commit) = `15e816d` (the intake-filing
  commit), tree **clean**, **1 ahead** of `origin/main` (`14c4a42`). This handover's
  commit lands on top → local goes **2 ahead**. **#22 pushes BOTH commits together to
  `origin/main` immediately after this commit lands**, once CI on `14c4a42` (run
  `29386901268`, confirmed `success` live) has already concluded green — it has.
- By the time #23 reads this, `origin/main` SHOULD already be at this handover's commit
  with CI green — but **do not trust that assumption**. Re-run the block above and
  confirm: (a) `git rev-list --left-right --count HEAD...origin/main` reads `0  0`
  (EVEN), and (b) the newest `gh run list` row for the current HEAD sha shows
  `status: completed, conclusion: success`, **before opening any new work**. If still
  ahead or CI still `in_progress`/pending, wait/re-poll; if CI concluded `failure`, stop
  and diagnose — never proceed over a red or pending main.
- Commit lineage this rotation (`15e816d` back to `14c4a42`): `15e816d` = "file 6 v0.15
  fix-first intake rows — t4 Confluence oid-drift defect + harness/infra noticings"
  (this rotation's own commit); `14c4a42` = manager #7's handover commit recording the
  t4 outcome and benchmark-ceiling ruling (not authored by this L0 rotation).

## 2. Wave/cycle state

| Item | Artifact | State | Commit(s) |
|---|---|---|---|
| Directive 1 — t4 real-backend cadence re-run (destructive, manager-authorized) | `quality/runners/run.py --cadence pre-release-real-backend` | **COMPLETE — verdict 4 PASS / 2 P0 FAIL.** Env-propagation gap FIXED for this run (`.env` sourced in the SAME invocation as `run.py` → all 6 real-backend rows executed for real, 0 silent skips). See §5 for the two FAILs and why they are NOT regressions of this rotation's own making. | none (no catalog mutation committed — tree restored after `--persist` tried to downgrade a genuinely-PASS row) |
| Intake filing — 6 v0.15 fix-first rows | `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (Items A/B/C/E/F) + `GOOD-TO-HAVES.md` (GTH-V15-19) | **DONE.** See §5 for the row contents. | `15e816d` |
| Manager relay — benchmark-spend ceiling | (recorded here + in manager's own handover) | **DONE** — owner-confirmed ceiling: up to 50 benchmark sessions on the existing subscription for the funded Q1 live MCP re-measurement (no new API dollars; escalate only past 50). | (relay only, no separate commit) |
| Item 1 — `/gsd-new-milestone` PROJECT.md re-anchor | — | **NOT STARTED.** Gated on Arc D (RATIFIED) — no longer blocked, just not yet begun. | — |
| Item 2 — v0.15 floor milestone definition + planning | — | **NOT STARTED.** Same gate status as item 1. Now has 6 more intake rows to route in vs. #22's handover. | — |
| Directive 2 — scratch-repo `reposix-scope-test-DELETEME` policy doc | `docs/reference/testing-targets.md` | **NOT STARTED**, confirmed live this handover (`grep DELETEME` → no hits). Low urgency, GSD-quick scale. | none |

**No named-incident / diagnostic pending beyond §5's t4 root-cause writeup** (read before
touching the t4 caveat again, or before scoping the v0.15 fix-first lane).

## 3. Binding constraints (unchanged)

- **ONE cargo invocation machine-wide** (prefer `-p <crate>`). Leaf isolation: `/tmp`
  clones, `cd` in the SAME Bash invocation, never the shared tree.
- **Uncommitted = didn't happen.** Push per phase → confirm `code/ci-green-on-main` (P0)
  green → **never open next work over a red or pending main.**
- You **route, don't work**: delegate opus (complex/security), sonnet (default), haiku
  (mechanical); never fable at a leaf. Report to the manager (w1:p7) at each boundary or
  when blocked. Relieve past ~100k own-context tokens (hard stop ~150k) at a clean wave
  boundary — write+commit a handover first.
- **No `--no-verify`. No tag push by any coordinator** — the MANAGER cuts tags. No git
  surgery on `main`.
- **Shared repo with the manager (w1:p7)** — both commit to the SAME working tree. Use
  TARGETED staging (`git add <explicit path>`, NEVER `git add -A`/`.`) so you never sweep
  the manager's uncommitted `MANAGER-HANDOVER.md` edits. **Do NOT touch
  `.planning/MANAGER-HANDOVER.md`** (separate owner).
- **Owner-only stays owner-only:** interactive sudo, new creds/scopes/spend beyond the
  50-session benchmark ceiling, outward publishing.
- **Arc D is RATIFIED** (`6aa734a`, under owner delegation) — normal GSD gates apply to
  items 1/2 below, no pipeline pause in effect.

## 4. Litmus / gate / REOPEN state

- `ci.yml` on `origin/main`: newest confirmed-green run at handover-write time is
  `29386901268` on `14c4a42` — **#23 must re-poll per §1** to confirm the post-push state
  once #22's two commits land.
- **`pre-release-real-backend` cadence — t4 row
  (`agent-ux/t4-conflict-rebase-ancestry-real-backend`, P0) NOW HAS A REAL DETERMINISTIC
  RESULT: FAIL, and it is a genuine product defect, not a harness gap.** Confluence
  `list_records`-vs-`get_record` oid drift on page `7766017` breaks partial-clone `git
  checkout` (requested oid `288fbcf…` vs backend-materialized `959a0393…`). Root cause
  traced to `crates/reposix-cache/src/builder.rs:610-618` (`read_blob`): the tree oid is
  computed from the list-body at cache init time, but the blob is materialized from the
  get-body at checkout time — the two bodies render different bytes, so the drift-check
  at line ~612 aborts. Confirmed deterministic (re-ran, byte-identical failure) on an
  UNMUTATED protected fixture (not a race). **Manager RATIFIED converting this to a
  v0.15 FIX-FIRST lane** (owner `b773c04`); manager independently spot-verified the trace
  site (`builder.rs:612`). **Do NOT re-run this row to "retire a caveat" — there is no
  caveat left to retire, this is a scheduled fix.**
- **`milestone-close-vision-litmus` (P0) FAIL this run = KNOWN mirror non-idempotency**,
  not a regression — the fixture mirror needs `scripts/refresh-tokenworld-mirror.sh` run
  BEFORE the cadence, which was NOT done on this run. The already-committed catalog PASS
  for this row stands untouched (manager accepted the explanation; no downgrade landed).
- `p93-partial-failure-recovery` (P0) + all 3 P1 real-backend rows genuinely GREEN this
  run. **Tree was RESTORED after the run** — `--persist` had tried to downgrade the
  `milestone-close-vision-litmus` PASS to FAIL/NOT-VERIFIED on the mirror-lag false
  negative; the opus subagent caught it and `git restore`d the catalog before this
  handover. No stray catalog mutation is in the diff (`git diff --stat origin/main HEAD`
  shows only the two intake `.md` files).
- **Open waiver clocks:** 8 hero-number doc-alignment rows expire **2026-08-15**
  (= the funded Q1 live MCP re-measurement's HARD DEADLINE — schedule it early in v0.15,
  see §6 Item 2). `structure/file-size-limits` waiver expires **2026-08-08**
  (`quality/catalogs/freshness-invariants.json`, verified live this handover) — it now
  ALSO covers the newly-oversized `SURPRISES-INTAKE.md` (21773 B > 20000 char `.md`
  ceiling, confirmed via `wc -c` this handover); the progressive-disclosure split is
  v0.17 bloat-remediation work, do not split it early out of turn. `perf-targets`
  self-declares `WAIVED until 2026-07-26` (pre-existing, unrelated to this rotation).

## 5. Mid-execution decisions + noticed-not-filed

**Decisions made live this rotation:**
- **t4 caveat reclassified: was "never run," is now "ran for real, found a real
  defect."** #21's handover framed t4 as blocked on an env-propagation gap with an
  unknown underlying product result. #22 obtained the manager's GO, delegated the
  corrected `.env`-sourced re-run to an **opus** subagent (complexity/security tier,
  per the destructive-op + real-backend-creds sensitivity), and got a real, deterministic
  answer: **product defect**, traced to a specific line range in
  `crates/reposix-cache/src/builder.rs`. The manager ratified folding it into v0.15 as a
  fix-first lane rather than treating it as an open caveat to keep re-testing.
- **Six intake rows filed this rotation, commit `15e816d`** (do NOT re-file — read the
  rows before scoping v0.15, don't re-diagnose from scratch):
  - `SURPRISES-INTAKE.md` **Item A (HIGH)** — the t4 Confluence oid-drift defect itself
    (`builder.rs:610-618`), the v0.15 fix-first lane's core ticket.
  - **Item B (MED)** — the t4 gate's own error message
    (`quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`) misattributes
    the failure to git version instead of surfacing the real oid-drift stderr; fix so the
    error teaches the actual defect (Rust-compiler-grade-UX charter).
  - **Item C (MED)** — the cadence needs a documented pre-step
    (`scripts/refresh-tokenworld-mirror.sh`) before running, or the vision-litmus row
    false-negatives on mirror lag every time; cross-references GTH-V15-09.
  - **Item E (HIGH)** — `run.py` doesn't source `.env` while
    `scripts/preflight-real-backends.sh` does, which is exactly the env-propagation gap
    that made earlier rotations believe t4 had "never run" when really it had silently
    skipped. Fix-it-twice: make `run.py` source `.env` directly AND update the doc
    references in `.planning/CLAUDE.md` + `docs/reference/testing-targets.md`.
  - **Item F (HIGH)** — `--persist` silently downgrades a genuinely-GREEN committed
    catalog row to a worse status on a skip/false-negative outcome (this is exactly what
    almost happened to `milestone-close-vision-litmus` this run, caught only by an alert
    subagent). Fix: gate skip-driven catalog writes behind an explicit opt-in flag.
  - `GOOD-TO-HAVES.md` **GTH-V15-19 (LOW, audit)** — the `sync --reconcile` recovery
    doc claims it fixes oid-drift, but that claim is dubious specifically for the
    list-vs-get class of drift (Item A's class); audit once Item A's fix lands.
- **`SURPRISES-INTAKE.md` is now 21773 B**, over the 20000-char `.md` progressive-
  disclosure ceiling — covered by the existing repo-wide `structure/file-size-limits`
  waiver (expires 2026-08-08, §4). This is a NOTICED-not-yet-split condition: the split
  belongs to v0.17 bloat remediation, not this rotation or v0.15 — do not attempt it
  early.
- **Manager relay recorded, not yet independently re-confirmed by #23:** owner-confirmed
  benchmark-spend ceiling for the funded Q1 live MCP re-measurement is **up to 50
  benchmark sessions on the existing subscription** (no new API-dollar spend; escalate
  only past 50 sessions, and only with manager/owner sign-off). Fold this explicitly into
  the v0.15 waiver-expiry lane's scoping (§6 Item 2) — do not let it get lost as a verbal
  relay.

**Noticed-not-filed:** none new beyond the six rows above — everything surfaced this
rotation that met the OP-8 bar was filed into intake/good-to-haves in `15e816d`, and the
oversized-intake-file noticing is folded into the existing waiver + v0.17 lane rather
than filed as a duplicate row.

## 6. Precise next steps (successor #23 runbook)

1. **Re-verify §1 ground truth live first.** Confirm `main` is EVEN with `origin/main`
   and the newest CI run on the current HEAD sha is `success` before opening any new
   work. If still ahead, wait for #22's push to land; if CI is `in_progress`, re-poll;
   if `failure`, stop and diagnose — do not proceed over a red/pending main.
2. **Item 1 — `/gsd-new-milestone` PROJECT.md re-anchor.** Fold the Arc D ADDENDUM
   (`.planning/milestones/audits/2026-07-12-reality-check.md`) in FULL. Reconcile the
   P112 ROADMAP prose-vs-artifact divergence (standing RAISE: docs/planning
   simplification as a first-class roadmap goal). Replace the PROJECT.md truth banner
   with real re-anchored content. **Reconcile the ADDENDUM's "cut two stalled tags
   (v0.13.0, v0.14.0)" phrasing against live reality — BOTH have SHIPPED publicly**, so
   that phrasing predates the ship; correct the prose, do NOT literally execute a tag
   cut.
3. **Item 2 — v0.15 floor milestone definition + planning.** Route ALL open v0.15
   intakes + good-to-haves rows in (OP-8), INCLUDING the 6 new rows filed in `15e816d`
   this rotation and the **t4 oid-drift defect (Item A) as a v0.15 fix-first LANE**
   (not a caveat to re-test — it's a scheduled fix per §4/§5).
   - **HARD DEADLINE — schedule EARLY:** the funded Q1 live MCP re-measurement must land
     before **2026-08-15** (8 hero-number doc-alignment waiver rows expire then; a late
     re-measurement re-blocks every docs push). Spend ceiling: **up to 50 benchmark
     sessions on the existing subscription** (owner-confirmed via manager relay,
     §5 — escalate only past 50, do NOT exceed without manager GO).
   - Include an **ADR-010 mirror fan-out DECISION PACKET** (options + tradeoffs,
     prepared by a lane) — **the MANAGER decides; do NOT implement before that ruling.**
   - Ratchet-first sequence for reference (canonical: the Arc D ADDENDUM — digest only,
     do not re-fetch): **v0.15 floor** (kill 4 LAUNCH-BLOCKERs: index.md category,
     filesystem-layer rewrite, `reposix list/refresh` errors, `reposix detach` fix/delete,
     token-fixture provenance; PLUS the t4 oid-drift fix-first lane; cut the two stalled
     tags — reconcile vs live per step 2 above) → **v0.17 meta-milestone** (5 gate shapes:
     pivot-vocabulary lint, nav-budget, hero-redundancy, framing-claim rows, persona
     whole-journey rubric; + subjective-runner Task-dispatch fix unfreezing 3 WAIVED
     meaning-gates; + waiver-escalation rule; + transcript retention; + **bloat
     remediation** — natural home for the SURPRISES-INTAKE/GOOD-TO-HAVES progressive-
     disclosure split) → **v0.19** truth purge + IA rebuild → **v0.21** benchmark honesty
     (re-fixture live baseline, CI job, headline-cross-check verifier) → **v0.23** journey
     slices → **v0.25** launch kit → Show-HN. Even v0.16–v0.26 = small stub milestones.
     **Q3 launch gate:** Show-HN gated on a walkable REAL-BACKEND journey (GitHub
     minimum), not sim-first. **Q5/Q7 mandate:** DELETE legacy/historical files outright,
     no keep-with-banners (git history is the archive). **Deep-survey calibration:**
     ~10% latent work per pass, ~10 passes to converge, recurring deep surveys are
     STANDING practice. **Q9 ceiling:** keep v0.15→v0.25 ≈ 6-milestone scale.
4. **Directive 2 (low urgency, GSD-quick scale):** write the scratch-repo
   (`reposix-scope-test-DELETEME`) KEEP-as-reusable-scratch-target policy into
   `docs/reference/testing-targets.md` (owner policy: reset via force-push, never delete;
   currently archived, unarchive via API on first reuse) — confirmed NOT yet present
   (§2, live `grep` check this handover).
5. **If/when the pre-release-real-backend cadence is re-run** (e.g. at v0.15
   milestone-close, or to verify the Item A fix): use the `.env`-sourced form
   (`set -a; . ./.env; set +a; python3 quality/runners/run.py --cadence
   pre-release-real-backend --persist`) with an in-process fail-loud env check, AND run
   `scripts/refresh-tokenworld-mirror.sh` FIRST to avoid the vision-litmus false
   negative. The t4 row should only flip to genuinely PASS once the `builder.rs` fix
   (Item A) lands — do not expect or force a PASS before that.
6. **Report to the manager (w1:p7)** at each boundary above or when blocked — do not
   silently continue past a checkpoint.
7. **Relieve past ~100k own-context tokens at the next clean wave boundary** — write and
   commit a fresh `.planning/SESSION-HANDOVER.md` (REPLACE, not append) naming successor
   #24, following this same §3 (of `ORCHESTRATION.md`) template.
