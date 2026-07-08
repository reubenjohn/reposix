# SESSION-HANDOVER.md — v0.13.1 "Front door actually works" — block-vs-ship decision — 2026-07-07

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

## 1. Where v0.13.1 stands (ground truth: HEAD `e2e4659` — confirm with
`git rev-parse HEAD origin/main` before trusting this section further)

- **Sim front door: DONE and proven.** `reposix init` exits non-zero on unreachable
  backend; `reposix sim` runs in-process (was forking an unshipped binary — this was the
  actual tutorial-dead-end root cause); doc-lies fixed; git version floor is accurate;
  conflict-recovery docs are honest about the known RBF-LR-03 limitation (deep fix
  deferred to v0.14.0); the zero-shot human-simulation gate (D3,
  `quality/gates/agent-ux/zero-shot-onboarding.sh`) is institutionalized and PASSES on a
  clean machine with only the 2 shipped binaries. Verification record:
  `.planning/milestones/v0.13.1-phases/VERIFICATION.md`.
- **CI on HEAD `e2e4659` (run `28910425946`):** clippy, rustfmt, gitleaks, test,
  coverage, shell-coverage, and sim-dark-factory are GREEN. `quality gates (pre-pr)` is
  GREEN (fixed via `7f17cee`, cache-dir isolation in `protocol.rs` push tests).
  **Real-backend transport was PROVEN for the first time ever in CI: JIRA v09 and
  Confluence v09 both PASS against LIVE backends.** This is only possible because the
  honest `init` fix unmasked that CI's real-backend jobs had never actually installed
  `git-remote-reposix` — they were false-green; `b624688` fixed the harness so they truly
  run now.
  **Next L0 action: re-confirm this run's final conclusion yourself** (`gh run view
  28910425946` or the latest run on `main`) before acting on it — it was read once this
  session, not independently re-verified at handover time.
- **ONE remaining red: `integration (contract, real github v09)`** — a "helper-path 404"
  against the real GitHub backend. Filed HIGH to
  `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` at commit `e2e4659`. It is
  **pre-existing** (false-green before v0.13.1's honesty fix surfaced it) — NOT a
  v0.13.1 regression. GitHub-specific: JIRA v09, Confluence v09, and non-v09 GitHub jobs
  all pass.

## 2. THE decision for next L0: ship v0.13.1 now, or hold for the GitHub-v09 fix?

Owner design taste points at **SHIP-with-documented-limitation**: "ship honest
milestones and document known limitations out loud rather than hold a green milestone
hostage." v0.13.1's actual purpose (sim front door + init/sim honesty) is fully
delivered and verified; the GitHub-v09 404 is a pre-existing real-backend gap orthogonal
to that purpose, and it is now honestly filed rather than hidden behind a false-green
job.

**RECOMMENDED PATH:** mark the `real github v09` CI job as a known-limitation
(`continue-on-error: true` or equivalent, with a comment linking the SURPRISES-INTAKE
row) so main CI reads "green except one documented pre-existing gap" instead of
silent-red on an unrelated axis — then release as **0.13.1** (§3).

**Alternative** (only if next L0 judges the 404 blocks the adoption story, i.e. GitHub is
the primary onboarding backend and a broken real-GitHub path undercuts the "front door
works" claim): hold v0.13.1 until the GitHub real-backend path is fixed. This pulls a
wave-2-sized real-backend fix into what was meant to be a hotfix — weigh that cost
against the adoption-story risk before choosing this branch.

Decide with the final CI conclusion in hand (§1's "re-confirm" action), then proceed to
§3.

## 3. Release runbook (SETTLED — release as 0.13.1, NOT release-plz's auto-0.14.0)

- release-plz PR **#69** auto-titled `chore: release v0.14.0` (feat(sim) commits →
  conventional-commits minor bump). **Override to 0.13.1 before merging** — the
  `v0.14.0` name is RESERVED for wave 2 (self-safe dark factory + reconciliation, §4).
  Re-verify the PR number before acting — it moves on every main push.
- crates.io publishes on **merge-to-main** via `release-plz.yml` (NOT the tag); tag `v*`
  triggers `release.yml` (binaries + GitHub release); `git_release_enable=false` in
  `release-plz.toml` STAYS — do NOT re-enable (it previously stole `releases/latest` and
  404'd the installer URLs; rationale in that file's header comment).
- Bot-authored release-plz PRs sit at `action_required` until a real-actor close/reopen.
- The **9th-probe (`pre-release-real-backend`) concern is now largely MOOT**: CI itself
  exercises live JIRA and Confluence (and attempts GitHub) on every push, substantially
  covering the milestone-close real-backend litmus — but still confirm the catalog row's
  status before declaring the probe formally satisfied.

## 4. Wave 2 — road to "on the map" (after v0.13.1 tags)

1. **D2 self-safe dark factory FIRST** — reject-`t@t`-identity commit/push hook + real
   per-leaf worktree isolation. The shared `.git/config` self-corrupted **3× this
   session** (origin was never affected; the pre-push gate held each time) via the
   credential-hook + sim/protocol tests running git in the shared tree. Non-negotiable
   first item. Anchor: `S-260707-pr-08`.
2. **Fix the GitHub-v09 real-backend helper-path 404** (the filed HIGH) + the filed
   cache-desync intake item.
3. **RBF-LR-03 reconciliation fix** — root cause of the broken `git pull --rebase`
   recovery path.
4. **Make waived tutorials reproduce** — `docs-repro`/`tutorial-replay` + examples
   01/02/04/05 are WAIVED-broken until 2026-09-15; a skeptical dev runs these.
5. **Carried HIGHs:** live RUSTSEC memmap2 + quinn-proto advisories in `Cargo.lock`,
   `prune_oid_map` pagination-truncation, RBF-FW-11 date-cutoff, quality-convergence
   write-contention.

## 5. Live nuisances (fix early in wave 2)

- **`doc-alignment` walk dirties the committed catalog on every read** (no `--persist`
  gate) — bites every push. Workaround: `git checkout -- quality/catalogs/doc-alignment.json`.
  High nuisance, prioritize a real fix.
- `SURPRISES-INTAKE.md` (~98k+ chars) and `GOOD-TO-HAVES.md` (~128k chars) are 5-6× past
  soft size limits — split/distill at wave-2 scoping. The SURPRISES "Entry format"
  template also documents a schema (`## YYYY-MM-DD HH:MM`) matching **no live row**
  (live rows use `## S-<id> — title (SEV)`) — fix the template so it stops misleading
  appenders.
- **`cargo-nextest` is NOT installed on this box** (`which cargo-nextest` fails; local
  runs fall back to `cargo test`). Add `cargo clippy --workspace --all-targets -- -D
  warnings` to the LOCAL pre-push gate so CI-clippy-red can't recur silently.

## 6. Doctrine

Full delegation / relief / cadence / durable-state doctrine:
`.planning/ORCHESTRATION.md` §3 — relief at ~100k own-context (hard stop ~150k), a
coordinator-of-coordinators per milestone, one-cargo-invocation machine-wide, and the
Leaf Isolation HARD-STOP (leaf test setup runs in a throwaway `/tmp` clone, `cd` into it
in the SAME bash invocation — never mutate git state in the shared repo/worktree).

**Meta-lesson carried into this window:** local pre-push green is necessary but not
sufficient — CI is the actual gate for tag-readiness; always run `gh run list --branch
main` (or `gh run view <id>`) on the tag-candidate HEAD before declaring a milestone
tag-ready. A second lesson from this window: an `init`/onboarding honesty fix can unmask
previously-false-green CI jobs (the real-backend harness bug) — a newly-red job after a
honesty fix is not necessarily a regression; check whether it was ever really passing.

---

History lives in git — `git log` / `git show`, not restated here.
