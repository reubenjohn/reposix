# RELIEF-HANDOVER-C2-wave-c.md — v0.13.1 "Front door actually works" onboarding hotfix, C2 relief handover, 2026-07-07

Written by the C2 milestone-coordinator for v0.13.1 relieving at the Wave C/D boundary
(context budget). A prior relief attempt hit an API error mid-write and committed
NOTHING — that attempt left no trace in git; this handover is written fresh from
ground-truth inspection, not from any prior (uncommitted, non-existent) draft.

**Successor's required reading order:** this file top-to-bottom, then
`.planning/milestones/v0.13.1-phases/ROADMAP.md` + `REQUIREMENTS.md`, then
`.planning/CONSULT-DECISIONS.md` 2026-07-07 entry (B5 doc-honesty edit spec — VERBATIM,
do not re-derive), then `.planning/ORCHESTRATION.md` §3 (this template) and § Leaf
isolation before dispatching any executor.

**Do not touch:** v0.13.2's phase numbering (P98–P107 nominally claimed there) — do NOT
renumber or edit v0.13.2 files now; that renumbering is v0.13.2's own problem at its own
plan time. **Do not start:** `git tag v0.13.1` / release.yml / crates.io publish — that
is L0's call, gated on this milestone's DoD going green (see STOP BOUNDARY below).

## 1. Ground truth (git)

- Branch `main`, HEAD `5ca79c5`, tree **clean** (`git status` confirms "nothing to
  commit, working tree clean").
- `git config user.email` = `reubenvjohn@gmail.com` (verified, correct — NOT `t@t`).
- Local branch is **5 commits ahead of `origin/main`** — nothing has been pushed yet
  this milestone. Commits since the last known-clean pre-milestone sha (`540fb1f`,
  itself a prior handover commit):
  1. `cdc0bef` docs(planning): scaffold v0.13.1 onboarding-hotfix milestone (P98-P101)
  2. `499ec67` docs(planning): triage post-conflict recovery crash as known RBF-LR-03
     limitation (v0.13.1 B5)
  3. `8781c7f` feat(sim): honest one-line startup banner on stderr
  4. `ba49482` fix(cli): reposix init exits non-zero when the backend is unreachable
  5. `5ca79c5` fix(cli,remote): make the documented init front door truthful (v0.13.1
     CHECKOUT-BREAK)
- **Numbered deviations the successor MUST know:**
  1. A checkout-break release-blocker was discovered mid-milestone (not in the original
     scaffold): the documented `git checkout origin/main` FAILS post-init. Fixed in
     `5ca79c5` with a canonical working replacement command (see Wave D below) — but the
     fix only touched the CLI/remote-helper truthfulness; the DOCS still need updating
     (that's Wave D's job, not yet done).
  2. B5 (post-conflict recovery crash) was triaged as a KNOWN LIMITATION, not fixed —
     the full doc-honesty edit (rewriting the docs to stop advertising the crashing
     recovery path) is planned in Wave D but NOT yet applied to the doc files
     themselves; only the planning-side triage/decision record is committed so far.
  3. No handover file previously existed for this milestone under this name or any
     other under `v0.13.1-phases/` — the prior relief attempt produced zero commits.

## 2. Wave/cycle state

| Wave | Scope | State | Commits |
|---|---|---|---|
| A (scaffold) | P98–P101 ROADMAP/REQUIREMENTS scaffold | DONE | `cdc0bef` |
| B (triage+fixes) | B5 triage, B3-sim banner, B4 init non-zero, checkout-break fix | DONE | `499ec67`, `8781c7f`, `ba49482`, `5ca79c5` |
| C (this coordinator's tenure) | Ground-truth verification + this handover | DONE (relief, no code changes) | (none — handover only) |
| D | Doc-truth lane (B1, B2, B3-docs, checkout cmd, B5 doc-honesty, intake append, mkdocs gate) | NOT STARTED | — |
| E | Quality (B6 new catalog row + verifier, B7 stale-doc refresh) | NOT STARTED | — |
| F | Push + zero-shot human-sim gate (THE DoD gate) | NOT STARTED | — |
| G | gsd-verifier catalog-row grading | NOT STARTED | — |

**Named-incident post-mortem to read before dispatching an executor:** the
checkout-break discovery (`5ca79c5`'s commit body) — it explains WHY the advertised
refspec had to change (`refs/heads/*:refs/reposix/origin/*`) and WHY
`git checkout -B main refs/reposix/origin/main` is the canonical replacement, plus the
eager-fixed RED test `dark_factory_sim_happy_path` that B4's exit-code change broke.
Read the commit message in full: `git show 5ca79c5 --stat` then the message body.

## 3. Binding constraints (unchanged)

- **One tree-writer at a time.** Wave D must run as a single sonnet `gsd-executor`
  (doc-truth lane) — do not fan out multiple simultaneous doc-editors on the same tree.
- **ONE cargo invocation machine-wide** (hook-enforced via `.claude/hooks/cargo-mutex.sh`
  — prefer `-p <crate> -j2`, never `--workspace`). Wave D is doc-only and should run
  **with NO cargo invocations at all**; Wave E's verifier work will need cargo — do not
  overlap Wave D and Wave E cargo usage.
- **No `--no-verify`** on any commit, ever.
- **Leaf isolation HARD-STOP**: any `reposix init` / sim-seed / `git commit`/`config`
  leaf test setup runs in a throwaway `/tmp/<uniq>` clone in the SAME bash invocation —
  never the shared repo tree. This applies directly to Wave F's zero-shot gate dispatch.
- **Verify `git config user.email` == `reubenvjohn@gmail.com` before every commit** —
  if it ever reads anything else (e.g. `t@t`), STOP and report to L0, do not commit, do
  not change the config yourself.
- **Push `origin main` BEFORE the verifier dispatch** (Wave F precedes Wave G) — the
  gsd-verifier grades RED if the phase shipped without the push landing.
- **Commit trailer format:** conventional-commit subject line +
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>` (or the
  applicable model's own attribution per the dispatching agent) on every commit this
  milestone produces.
- **Model tiering:** Wave D (mechanical doc edits) → sonnet `gsd-executor`. Wave E
  (catalog-row + verifier authoring, judgment-heavy) → sonnet, possibly opus if the
  verifier design proves subtle. Wave F's zero-shot dispatch → a FRESH general-purpose
  agent with explicitly NO repo context (this is the whole point of the gate — do not
  brief it beyond "here are the published docs"). Wave G → `gsd-verifier` per OP-7.

## 4. Litmus / gate / REOPEN state

- No quality gate has been run yet this milestone (Wave D's mkdocs-strict.sh /
  mermaid-renders.sh gate has not been invoked; Wave E's catalog rows do not exist yet;
  Wave F's zero-shot gate — the actual DoD-defining litmus — has not been dispatched;
  Wave G's gsd-verifier has not been dispatched). **No PASS/FAIL/REOPEN history exists
  for this milestone as of this handover.**
- Pre-existing catalog drift noticed (not yet fixed, carried into Wave E as B7): two
  binstall rows in `quality/catalogs/doc-alignment.json` went STALE_TEST_DRIFT after PR
  #70 (`4b564e4`). This file currently shows as modified in `git status` at the START of
  this coordinator's tenure per the task brief — but as of THIS handover's ground-truth
  check, `git status` reports the tree fully clean, meaning either a pre-commit hook
  auto-reverted the modification (documented elsewhere as a known hook behavior — see
  Wave E note below) or it was never actually dirty at handover time. **Successor MUST
  re-check `quality/catalogs/doc-alignment.json` drift status fresh in Wave E** — do not
  assume either way from this note.
- No waivers are open; no expiry clocks are running for this milestone.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**Decisions already formalized (committed) — successor should NOT re-litigate:**
- B5 VERDICT (known-RBF-LR-03-limitation, doc-honest) — full edit spec is committed
  verbatim in `.planning/CONSULT-DECISIONS.md` 2026-07-07 entry. Apply it in Wave D
  exactly as written; do not redesign it.
- Checkout-break VERDICT (hotfix-honest-fallback) — canonical command
  `git checkout -B main refs/reposix/origin/main` is settled; do not re-derive.

**Noticed, not yet filed (successor must route, not drop):**
- SURPRISES-INTAKE.md is 86k chars / 4.3x the soft size limit — pre-existing bloat,
  already tagged for a v0.14.0 ingest/split but not yet acted on. Not this milestone's
  job to fix, but flag again to L0 at close if still true.
- On-box git is 2.25.1, below the CLAUDE.md-documented `>=2.34` floor, yet the
  documented flow reportedly works on it. This needs empirical re-verification during
  Wave F (the zero-shot gate dispatch is the natural place to observe this) — if the
  flow truly works on 2.25.1, either the floor doc is overly conservative or under-
  tested; do not silently resolve either way, relay the observation to L0.
- Phase-number collision: v0.13.1 claims P98–P101; v0.13.2 nominally claims P98–P107.
  Resolution is deferred to v0.13.2's own planning (renumber to P102–P111 there) — do
  NOT touch v0.13.2 files from this milestone's context.
- v0.14.0-bound filed items (already filed, just tracking): `S-260707-rbf-lr03-external-
  write-crash` (HIGH, in `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`) plus
  two sub-risks that Wave D must ADD to that same entry (sync --reconcile teaching-free
  false-success; push-exit-0-after-failed-pull data-loss window) — these sub-risks are
  NOT yet filed, Wave D is responsible for filing them (see Wave D task list below,
  "Intake append").
- Pure-git checkout ergonomic follow-up (MEDIUM, v0.14.0) — filed already per the
  checkout-break commit; no further action needed this milestone beyond what's in Wave D.

## 6. Precise next steps (successor runbook)

Dispatch order: Wave D → Wave E → Wave F → Wave G → report to L0. Each wave is a single
tree-writer; confirm `git config user.email` before every commit in every wave.

### WAVE D — doc-truth lane (single sonnet `gsd-executor`, doc-only, NO cargo)

1. **B1 (seed-doc mismatch):** Real seed data is `issues/1.md`..`6.md` (unpadded
   numbering, 6 issues total; seed issue 1's title is `A-edit-conflict-test`). Fix
   `docs/tutorials/first-run.md` and `docs/index.md` — in `docs/index.md` fix BOTH the
   "After — one commit" block AND the build-from-source block. Replace any reference to
   `issues/0001.md`, "5 issues", or "Add user avatar upload" with the real values.
2. **B2 (audit SQL + narrative):** Tutorial step-8's audit SQL references a nonexistent
   column `decision`. Real `audit_events` columns are: `ts, op, backend, project,
   issue_id, oid, bytes, reason`. Fix the query. Also fix the narrative claiming "No
   helper_push_sanitized_field" row appears — a NORMAL push DOES write that row (seed
   frontmatter carries server-controlled `id`/`version` fields that get stripped on
   push) — correct the narrative to match reality, don't just patch the SQL.
3. **B3-docs (seed-file path):** Tutorial step-2 references the source-tree path
   `--seed-file crates/reposix-sim/fixtures/seed.json`, which is ABSENT from release
   archives. Align to the built-in default seed, following the README's pattern:
   `reposix sim --bind 127.0.0.1:7878 &`. Then actually RUN the documented command and
   transcribe the REAL banner line into the docs — the `{source}` token in the banner
   format `reposix-sim: listening on http://{addr} (seed: {source}, {n} issues) —
   Ctrl-C to stop` depends on whether `--seed-file` is passed:
   `{source} ∈ {none (no --seed-file), seed-file, disabled (--no-seed)}`. Do not
   hand-write a plausible-looking banner — run it and copy the actual output.
4. **Checkout command fix:** Replace the broken `git checkout origin/main` with the
   canonical `git checkout -B main refs/reposix/origin/main` at the INITIAL-checkout
   call site `docs/concepts/dvcs-topology.md:124`.
   **CAUTION — do not blindly find/replace:** the blob-limit RECOVERY contexts at
   `docs/guides/troubleshooting.md:65,74`, `docs/guides/integrate-with-your-agent.md:83`,
   and `docs/how-it-works/git-layer.md:112` are re-materialize-on-EXISTING-main (i.e.
   they run AFTER `git sparse-checkout set`, not at initial checkout) — RUN each of
   those contexts for real and use whatever the correct re-materialize command actually
   is there (likely plain `git checkout main`, but verify, don't assume). The primary
   onboarding call sites `docs/index.md:131` and
   `docs/concepts/mental-model-in-60-seconds.md:62` are ALREADY correct — leave them
   untouched.
5. **B5 doc-honesty:** Apply the VERBATIM edit spec recorded in
   `.planning/CONSULT-DECISIONS.md` under the 2026-07-07 entry to
   `docs/guides/troubleshooting.md` § "Bus-remote fetch first rejection" and
   `docs/concepts/dvcs-topology.md` around line 93 — stop advertising the crashing
   recovery path (the full documented `reposix sync --reconcile` → `git pull --rebase`
   → `git push` sequence CRASHES; `sync --reconcile` is the trigger), document the
   honest workaround (fresh `reposix init`), and point readers to v0.14.0 for the real
   fix. **There is a THIRD location** advertising the same broken `git pull --rebase`
   recovery that is NOT in the original spec's location list: `docs/index.md:152` — fix
   this one too, using the same honesty pattern as the other two.
6. **Intake append:** In `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`,
   under the existing `S-260707-rbf-lr03-external-write-crash` entry, add two v0.14.0
   sub-risks: (a) `sync --reconcile` can produce a teaching-free false-success (exits 0
   while leaving unusable non-descendant state); (b) push-exits-0-after-failed-pull
   data-loss window (an external edit can be silently overwritten) — suggest considering
   a guard. This is a planning-file edit by the SAME tree-writer, not a separate lane.
7. **Gate:** After all edits, run `bash quality/gates/docs-build/mkdocs-strict.sh` AND
   `bash quality/gates/docs-build/mermaid-renders.sh` — both must pass before this wave
   is considered closed. Commit (conventional message, correct trailer) only once these
   pass.

### WAVE E — quality (B6 + B7), catalog-first rule applies

1. **B7 (stale binstall rows):** Two rows in `quality/catalogs/doc-alignment.json`
   drifted `STALE_TEST_DRIFT` after PR #70 (`4b564e4`). Refresh honestly — use
   `/reposix-quality-refresh` for the affected doc, or hand-bind if it's agent-ux-scoped.
   **Note:** this file may get auto-reverted to HEAD by a pre-commit hook (observed
   behavior in earlier waves per the task brief) — after any edit, re-check
   `git status`/`git diff` on this file before assuming the edit stuck, and re-apply if
   reverted. Get the tree fully clean before moving on.
2. **B6 (new agent-ux catalog row):** Add a row "zero-shot human-simulation onboarding"
   to `quality/catalogs/agent-ux.json` — HAND-EDIT per its provenance note, mirroring the
   `agent-ux/dark-factory-sim` row's schema. Author a verifier under
   `quality/gates/agent-ux/` that runs the documented flow against a local sim in `/tmp`
   (leaf-isolation HARD-STOP applies) and greps for the doc-lie signatures fixed in Wave
   D. Wire it as a milestone-close gate. **Catalog-first rule: commit the GREEN-contract
   row BEFORE the verifier implementation exists** — do not write the verifier first.

### WAVE F — push + THE zero-shot gate (the DoD-defining litmus)

1. `git push origin main` — this MUST land before Wave G's verifier dispatch.
2. Dispatch a genuinely FRESH general-purpose agent with **zero repo context** — give it
   ONLY the published docs (as a fresh clone/checkout of the pushed docs would present
   them) — and have it copy-paste the documented sim getting-started flow against a
   **locally-built binary**, completing read + write + push with **ZERO manual
   fixups**. If it needs ANY fixup at all, that is a FAIL — loop back to Wave D with the
   specific friction point, fix it, and re-run this gate; do not patch around it live in
   the gate transcript.
3. This box runs git 2.25.1, below the documented `>=2.34` floor — if the flow succeeds
   anyway, flag the discrepancy to L0 in the close report; do not let it block Wave F if
   the flow itself succeeds.

### WAVE G — verifier

1. Dispatch `gsd-verifier` (per OP-7) to grade the catalog rows (including the new B6
   row from Wave E) PASS. Any RED loops back to the specific owning wave (D, E, or F) —
   do not attempt to fix RED items directly from Wave G.
2. Once green, hold at the STOP BOUNDARY below and report up.

## STOP BOUNDARY (do NOT cross)

The successor coordinator does **NOT** run `git tag v0.13.1`, trigger `.github/
workflows/release.yml`, or publish to crates.io — that irreversible action belongs to
L0 alone. When the DoD is green and the milestone is tag-ready, report to L0 with: the
exact tag-ready SHA, per-DoD-item evidence (doc-truth fixes verified, B5 decision
recorded, catalog rows PASS, zero-shot gate transcript showing zero fixups, clean tree
pushed to `origin/main`), the B5 decision summary, and the release-runbook state below.

## RELEASE RUNBOOK STATE (relay to L0 at close)

crates.io publish fires on MERGE-to-main via `release-plz.yml` (NOT the tag); tag `v*`
triggers `.github/workflows/release.yml` (multi-platform build). Gotchas: release-plz
has `git_release_enable=false` (per-package zero-asset releases previously stole
`releases/latest` and 404'd installer URLs — do not re-enable without reading that
file's header comment); a bot-authored release-plz push leaves `pull_request`-triggered
workflows stuck at `action_required` (a real-actor close/reopen of the PR unblocks
them); release-plz regenerates its PR on every `main` push, so the PR number moves —
don't hardcode a PR number in any runbook step.

## MILESTONE GOAL + DoD (verbatim, for successor's reference — do not reinterpret)

Hotfix so a FRESH zero-shot human-sim agent (no repo context, no system prompt, given
ONLY published docs) copy-pastes the documented sim getting-started flow on a
locally-built binary and completes read+write+push with ZERO manual fixups. Plus:
`reposix init` non-zero on unreachable backend (DONE, B4); no documented command errors
on copy-paste; B5 resolved honestly (DONE, triage-side; doc-side pending Wave D); clean
tree pushed to `origin/main`; unbiased `gsd-verifier` confirms catalog rows PASS.
