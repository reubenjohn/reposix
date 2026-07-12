# SESSION-HANDOVER.md — v0.14.0 wave-2, CI-green restored + fix-twice landed — 2026-07-12

For the incoming top-level orchestrator (L0). Map, not territory — detail lives in git
and linked files. HEAD = live state only; history is in `git log`. Bound to live state;
delete closed/superseded entries rather than appending.

## 0. Owner calibration — READ FIRST (over-ask LESS)

Decide-and-record, not gating questions. Pick the path the owner's model implies, log to
`.planning/CONSULT-DECISIONS.md`, proceed — the owner vetoes if you misread. Reserve STOPs
for the genuinely-owner class: irreversible/destructive, external-backend mutations,
credential/spend (E1/E3) — never cut a real tag or fire a real-backend call without the
owner. Prefer surfacing a reversible default-to-veto over a blocking question.

Owner design taste: backend owns identity, client works in slugs; model client↔server as
git-native self-reconciling commit sequences; big questions are pivots to
explore/prototype/converge; ship honest milestones with limitations documented out loud,
never suppress a gate; guard context aggressively (fork, prune, lean on git).

## 1. Current state (ground truth — confirm with `git rev-parse origin/main`)

- `origin/main` was `ab51024` at wrap (the `code/ci-green-on-main` dogfood-PASS commit;
  parent `335f9b7` CI-confirmed **GREEN, all 6 jobs**). Tree CLEAN except the
  persistent FOREIGN untracked dirs from a concurrent session (`.planning/phases/21-*`,
  `22-*`, `scripts/demos/`, `scripts/dev/`) + a foreign `stash@{0}` — NOT ours; NEVER
  `git add .`/`git clean`/`git stash drop`; explicit-path commits only. NB: the concurrent
  session actively pushes planning-doc commits to main ("manager handover" edits) — main
  MOVES under you; `git fetch && git pull --rebase` before any push.
- v0.14.0 landed GREEN so far: P102 D2 gate; P103 early wins; P104 gh404 (sim/unit +
  honest known-limitation); P105 RBF-LR-03; Phase-0 D2 RE-SEAL; fleet-safety persist-gate;
  P108 prune-gate; P109(a) RBF-FW-11.

## 2. This session's two SERIALIZING deliverables — BOTH DONE + verified green

1. **CI-green restored, HONESTLY.** The `code/shell-coverage` ratchet gate was FAILing in
   CI (12.54% vs 13% floor — new fleet-safety guard scripts diluted the corpus). Fixed by
   COVERING the corpus: new kcov harness
   `quality/gates/code/shell-coverage-tests/11-fleet-safety-guards.sh` genuinely exercises
   `leaf-isolation-guard.sh`'s 3 guards + 4 fleet-safety gates; aggregate 11.80% → 14.78%.
   **Floor untouched at 13; no waiver, no silent lowering.** Fix `15bbe88`, verified green
   on CI run 29198056598. Filed noticing: kcov nested-attribution flake = GOOD-TO-HAVES-01.
2. **Fix-twice the systemic hole** (8 GREEN phases had shipped over a red main because
   phase-close verified push-LANDED but not CI-green-on-main). Landed (`67f3efe`→`335f9b7`):
   - Executable P0 probe `quality/gates/code/ci-green-on-main.sh` (row `code/ci-green-on-main`),
     run at phase-close via `python3 quality/runners/run.py --cadence post-push --persist`;
     asserts main's newest `ci.yml` run concluded success, else phase verdict RED. NON-circular
     (runs orchestrator-side AFTER push, unlike the demoted D-CONV-1 in-CI check).
   - `.github/workflows/docs.yml` now gated on CI: `workflow_run: workflows:["CI"]` +
     job-level `if success` + a paths-preserving detect step (D-CONV-4). A red main no
     longer deploys docs.
   - Doctrine fix-twice'd into `.planning/CLAUDE.md` (push-cadence bullet, ll.69-75), root
     `CLAUDE.md`, `.planning/ORCHESTRATION.md` §3/§6. **New standing rule: never open the
     next phase over a red main.**

## 3. IN FLIGHT — v0.14.0 READY queue owned by a milestone C2 (opus phase-coordinator)

A milestone-scoped C2 (`SendMessage` to `add159944b57d8a99`) is driving the owner's fixed
order autonomously; phase rotations absorb below the top. Its charter (queue detail:
`.planning/milestones/v0.14.0-phases/ROADMAP.md`):

- **P106** (START/in-progress; deadline 2026-09-15) — clear the 6 WAIVED
  `quality/catalogs/docs-reproducible.json` rows (tutorial-replay + examples 01/02/04/05).
  Real tutorial/examples repro work, NOT started before this session; needs sim runs +
  /tmp-clone leaf isolation. (Product-layer incident already sealed by Phase 0; orthogonal.)
- **P107** — RUSTSEC memmap2/quinn-proto: C2 does the non-gated part (fresh `cargo audit`,
  document posture). Dependabot **#64/#65/#66** merges are OWNER-GATED — C2 surfaces, never
  self-merges. If P107 can't green without the merges, C2 STOPS and surfaces.
- **P110/P111** — OP-8 drains + milestone-close (RETROSPECTIVE/OP-9, CHANGELOG, tag-script,
  9th probe). BLOCKED until P106+P107 GREEN. C2 STOPS at the tag boundary (owner cuts `v*`).
- **P112** — OD-4 launch-readiness SCOPE-ONLY stub, DO-NOT-START.

RESIDUAL follow-up (defense-in-depth, observed vector already closed): worktree-shared
`.git` object-store self-safety + non-Bash subprocess boundary — not yet lane'd.

**Net-new noticing from the WS2 fix-twice (handed to C2 to file into SURPRISES-INTAKE):**
`.github/workflows/release.yml` (tag `v*`) is the THIRD ungated deploy path — a tag cut
over a red main would still publish (phase-close + docs-deploy are now CI-gated; the tag
publish is not). MEDIUM. Matters because the aggregate `v*` tag is owner-cut at
milestone-close (P111) — worth gating before the next tag.

## 4. OWNER-GATED — pending owner decision (surfaced, NOT executed)

1. Land `424d367` (lost-update HIGH fix, clean 4-commit ff) to GitHub main — recommend yes,
   from a clean `/tmp` clone through the real pre-push gate. Anchored LOCAL-ONLY on branch
   `backup-lost-update-424d367`.
2. Dependabot #64/#65/#66 — `cargo audit` = 0 live advisories, none touch memmap2/quinn-proto,
   stale-base bumps — recommend close-as-redundant.
3. gh404 live-GitHub read-only verify — recommend defer; sim/unit stands.
4. GTH-09 ADR-010 slug→id durable-create (MEDIUM-HIGH, UNSTARTED v0.14.0 headline) — ship
   this milestone or defer? Owner scope call.

## 5. Release/ops facts (settled)

crates.io publishes on MERGE-to-main via `release-plz.yml`; tag `v*` triggers `release.yml`;
`git_release_enable=false` STAYS (re-enabling stole `releases/latest` + 404'd installers);
aggregate `v$VERSION` tag is owner-gated (L0 cuts it); bot release-plz PRs sit at
`action_required` until a real-actor reopen; watch release-plz auto-titling for unintended
minor bumps.

## 6. Doctrine

Full delegation/relief/cadence/durable-state: `.planning/ORCHESTRATION.md` §3 + §11
(no-fable opus L0 recursion). Relief at ~100k own-context (hard 150k, absolute). Leaf
isolation HARD-STOP: leaf test setup in a throwaway `/tmp` clone, `cd` in the SAME bash
invocation. ONE cargo invocation machine-wide. **Phase-close now requires CI-green-on-main
after push (§2.2) — dogfood it.**

---

History lives in git — `git log` / `git show`, not restated here.
