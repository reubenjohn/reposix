# SESSION-HANDOVER.md — v0.13.0 tag-ready (owner decisions settled), 2026-07-06

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
- **Tag is READY** — the two owner gates that halted it are settled (§2). Remaining work is
  a checklist the orchestrator drives, then the owner (not the coordinator) cuts the tag.
- **Git:** local `main` was in sync with `origin/main` at session start; this session's
  ledger + handover commits sit on top — **push `main` early** before new work (a phase
  grades RED for shipping without it). Use `git log --oneline` / `git rev-parse` for live
  SHAs; do not trust pinned SHAs in prose.

## 2. Owner decisions — SETTLED this session

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

## 3. Pre-tag checklist (orchestrator drives; owner cuts the tag)

1. **Document RBF-LR-03 as a known limitation** — ADR-010 §3 (mark WAIVED-for-v0.13.0,
   pointer to the v0.14.0 pivot) + a user-facing known-limitations note ("on a real backend,
   a create interrupted mid-batch may leave a duplicate on retry; check before re-pushing").
   This is what makes T1 honest rather than suppression.
2. **dvcs-cold-reader doc review** — `/doc-clarity-review` on the DVCS-facing pages.
3. **Disposition the open SURPRISES / GOOD-TO-HAVES backlog** (§5) — eager-fix <1h items,
   file the rest; nothing silently skipped.
4. **Regenerate PR #61** (release-plz; held to P97 — regenerate from the current release-plz
   baseline, review, owner-gated crates.io publish).
5. **Owner cuts the v0.13.0 tag** — canonical multi-platform release is `.github/
   workflows/release.yml` (tag `v*`); tag-cut is an owner action (E1).

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
  docs/index.md`. A C2 / brittle-gate target.

## 7. Doctrine

C2 / relief-threshold doctrine is in `.planning/ORCHESTRATION.md` (pointer only; do not
restate or edit here). t4-real (real-backend T4 litmus) remains unimplemented — Option B
in `.planning/milestones/v0.13.0-phases/97-HANDOVER.md` (~1.5h `#[ignore]` Rust smoke);
opt-in, not tag-blocking.

---

History lives in git — `git log` / `git show`, not restated here.
