# SESSION-HANDOVER.md — v0.14.0 wave-2 in progress, post-incident clean — 2026-07-12

For the incoming top-level orchestrator (L0). This is the map, not the territory —
detail lives in git and the linked files. HEAD = live state only; history is in `git
log`. No doc carries an unbounded-growth policy: bound to live state, delete
closed/superseded entries rather than appending.

## 0. Owner calibration — READ FIRST (over-ask LESS)

The owner wants **decide-and-record, not gating questions.** Pick the path the owner's
model implies, log it to `.planning/CONSULT-DECISIONS.md` with reasoning, and proceed —
the owner vetoes if you misread. Reserve owner STOPs for the genuinely-owner class only:
**irreversible/destructive moves, external-backend mutations, and credential/spend
authorization** (E1/E3) — e.g. never cut a real tag or fire a real-backend call without
the owner. When you would ask, prefer surfacing a **reversible default to veto** over a
blocking question. "Not a decision, go verify" is not an escalation.

**Owner design taste** (use to make calls autonomously): backend owns identity, client
works in **slugs** (client-side ID remapping is a smell); model multi-step client↔server
interactions as **git-native commit sequences that self-reconcile on partial fail**; big
design questions are **pivots to explore/prototype/converge**, not point-patches; **ship
honest milestones and document known limitations out loud** rather than suppress gates or
hold a green milestone hostage; **guard context aggressively** (fork, prune, lean on git,
least-complex path).

## 1. Current state (ground truth — confirm with `git rev-parse HEAD origin/main`)

- `origin/main == HEAD == c779a629`, tracked tree CLEAN. Only untracked = 4 FOREIGN dirs
  from a concurrent session (`.planning/phases/21-*`, `22-*`, `scripts/demos/`,
  `scripts/dev/`) + a foreign `stash@{0}` (`WIP on main: faf3d16 docs(22): create phase
  plan — 3 plans across 2 waves for BENCH-01..04`) — NOT ours; NEVER `git add .`/`git
  clean`/`git stash drop`; explicit-path commits only.
- v0.14.0 milestone established (P102–P112). Landed GREEN this session: P102 D2
  self-safe gate; P103 early wins; P104 gh404 (sim/unit + honest known-limitation); P105
  RBF-LR-03; Phase 0 D2 RE-SEAL (`2ad2bf5`, `3206a2b`); fleet-safety persist-gate
  (`309f0b6`, `72ae517`); P109(a) RBF-FW-11 (`1cb9dd1`, `10bd508`); P108 prune-gate
  (`13de686`, `b037876`).
- The lost-update HIGH fix is a clean 4-commit ff of prior main, anchored LOCAL-ONLY on
  branch `backup-lost-update-424d367` (`424d367`). LANDING IT TO GITHUB MAIN IS
  OWNER-GATED — not yet pushed.

## 2. INCIDENT (resolved) — read before dispatching any fleet

A P106 leaf ran `reposix init` inside `.claude/worktrees/...` of the SHARED repo
(subprocess path bypassing the Bash-only D2 hook), flipping `.git/config` to bare=true,
repointing origin to the sim (127.0.0.1:7988), injecting `[user] t@t`, thrashing HEAD to
e18df81. L0 REPAIRED it (rewrote `.git/config` clean, `git reset --hard` to real main,
pruned `refs/reposix/*`). ZERO data loss (sim never had GitHub write access). Phase 0
re-sealed the leak at the PRODUCT layer (`reposix init` refuses an existing worktree
root, fail-closed) + guard extensions + fixed a D2 guard false-positive (it had blocked
read-only `git config --get`). RESIDUAL (defense-in-depth, observed vector already
closed): worktree-shared `.git` object-store self-safety + non-Bash subprocess boundary
still open — recommend a follow-up lane.

## 3. NEXT — resume v0.14.0 wave-2 (dispatch a fresh milestone-scoped C2, opus,
phase-coordinator)

READY queue (from
`.planning/milestones/v0.14.0-phases/RELIEF-HANDOVER-C2-wave-2.md` @ `c779a629`):
tutorials 01/02/04/05 self-start-sim rewrite (waiver deadline 2026-09-15) → git-2.34 CI
boundary job (scope brief embedded in that handover; "verified to 2.25" is currently
prose-only) → shell-coverage floor (MEDIUM, filed) → OP-8 file splits
(SURPRISES-INTAKE/GOOD-TO-HAVES 5-6x oversize, structure waiver expires 2026-08-08) →
OP-9 distill → OD-4 launch-readiness SCOPE-ONLY stub → milestone-close 9th probe
(pre-release-real-backend, non-skippable). Plus the Phase 0 residual follow-up lane
above.

## 4. OWNER-GATED — pending owner decision (surfaced, not executed)

1. Land `424d367` to GitHub main (recommend: yes, from a clean `/tmp` clone that passes
   the real pre-push gate).
2. Dependabot #64/#65/#66 — `cargo audit` = 0 live advisories, none touch
   memmap2/quinn-proto, stale-base bumps (recommend: close-as-redundant).
3. gh404 live-GitHub read-only verify (recommend: defer; sim/unit stands).
4. NEW — GTH-09 ADR-010 slug→id durable-create (MEDIUM-HIGH): an UNSTARTED v0.14.0
   headline item — ship this milestone or defer? Owner scope call.

## 5. Release/ops facts (settled)

crates.io publishes on MERGE-to-main via `release-plz.yml`; tag `v*` triggers
`release.yml`; `git_release_enable=false` STAYS (re-enabling stole `releases/latest` +
404'd the installers); the aggregate `v$VERSION` tag is owner-gated (L0 cuts it);
bot-authored release-plz PRs sit at `action_required` until a real-actor reopen;
release-plz auto-titles from conventional-commits (watch for unintended minor bumps —
v0.13.1 had to be forced down from an auto-computed v0.14.0).

## 6. Doctrine

Full delegation / relief / cadence / durable-state doctrine:
`.planning/ORCHESTRATION.md` §3 — relief at ~100k own-context (hard stop ~150k), a
coordinator-of-coordinators per milestone, one-cargo-invocation machine-wide, and the
Leaf Isolation HARD-STOP (leaf test setup runs in a throwaway `/tmp` clone, `cd` into it
in the SAME bash invocation — never mutate git state in the shared repo/worktree).

---

History lives in git — `git log` / `git show`, not restated here.
