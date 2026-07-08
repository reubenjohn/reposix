# SESSION-HANDOVER.md — v0.13.1 onboarding-hotfix, CI-red pre-tag — 2026-07-08

For the incoming top-level orchestrator (L0). This is the map, not the territory — detail
lives in git and the linked files. HEAD = live state only; history is in `git log`.

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

- **No doc carries an unbounded-growth policy:** bound every doc to **live state**; git
  history is the only archive. Delete closed/superseded entries rather than appending or
  relocating them to a child file (a child file just relocates the growth). Exempt:
  code-enforced `audit_events` tables (operational forensic data, not docs).

### Calibration examples (the decide-vs-ask boundary)

| Situation | Right call | Why |
|---|---|---|
| Tag timing (ship v0.13.0 now vs hold for the pivot), owner said "leave it to you" | DECIDE (chose T1, ship now), record to ledger, proceed | Owner delegated + reasoning was available. Asking was over-asking — should've surfaced T1 as a reversible default to veto, not a gating question. |
| Reconciliation blocker: which fix mechanism | ASK (correctly) — but frame as a proposal | Architecture-shaping (E2); owner turned it into a design pivot no agent would've invented. Genuine owner input. Still: lead with a recommendation, let owner redirect. |
| Authorize a real-Confluence probe (credentials + real-backend call) | STOP for owner | Credential/spend + external mutation (E1/E3) — never self-authorize, even when confident. |
| "9th probe says NOT-VERIFIED but owner recalls it passing" | INVESTIGATE, don't ask | Not a decision — a fact to establish from committed evidence. Go find the crux (stale status vs real gap); only surface if evidence is genuinely absent. |
| Force-push main to correct one commit falsely authored by `t<t@t>` | ASK owner (chose amend+force-push) | Force-push to the primary branch is external + semi-irreversible (E1-class) — correctly surfaced as a reversible-default-to-veto, not self-authorized. |
| Post-release gate RED but delegation harness failed 3x on the log-read | L0 read the one CI log itself | A single decision-critical read-only fact is within L0's short-read allowance when delegation is failing — not the fleet-running work that is correctly delegated. |

Throughline: **default to decide-and-record; escalate only irreversible / external /
credential / spend; and "not a decision, go verify" is not an escalation.**

## 1. Where v0.13.1 stands (ground truth: `git rev-parse HEAD origin/main` both `dcb4117`, tree clean)

- **v0.13.1 "Front door actually works" is functionally COMPLETE and locally-verified
  GREEN** across 3 coordinator tenures (Waves A–G, handovers at
  `.planning/milestones/v0.13.1-phases/RELIEF-HANDOVER-C2-wave-{c,f}.md`,
  phase-close grade at `.../VERIFICATION.md`, verified `dcb4117`): `reposix init` now
  exits non-zero on unreachable backend (+test, `ee5f909`); the REAL front-door fix was
  making `reposix sim` run **in-process** (it was forking a `reposix-sim` binary that no
  install ships — dead-ended the tutorial at step 2 from any real install); doc-lies
  fixed (`issues/1.md` not `0001.md`, audit SQL columns, built-in seed); git floor
  softened (sim flow verified on git 2.25.1; 2.34+ now "recommended for partial-clone
  reads/stateless-connect", doctor treats sub-2.34 as WARN); B5 conflict-recovery
  confirmed as the known RBF-LR-03 limitation → docs made honest, deep fix filed to
  v0.14.0. A **zero-shot human-simulation gate** (D3) was institutionalized
  (`quality/gates/agent-ux/zero-shot-onboarding.sh` + catalog row) and passed on a clean
  machine with only the 2 shipped binaries. Verifier scored **6/6 DoD items GREEN**.
- **BUT it is NOT tag-ready: GitHub CI is RED on HEAD `dcb4117`** (confirmed live via
  `gh run view 28907728970`, run started 2026-07-08T00:11Z) — local gates missed it (no
  `nextest` installed on this box — `cargo-nextest not found`; local pre-push doesn't run
  `clippy --workspace -D warnings`). Two real, reproducible failures:
  1. **`clippy` job** — `cargo clippy --workspace --all-targets -- -D warnings` fails.
     The in-process-sim change (`ee5f909`) introduced lint warnings; this also fails the
     `code/clippy-lint-loaded` + `code/cargo-clippy-warnings` catalog rows.
  2. **`test` job** — step "Test pre-push credential hook" fails (`hook self-scan
     exclusion honored: expected exit=0, got 1`).
  `shell-coverage`, `rustfmt`, `gitleaks` jobs are GREEN; `quality gates (pre-pr)` job
  also fails (downstream of the above). **No CI-fix coordinator worktree or branch was
  found live in this repo at handover time** — if L0 believes one was dispatched, verify
  it independently (`git worktree list`, `gh pr list`, recent commits) before assuming
  progress; ground truth right now is CI RED, nothing in flight to fix it.
  **LESSON (already worth encoding): local pre-push green ≠ CI green; always see CI
  green before treating a milestone as tag-ready.**

## 2. Release decisions HELD by L0 (SETTLED — do not re-litigate)

- **Version: force the release out as `0.13.1` (patch), NOT release-plz's computed
  `0.14.0`.** release-plz PR **#69** (`chore: release v0.14.0`, branch
  `release-plz-2026-07-07T03-15-08Z`) is open and computed a MINOR bump because the
  milestone contains `feat(sim):`-tagged commits (conventional-commits →
  minor). PR #69 also currently shows a `reposix-cli` **semver-breaking-change**
  warning (removed `binpath::resolve_bin` / `sim` module paths) from
  `cargo-semver-checks` — inspect before merging regardless of version number.
  **v0.14.0 the NAME is RESERVED** for the big orchestration-hardening +
  RBF-LR-03-reconciliation milestone (§3 below). Override release-plz to `0.13.1`
  (explicit workspace version / edit the release PR) before merging. Do this **only
  after CI is green** (§1).
- **Release runbook (verified from `.planning/STATE.md` + prior sessions):**
  crates.io publishes on MERGE-to-main via `release-plz.yml` (NOT the tag); tag `v*`
  triggers `release.yml` (binaries + GitHub release); `git_release_enable=false` in
  `release-plz.toml` — do NOT re-enable (it previously stole `releases/latest` +
  404'd installers, see that file's header comment); bot-authored release-plz PRs sit
  at `action_required` until a real-actor close/reopen; the release-PR number moves on
  every main push (already moved twice this window: #61 → #68 → #69).
- **9th probe `pre-release-real-backend` = NOT-VERIFIED** (row
  `agent-ux/milestone-close-vision-litmus-real-backend`,
  `quality/catalogs/agent-ux.json:1323`: env-gated, creds/allowlist unset on this box;
  `last_real_grade: PASS`, `last_verified: 2026-07-06T05:03:59Z` — that PASS dates to
  the v0.13.0 close window, not v0.13.1's). It's an external-backend +
  credential/spend action = genuinely OWNER-CLASS — do NOT self-run. Per
  `VERIFICATION.md` §"FOR L0": v0.13.1's change surface is 100% sim-only (in-process
  front door + builtin seed) and touches zero real-backend transport code, so a fresh
  real-backend litmus is arguably not implicated. RECOMMENDATION to surface to owner
  as a veto-default: "tag v0.13.1 without re-running the real-backend probe since the
  change is sim-only; say the word to run it (needs your creds)." Owner has
  PRE-AUTHORIZED releases generally, but real-backend calls specifically remain
  owner-gated.

## 3. Immediate runbook — get to a real tag decision

1. **Fix CI red first** — dispatch a coordinator (or do it directly if small) to:
   (a) run `cargo clippy --workspace --all-targets -- -D warnings` locally, fix the
   warnings introduced by `ee5f909`'s in-process-sim change; (b) reproduce and fix the
   "Test pre-push credential hook" self-scan-exclusion failure; (c) push, then verify
   `gh run list --branch main -L 3` shows GREEN on the new HEAD before touching
   anything release-related.
2. **Once CI is green:** surface the sim-only real-backend-probe recommendation (§2) to
   the owner as a reversible default to veto; on silence/approval, proceed.
3. **Force the version to `0.13.1`** on the release-plz PR (override the computed
   `0.14.0`), verify the `reposix-cli` semver-breaking-change warning is either
   expected/acceptable for a patch release or needs addressing, merge, confirm
   crates.io publish, then cut tag `v0.13.1` (owner/L0-gated per §2 runbook).
4. **Only after v0.13.1 tags:** move to Wave 2 (§4).

## 4. Wave 2 (after v0.13.1 tag) — the road to "on the map"

1. **v0.14.0 — self-safe dark factory + reconciliation:** the D2 `reject-t@t`-identity
   commit/push hook + real per-leaf worktree isolation (the repo self-corrupted twice
   from shared `.git/config` in an earlier session); PLUS the deep RBF-LR-03
   reconciliation fix (root cause of the broken `git pull --rebase` recovery, filed
   HIGH). Anchor intake: `S-260707-pr-08`.
2. **Tutorials must actually reproduce:** `docs-repro`/`tutorial-replay` + examples
   01/02/04/05 are WAIVED-as-broken until 2026-09-15 — a skeptical dev runs those; they
   must go green.
3. **Real-backend proof end-to-end** (Confluence TokenWorld / GitHub / JIRA) + the 5
   carried HIGHs (live RUSTSEC memmap2 + quinn-proto advisories in `Cargo.lock`,
   `prune_oid_map` pagination-truncation, RBF-FW-11 date-cutoff, quality-convergence
   write-contention).

## 5. Live nuisances to fix early in Wave 2 (noticed across this milestone)

- **`doc-alignment` walk dirties the committed catalog on EVERY read** (no `--persist`
  gate) — bites every push (workaround: `git checkout -- quality/catalogs/doc-alignment.json`).
  Confirmed still live: `git status` shows `M quality/catalogs/doc-alignment.json` at
  handover time with no substantive diff intended. High nuisance, prioritize.
- `SURPRISES-INTAKE.md` and `GOOD-TO-HAVES.md` are several times past soft limits —
  split/distill at v0.14.0 scoping; the SURPRISES "Entry format" template also documents
  a schema (`## YYYY-MM-DD HH:MM`) that matches NO live row (live rows use
  `## S-<id> — title (SEV)`) — fix so it stops misleading appenders.
- **`cargo-nextest` NOT installed on this box** (`which cargo-nextest` → not found;
  local verification runs fall back to `cargo test`); CLAUDE.md/commands reference
  nextest — install it or correct the docs. This is also *why* local gates missed the
  CI-red clippy/hook failures — **add `cargo clippy --workspace --all-targets -- -D
  warnings` to the LOCAL pre-push gate** so CI-clippy-red can't recur silently.
- ~180-row doc-alignment false-BOND re-grade backlog filed to v0.14.0 (per
  `VERIFICATION.md` notes).

## 6. Known brittle gates + hazards

- **p94-badges** — was a genuine red in the v0.13.0 window (`S-260707-pr-07`); re-check
  before dismissing as brittle.
- **doc-alignment walker** re-drifts `last_walked`/counters on every pre-push read — see
  §5, filed as a GTH, not yet fixed.
- **"No tools needed for summary"** — a recurring subagent-dispatch harness flake seen
  in earlier v0.13.0 sessions (agent executes zero tools, returns early); retry once
  with a trivial `echo ok` health-check dispatch before escalating.
- **Worktree/identity-corruption hazard (v0.13.0-era, repaired, root-caused into D2)** —
  a dispatched leaf can corrupt the shared repo (`core.bare`, `user.email`/`user.name`)
  if it doesn't isolate into `/tmp`. Root rule: root `CLAUDE.md` § "Leaf test setup" +
  `.planning/ORCHESTRATION.md` § "Leaf isolation". **Always confirm
  `git config user.email == reubenvjohn@gmail.com` before any commit.**

**Waiver clocks:** `structure/file-size-limits` WAIVED until **2026-08-08** (renew
before); `docs-repro`/tutorial rows WAIVED until **2026-09-15**.

## 7. Verification honesty (read before trusting §1)

- The 6/6 GREEN DoD grade in §1 is from the unbiased `gsd-verifier` subagent
  (`VERIFICATION.md`, graded at `a374a4b`, one commit before current HEAD `dcb4117`) —
  it is a real independent grade, not self-reported by the executing coordinator.
  However **it graded local gates only** (pre-push cadence, 55 PASS/0 FAIL/1 WAIVED);
  it did not and could not see the CI-red state discovered afterward (§1) — CI ran
  fresh on the next push and failed. Treat "6/6 DoD GREEN" and "CI RED" as both true and
  not contradictory: the milestone's functional work is sound, but the tree currently
  fails a check the local verifier doesn't run (clippy `-D warnings`, nextest-based
  hook test).
- Real-backend workflows (Confluence/GitHub/JIRA) were NOT re-verified for v0.13.1 — the
  9th probe's `last_real_grade: PASS` is a carry from the v0.13.0 close window (§2).
- The "CI-green fix coordinator in flight" claim in the dispatching instructions for
  this handover was **not independently confirmed** by ground truth at write time — no
  matching worktree, branch, or WIP commit was found. Next L0 must verify directly
  (`gh run list --branch main`, `git worktree list`, `gh pr list`) rather than assume
  progress happened between this handover and the next session start.

## 8. Doctrine

Full delegation / relief / cadence / durable-state doctrine:
`.planning/ORCHESTRATION.md` — relief at ~100k own-context (hard stop ~150k), a
coordinator-of-coordinators per milestone, one-cargo-invocation machine-wide, and the
Leaf Isolation HARD-STOP.

**Meta-lesson carried into this window:** local pre-push green is necessary but not
sufficient — CI (which runs `clippy -D warnings` and the full pre-push-hook test suite
via `cargo test`, not `nextest`, since nextest isn't installed locally) is the actual
gate for tag-readiness. Always run `gh run list --branch main` on the tag-candidate HEAD
before declaring a milestone tag-ready.

---

History lives in git — `git log` / `git show`, not restated here.
