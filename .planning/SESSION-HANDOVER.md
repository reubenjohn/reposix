# SESSION-HANDOVER.md — v0.13.1 onboarding hotfix + v0.14.0 hardening runbook — 2026-07-07

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

## 1. Current state

- **v0.13.0 SHIPPED and VERIFIED SOUND.** `origin/main == HEAD == 5fd4731`
  (confirm live via `git rev-parse HEAD origin/main`). Tag `v0.13.0` exists at `3423b18f`.
  GitHub release is Latest with 8 assets. All 9 crates published at `0.13.0`
  (crates.io verified). PRs **#68** (release), **#70** (binstall gate fix, merged
  `4b564e4`), **#71** (post-release findings, merged `bfdba9a`) are all MERGED.
- **Both original release-blocking scares were VERIFIED FALSE ALARMS:**
  (a) the `release/cargo-binstall-resolves` RED was a stale-literal-string brittle gate —
  the installer works; fixed in PR #70 to assert the actual invariant instead of a
  hardcoded string.
  (b) the CI failure `crlf_blob_body_round_trips_byte_for_byte` is a **wiremock
  test-harness artifact** under CI CPU starvation, NOT byte corruption — production
  preserves bytes byte-for-byte. Root cause filed **S-260707-rbf-01**, still OPEN as a
  monitor (see §5 for the next experiment).
- **Repo-corruption hazard this session (repaired, not carried forward as live damage):**
  a dispatched sim/seed leaf CORRUPTED the local shared repo TWICE (flipped
  `core.bare=true`; set `user.email`/`user.name` to `t<t@t>`). Both repaired by L0;
  **origin was never affected.** Root cause: agent worktrees are NOT isolated here (shared
  `.git/config` + object store) + cwd resets between Bash calls. HARD-STOP rule now lives
  in `.planning/ORCHESTRATION.md` § "Leaf isolation" + root `CLAUDE.md` § Non-negotiables.
  **Guard for the next session: run `git config user.email` and confirm it is NOT `t@t`
  before any commit.**

## 2. Decisions SETTLED this session (D1/D2/D3)

Full text + rationale + evidence: `.planning/CONSULT-DECISIONS.md`.

- **D1 — v0.13.1 onboarding hotfix, sequenced BEFORE the v0.14.0 pivot.** Binary-install
  onboarding is 100% broken: `reposix-sim` (the documented DEFAULT backend, OP-1) ships in
  NO prebuilt distribution, so a new user following the docs hits a wall — and
  `reposix init` MASKS the failure behind **exit 0** (3 independent zero-shot
  reproductions this session, see §4). An adoption-blocker on `releases/latest` cannot
  wait behind a milestone-sized pivot.
  **Acceptance (end-state; mechanism converges in discuss/plan):**
  (i) the documented getting-started flow completes end-to-end on the SHIPPED binary
  (not a source build);
  (ii) `reposix init` exits NON-ZERO when the backend is unreachable — no silent exit-0
  masking;
  (iii) the release-path sim→cargo fallback that hides the gap is removed;
  (iv) verified by a fresh zero-shot human-simulation agent (D3).
  **Bias:** ship `reposix-sim` in the release matrix (OP-1 makes it canonical), but
  honest de-advertisement is an acceptable convergence if shipping sim is disproportionate.
- **D2 — v0.14.0 orchestration-hardening: reject-`t@t`-identity commit hook + real
  worktree-isolation enforcement are P0.** The corruption recurred twice; the doc rule
  alone won't stop it. Sketch: a pre-commit/pre-push hook that hard-rejects any commit
  authored by `t<t@t>` (or any non-allowlisted identity), plus per-leaf isolated `/tmp`
  clones and unique `REPOSIX_CACHE_DIR` enforcement. Anchor: intake **S-260707-pr-08**
  (HIGH).
- **D3 — Zero-shot human-simulation testing becomes a STANDING milestone-close gate**
  (new agent-ux catalog row), not a one-off. Every milestone-close dispatches N fresh,
  context-free agents (no system prompt, no repo context) that install the shipped
  artifact THE WAY THE DOCS SAY and attempt the documented workflows (read path:
  init/attach → clone → grep/cat; write path: edit → commit → push; recovery:
  conflict-rebase, blob-limit sparse-checkout). Any doc-lie or broken path = RED. This is
  exactly what caught the sim-onboarding break; institutionalize it.

## 3. v0.13.1 runbook — end state (definition of done)

Enter via GSD (new-milestone or a scoped hotfix milestone); discuss → plan → execute the
three acceptance items in D1. The milestone is DONE only when:

1. A fresh zero-shot human-sim agent completes the documented getting-started flow on
   the shipped/installed binary with **zero manual fixups**.
2. `reposix init` returns **non-zero** on an unreachable backend.
3. The binstall/post-release gates are **green** on the v0.13.1 tag.

Then tag `v0.13.1` (L0/owner-gated, same runbook shape as v0.13.0: PR merge → crates.io
publish via `release-plz.yml` on merge-to-main → tag → `release.yml`).

**Release-plz operational gotchas from last session (still apply):**
- A bot-authored (`GITHUB_TOKEN`) release-plz push leaves `pull_request`-triggered
  workflows at `action_required` — a real-actor close/reopen of the PR unblocks it.
- release-plz **regenerates the PR on every main push** — expect the PR number to move.
- crates.io publish fires on **MERGE-to-main** (`release-plz.yml`), NOT on the tag
  (`release.yml` only builds binaries + the GitHub release).

## 4. Standing practice — zero-shot human-simulation testing (D3 detail)

**How to run it:** dispatch fresh general-purpose agents with minimal context (no system
prompt, no repo history) that follow ONLY the published docs — install the shipped
artifact, run the documented getting-started flow against the sim backend (OP-1 default).
Do not seed them with any repo knowledge beyond the install instructions.

**What it found this session:** the sim-onboarding break (§1/D1) — 3 independent
zero-shot reproductions confirmed `reposix-sim` is absent from every prebuilt
distribution and `reposix init` silently exits 0 on the resulting failure.

**Institutionalize, don't ad-hoc:** this must become a catalog-backed milestone-close
gate (D3), not a one-off session activity — file the new agent-ux catalog row as part of
v0.13.1 or v0.14.0 scoping.

**Tool-harness gotcha:** several subagent lanes hit a recurring `"No tools needed for
summary"` error (the agent executes zero tools and returns early). Retry once with a
trivial `echo ok` health-check dispatch; if it recurs, escalate rather than burning a
whole fleet retrying the same broken lane.

## 5. Live deferred backlog

Registry: `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (open rows
`S-260707-pr-01`..`08` + `S-260707-rbf-01`). Notable:

- **pr-07** — p94-badges is a GENUINE red (low-impact bookkeeping), not brittle-gate
  noise; distinguish from the badges brittleness in §6.
- **pr-08** — worktree-isolation, HIGH (feeds D2).
- A **test-isolation MEDIUM**: integration tests share one on-disk cache + sqlite DB
  across parallel threads AND across the gate's two test binaries — an OP-4 violation;
  fix is a unique `REPOSIX_CACHE_DIR` per test.
- **rbf-01** — the crlf flake, OPEN as a monitor. Next experiment: a throwaway
  `debug/crlf-capture` branch that loops the CRLF push ~60x with an unconditional
  `eprintln` of the captured body, to confirm the wiremock-truncation hypothesis.

**SURPRISES-INTAKE.md itself needs a distill/split pass at v0.14.0 scoping** — it is
~84k chars (4x the 20k soft limit); re-triage alone won't fix the size. It ALSO contains
a doc-lie in its own "Entry format" template (the `## YYYY-MM-DD HH:MM | …` schema
documented there matches NO live row — every live row actually uses `## S-<id> — title
(SEV)`); fix the template in the same cleanup pass so it stops misleading appenders.

**5 carried-forward HIGHs for v0.14.0 scoping:** RUSTSEC memmap2 + quinn-proto
(advisories still present in `Cargo.lock`), `prune_oid_map` pagination-truncation,
RBF-FW-11 date-cutoff, quality-convergence write-contention.

**Queued behind v0.13.1:** the v0.14.0 RBF-LR-03 reconciliation pivot (headline
milestone — explore → prototype-on-real-backend → converge; gate the real-backend spend
before prototyping), the OD-4 launch-readiness milestone, and workstream B (v0.13.2,
P98+).

## 6. Known brittle gates + hazards

- **p94-badges** — genuine red this time (`S-260707-pr-07`), do not dismiss as brittle
  without checking.
- **doc-alignment walker** re-drifts `last_walked` on every pre-push — benign; recovery
  is `/reposix-quality-refresh docs/index.md`.
- **"No tools needed for summary"** harness flake (§4) — retry once, then escalate.
- **Worktree-corruption hazard** — check `git config user.email` before every commit
  this session (see §1).

**Waiver clocks:** `structure/file-size-limits` WAIVED until **2026-08-08** (renew
before); docs-repro rows WAIVED until **2026-09-15**.

## 7. Doctrine

Full delegation / relief / cadence / durable-state doctrine:
`.planning/ORCHESTRATION.md` — relief at ~100k own-context (hard stop ~150k), a
coordinator-of-coordinators per milestone, one-cargo-invocation machine-wide, and the
Leaf Isolation HARD-STOP (added this session).

**This session's meta-lesson:** L0 must delegate leaf/fleet work and reserve its own
context for decisions + integration + decision-critical short reads. Two costly failure
modes are now documented so the next session doesn't rediscover them from scratch: the
"No tools needed for summary" harness flake (§4) and the shared-worktree identity
corruption (§1).

---

History lives in git — `git log` / `git show`, not restated here.
