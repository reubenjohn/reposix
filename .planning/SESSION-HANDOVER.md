# SESSION-HANDOVER.md — v0.13.0 release runbook, owner-delegated to L0 — 2026-07-06

For the incoming top-level orchestrator (L0). This is the map, not the territory — detail
lives in git and the linked files. HEAD = live state only; history is in `git log`.

## 0. Owner calibration — READ FIRST (over-ask LESS)

The owner wants **decide-and-record, not gating questions.** Pick the path the owner's
model implies, log it to `.planning/CONSULT-DECISIONS.md` with reasoning, and proceed —
the owner vetoes if you misread. Reserve owner STOPs for the genuinely-owner class only:
**irreversible/destructive moves, external-backend mutations, and credential/spend
authorization** (E1/E3) — e.g. never cut the actual tag or fire a real-backend call
without the owner. When you would ask, prefer surfacing a **reversible default to veto**
over a blocking question.

**Owner design taste** (use to make calls autonomously): backend owns identity, client
works in **slugs** (client-side ID remapping is a smell); model multi-step client↔server
interactions as **git-native commit sequences that self-reconcile on partial fail**; big
design questions are **pivots to explore/prototype/converge**, not point-patches; **ship
honest milestones and document known limitations out loud** rather than suppress gates or
hold a green milestone hostage; **guard context aggressively** (fork, prune, lean on git,
least-complex path).

- **No doc carries an unbounded-growth policy** (ratified this session): bound every doc to
  **live state**; git history is the only archive. Delete closed/superseded entries rather
  than appending or relocating them to a child file (a child file just relocates the
  growth). Applied to `CONSULT-DECISIONS.md` this session (now holds open decisions only).
  Exempt: code-enforced `audit_events` tables (operational forensic data, not docs).

### Calibration examples (from this session — the decide-vs-ask boundary)

| Situation | Right call | Why |
|---|---|---|
| Tag timing (ship v0.13.0 now vs hold for the pivot), owner said "leave it to you" | DECIDE (chose T1, ship now), record to ledger, proceed | Owner delegated + reasoning was available. Asking was over-asking — should've surfaced T1 as a reversible default to veto, not a gating question. |
| Reconciliation blocker: which fix mechanism | ASK (correctly) — but frame as a proposal | Architecture-shaping (E2); owner turned it into a design pivot no agent would've invented. Genuine owner input. Still: lead with a recommendation, let owner redirect. |
| Authorize a real-Confluence probe (credentials + real-backend call) | STOP for owner | Credential/spend + external mutation (E1/E3) — never self-authorize, even when confident. |
| "9th probe says NOT-VERIFIED but owner recalls it passing" | INVESTIGATE, don't ask | Not a decision — a fact to establish from committed evidence. Go find the crux (stale status vs real gap); only surface if evidence is genuinely absent. |

Throughline: **default to decide-and-record; escalate only irreversible / external /
credential / spend; and "not a decision, go verify" is not an escalation.**

## 1. Current state

- **v0.13.0 autonomously GREEN** — P78–P97, 20/20 phases shipped; milestone verdict at
  `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`.
- **Pre-tag checklist items 1–3 are DONE and pushed.** `HEAD == origin/main == aff5233`
  (verified live this session via `git log --oneline -8` / `git rev-parse`). Working tree
  is clean (`git status` — nothing to commit). **No `v0.13.0` tag exists**
  (`git tag -l 'v0.13*'` empty — confirmed live). Recent commits:
  - `aff5233` chore(planning): release runbook live status — PR #61 regen clean, CI re-triggered
  - `5ade713` docs(handover): L0 relief — v0.13.0 release runbook, owner-delegated
  - `f686ab2` chore(planning): owner delegated the release decision to L0 — release runbook
  - `13c922f` chore(planning): STATE cursor — pre-tag doc/planning items cleared
  - `56307be` docs: v0.13.0 intake OP-8 disposition + bound-to-live-state sweep
- **The owner DELEGATED the full release decision to L0** (2026-07-06): PR #61 merge, the
  **crates.io publish (IRREVERSIBLE — published versions can only be yanked, not undone)**,
  and cutting the **v0.13.0 tag**. This extends the OD-3 tag-push delegation to the
  publish spend.
- **PR #61 regenerated clean** (live, checked this session): `state: OPEN`,
  `mergeable: MERGEABLE`, `headRefOid: 2d1f55f6`. Diff is uniform `0.12.0→0.13.0` churn-only
  (per-crate version bumps + accurate CHANGELOGs, no stray source), steward-reviewed.
  The earlier **bot-push CI-gap** (a GITHUB_TOKEN-authored release-plz push leaves
  `pull_request`-triggered workflows at `action_required` instead of running them) was
  **resolved by an L0 close/reopen** of PR #61 as a real actor — this is structural to
  every release-plz regen here; close/reopen (or an equivalent real-actor push) is the
  standard unblock.
- **CI run `28819166220` is COMPLETE, NOT all-green** (`gh pr checks 61`, observed live
  this session): `test`, `clippy`, `rustfmt`, `shell-coverage`, `cargo-audit`,
  `gitleaks`, `quality gates (pre-pr)`, `coverage`, `bench-latency-v09`, `CodeQL`, and all
  three `*-v09` real-backend integration jobs — **PASS**. Two checks **FAIL**:
  - **`integration (contract, real confluence)`** — `contract_confluence_live` and
    `contract_confluence_live_hierarchy` panic: durable TokenWorld fixture `7798785` lost
    its expected `parent_id == Some(7766017)` (`left: None`). This is **live-backend fixture
    drift in Confluence TokenWorld**, not a code regression from this PR (the `-v09` sibling
    job against the same backend passes). Needs investigation/repair of the fixture state
    (or the assertion) before this check can go green — **not something to route around**.
  - **`codecov/project`** — FAIL (coverage-ratio check on the PR; not yet triaged this
    session).
  - This supersedes STATE.md's `2026-07-06` snapshot of run `28819166220` as "all pending" —
    that was true when STATE.md was last written; the run has since finished with these two
    reds. STATE.md § Workstream A is still the durable pointer for the runbook shape, but
    its per-check status line is now stale versus this live read.
- **Net: current GO/NO-GO read is NO-GO** on the release runbook (§3) until the two failing
  checks are resolved (fixture repair for the Confluence contract test; triage for
  `codecov/project`) and a fresh `gh pr checks 61` shows all green.

## 2. Owner decisions — SETTLED

- **Tag timing → T1 (ship now).** v0.13.0 tags now; **RBF-LR-03 ships as an
  honestly-WAIVED, documented known-limitation** (narrow: real backend + mid-batch-create
  network drop → one hand-deletable duplicate); the reconciliation redesign becomes the
  **v0.14.0 headline milestone.** No gate suppression — the waiver is honest + owner-signed.
  Ledger: `CONSULT-DECISIONS.md` § "Tag-timing: T1".
- **RBF-LR-03 → v0.14.0 pivot (directional inspiration, NOT a spec).** Owner directed a
  **coordinator-of-coordinators** effort: explore candidate mechanisms → prototype top few
  **against a real backend** → stress-test survivors on **all 3 backends** with injected
  mid-sequence failures → converge → clean debt-free implementation (accepting large
  refactors + docs + quality/CI changes). Owner's slug/symlink/commit-sequence vision is
  *inspiration for the direction*; the exploration **owns the outcome** and may converge on
  a different mechanism. **~Milestone-sized; gate the spend before the prototype phase**
  (real-backend calls cost). Vision + directive: `CONSULT-DECISIONS.md` § "RBF-LR-03 pivot"
  (131315c + amendment). ADR-010 §3 is revised only AFTER convergence.
- **Release decision delegated to L0 (2026-07-06).** PR #61 merge, crates.io publish
  (irreversible), and the v0.13.0 tag cut are all L0-owned now — not owner-blocking. See
  §3 for the runbook.

## 3. Release runbook (L0-owned; authority already delegated — execute the DECISION,
   dispatch the mechanics)

Full durable copy: `.planning/STATE.md` § Workstream A "**RELEASE RUNBOOK (L0-owned
tail) — LIVE STATUS**" — this section is a map pointing at that, not a duplicate to
maintain.

1. **Fix the GO blocker first.** CI run `28819166220` currently has two reds (§1):
   the real-Confluence contract-test fixture drift and `codecov/project`. Dispatch a
   subagent to investigate/repair the TokenWorld fixture (or confirm+fix the assertion)
   and triage the codecov failure. Do NOT merge or publish while either is red.
2. **Re-check PR #61 live** — `gh pr view 61 --json state,mergeable,files,statusCheckRollup`
   and `gh pr checks 61` (read the steward's regen review on the PR itself, not from
   session memory) after the fix lands and a fresh run completes.
3. **GO criteria (all must hold):** the regenerated diff is release-churn-only (per-crate
   version bumps + CHANGELOG entries, no stray source/logic changes), version bumps are
   sane for the shipped v0.13.0 work, and CI is **fully green** (no reds, no pending).
4. **If GO:** merge PR #61 → **crates.io publish (IRREVERSIBLE — verify each crate
   actually published before proceeding)** → cut the **v0.13.0 tag** (the tag script
   `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` stays disabled — do NOT
   run it; canonical release is `.github/workflows/release.yml` on tag `v*`) → push the
   tag → `gh run watch` the release workflow to green.
5. **If NO-GO:** loop back / fix the regenerated PR; do NOT publish.
6. **Non-blocking tail after the tag lands:** the 6 env-gated real-backend rows (accept
   via creds or leave honestly NOT-VERIFIED — see §4, this is not a gap); renew the
   `structure/file-size-limits` waiver before 2026-08-08 (§6-adjacent, see STATE.md);
   then scope the v0.14.0 pivot (§2) + the launch-readiness milestone (OD-4) + resume
   workstream B (P98+, `.planning/milestones/v0.13.2-phases/`).

**DELEGATION DISCIPLINE FOR THE NEXT L0 (lesson from this session):** the executing L0
must NOT read workflow specs itself (e.g. invoking `/gsd-quick` directly, walking
`quality/PROTOCOL.md` line-by-line) or run mechanical git/gh commands in its own context.
Dispatch a steward/executor subagent for the merge/publish/tag mechanics and the fixture
investigation, and keep the L0's own context budget for the release *decision* (GO/NO-GO
read, risk calls) and cross-cutting integration. The prior L0 ran low on context precisely
because it did this mechanical work itself instead of delegating it — do not repeat that.

## 4. Real-backend 9th probe — VERIFIED (owner was right)

The real-Confluence probe **genuinely ran green.** The committed catalog row
`agent-ux/milestone-close-vision-litmus-real-backend` (`quality/catalogs/agent-ux.json`)
carries `last_real_grade: "PASS"`, and a fresh ephemeral PASS transcript exists at
`quality/reports/…/…-2026-07-06T06-28-00Z.*` (real Confluence page 2818063 round-trip).
The mechanical `status: NOT-VERIFIED` is **honest-by-design**: this P0 row has NO waiver
and fails-closed to NOT-VERIFIED when re-graded in a shell without creds (env-gate, exit
75), preserving `last_real_grade`. **NOT-VERIFIED ≠ never-passed.** No new real-backend
call is required to tag; treat the probe as satisfied via the committed `last_real_grade`.

**Real-backend creds (for reference / the v0.14.0 pivot):** a local **`.env` at the repo
root** (present + ready; gitignored). Confluence needs `ATLASSIAN_API_KEY`,
`ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, and `REPOSIX_ALLOWED_ORIGINS` **must
include** `https://reuben-john.atlassian.net` (allowlist is fail-closed). No auto-dotenv
loader — `set -a; source .env; set +a` first. CI uses GitHub Actions secrets separately
(already provisioned; `gh secret list`). Sanctioned mutable target: Confluence TokenWorld
(spare fixture pages `7766017` / `7798785`).

## 5. Live deferred backlog

The pruned intakes ARE the live registry (open-only; resolved items are DELETED — git is
the archive):

- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` (incl. doctrine follow-ups + a new
  **next-session sweep**: audit the other append-only docs — `SURPRISES-INTAKE.md`,
  `GOOD-TO-HAVES.md`, `RETROSPECTIVE.md` — against the **bound-to-live-state** rule (§0);
  delete closed/resolved entries, git retains them. `CONSULT-DECISIONS.md` already done.)

**Git-only relocated items** (deleted with the SURPRISES archive during the prune, NOT
proven resolved; full text: `git show 3109fbb:.planning/milestones/v0.13.0-phases/
SURPRISES-INTAKE-ARCHIVE-P89-P97.md`): **P84-01-T05** (HIGH) · **P89 cross-AI Claude-leg** ·
**steward-window** (MEDIUM) · **quality-convergence** (HIGH) · **P91 T2-REOPEN** (MEDIUM) ·
**Entry-27** walker forward pre-audit (post-v0.14.0 gate).

## 6. Known brittle gate — badges (p94 + doc-alignment)

Two related brittle-badge misfires; fix by asserting the **invariant**, not the surface:

- `quality/gates/docs-build/p94-badges-real-vs-transient.sh:78` greps `GOOD-TO-HAVES.md`
  for an h2 heading the OP-8 drain relocated → false pre-push FAIL.
- The **doc-alignment walker** re-flags the 2 `docs/index.md` badge rows
  `BOUND → STALE_DOCS_DRIFT` on re-walk despite `badges-resolve.py 8/8 PASS` (hash
  re-extraction drift, not a broken badge). Seen + reverted this session; **the next
  session's push may hit STALE_DOCS_DRIFT** — recovery is `/reposix-quality-refresh
  docs/index.md`. A C2 / brittle-gate target. (Working tree is clean this session — no
  live `doc-alignment.json` drift to carry forward, but the failure mode recurs.)

## 7. Doctrine

C2 / relief-threshold doctrine is in `.planning/ORCHESTRATION.md` (pointer only; do not
restate or edit here). t4-real (real-backend T4 litmus) remains unimplemented — Option B
in `.planning/milestones/v0.13.0-phases/97-HANDOVER.md` (~1.5h `#[ignore]` Rust smoke);
opt-in, not tag-blocking.

---

History lives in git — `git log` / `git show`, not restated here.
