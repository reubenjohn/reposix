# COORD-HANDOFF-P95-P97.md — v0.13.0 milestone-close coordinator succession, 2026-07-05

**Type: COORDINATOR-SUCCESSION** (new coordinator identity taking over, not a
same-coordinator pause). Written by the outgoing coordinator per
`.planning/ORCHESTRATION.md` §3 (relief/handover protocol), adapted to a
multi-phase milestone-close charter. Read this file in full before any dispatch —
do not skim to "next steps" and start working; the hazards section (below) has
already bitten this session 5 times.

**Do-not-touch guardrails up front:**
- Do **not** trust any `quality/catalogs/*.json` row status on disk without
  cross-checking a verdict file — see Hazard #1.
- Do **not** run `tag-v0.13.0.sh` (it's `.disabled`) or push a `v0.13.0` git tag —
  that is L0's action per OD-3, gated on P97 GREEN, never this coordinator's.
- Do **not** re-litigate ADR-010 or the P94 pagination Fork A/B decision — both are
  ACCEPTED/ratified; implement forward, don't reopen.

---

## Role & reporting

You are the **L2 coordinator** owning v0.13.0 milestone-close, phases **P95 → P96
→ P97**, in that order. Your mandate:

1. Get **P95 CLOSED GREEN**, then **P96 CLOSED GREEN**, then **P97 CLOSED GREEN**
   (each phase closes with a push + verifier grade per the project's standing
   cadence — see Operating constraints).
2. At P97, additionally run the **non-skippable milestone-close 9th probe**
   (`pre-release-real-backend`).
3. At P97, distill the **OP-9 RETROSPECTIVE** — a NEW section appended to the
   existing `.planning/RETROSPECTIVE.md` (17859 bytes, last touched 2026-05-01 for
   a prior milestone) — BEFORE any archive step.
4. Once P97 is GREEN, **hand back to L0** for the v0.13.0 tag push. You do not
   push the tag yourself (OD-3: tag push is delegated to the orchestrator/L0,
   contingent on a GREEN milestone verdict).

**You ROUTE, you don't work** — dispatch coordinators/executors for actual file
edits, reads, and test runs; your own tool calls stay to git one-liners and
reading short reports/handovers. **Relieve yourself at ~50% of your own context**
(dispatch a fresh handover-writer at the next wave boundary if you're approaching
it — don't ride it to exhaustion).

**FIRST ACTIONS (in order):**
1. Load the `coordinator-dispatch` skill and the `decision-procedures` skill
   (`.claude/skills/coordinator-dispatch/`, `.claude/skills/decision-procedures/`)
   — both exist on disk, confirmed.
2. Read `.planning/STATE.md` (entry point / machine-readable cursor).
3. Read this handover file in full (you're doing that now).
4. Read `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
   `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` — both are oversized
   (see Remaining scope, P96) but you need their current content before touching
   either.
5. Then proceed to Remaining scope § P95.

---

## Ground truth (git)

Verified directly at handoff time (not taken on faith):

- **HEAD:** `865a492ac9fe28bed1ffe7112357049454cac238` (`865a492`)
- **`git rev-parse origin/main`:** identical — `865a492...` — **local HEAD ==
  origin/main, 0 ahead / 0 behind.**
- **Tree:** clean (`git status` → "nothing to commit, working tree clean").
- **Branch:** `main`.

**Last 5 commits (newest first):**

| SHA | Message |
|---|---|
| `865a492` | docs(94): file 3 P94 close-out intake items (2 new, 1 escalation) |
| `eea309f` | docs(94): advance STATE cursor — P94 CLOSED GREEN, next P95 |
| `0d3c2f9` | docs(94): P94 phase-close verdict |
| `46bd1fa` | docs(94): re-bind docs-alignment claims drifted by Fork A backend.rs Listing/list_records_complete |
| `b98e8c5` | docs(94): P94 catalog-freshness sweep evidence + classification (D4) |

**Full P94 commit chain** (for provenance if you need to trace a decision back to
its origin): `3478dcc` ([FABLE] pagination consult) → `a5eaf7b` (rows+plan) →
`bb302ac`/`5cb9a14` (D1 pagination Fork A+B) → `c136795`/`557d45a` (D2 git-2.43) →
`28855b8` (test-isolation blocker) → `221c83e`/`78e4657` (jira hardening+intake) →
`34b4978`/`b98e8c5` (badges+sweep) → `46bd1fa` (docs-align refresh) → `0d3c2f9`
(verdict) → `eea309f` (STATE) → `865a492` (intake).

**P94 status:** CLOSED GREEN. Verdict at `quality/reports/verdicts/p94/VERDICT.md`
(commit `0d3c2f9`, 8097 bytes on disk, confirmed present) — all 4 P94 rows
PASS-by-executed-evidence. **The catalog rows themselves are NOT-VERIFIED on
disk** — see Hazard #1 below; the verdict file, not the catalog, is the
authoritative GREEN record for P94.

**STATE.md frontmatter (confirmed by direct read):** `phases_completed: 17`
(P78–P94), `next_phase: P95`, `status: executing-p94-post-close-drain`,
`blocks_tag: true` (v0.13.0 tag withheld until P97 GREEN).

---

## Remaining scope (THE LIVE PLAN)

**IMPORTANT:** `.planning/milestones/v0.13.0-phases/ROADMAP.md`'s prose for
P94–P97 is **STALE / orphaned "RBF-cluster" framing** — confirmed by direct read:
it still describes P94 as "Cluster D" DVCS-docs work, P95 as "Cluster E/F/H/I"
init-UX/honesty work, P96 as the old P87-superseding intake-drain phase, P97 as
the old P88-superseding milestone-close phase. **None of that matches what
actually happened at P94 or what STATE.md now says is coming.** STATE.md and this
handover are authoritative for what P95/P96/P97 actually contain; reconcile
ROADMAP.md's prose as part of P97's milestone-close (it's exactly the kind of
stale-doc drift the docs-alignment dimension exists to catch — don't let it ship
unreconciled).

### 1. P95 — marker footgun + docs-alignment refresh

- **"Marker footgun"** — this term appears verbatim in STATE.md's P94
  carry-forward line and nowhere else I could find on disk (no dedicated
  SURPRISES-INTAKE/GOOD-TO-HAVES entry, no P95 phase directory yet — P95 hasn't
  started). **Treat this as an open question, not a known quantity:** your first
  P95 action should be tracking down what "marker footgun" actually refers to
  (grep recent session transcripts / ask L0 if truly untraceable) before
  scoping a fix for it. Do not invent a meaning for it.
- **Docs-alignment refresh (~17 rows)** — a top-level `/reposix-quality-refresh`
  operation (orchestration-shaped: Opus-grader fan-out, unreachable from inside a
  `gsd-executor`, per `.planning/CLAUDE.md` "Orchestration-shaped phases run at
  top-level"). Covers:
  - The **5 STALE_TEST_DRIFT rows** the pre-push walk keeps reverting (all in
    `quality/catalogs/doc-alignment.json`, confirmed present at these row `id`s):
    `docs/decisions/009-stability-commitment/exit-codes-locked` (line ~3221),
    `docs/index/ci-badge` (line ~2182), `docs/index/quality-score-badge` (line
    ~6268), `docs/index/quality-weekly-badge` (line ~6293),
    `git-remote/bulk-delete-override-tag` (line ~1731). Genuine hash drift
    (additive/mechanical: badge URL bump, test-body reformat, exit-code table
    edit) — expect clean re-binds, not contested content.
  - The **P94 Fork-A `backend.rs` `list_records_complete` hash-drift** (the 8
    `docs/connectors/guide/*` claims).
  - The **3 older `bench-cron`/`release.yml` STALE_TEST_DRIFT rows** (filed
    separately, `DEFERRED-P95` tagged in SURPRISES-INTAKE).
  - Run one `/reposix-quality-refresh` session per drifted doc; re-run
    `quality/gates/docs-alignment/walk.sh` until it leaves the tree clean (no
    post-push `git checkout` needed for these specific rows once re-bound).

### 2. P96 — OP-8 Slot 1 (drains SURPRISES-INTAKE.md)

- **Quality-runner catalog self-mutation bug (HIGH — the real fix, not a 6th
  workaround).** Fired 5× this session already. It now (a) self-GRADES phase
  rows and (b) corrupts mid-sweep catalog reads. The fix must **validate-before-
  persist and never write phase-row grades** as a side effect of a read/walk
  operation — this is a `quality/runners/run.py` / `verdict.py` code change, not
  another `git checkout HEAD -- quality/catalogs/` band-aid. This is the item
  that currently forces every push in this drive to be followed by a catalog
  checkout (see Hazard #1) — fixing it retires that workaround for P96 onward.
- **Intake-file bloat split.** Both intake files are far over the 20k-byte
  budget: `SURPRISES-INTAKE.md` is **180,103 bytes** (confirmed by direct
  `wc -c`, ~9× over budget), `GOOD-TO-HAVES.md` is **79,613 bytes** (~4× over
  budget). These are first-class split targets in their own right — not just the
  smaller per-phase report logs. Design and execute a split (e.g. by
  milestone-phase-range, or active vs. archived rows) that keeps each resulting
  file under budget without losing any row.
- **`list_changed_since` under-materialization** (MEDIUM, filed).
- **`source_hashes`-empty walker false-negative** at
  `crates/reposix-quality/src/commands/doc_alignment.rs:1119-1122` (MEDIUM,
  confirmed present at that line range by direct read): the walk's per-source
  drift compare skips drift detection whenever `row.source_hashes` is empty,
  keyed on `is_empty()` rather than actual bind-state — so a row with `source`
  citations but an empty `source_hashes` array (hand-edited catalog, partial
  migration) silently escapes `STALE_DOCS_DRIFT` detection forever. Fix sketch
  already in the intake entry: backfill `source_hashes` at `Catalog::load` time,
  then key the skip on bind-state (`UNBOUND`/retire-proposed) instead of
  emptiness, plus a regression fixture proving the false-negative window closes.

### 3. P97 — OP-8 Slot 2 + milestone-close

- **Drain GOOD-TO-HAVES.md** (XS items close now; M items default-defer to
  v0.14.0 per OP-8 sizing rules), specifically named items: the meta-infra
  doctrine 4-edit proposal (scope as a tracked `/gsd-quick`); the
  `dispatch-doctrine.sh` session-guard; badges; the GitHub `list_records` →
  `list_records_complete` recursion footgun; the `STATE.md` strict-YAML
  frontmatter footgun.
- **Non-skippable 9th probe `pre-release-real-backend`** (RBF-FW-03, catalog row
  `agent-ux/milestone-close-vision-litmus-real-backend`, `blast_radius: P0`,
  NEVER carries a waiver): TokenWorld two-writer verifier + **RBF-LR-03**
  (was NOT-VERIFIED-deferred at P93 — this is where it actually gets verified).
  If real-backend creds are unset, mark **NOT-VERIFIED** honestly per OD-2 —
  **never skip-as-pass, never grade FAIL for an env-gated absence.** Verifier:
  `quality/gates/agent-ux/milestone-close-vision-litmus.sh`; verdict skeleton:
  `quality/dispatch/milestone-close-verdict.md`.
- **OP-9 RETROSPECTIVE.** `.planning/RETROSPECTIVE.md` already exists (17859
  bytes, last written 2026-05-01 for a prior milestone) — this is a NEW section
  **appended** for v0.13.0's extension (P89–P97), not an overwrite, and it must
  land BEFORE any archive step. The ratification subagent grades RED if it's
  missing. Five OP-9 subheadings expected (per ROADMAP's P97 acceptance
  criteria, still valid even though the surrounding prose is stale): What Was
  Built / What Worked / What Was Inefficient / Patterns Established / Key
  Lessons — sourced from SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts
  (P89–P96) + this session's autonomous-run findings.
- **PR #61** (`chore: release v0.13.0`, release-plz crates.io publish) — confirmed
  **currently OPEN** on `reubenjohn/reposix`. Owner-HELD until P97 GREEN; decide
  merge/close once P97 is GREEN, not before.
- **Then**: report milestone-close to L0 (see Report-to-L0 triggers) for the tag
  push. You do not run `tag-v0.13.0.sh` or push the tag.

---

## CRITICAL operating hazards (read before first push)

1. **Quality-runner self-mutation bug (HIGH).** Do **not** trust catalog row
   statuses on disk. The pre-push walk auto-flips rows — the 5 known
   STALE_TEST_DRIFT rows named above, **and** unpredictably a phase's own row
   (cadence-scope-dependent — it has self-graded a phase's row mid-sweep before).
   **At every push:** run it WITHOUT `--no-verify`, then immediately
   `git checkout HEAD -- quality/catalogs/` to drop the runner's writeback before
   your next commit. The REAL fix (validate-before-persist, never write
   phase-row grades) is a P96 deliverable — until P96 ships, this workaround is
   mandatory at every single push in P95 too.
2. **P94's 4 catalog rows are NOT-VERIFIED-on-disk but verdict-backed GREEN.**
   `quality/reports/verdicts/p94/VERDICT.md` (commit `0d3c2f9`) is authoritative.
   Do **not** attempt to re-grade or "fix" these 4 rows to PASS on disk — a
   clean 4-row catalog mint is blocked by Hazard #1 above, and P96 unblocks it.
   Don't spend P95/P96 time chasing this before the self-mutation fix lands.
3. **`p92-litmus` FAIL = git<2.34 env-gate.** CONFIRMED benign — the local dev
   env's git version predates 2.34, but CI runs git 2.54.0 and PASSES there.
   This is not a regression to chase; don't let a fresh coordinator mistake it
   for new breakage.
4. **ROADMAP.md P94–P97 prose is stale** — see Remaining scope preamble above.
   STATE.md + this handover are the live plan; don't let a subagent "correct"
   your scoping back to ROADMAP's orphaned RBF-cluster framing.

---

## Operating constraints

- **One tree-writer at a time.**
- **ONE cargo invocation machine-wide** (VM has OOM-crashed on parallel builds) —
  prefer `-p <crate>`; a hook (`cargo-mutex.sh`) backstops this but orchestration
  discipline is the primary control.
- **No `--no-verify`, no `-c commit.gpgsign=false`, no `--force` without explicit
  owner ask.**
- **Push cadence:** every phase closes with `git push origin main` BEFORE the
  unbiased `gsd-verifier` subagent grades it. The verifier grades RED if the
  phase shipped without the push landing.
- **Phase-close = catalog-row PASS, or verdict-backed** (per Hazard #2 pattern —
  a verdict file can be the authoritative record when the catalog itself is
  compromised). RED loops back — the orchestrator never waives a RED.
- **Model tiering:** opus (complex/security-judgment lanes), sonnet (default
  implementation), haiku (mechanical/leaf/reads that return a digest). **Never
  fable at a leaf.** In no-fable mode (which this drive may be running under),
  the dispatcher passes an explicit `model: opus` for a coordinator role — you
  are that coordinator.
- **The single-shot fable consult valve is already spent for this drive** — the
  P94 pagination Fork A/B decision used it (`3478dcc`, ratified). Any genuinely
  NEW E2-class architectural decision in P95/P96/P97 goes through the
  `decision-procedures` skill's escalation path, not a fresh ad-hoc fable
  consult.
- **Commit trailer:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` (or the equivalent tag for whichever model executes)
  on every commit.

---

## Report-to-L0 triggers

Report to L0 **only** on:
1. An owner-level E-escalation (a decision genuinely outside this coordinator's
   delegated authority — see `decision-procedures` skill's E1–E4 valve).
2. **Milestone-close**: P97 GREEN + RETROSPECTIVE section landed + 9th-probe
   result recorded (GREEN or honest NOT-VERIFIED) + a PR #61
   merge/close recommendation — hand to L0 for the v0.13.0 tag push.
3. A **successor pointer** if you relieve yourself mid-drive (write + commit a
   fresh handover first, per the relief protocol).

Intermediate P95 GREEN or P96 GREEN → **do not** report to L0. Just advance
STATE.md's cursor and continue to the next phase yourself.

---

## Decision ledger + binding artifacts

**`.planning/CONSULT-DECISIONS.md`** — P94's `[FABLE] pagination-truncation
prune-safety fork (A/B/C/D)` entry (2026-07-05, commit `3478dcc`): **Decision =
Fork A** (add `Listing { records, is_complete }` completeness signal, gate BOTH
`prune_oid_map` call sites on it) **+ Fork B as cheap defense-in-depth**
(idempotent delete-of-NotFound), **without reverting `272882c`**. C and D
rejected (see the ledger entry for full rationale — C still deletes live rows
beyond a `--reconcile` full-rebuild's own cap; D's count-margin heuristic has
structural false negatives on PR-heavy repos). This decision is ACCEPTED —
implement forward, do not re-litigate.

**Read in this order** before your first dispatch:
1. `.planning/STATE.md`
2. This handover
3. `quality/reports/verdicts/p94/VERDICT.md`
4. `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` +
   `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md`
5. `.planning/ORCHESTRATION.md`
6. `.planning/RUNBOOK-TO-V1/02-loops-and-context.md`
7. `.planning/PRACTICES.md` §OP-8 / §OP-9

**Embed root `CLAUDE.md`'s Ownership charter (OD-3's 5 points — acceptance
criteria are the floor not the ceiling, noticing is a deliverable, eager-fix or
file never silently skip, verify against reality, north-star polish-for-adoption)
verbatim in every sub-charter you write.**

---

## RAISE LIST already filed (don't re-file; drain in the named phase)

- 5-row STALE_TEST_DRIFT catalog refresh (`exit-codes-locked`, `ci-badge`,
  `quality-score-badge`, `quality-weekly-badge`, `bulk-delete-override-tag`) +
  P94 Fork-A `backend.rs` drift + 3 older bench-cron/release.yml rows →
  **P95**.
- Quality-runner self-mutation bug (HIGH) + intake-file bloat split
  (SURPRISES 180,103 bytes / GOOD-TO-HAVES 79,613 bytes) + `list_changed_since`
  under-materialization + `source_hashes`-empty walker false-negative
  (`doc_alignment.rs:1119-1122`) → **P96**.
- `STATE.md` strict-YAML frontmatter footgun + meta-infra doctrine 4-edit
  proposal + `dispatch-doctrine.sh` session-guard + GitHub `list_records`
  recursion footgun + badges → **P97**.

These are all already filed in `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` —
**do not re-file them**; just drain each in its named phase. Note:
`gsd-sdk` is not on `PATH` in this environment — STATE.md was hand-advanced
this session, not via SDK helpers; account for that if you script anything
against it.
