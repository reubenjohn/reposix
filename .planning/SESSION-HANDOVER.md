# SESSION-HANDOVER.md — items 0–1 CLOSED green @ `6dc47a3`; successor #18 resumes items 2–5 — 2026-07-13

Written by **workhorse successor #17** (L0 orchestrator, pane w1:p5, herded by the
manager in w1:p7). Relief reason: past ~100k own-context tokens at a clean wave
boundary (items 0–1 closed green, main green, tree clean). This **REPLACES** (does not
append to) the prior `SESSION-HANDOVER.md` (successor #17's own charter file, written
by successor #16 naming "successor #17 resumes post-tag queue items 0–5" — that charter
is now discharged for items 0–1; this file re-issues it for items 2–5 under successor
#18).

**Read order:** this file → §1 (verify live) → §6 (runbook) → dip into §2/§4/§5 as
needed. **Guardrails:** do NOT touch `.planning/MANAGER-HANDOVER.md` — that is the
MANAGER's own handover file (pane w1:p7), a separate document with a separate owner;
this file governs only the L0 orchestrator seat. No tag push by any coordinator — the
manager cuts tags, never L0.

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main
```
**Verified this session (2026-07-13, re-checked independently of the outgoing
coordinator's own numbers):** HEAD = `6dc47a3`, tree **clean** (`git status --porcelain`
empty), `0 0` vs `origin/main` — i.e. `6dc47a3` is fully pushed, nothing local ahead or
behind.

**CI verified GREEN on `6dc47a3` independently this session:**
- `gh run view 29304998456` → `workflowName: "CI"`, `headSha:
  6dc47a34b827fe60f121fe9e231c84dceeb3f59b` (matches `6dc47a3`), `status: completed`,
  `conclusion: success`.
- Post-push cadence P0 probe confirmed via its persisted artifact (not re-derived):
  `quality/reports/verifications/code/ci-green-on-main.json` — `ts:
  2026-07-14T04:09:07Z`, `exit_code: 0`, `asserts_passed: ["latest ci.yml run on main
  concluded success"]`. This is the freshest file under `quality/reports/verifications/`
  as of this handover.
- `gh run list --branch main --limit 3` also shows Docs / release-plz / CodeQL all
  `success` on the same sha (side confirmation, not the P0 probe itself).

Commit chain this session, oldest → newest (baseline: prior handover's HEAD `ffb9d25`):
- `ff7be56` — item 0, GSD cursor refresh + fold of 4 v0.15.0 noticings. Quick lane
  `.planning/quick/260713-c0r-cursor-refresh/` (`260713-c0r-PLAN.md` +
  `260713-c0r-SUMMARY.md`, both present on disk, confirmed).
- `69b9da7` — **NOT L0's work.** This is the MANAGER's own rotation-#6 handover refresh
  (`Co-Authored-By: Claude Fable 5`), touching only `.planning/MANAGER-HANDOVER.md` (42
  lines changed, confirmed via `git show --stat`) — it corrected the stale
  v0.13.0-tag queue-item description on the manager's own document. Interleaved in the
  chain because both seats commit to the same branch; do not attribute it to L0 and do
  not re-touch `MANAGER-HANDOVER.md` from this seat.
- `370310d` — item 1, `release.yml` `make_latest` fix (the substantive code change).
- `a5081a1` — item 1 riders: STATE.md Workstream C de-stale + PROJECT.md truth banner.
- `6dc47a3` (HEAD) — item 1 quick-lane SUMMARY commit. Quick lane
  `.planning/quick/260713-mlh-make-latest-hardening/` (`260713-mlh-PLAN.md`,
  `260713-mlh-SUMMARY.md`, `make-latest-proof.md`, all present on disk, confirmed).

**Numbered facts the successor MUST know:**
1. **v0.14.0 is SHIPPED + public + Latest** (crates.io 0.14.0; GH release marked
   "Latest" 2026-07-14) — confirmed via `PROJECT.md`'s own banner (line 3, see §5) and
   `STATE.md` workstream_c block (`status: shipped-green`).
2. **v0.13.0 (2026-07-07) and v0.13.1 (2026-07-08) are ALREADY tagged+released** —
   confirmed via `STATE.md` workstream_a block (`blocks_tag: false`, `next_phase: P98
   # v0.13.0 SHIPPED — tagged/released 2026-07-07 (commit 3423b18)... v0.13.1 hotfix
   shipped 2026-07-08 (04640d5)`). **Do not re-run a v0.13.0 tag-prep sequence.**
3. **No v0.13.2 tag exists; `workstream_b` has zero code** — confirmed via `STATE.md`
   workstream_b block: `status: queued`, `phases_completed: 0`, `next_phase: P98`
   (placeholder), `blocks_tag: false`. It is queued behind workstream C and the
   not-yet-scoped launch-readiness milestone per OD-4 — nothing to do here now.
4. **`release.yml`'s `make_latest` fix is live in the tree** (commit `370310d`) —
   confirmed by reading the diff directly: it now computes the highest published
   app-release semver via `gh api .../releases --paginate` + `sort -V`, and passes
   `--latest=true|false` explicitly on both the `gh release create` and `gh release
   edit` paths, with a fix-it-twice comment at the site pointing back to
   `make-latest-proof.md`. This is a **preventive fix, not a tag cut** — no tag was
   pushed this session.

## 2. Wave/cycle state

| Item | Artifact | State | Commit(s) |
|---|---|---|---|
| 0 — GSD cursor refresh + intake fold | quick `260713-c0r-cursor-refresh/` | DONE, pushed | `ff7be56` |
| (manager's own rotation-#6 handover refresh — NOT an L0 queue item, listed for chain context only) | `.planning/MANAGER-HANDOVER.md` | N/A (manager-owned) | `69b9da7` |
| 1 — `make_latest` preventive hardening (RE-SCOPED; no tag cut) | quick `260713-mlh-make-latest-hardening/` (incl. `make-latest-proof.md`) | DONE, pushed, CI-verified green | `370310d`, `a5081a1`, `6dc47a3` |
| 2 — Q1c interim hero qualifiers | README + `docs/index.md:17` | **NOT STARTED** | — |
| 3 — `.playwright-mcp/audit-03..08*` droppings sweep | filesystem housekeeping (gitignored, not GSD-tracked) | **NOT STARTED** | — |
| 4 — `/gsd-cleanup` archival cascade | — | **NOT STARTED** | — |
| 5 — ORCHESTRATION.md size split + CONSULT-DECISIONS.md archival rider | — | **NOT STARTED** — philosophy conflict to reconcile first, see §6 | — |

**Named-incident post-mortem to read before touching `release.yml` again:**
`.planning/quick/260713-mlh-make-latest-hardening/make-latest-proof.md` — the
`make_latest` hazard was PROVEN, not assumed: `gh` (v2.62.0) omits `make_latest` from
the create/edit POST unless `--latest` is explicitly passed, and GitHub's REST
omit-default is `make_latest="true"` (set THIS release as Latest), **not** the
date/version-aware `legacy` that `gh`'s own help text implies. The proof was executed
against the sanctioned `reubenjohn/reposix` repo using drafts/scratch releases only —
the live v0.14.0 "Latest" marker was never touched, and residue was cleaned to 0 (one
side-effect residue is tracked separately in §5, not a code residue).

## 3. Binding constraints (unchanged)

- Reality-check arc is **NOT owner-ratified** — no defect-fixing lanes beyond
  tag-blockers; OPEN intakes (v0.15.0 or otherwise) route forward, do NOT drain them now.
- **ONE cargo invocation machine-wide** (prefer `-p <crate>`). Leaf isolation: `/tmp`
  clones, `cd` in the SAME Bash invocation, never the shared tree.
- **Uncommitted = didn't happen.** Push per queue-item cadence → then
  `python3 quality/runners/run.py --cadence post-push --persist` → confirm
  `code/ci-green-on-main` (P0) green → **never proceed over a red main.**
- You **route, don't work**: delegate opus (complex/security), sonnet (default), haiku
  (mechanical); never fable at a leaf. Report to the manager (w1:p7) at each queue-item
  boundary or when blocked. Relieve past ~100k own-context tokens (hard stop ~150k) at a
  wave boundary — write+commit a handover first.
- **No `--no-verify`. No tag push by any coordinator** — the MANAGER cuts tags, never
  the coordinator, even at READY-TO-TAG.
- **zsh gotcha (bit this session's outgoing coordinator):** `read -r status ...` FAILS
  in poll loops — `status` is a read-only var in zsh. Use a different variable name.

## 4. Litmus / gate / REOPEN state

- `ci.yml` run `29304998456` (push, sha `6dc47a34b827fe60f121fe9e231c84dceeb3f59b` =
  `6dc47a3`) — **SUCCESS**, re-verified independently this session via `gh run view`.
- Post-push cadence P0 probe `code/ci-green-on-main` — **PASS, exit_code 0**, persisted
  artifact `quality/reports/verifications/code/ci-green-on-main.json`,
  `ts: 2026-07-14T04:09:07Z` (freshest file under `quality/reports/verifications/` as of
  this handover — re-run `python3 quality/runners/run.py --cadence post-push --persist`
  yourself before trusting a stale timestamp).
- `make_latest` hazard proof — see §2 pointer, `make-latest-proof.md`, fully committed
  (`6dc47a3`), no open verification gap.
- No open REOPEN-gate clock. No P0 row carries a waiver from this session's work.

## 5. Mid-execution decisions + noticed-not-filed

**De-facto decisions made live this session:**
- **Item 1 re-scope (manager-ratified).** The queue's original "item 1 = v0.13.0 tag
  sequence" was DROPPED — v0.13.0 and v0.13.1 were already shipped (§1 facts 2–3), so
  there was nothing to tag. Re-scoped in place to the `make_latest` preventive-hardening
  work that the tag-sequence investigation surfaced as a real hazard.
- **PROJECT.md wholesale reconcile DEFERRED, not done.** Per manager ruling, the
  wholesale re-anchor of `.planning/PROJECT.md` (structurally stale, 22.6k chars vs the
  20k progressive-disclosure guideline) is deferred to `/gsd-new-milestone`. This
  session's interim OP-8 fix was a one-line VERBATIM dated truth banner at the top:
  `> **Status (2026-07-14):** v0.14.0 shipped + Latest (2026-07-14); v0.13.0/v0.13.1
  already released; release.yml make_latest hardening in progress to protect future
  back-tags; wholesale re-anchor pending at /gsd-new-milestone.` — confirmed present at
  `PROJECT.md` line 3.

**Noticings to ROUTE, not fix (reality-check arc is NOT owner-ratified for defect
lanes — file/route only, do not act):**
1. **[OWNER ACTION — needs interactive auth, L0 cannot self-authorize] Archived scratch
   repo needs deletion.** Item 1's steal-demo probe created
   `reubenjohn/reposix-scope-test-DELETEME`. Confirmed still present this session via
   `gh repo view`: `isArchived: true`, `visibility: PRIVATE`. The token lacks
   `delete_repo` scope, so it was archived (not deleted) as the safe fallback. Recovery,
   for the owner or a session with elevated auth: `gh auth refresh -h github.com -s
   delete_repo && gh repo delete reubenjohn/reposix-scope-test-DELETEME --yes`. Surface
   to the owner/manager; this is an external mutation requiring named-target approval
   (ORCHESTRATION §9), already partially executed (archive) under the reversible
   fallback path.
2. **[v0.15.0 intake candidate, low severity] `gh release create --help`'s `--latest`
   flag description is misleading** — it reads "automatic by date and version" but `gh`
   never actually sends `make_latest=legacy`; this is very likely the root cause the
   original `release.yml` author trusted when omitting the flag. Worth a GOOD-TO-HAVE
   note (context for the next release.yml reader), not an action item.
3. **Non-blocking WARNs, pre-existing (not this session's regressions), do not act:**
   pre-push wall-time ~97s measured against a 60s budget note in `quality/CLAUDE.md`;
   `PROJECT.md` size vs the 20k guideline (see decision above — wholesale reconcile
   deferred to `/gsd-new-milestone`, the banner is the interim fix).
4. **Already-triaged (closed) — no further action:** `STATE.md` Workstream C body
   staleness → absorbed + fixed in item 1 (`a5081a1`, confirmed this session by reading
   the current block, which correctly reads `status: shipped-green` /
   `blocks_tag: false`).

**Confirmed landed this session (verification, not new noticing):** the 4 v0.15.0
noticings item 0 promised to route were independently re-verified present on disk —
sim-leak-on-SIGKILL is in
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (grep-confirmed at line 48);
`GTH-V15-10` (harness rc/exit_code mismatch + sim-readiness race) and `GTH-V15-11`
(`.sim-*.log` gitignore gap) are both in
`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (lines 70, 75). Note: these
intake files live under the **v0.15.0 milestone directory**, NOT at
`.planning/SURPRISES-INTAKE.md` / `.planning/GOOD-TO-HAVES.md` top level (only
`GOOD-TO-HAVES.md` has a top-level copy; `SURPRISES-INTAKE.md` does not — grep the
milestone dir, not the repo root, when routing future noticings).

## 6. Precise next steps (successor #18 runbook)

Manager herds from w1:p7; report at each numbered boundary or if blocked.

1. **Re-verify §1 ground truth live** before touching anything — do not trust this
   file's timestamps.
2. **Item 2 — Q1c interim hero qualifiers.** README "Three measured numbers" section +
   `docs/index.md:17` synthetic-baseline caveat. Run a cold-reader pass via
   `/doc-clarity-review` on the touched pages before calling it done (root CLAUDE.md §
   "Cold-reader pass on user-facing surfaces").
3. **Item 3 — `.playwright-mcp/audit-03..08*` droppings sweep.** 6 files confirmed
   present this session (gitignored, all dated 2026-07-12): `audit-03-landing-narrow-
   viewport.png`, `audit-04-confluence-reference.png`, `audit-05-first-run.png`,
   `audit-06-token-economy.png`, `audit-07-git-layer.png`, `audit-08-filesystem-
   narrow.png`. Note: `.playwright-mcp/` also holds many OTHER unrelated files (older
   console logs, page snapshots, hero screenshots dating back to April) — those are
   OUT OF SCOPE for this item; verify nothing depends on the named 6, then `rm` only
   those 6.
4. **Item 4 — `/gsd-cleanup` archival cascade.**
5. **Item 5 — ORCHESTRATION.md progressive-disclosure size split + CONSULT-DECISIONS.md
   archival rider.** `CONSULT-DECISIONS.md` confirmed this session at 53,778 chars / 400
   lines (vs the ~20k progressive-disclosure guideline). **RECONCILE THIS PHILOSOPHY
   CONFLICT FIRST, before executing:**
   - The `decision-procedures` skill (`.claude/skills/decision-procedures/SKILL.md`
     line 147) states the ledger is **"bounded, NOT append-only: the ledger holds only
     OPEN / live decisions — DELETE an entry on close / implementation / supersession;
     `git log` / `git show` is the archive (reversible)."**
   - The manager's rider (per the outgoing coordinator's report, not independently
     re-confirmed by this handover's author against a manager transcript) says
     move-older-entries-to-`.planning/archive/` instead.
   - These are two different archival philosophies (delete-relying-on-git-history vs.
     move-to-archive-file). **Confirm with the manager (w1:p7) which to apply before
     executing item 5** — do not default to either silently.
6. **Do not drain the reality-check-arc intakes** (v0.15.0 or otherwise) beyond
   tag-blockers — that arc is not owner-ratified for defect-fixing lanes yet.
7. **At each queue-item boundary, or if blocked, report to the manager (w1:p7)** — do
   not silently continue past a boundary without a checkpoint.
8. **Relieve past ~100k own-context tokens at the next clean wave boundary** — write and
   commit a fresh `.planning/SESSION-HANDOVER.md` (REPLACE, not append) naming successor
   #19, following this same §3 template.
