# SESSION-HANDOVER.md — v0.14.0 wave-2 CLOSED 11/11 GREEN, at owner tag boundary — 2026-07-12

For the incoming top-level orchestrator (L0). Map, not territory — detail lives in git and
the linked files. HEAD = live state only; history is in `git log`. Bound to live state;
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
never suppress a gate; guard context aggressively.

## 1. Current state (ground truth — confirm with `git rev-parse origin/main`)

- `origin/main == cf673ba`, **CI GREEN** (P112 run `29211572517`; P111 grade `29210988258`).
  **No `v0.14.0` tag exists** (local or remote) — clear for the owner to cut.
- **v0.14.0 wave-2 is 11/11 GREEN and STOPPED at the owner tag boundary.** Phases P102–P112
  all closed GREEN this session (P110 verifier `3067f15`; P111 milestone-close chain
  `6f6c2bf`→`c259718`, OP-9 ratification GREEN; P112 OD-4 scope stub `cf673ba`, DO-NOT-START).
- **Foreign uncommitted session work in the tree — DO NOT TOUCH.** `M quality/catalogs/code.json`
  (+ unpersisted status flips owned by another session/P113), untracked `.planning/phases/21-*`,
  `22-*`, `scripts/demos/`, `scripts/dev/`, `quality/reports/verifications/docs-repro/`, and a
  foreign `stash@{0}`. NEVER `git add .`/`git clean`/`git stash drop`; explicit-path commits only.

## 2. OWNER-ONLY — remaining to cut the v0.14.0 tag (E1/E3, do NOT self-action)

In order:
1. **9th probe `pre-release-real-backend`** currently reads NOT-VERIFIED (no real-backend
   creds). Owner provides TokenWorld creds + a non-default `REPOSIX_ALLOWED_ORIGINS`, then runs
   `python3 quality/runners/run.py --cadence pre-release-real-backend`. Non-skippable (RBF-FW-03,
   catalog row `agent-ux/milestone-close-vision-litmus-real-backend`, blast_radius P0).
2. **Mint + ratify** the aggregate `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`.
3. **Author + run the tag script**, then **cut `v0.14.0`** (`release.yml` fires on tag `v*`;
   `git_release_enable=false` STAYS — re-enabling stole `releases/latest` + 404'd installers).

## 3. RAISE LIST — for the owner / next L0 (none dropped)

1. **Shared-tree contention — HIGH.** Multiple sessions writing one tree + `.git` this session
   forced a lane to handle a foreign `code.json` to pass a rebase; the CI-`gh run watch` waiter
   hung twice (`bulqmsyrv`/`biy9yxt33`) needing manual nudges. The waiter is now fixed
   (`scripts/ci-wait.sh`, bounded-poll, dogfooded 3×). Still recommend **git-worktree isolation
   or session serialization** before the next parallel-writer fleet run.
2. **`STATE.md` (21k) + `ROADMAP.md` (37k) over the 20k soft limit** — GOOD-TO-HAVE
   progressive-disclosure split (archive closed workstream-A pre-tag checklist) via a dedicated
   `/gsd-quick` next session.
3. **ROADMAP P112 prose slightly ahead of artifact** (hinted a v0.15.0 dir; stub landed in
   `v0.14.0-phases/` per charter) — reconcile when the real launch-readiness milestone dir is
   minted via `/gsd-new-milestone`.

## 4. Forward pointer — v0.15.0 (scheduled, not started)

`.planning/milestones/v0.15.0-phases/ROADMAP.md` already holds two HEADLINE phases scheduled
this session: (a) the **UX error-message audit** (all CLI subcommands + `reposix-remote` helper
to the `init.rs` Rust-compiler-grade standard); (b) **error codes + `reposix explain <code>`**
(stable `RPX-xxxx` namespace + `rustc --explain`-style subcommand). Plus **GTH-09** (ADR-010
slug→id durable-create) DEFERRED here to v0.15.0 (recorded honestly in the v0.14.0 RETROSPECTIVE).
The **standing UX mandate** (Rust-compiler-grade UX: teach the fix / suggest the alternative /
copy-paste recovery) is now fix-twice'd into root + `crates/CLAUDE.md` and is active immediately.

## 5. Release/ops facts (settled)

crates.io publishes on MERGE-to-main via `release-plz.yml`; tag `v*` triggers `release.yml`;
`git_release_enable=false` STAYS; the aggregate `v$VERSION` tag is owner-cut (L0 never cuts it);
bot release-plz PRs sit at `action_required` until a real-actor reopen; watch release-plz
auto-titling for unintended minor bumps.

## 6. Doctrine

Full delegation/relief/cadence/durable-state: `.planning/ORCHESTRATION.md` §3 + §11 (no-fable
opus L0 recursion). Relief at ~100k own-context (hard 150k, absolute). ONE cargo invocation
machine-wide. Leaf isolation: leaf test setup in a throwaway `/tmp` clone, `cd` in the SAME bash
invocation. **Phase-close requires CI-green-on-main AFTER push** (`code/ci-green-on-main` P0
probe, `post-push` cadence) — never open the next phase over a red main. To resume a specific
agent's context, **SendMessage it — never `fork`** (a fork inherits the caller's context, not the
target's).

---

History lives in git — `git log` / `git show`, not restated here.
