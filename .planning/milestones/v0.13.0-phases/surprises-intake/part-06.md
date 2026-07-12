# v0.13.0 Surprises Intake (P96 source-of-truth) — Part 6 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## S-260707-rbf-lr03-external-write-crash | 2026-07-07 | discovered-by: v0.13.1 B5 TRIAGE | severity: HIGH | tag: v0.14.0 RBF-LR-03 pivot

**What:** The documented post-conflict recovery CRASHES when the SoT moved via an
**external REST write** (web UI / direct `PATCH`) rather than a git-side push. Reproduced
leaf-isolated in `/tmp` (sim `--ephemeral` seeded, git 2.25.1, `git-remote-reposix` on
PATH): after a local commit + an external `PATCH /projects/demo/issues/1`, the FULL
documented sequence `reposix sync --reconcile` → `git pull --rebase` → `git push` aborts
at the pull with:
```
warning: Not updating refs/reposix/origin/main (new tip 764e1a70… does not contain bd848c1…)
fatal: error while running fast-import
```
`reposix sync --reconcile` (exit 0) does NOT help — it is the trigger: it mints a fresh
"Sync from REST snapshot" synthesis commit that is NOT a descendant of the tracking tip,
so git's fast-import refuses the ref update. The bare `git pull --rebase` (no reconcile)
crashes identically. Data-safety kicker: the follow-on `git push` returns exit 0 and
would push the tree that never absorbed the external edit — silently overwriting the
external writer (the exact overwrite reconcile was documented to prevent). The only
current recovery is a fresh `reposix init` into a new dir (verified: fresh tree shows the
external edit), losing unpushed local commits.

**Why out-of-scope for v0.13.1 hotfix:** No `<1h`/no-new-dependency fix. A correct fix
must make the cache build snapshot commits as *descendants* of the prior synthesis tip
(lineage + dedup + push-conflict semantics) — precisely the RBF-LR-03 reconciliation
redesign already ratified as the v0.14.0 owner pivot (CONSULT-DECISIONS 2026-07-06). A
point patch here re-entrenches the placeholder-synthesis design the pivot exists to
replace. HOTFIX-conservative bias → docs made honest in v0.13.1, deep fix deferred.

**Sketched resolution (v0.14.0 RBF-LR-03):** In the reconciliation redesign, the
cache-side "Sync from REST snapshot" commit MUST be parented on the prior
`refs/reposix/origin/main` tip so `git pull --rebase` sees a fast-forwardable /
rebaseable lineage. Model the external-write reconciliation as a commit *appended* to the
existing history (owner's commit-sequence model) rather than a fresh-root snapshot. Add a
leaf-isolated regression proving `external PATCH → pull --rebase` recovers without
`fatal: error while running fast-import`. Repro scripts + full transcript:
CONSULT-DECISIONS.md 2026-07-07 entry.

**STATUS:** OPEN (deferred to v0.14.0 RBF-LR-03 pivot; docs-honesty spec handed to the
v0.13.1 doc-truth lane)

**Sub-risks surfaced by the v0.13.1 Wave D doc-truth lane (add to the v0.14.0 RBF-LR-03
scoping, same tree — not a separate lane):**

(a) **`sync --reconcile` can produce a teaching-free false-success.** The command exits
0 even when it leaves the cache in the non-descendant "Sync from REST snapshot" state
that then makes the very next `git pull --rebase` crash with `fatal: error while running
fast-import`. An agent (or human) reading only the exit code sees "reconcile succeeded"
and has no signal that the state it produced is unusable until the *next* command fails
opaquely. The v0.14.0 redesign should either (i) make `sync --reconcile` itself fail
loudly when it cannot produce a rebaseable/fast-forwardable lineage, or (ii) make the
synthesis commit genuinely rebaseable so the crash never happens — per the ratified
commit-sequence direction, (ii) is the real fix, but (i) is a cheap defensive teaching
improvement worth keeping even after (ii) lands.

(b) **Push-exits-0-after-failed-pull is a silent data-loss window.** Reproduced in this
lane's own repro: after the `sync --reconcile` → `git pull --rebase` sequence crashes,
the local tree still has the local edit but never incorporated the external REST edit;
the follow-on `git push` returns exit 0 (`[new branch] main -> main`) and silently
overwrites the external writer's change — the exact overwrite the whole recovery
sequence exists to prevent. Consider a guard: the bus/init push path could refuse (or at
minimum warn loudly) when it detects the local branch's cache-side lineage predates a
known-crashed reconcile attempt, rather than pushing a stale-relative-to-SoT tree with a
clean exit code. Worth scoping as part of the v0.14.0 reconciliation redesign's
acceptance criteria, not a bolt-on afterward.

## 2026-07-07 | discovered-by: v0.13.1 Lane E2 (agent-ux/zero-shot-onboarding) | severity: HIGH

**What:** `quality/gates/agent-ux/dark-factory/sim.sh` (catalog row `agent-ux/dark-factory-sim`,
committed `status: PASS`, `last_verified: 2026-05-01`) now FAILS for real every run: it
spawns `reposix-sim` on `127.0.0.1:7779` but never exports `REPOSIX_SIM_ORIGIN`, so the
`reposix init sim::demo <path>` call inside it resolves against the hardcoded
`DEFAULT_SIM_ORIGIN` (`127.0.0.1:7878`, `crates/reposix-cli/src/init.rs:23`) instead —
`init` tries to fetch from a port nothing is listening on and fails with `error: cannot
list issues for import: blocked origin: ...` / `fatal: error while running fast-import`.
Confirmed via a real `bash quality/gates/agent-ux/dark-factory.sh sim` run in this
session (exit 1) and by reading `init.rs:56-65`'s own comment, which says `init` added
`REPOSIX_SIM_ORIGIN` honoring specifically "so an isolated-port sim... can be init'd
against" — `sim.sh` was never updated to set it, so this appears to have silently broken
the moment that `init.rs` change landed, and the catalog's stale `PASS` from 2026-05-01
never caught it because no session re-ran `--persist` against it since.

**Why out-of-scope for v0.13.1 Lane E2:** dark-factory-sim.sh is a different catalog
row's verifier (`agent-ux/dark-factory-sim`), not a file this lane's dispatch (add
`agent-ux/zero-shot-onboarding`) touched or owns. My own verifier
(`quality/gates/agent-ux/zero-shot-onboarding.sh`) hit the identical hardcoded-port trap
during its own `reposix init` call and worked around it by exporting
`REPOSIX_SIM_ORIGIN` (see that script) — the fix pattern is proven, just not applied here
because it's a one-line change to a sibling row's committed-PASS verifier that this
dispatch wasn't chartered to touch.

**Sketched resolution:** add `export REPOSIX_SIM_ORIGIN="${SIM_URL}"` to
`quality/gates/agent-ux/dark-factory/sim.sh` (mirroring `dvcs-third-arm.sh`'s existing
`export REPOSIX_SIM_ORIGIN="${SIM_URL}"` line), re-run `bash
quality/gates/agent-ux/dark-factory.sh sim` to confirm PASS, then `python3
quality/runners/run.py --cadence on-demand --persist` to re-mint the row honestly (it is
currently phantom-green in the committed catalog).

**STATUS:** RESOLVED (v0.13.1 Lane E3, 2026-07-07) — applied the sketched
`export REPOSIX_SIM_ORIGIN="${SIM_URL}"` fix to `sim.sh` verbatim, confirmed the OLD
script (fix reverted) concretely FAILs against the current codebase (`error: cannot list
issues for import: blocked origin: http://127.0.0.1:7878/...`), then confirmed the FIXED
script exits 0 with `reposix init` correctly targeting `127.0.0.1:7779`. Row
`agent-ux/dark-factory-sim` re-minted honestly in `quality/catalogs/agent-ux.json`
(`last_verified`/`minted_at`: 2026-07-07T22:22:58Z, `coverage_kind: mechanical`,
`transport_claim: false`, `claim_vs_assertion_audit` documents the before/after proof).

## 2026-07-07 | git-floor drift in planning artifacts — `.planning/PROJECT.md` and `docs_reproducible_catalog.json` still assert a HARD `git >= 2.34` floor | discovered-by: v0.13.1 mechanical filing lane | severity: LOW

**What:** `.planning/PROJECT.md:41` ("Runtime requires git >= 2.34") and
`.planning/docs_reproducible_catalog.json:196` (a repro entry's preconditions list a
bare `git >= 2.34`) still assert a HARD floor, now inconsistent with the softened
README/doctor story shipped this milestone (git 2.34+ *recommended* for reliable
partial-clone reads / stateless-connect; sim quickstart empirically verified working on
git 2.25.1; `doctor` treats sub-2.34 as WARN, not ERROR — landed commits `f9d489a` +
`6dee426`).

**Why out-of-scope for eager-resolution:** both hits live in planning-artifact files
(not `docs/**`, which this lane is explicitly barred from touching) and rewording them
correctly requires cross-checking the exact softened phrasing the README/doctor now use
— a small but deliberate doc-consistency edit, not a mechanical filing action.

**Sketched resolution:** reword both to the softened split (sim flow works on 2.25+;
2.34+ recommended for reliable partial-clone/stateless-connect real-backend paths); the
`docs_reproducible_catalog.json` precondition can reference the `doctor` WARN behavior
instead of a bare version-string gate.

**Default disposition:** LOW — doc-consistency only, no runtime hazard. Target:
v0.14.0 scoping session or the next planning-artifact-touching phase.

**STATUS:** OPEN

## 2026-07-07 | Cache keyed by project name, not target dir — repeat-use friction on 2nd tutorial attempt | discovered-by: v0.13.1 zero-shot re-gate | severity: MEDIUM

**What:** `reposix init sim::demo <path>` keys its on-disk cache under a fixed
`~/.cache/reposix/<project>.git` regardless of the target directory, so two independent
"fresh" tutorial attempts on the same account silently SHARE backend/cache state. A user
re-running the tutorial after a failed first attempt sees pre-mutated data (issue
already `in_progress`, a stale comment) and may reasonably think something broke, since
nothing in the front-door UX signals the cache is being reused rather than freshly
seeded.

**Why out-of-scope for eager-resolution:** not a first-run blocker — the zero-shot gate
PASSED end-to-end on a clean machine — so it doesn't block v0.13.1. Fixing it properly
forks into a real design decision (key cache by target path vs. ship a cache-reset
command vs. a tutorial-only doc note), which is Rule-4 territory, not a mechanical
filing action.

**Sketched resolution:** either (a) doc the cache-keying behavior + offer a
`reposix` cache-reset command, (b) key the cache by target path instead of project name
so independent checkouts get independent caches, or (c) add a tutorial note ("re-running?
clear `~/.cache/reposix/<project>.git` first"). (b) is the most robust fix but touches
cache-bootstrap code; (c) is the cheapest stopgap.

**Default disposition:** MEDIUM — real repeat-use friction that undermines "front door
actually works" on a second run, even though first-run zero-shot passed. Target:
v0.14.0 scoping session.

**STATUS:** OPEN

## 2026-07-07 | S-260707-gh404 — GitHub real-backend helper path 404s on owner/repo: cache feeds filesystem-sanitized project into backend REST call | discovered-by: p94/protocol.rs CRLF-flake fix executor (CI run 28909417360) | severity: HIGH

**What:** The real-GitHub-via-helper path is broken. CI run 28909417360, job
`integration (contract, real github v09)`, test `dark_factory_real_github` panics at
`crates/reposix-cli/tests/agent_flow_real.rs:140`:
`reposix init github::reubenjohn/reposix failed: backend: github returned 404 Not Found
for GET https://api.github.com/repos/reubenjohn-reposix/issues?state=all&per_page=100`
— note the path is `reubenjohn-reposix` (DASH) not `reubenjohn/reposix` (SLASH).

**Root cause:** `crates/reposix-remote/src/main.rs:299` calls
`Cache::open(state.backend.clone(), &state.backend_name, &state.cache_project)` —
`cache_project` is the FILESYSTEM-sanitized form (`owner-repo`, dashes, from
`sanitize_project_for_cache`, main.rs:141). `Cache::build_from` then forwards that string
to `backend.list_records_complete(&self.project)`, which builds `{base}/repos/{project}/issues`
= `repos/owner-repo/issues` → 404. The slashed `state.project` (main.rs:142/153) is the
backend-correct form. Same latent bug in `crates/reposix-cli/src/attach.rs` (its
`Cache::open` call).

**Why it only surfaced now:** pre-existing since commit cd1b0b6 (Phase 32, backend
dispatch via URL scheme). Never caught because CI never exercised the real-GitHub-via-helper
path until commit b624688 (#71) built + PATH'd `git-remote-reposix` in the v09 jobs. The
non-v09 `integration (contract, real github)` job passes because it runs `reposix-github`'s
contract test DIRECTLY (never through the helper/cache).

**Why out-of-scope (deferred, not hotfixed):** NOT a safe one-line arg swap. `Cache` holds
ONE `project` field used for BOTH the on-disk cache dir path (`resolve_cache_path`, needs
the sanitized dash form) AND backend REST calls (need the slashed form). The correct fix
separates the two concerns (Cache carries a cache-path key distinct from the
backend-project, or sanitizes only at path-resolution time), applied in BOTH main.rs and
attach.rs, plus real-GitHub re-verification. That is transport-layer hardening → v0.14.0,
out of the v0.13.1 hotfix scope.

**Impact:** JIRA + Confluence real-backend transport are now PROVEN through the helper (v09
jobs green); the GitHub helper front-door is broken against real GitHub. Sim default and
direct `reposix-github` contract are unaffected.

**Default disposition:** HIGH — real broken real-backend front-door; route to v0.14.0
transport-layer hardening (fix in both main.rs and attach.rs + real-GitHub re-verify).

**STATUS:** OPEN

## 2026-07-07 | S-260707-desync — cache-desync on push produces confusing PATCH-404 instead of a diagnostic | discovered-by: p94/protocol.rs CRLF-flake fix executor | severity: MEDIUM

**What:** When the cache holds a `last_fetched_at` cursor + `oid_map` row for a record that
the backend GET no longer returns, `precheck.rs` Step 5
(`precheck_export_against_changed_set`, `crates/reposix-remote/.../precheck.rs:243`) trusts
the cache as the authoritative prior and emits an Update → PATCH, which then 404s as
`patch issue N: not found: <project>/N` rather than surfacing the D-01 recovery
(`reposix sync --reconcile`). Same error family as S-260707-gh404 above.

**Why out-of-scope:** low/medium severity; fold into the already-deferred v0.14.0
cache-desync hardening rather than mid-hotfix.

**Sketched resolution:** in `precheck.rs` Step 5, when a cached record's backend GET returns
NotFound, surface the D-01 `reposix sync --reconcile` recovery (teaching stderr) instead of
blindly emitting a PATCH that 404s. Pairs with the v0.14.0 cache-desync hardening.

**Default disposition:** MEDIUM — confusing-error UX, no data loss; route to v0.14.0
cache-desync hardening.

**STATUS:** OPEN

## 2026-07-07 | S-260707-relplz-tmp-collision — `/tmp` isolated git worktree not immune to concurrent leaf seed-commit corruption | discovered-by: v0.13.1 release Step-2 version-force executor | severity: MEDIUM

**What:** A version-force executor in an isolated `/tmp/relplz-force-$PID` worktree had a
rogue `dae3fea "seed"` commit appear atop its pushed HEAD, deleting the tree — a concurrent
shell-coverage-kcov worktree/pre-push quality-gate seeded into the same checkout dir.
CONTAINED (seed never pushed; remote PR branch stayed clean at 04640d5).

**Root cause:** `/tmp/<pid>` worktree root was not collision-immune vs a concurrent
quality-gate/coverage worktree sharing the same `.git`.

**Impact:** Hazard for any release/branch-edit leaf running while pre-push gates/shell-coverage
worktrees are active; contained this time but not mechanically prevented.

**Default disposition:** MEDIUM — sketch: unique-per-invocation worktree roots + lock, or
serialize worktree lanes. Route v0.14.0 hardening.

**STATUS:** OPEN

## 2026-07-07 | S-260707-relplz-tagcut-doc-gap — release-plz creates crates.io publish + per-package tags but NOT the plain vX.Y.Z aggregate tag release.yml needs — mandatory owner-gated manual tag-cut undocumented in executor-facing runbook | discovered-by: v0.13.1 release Step-3 publish executor | severity: MEDIUM

**What:** Merging the release-plz PR published 9 crates at 0.13.1 + created 9 per-package
tags, but no plain `v0.13.1` tag → release.yml (binaries + git-remote-reposix + GitHub
release) never triggered. release-plz pushes tags with GITHUB_TOKEN (no downstream-workflow
recursion); per repo convention the aggregate `vX.Y.Z` tag is OWNER-gated (P88 SC6
STOP-at-tag-boundary; RELIEF-HANDOVER-C2-wave-f.md:123; STATE.md:37; tag-v0.13.0.sh.disabled).

**Note:** the owner-gate is DELIBERATE — do NOT propose automating the tag away (that
bypasses the control); the gap is DOCUMENTATION: the release runbook/coordinator-dispatch
guidance must state the mandatory owner-gated aggregate-tag-cut step so future executors
don't assume auto-trigger and don't stall.

**Default disposition:** MEDIUM, route v0.14.0 doc hardening.

**STATUS:** OPEN

<!-- SKIPPED as true duplicate (2026-07-07, doc-executor lane): a third candidate row —
"shared .git/config corrupted mid-release (core.bare=true + identity t@t) by a concurrent
git-config-in-shared-repo leaf" — was NOT appended here. It is already fully covered by
`S-260707-pr-08` above (same root cause: worktree/cwd isolation not enforced, same
core.bare=true + `t <t@t>` identity fingerprint, same HIGH severity, same v0.14.0
enforcement-hook sketch). Filing it again would fragment the same incident across two rows;
if a NEW instance of this corruption recurs, prefer amending S-260707-pr-08's STATUS/count
rather than opening a fourth near-duplicate row. -->

## 2026-07-11 | S-260711-prerelease-swallow — quality-pre-release.yml runs `verdict.py --cadence pre-release || true`, swallowing a genuine RED | discovered-by: post-release verdict mint (cargo-binstall row) | severity: MEDIUM

**What:** `.github/workflows/quality-pre-release.yml:~51` invokes `verdict.py --cadence
pre-release || true`, so EVERY exit code — including a genuine RED verdict — is masked to
success. This is inconsistent with the weekly workflow's honest `verdict.py ... --fail-on
red`; a real pre-release red would ship silently.

**Why out-of-scope:** discovered mid-mint of an unrelated catalog row; editing a workflow
file is out of scope for a catalog-mint + intake-filing dispatch.

**Sketched resolution:** replace `|| true` with `--fail-on red` to align pre-release with
the weekly convention (D-CONV-2). Route v0.14.0 CI-honesty hardening.

**Default disposition:** MEDIUM — masks a real gate failure; no data loss but defeats the
gate's purpose.

**STATUS:** OPEN

