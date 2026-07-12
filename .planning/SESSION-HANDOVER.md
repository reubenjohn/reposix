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

**③ release-plz RED on main — RESOLVED, verified green** (owner eyes-and-ears BLOCKER).
release-plz refused on a dirty CI checkout: three
`quality/reports/verifications/agent-ux/fleet-safety-*.json` regenerate at grade time with
env-dependent assert outcomes, so byte-stability (309f0b6's approach) was fundamentally
fragile. Fix = **untrack** them (approach 2, P102 precedent `fbe02c8`; nothing reads them
as a baseline, already matched `.gitignore:72`): `git rm --cached` the 3 + regression pytest
`quality/runners/test_fleet_safety_verdicts_untracked.py`. Commit `3d3e60e` — release-plz
run `29199764486` **success** (prior two runs failure; the fix flipped it). NEW filed items:
(a) **fold release-plz into the `code/ci-green-on-main` bar** — decided YES-in-principle but
NON-TRIVIAL (probe hardcodes `WORKFLOW=ci.yml`; needs parameterization + false-RED design re
"does release-plz run every push / is no-release success-or-skipped") → CONSULT-DECISIONS D5
+ SURPRISES-INTAKE with sketch; (b) **runner unit tests uncollected by CI** (regression
guards inert in CI) → SURPRISES-INTAKE MEDIUM (`856f52f`). NB: `SURPRISES-INTAKE.md` now
30k chars, over its 20k soft limit — the C2's P110 intake drain should split it.

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

## 4. Parked owner calls — 3 delegated + executing (2026-07-12 manager relay), 1 still owner-held

1. **Land `424d367`** (lost-update HIGH fix) → OWNER-AUTHORIZED, **IN FLIGHT** (`SendMessage`
   → `a35d33efbec483f86`, opus, isolated `/tmp`-clone). NB: it's NO LONGER an ff onto current
   main — the lane rebases the 4 commits (code fix `34cfbea`/`39f9d64` + catalog `5028542` +
   ROADMAP-renumber `424d367`) onto current main. The renumber (106→113) likely conflicts with
   the C2's live ROADMAP (P106=tutorials now) — lane lands the CODE+catalog cleanly and DEFERS
   the planning-renumber to SURPRISES-INTAKE if ambiguous. Confirms ci.yml+release-plz green on
   the landed SHA. Local branch `backup-lost-update-424d367` KEPT as safety anchor (owner prunes).
2. **Dependabot #64/#65/#66** → **CLOSED** as redundant (owner-authorized; #64 tower-http, #65
   gix, #66 rusqlite — all verified CLOSED, comments posted, no cargo run). Ledger `2ecbea2`.
3. **gh404 live-GitHub verify** → **DEFERRED** by owner (record-only, no action). Sim/unit
   coverage from P104 stands as the honest known-limitation; re-open only if the owner asks.
4. **GTH-09** ADR-010 slug→id durable-create → **DEFERRED to v0.15.0** by owner call
   (2026-07-12). Named-headline deferral, NOT a silent slip: the C2 (`add159944b57d8a99`) is
   recording it in the v0.14.0 ROADMAP + GOOD-TO-HAVES and must document it out loud at
   milestone-close (P111) so v0.14.0 ships honest-without-it.

## 4b. STANDING UX MANDATE (owner, 2026-07-12) — north star for ALL tooling

End-user experience is the north star all tooling serves (docs, error-messages-with-fix-hints,
onboarding). Bar = **Rust-compiler-grade UX**: teach the fix / suggest the alternative /
copy-paste recovery. `init.rs` is the exemplar; the rest of the CLI + remote helper must reach
it. UX polish is scheduled FIRST-CLASS, never a leftover. Being encoded now (lane
`aaa7268efc0a12311`, opus): (a) **fix-twice into CLAUDE.md** — root (strengthen OD-3 #5 into the
concrete 3-part bar) + `crates/CLAUDE.md` (error-message convention, init.rs pattern); (b)
**scheduled as a first-class v0.15.0 phase** — audit every CLI subcommand + remote helper error
surface to the init.rs standard; kept in a v0.15.0 backlog stub (separate from the C2's live
v0.14.0 ROADMAP to avoid two-writer conflict); decision recorded `[SELF]` in CONSULT-DECISIONS,
reversible (owner may pull into v0.14.0). Mandate is active IMMEDIATELY — P106's remaining
tutorial work inherits the bar. Tutorials/onboarding-friction stays on the roadmap.

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
