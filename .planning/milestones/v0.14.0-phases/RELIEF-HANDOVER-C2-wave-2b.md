# RELIEF-HANDOVER-C2-wave-2b.md — v0.14.0 wave-2 hardening, C2 relief, 2026-07-12

Written by the C2 coordinator-of-coordinators for the v0.14.0 wave-2 hardening
milestone, relieving proactively past the ~150k absolute-token hard stop (measured
~220k own context; ORCHESTRATION.md §3 rule 5). This is the **successor** to
`RELIEF-HANDOVER-C2-wave-2.md` at this same directory — that file is NOT overwritten,
it is superseded; read THIS file, not the older one (its git-status/incident-boundary
snapshot is now stale — ten-plus commits landed since, including a P0 incident close,
P106, P107, and the P110 drain).

**Required reading order for the successor:** (1) this file top-to-bottom, (2)
re-verify §1 ground truth LIVE before any tool call that mutates state — shared-tree
contention is active and this snapshot may already be stale by the time you read it,
(3) `.planning/milestones/v0.14.0-phases/ROADMAP.md` Phases 106–112 for full success
criteria, (4) `.planning/phases/110-op-8-slot-1-surprises-drain/110-01-PLAN.md` +
`honesty-spot-check.md` before dispatching the P110 verifier (§6 step 1).

**Do-not-touch guardrails (see §3 for the full hard-constraints block):** the foreign
untracked dirs `.planning/phases/21-*`, `22-*`, `scripts/demos/`, `scripts/dev/`;
`stash@{0}`; the foreign uncommitted modification to `quality/catalogs/code.json`.
Never `git add .`/`-A`/`clean`/`stash`.

## 1. Ground truth (git) — re-verify with `git rev-parse origin/main` before trusting

Verified live this session (2026-07-12, via `git fetch origin main` + `git status` +
`git rev-parse` + `gh run view`):

- `origin/main = 3a72fa0` (`docs(planning): refresh manager handover`). CI run
  `29204267618` (sha `3a72fa0`, workflow `CI`, trigger `push`) = **success**, all 15
  jobs green (clippy, gitleaks, quality gates pre-pr, test, shell-coverage, rustfmt,
  runner unit tests, coverage, 4x real-backend integration contract jobs, dark-factory
  regression, bench-latency-v09). **main is GREEN.**
- Local `HEAD = e5b969d` (`docs(v0.15.0-roadmap): add error codes + reposix explain to
  HEADLINE scope`), exactly **ONE commit ahead of origin/main**, deliberately held
  UNPUSHED by the prior rotation so it would not churn the P110 CI probe. It is SAFE
  to carry up on the next push and **MUST NOT be dropped** — an ordinary
  `git push origin main` from this shared repo will include it automatically (no
  extra action needed); it will trigger a fresh CI run, and the successor confirms
  that run green before closing P110 (see §6 step 2/4).
- `git status` on the tracked tree: ONE tracked modification —
  `M quality/catalogs/code.json` — this is a **foreign, uncommitted edit**: the
  `code/ci-green-on-main` row's `status` field flipped `PASS` → `NOT-VERIFIED` and
  `last_verified` bumped from `2026-07-12T15:52:10Z` to `2026-07-12T17:23:19Z`
  (`minted_at` unchanged at `15:39:06Z`). Almost certainly a concurrent session's
  local `post-push` cadence run that has not committed. **DO NOT commit or discard
  this diff** — it is not this rotation's work; a future `run.py --persist` (yours or
  a concurrent session's) will resolve it naturally. Don't trust it as the live
  litmus state either way — re-run the post-push probe yourself after your own push
  (§6 step 2).
- Untracked, all foreign / pre-existing, confirmed still present, still NOT ours:
  `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/`,
  `.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/`,
  `scripts/demos/`, `scripts/dev/`. Plus `stash@{0}` ("WIP on main: faf3d16 docs(22):
  create phase plan — 3 plans across 2 waves for BENCH-01..04") — do not touch.
- ALSO untracked (this one is OUR debt, not foreign):
  `quality/reports/verifications/docs-repro/` (P106 evidence). Verified: the 8 `.json`
  files inside ARE already covered by the existing `.gitignore` glob
  `quality/reports/verifications/*/*.json` (by-design ephemeral evidence, matches the
  established pattern — P106 closed GREEN without committing these). The 4
  `.sim-docs-repro-*.log` dotfiles are NOT covered by any ignore rule, which is why
  `git status` still surfaces the whole directory as untracked. LOW severity, does not
  block anything; optional P111 hygiene (add a `*.log` glob line, or commit the 4 logs
  explicitly) — not urgent enough to interrupt the P110→P111 sequence.
- Commit log since the last known-clean anchor (newest first, all on `origin/main`
  except the top one):
  1. `e5b969d` (LOCAL ONLY, UNPUSHED) docs(v0.15.0-roadmap): add error codes +
     reposix explain to HEADLINE scope
  2. `3a72fa0` docs(planning): refresh manager handover
  3. `24bb079` chore(evidence): commit orphaned P107 verification artifact — P110
     honesty closure
  4. `51adbc8` docs(planning): P110 OP-8 Slot 1 — drain SURPRISES-INTAKE to terminal
     status + F-K5 honesty spot-check
  5. `655eb5f` chore(quality): P110 catalog-first — mint surprises-absorption
     verifier + row (FAIL)
  6. `3f1458d` fix(ci): collect env-safe quality/runners unit tests in CI — P110 OP-8
     intake row 10
  7. `0d05d7f` fix(ci): gate release.yml publish on green CI for the tagged commit —
     P110 OP-8 intake row 8
  8. `d6a0945` docs(mandate): encode Rust-compiler-grade UX north-star (fix-twice) +
     schedule v0.15.0 error-message audit phase
  9. `61c9c91` docs(planning): record GTH-09 (ADR-010 slug→id) DEFERRED-TO-v0.15.0 —
     owner scope call
  10. `8b488dc` docs(handover): GTH-09 deferred to v0.15.0 + standing UX mandate
      encoded (both in flight)
  11. `9007ca2` docs(planning): record UX mandate + resolved owner decisions in
      manager handover
  12. `ed42ece` docs(consult): [OWNER] record authorized external mutation — land
      lost-update shared-cursor HIGH security fix onto origin/main
  13. `4dd7e10` docs(113-01): renumber lost-update phase 106→113 (ROADMAP 106 taken)
      + mark intake RESOLVED
  14. `61e8222`/`fb563a6` fix(106-01): lost-update guard — shared cursor no longer
      gates conflict detection (the P113 landed fix)

**Numbered deviations the successor MUST know (found this rotation, corrections to
the outgoing brief):**
1. **P113 plan path CORRECTION.** The lost-update-shared-cursor plan is at
   `.planning/milestones/v0.14.0-phases/113-lost-update-shared-cursor/PLAN.md` —
   **NOT** `.planning/phases/113-lost-update-shared-cursor/PLAN.md` (verified via
   `git show --name-only 4dd7e10`; the latter path does not exist). Use the correct
   path when authoring the ROADMAP `### Phase 113` entry (§6 step 5a).
2. **Pre-push cargo-trigger CORRECTION (verified, see §5 item 2 for detail).** The
   land-lane's premise "invokes cargo fmt+clippy when Rust files changed" does NOT
   hold — `.githooks/pre-push` runs the full `--cadence pre-push` runner
   UNCONDITIONALLY on every push, and the composed `code/cargo-fmt-check` /
   `code/cargo-clippy-warnings` catalog rows are themselves unconditional
   whole-workspace invocations (`cargo fmt --all -- --check`, `cargo clippy
   --workspace --all-targets -- -D warnings`) — no diff/file-type gate exists
   anywhere in this path. Fix `crates/CLAUDE.md` against the VERIFIED trigger, not
   the unverified premise (§6 step 5b).
3. The `agent-ux/p110-surprises-absorption` catalog row (full id, not the shorthand
   "surprises-absorption") is currently `status: FAIL` in the committed catalog
   (catalog-first mint, `655eb5f`) — this is EXPECTED pre-verifier-run state, not a
   regression; grading it is the successor's first action (§6 step 1).

## 2. Wave/cycle state

| Wave / Item | Plan | State | Commits |
|---|---|---|---|
| P106 docs-repro | wave-2 planned | DONE, 5/5 catalog rows PASS | `7827d36` |
| P107 cargo-audit posture | wave-2 planned | DONE, 0 live advisories (RUSTSEC-2026-0185/0186 both non-live) | `d61cbb7` |
| P110 OP-8 Slot 1 (SURPRISES-INTAKE drain) | +2 reservation | **WORK LANDED, VERIFIER NOT YET RUN — successor's FIRST ACTION** | `655eb5f`, `51adbc8`, `0d05d7f`, `3f1458d`, `24bb079` |
| P111 milestone-close (GOOD-TO-HAVES drain, OP-9 RETROSPECTIVE, CHANGELOG, tag-script, 9th probe) | +2 reservation | **BLOCKED until P110 PASS** — no `.planning/phases/111-*` dir exists yet, confirmed unplanned | none |
| P112 OD-4 launch-readiness stub | scope-only | **DO NOT START** — no `.planning/phases/112-*` dir exists yet | none |
| v0.15.0 roadmap headline (error codes + `reposix explain`) | ad-hoc/owner | DONE locally, deliberately UNPUSHED, safe to carry up on next push | `e5b969d` |

**Named incident:** none new this rotation. The only prior-rotation incident (Phase 0
D2 leaf-isolation re-seal, P102) is already RESOLVED-in-P102 per SURPRISES-INTAKE.md
(commit `39a8500`) — no action needed, do not re-open.

## 3. Binding constraints (unchanged, repeated verbatim for the successor)

- **SHARED-TREE CONTENTION IS ACTIVE** — multiple sessions write the same working
  tree + `.git`. Before any rebase, `git status`; if dirty with FOREIGN files you
  didn't touch, do NOT stash/discard/reapply them — commit only explicit paths, and
  if a rebase refuses due to a foreign dirty file, STOP and surface rather than
  manipulating foreign work. **RAISED HAZARD for owner/L0: recommend git worktree
  isolation or session serialization** (repeated from the prior rotation — still
  unresolved, still worth raising).
- **Foreign, DO NOT TOUCH:** untracked `.planning/phases/21-*`, `22-*`,
  `scripts/demos/`, `scripts/dev/`; `stash@{0}`; the foreign uncommitted modification
  to `quality/catalogs/code.json` (see §1).
- ONE cargo invocation machine-wide; prefer `-p <crate>`.
- NEVER `git add .`/`-A`/`clean`/`stash`; explicit-path commits only.
- `origin/main` MOVES: `git fetch && git pull --rebase origin main` before every push
  (heed the contention rule above).
- Push cadence: push BEFORE the verifier; then
  `python3 quality/runners/run.py --cadence post-push --persist` and confirm main's
  latest `ci.yml` concludes success (red main = phase RED). No `--no-verify`.
  Leaf-isolation hook is fail-closed.
- Commit-trailer format: `Co-Authored-By` + `Claude-Session`. Model tiering: fable →
  opus (security/complex) / sonnet (default) / haiku (mechanical); never fable at a
  leaf.
- A stale background `gh run watch` waiter (`bulqmsyrv`) may fire a late redundant
  task-notification — ignore it.

## 4. Litmus / gate / REOPEN state

- `agent-ux/p110-surprises-absorption` (catalog: `quality/catalogs/agent-ux.json`,
  line ~1099): committed `status: FAIL` (catalog-first mint before the drain landed,
  per `quality/CLAUDE.md`). The drain work (16/16 SURPRISES-INTAKE entries terminal,
  honesty-spot-check present at
  `.planning/phases/110-op-8-slot-1-surprises-drain/honesty-spot-check.md`) has now
  landed on main — **the row has not been re-run/re-graded since**. Verified this
  rotation: SURPRISES-INTAKE.md's single `STATUS: OPEN` string occurrence is inside
  the `## Entry format` markdown TEMPLATE's fenced code block (line 28), not a real
  entry — 0 real OPEN entries, 16 real entries all carry terminal STATUS
  (RESOLVED-in-P*/DEFERRED-TO-v0.15.0/RESOLVED by commit SHA). The verifier's
  fence-aware awk design is built to skip the template line; a live re-run should
  find it PASS. This is the successor's job to confirm, not assume (§6 step 1).
  No `quality/reports/verdicts/p110/` directory exists yet — no verdict minted.
- No REOPEN state currently active; nothing failed and got reopened this rotation.
- **No open waiver-clock data was independently verified this rotation** — the
  outgoing brief did not carry any; successor should sweep `quality/catalogs/*.json`
  for near-term TTLs as part of the P111 close pass, not assumed clear.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**Formalized decisions carried forward (already landed, do not re-litigate):**
- **GTH-09 (ADR-010 slug→id durable-create) DEFERRED-TO-v0.15.0** — owner scope call
  2026-07-12, recorded at `61c9c91` in ROADMAP Phase 108 + milestone
  `GOOD-TO-HAVES.md` GTH-09 + root `GOOD-TO-HAVES.md` GTH-09. **The RETROSPECTIVE /
  milestone-close artifact (P111, OP-9) MUST name it as a known owner-chosen
  deferral** — a verifier grading P111 should see a recorded deferral, not a gap.
  Verified live: `GOOD-TO-HAVES.md` line 248 carries the terminal status text
  exactly as described.

**Noticed-not-yet-filed this rotation (routed here, not yet independently filed —
successor triages):**
1. **P113 plan path was wrong in the outgoing dispatch brief** (see §1 deviation 1).
   Low severity, purely a pointer — fix when authoring the ROADMAP Phase 113 entry,
   no separate intake row needed.
2. **Pre-push cargo-trigger claim was wrong in the outgoing dispatch brief** (see §1
   deviation 2). Verified via direct read of `.githooks/pre-push` (line 62:
   `python3 "$REPO_ROOT/quality/runners/run.py" --cadence pre-push`, unconditional,
   no diff check anywhere in the hook body) and `quality/catalogs/code.json`
   (`code/cargo-fmt-check` / `code/cargo-clippy-warnings` rows: unconditional
   `--workspace`/`--all` invocations, no file-type gate). This matters because the
   PRACTICAL conclusion the outgoing brief wanted (a lane landing code while another
   holds the cargo mutex must not run the full pre-push; rely on post-push CI) is
   still correct — but for a stronger reason (it ALWAYS runs cargo, not just on Rust
   diffs) than the brief stated. Fold the corrected wording into the `crates/CLAUDE.md`
   fix-twice edit (§6 step 5b) rather than parroting either premise uncritically.
3. **`quality/reports/verifications/docs-repro/` partial-gitignore gap** (see §1) —
   4 `.log` dotfiles not covered by any ignore glob, LOW severity, optional P111
   hygiene, does not block P110 or P111 critical path.
4. **CHANGELOG.md confirmed genuinely NOT STARTED for v0.14.0** — only `[Unreleased]`
   exists at line 78; no `[v0.14.0]` section anywhere. Matches P111 ROADMAP success
   criterion 3 expectation, not a surprise, just confirmed via direct read so the
   successor doesn't have to re-check.
5. **GOOD-TO-HAVES.md entry count, verified live:** 12 entries currently `STATUS:
   OPEN`, 1 (GTH-09) terminal `DEFERRED-TO-v0.15.0`. GTH-11 (P106 example-05
   non-cone sparse-checkout warnings, LOW) and GTH-12 (P107 orphan quinn Cargo.lock
   entries, XS/LOW, warns against opportunistic `cargo update`) are both present and
   OPEN as expected.

## 6. Precise next steps (successor runbook)

1. **Re-verify §1 ground truth live** (`git fetch`, `git status`, `git rev-parse HEAD
   origin/main`, `git stash list`) before any mutating tool call — contention is
   active and this snapshot may already be stale.
2. **Confirm main is still green** (`gh run list --branch main --limit 3`) before
   doing any gate-relevant work — never open a phase over a red main.
3. **Dispatch an unbiased `gsd-verifier` subagent to grade the P110 catalog row**
   `agent-ux/p110-surprises-absorption` (`bash
   quality/gates/agent-ux/p110-surprises-absorption.sh`), reading only committed
   artifacts (`SURPRISES-INTAKE.md`,
   `.planning/phases/110-op-8-slot-1-surprises-drain/honesty-spot-check.md`). RED
   loops back to fix the gap — do not silently downgrade or hand-wave a PASS.
4. **On PASS:** write `quality/reports/verdicts/p110/VERDICT.md`, and — since HEAD
   already carries the P110 work — push (this also carries the still-unpushed
   `e5b969d`), then run `python3 quality/runners/run.py --cadence post-push
   --persist` and confirm main's newest `ci.yml` run is green before closing P110.
5. **Only after P110 PASS, unblock P111.** Land these THREE folded hygiene items
   BEFORE any other P111 work, in this order:
   a. Add a `### Phase 113` entry to the v0.14.0 ROADMAP reflecting the LANDED
      lost-update shared-cursor fix (status done; code on main `61e8222`/`fb563a6`,
      CI-green; PLAN at the CORRECTED path
      `.planning/milestones/v0.14.0-phases/113-lost-update-shared-cursor/PLAN.md` —
      see §1 deviation 1). Don't leave a PLAN with no roadmap slot.
   b. Fix-twice `crates/CLAUDE.md`'s Build-memory-budget section using the VERIFIED
      trigger from §5 item 2: pre-push runs `cargo fmt --all -- --check` +
      `cargo clippy --workspace --all-targets -- -D warnings` UNCONDITIONALLY on
      every push (not diff-gated on Rust-file changes) — a lane landing code while
      another holds the machine-wide cargo token must NOT run the full pre-push
      (OOM trap); rely on post-push CI for cargo validation instead.
   c. Prune `.planning/CONSULT-DECISIONS.md` (27,931 bytes, over the 20k soft limit)
      per its own delete-on-close policy — delete implemented/closed `[SELF]`/
      `[OWNER]` entries (dependabot-close, ci-green probe, release-plz untrack, the
      land); git history is the archive. Apply the same delete-on-close treatment to
      SURPRISES-INTAKE.md / GOOD-TO-HAVES.md entries as they're drained in the steps
      below.
6. **Drain `GOOD-TO-HAVES.md`** (Slot 2, OP-8): 12 real `OPEN` entries verified live
   this rotation (§5 item 5), plus GTH-09 already correctly terminal (DEFERRED, do
   not re-litigate). Eager-fix <1h/no-new-dep, else terminal-status with named
   carry-forward.
7. **OP-9 RETROSPECTIVE distill BEFORE archive** — new `.planning/RETROSPECTIVE.md`
   v0.14.0 section (What Was Built / What Worked / What Was Inefficient / Patterns
   Established / Key Lessons). **MUST explicitly name GTH-09 as a known owner-chosen
   deferral**, not a gap (§5).
8. **Reconcile ROADMAP.md `Plan: TBD` placeholders for P103–P109** at close.
9. **Split oversize planning docs** — all three confirmed over the 20k soft limit
   this rotation: `STATE.md` (20,216 B), `GOOD-TO-HAVES.md` (21,677 B pre-drain),
   `SURPRISES-INTAKE.md` (41,941 B pre-any-further-pruning). Doc-split candidates,
   fold into P111.
10. **CHANGELOG `[v0.14.0]` finalize**; author `tag-v0.14.0.sh` (≥6 safety guards
    mirroring v0.13.0/v0.12.0 precedents); run the milestone-close verifier
    including the non-skippable 9th probe
    `python3 quality/runners/run.py --cadence pre-release-real-backend` (reads
    NOT-VERIFIED honestly if env unset, never FAIL/skip-as-pass). **STOP AT THE TAG
    BOUNDARY — the OWNER cuts the aggregate `v*` tag, never the coordinator.**
11. **P112 last:** author the SCOPE-ONLY stub with a DO-NOT-START banner. No
    implementation, no verifier dispatch — this phase produces planning prose only.
12. **Push-cadence discipline for every phase close, repeated:** `git fetch && git
    pull --rebase origin main` (heed contention) → push BEFORE the verifier dispatch
    → `python3 quality/runners/run.py --cadence post-push --persist` → confirm
    main's latest `ci.yml` run is green → THEN advance `.planning/STATE.md` cursor →
    write the phase's RAISE LIST + intake disposition.
13. **Relieve proactively** at ~100k of your own absolute context (hard stop
    ~150k) — dispatch `relief-handover-writer` for the next successor file before
    hitting the hard stop, same discipline that produced this file.

## Ownership charter (OD-3) — binding on every subagent this successor dispatches

Every subagent (executor, verifier, researcher, code-reviewer) that touches a real
surface OWNS it, not just its acceptance criteria: (1) acceptance criteria are the
floor, not the ceiling — done means "I'd defend this in review as excellent." (2)
Noticing is a deliverable — every report names what it noticed near its work; an
empty noticing section from code-touching work is itself a red flag. (3) Eager-fix
(<1h, no new dependency) or file with severity+sketch — never silently skip. (4)
Verify against reality — run the thing, render the page, hit the backend; a claim
without an artifact is not done. (5) North star: would a skeptical dev hitting this
milestone's close for the first time come away trusting it?

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
