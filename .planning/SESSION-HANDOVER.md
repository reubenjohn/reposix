# SESSION-HANDOVER.md — v0.14.0 TAG still BLOCKED; B1+B4+B5 CLOSED this session — 2026-07-13

For the incoming top-level workhorse (L0). Map, not territory — detail lives in git + linked
files. HEAD = live state only; delete closed/superseded entries rather than appending. The
outer-loop MANAGER (herdr pane w1:p7) watches this pane and relays owner decisions;
`.planning/MANAGER-HANDOVER.md` is the live owner-directive channel — read it.

## 1. Current state (confirm with `git log --oneline -8` + `git rev-parse origin/main`)
- v0.14.0 tag remains MANAGER-delegated and BLOCKED until the `pre-release-real-backend`
  cadence exits 0 honestly + an unbiased ratification passes. NO tag push ever by the workhorse.
- This session shipped B1, B4, B5. B2, B3, and item-4 (tag sequence) REMAIN.
- Working tree should be clean and pushed; `ci.yml` on main should be green (verify:
  `gh run list --branch main -L 3`). Never open work over a red main.

## 2. CLOSED THIS SESSION
- **B4+B5 (DONE, commits b635c3b, 8c48fc5, 1467eb2, fe8febb + docs finalize commit).** Both
  missing verifier scripts authored + git-tracked: `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`
  (+ `lib/t4-real-backend-flow.sh`) and `quality/gates/agent-ux/github-front-door-real-backend.sh`.
  Hermetic self-test `quality/gates/agent-ux/real-backend-env-gate.selftest.sh` (4/4 pass) +
  kcov harness `quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh` (coverage
  15.72% ≥ floor 13, floor untouched). Both catalog rows now grade NOT-VERIFIED/env-missing
  (no longer "verifier not found"). B5 fix-twice: RFC3339 transcript assert corrected in
  agent-ux.json (still valid JSON, 66 rows). NOTE: these rows still read NOT-VERIFIED until
  item 4 runs them WITH creds — they are now GRADEABLE, not yet GRADED.
  - **Noticing (file if not done): 3 OTHER catalog rows still carry the stale `-<RFC3339>`
    transcript-filename pattern** — `kind-shell-subprocess-worked-example`, `attach-sync-real-backend`,
    `rebase-recovery-reconciles`. Left untouched per scope fence; candidate GTH.
- **B1 (DONE — manager Branch-2 decision, recorded in CONSULT-DECISIONS 2026-07-13).**
  Diagnosis: page 2818063 legitimately deleted out-of-band; Confluence SoT authoritative,
  GitHub mirror stale (NOT a reposix bug; mass-delete guard worked as designed). Manager chose
  restore. Executed: `PUT /wiki/api/v2/pages/2818063` status→current → HTTP 200. Space 360450
  current pages now = {2818063, 7766017, 7798785} (known-good 3-page), API-verified. Only 2818063
  touched. **Follow-up: the reposix-side vision-litmus PASS (matched=3/backend_deleted=0) is NOT
  yet re-confirmed** — successor runs the vision-litmus first thing (see item 4). Self-healing-
  fixture GTH-V15-09 filed.

## 3. REMAINING (successor, in order)
1. **Confirm B1 end-to-end (fast):** run `bash quality/gates/agent-ux/milestone-close-vision-litmus.sh`
   with real creds (source `.env`; preflight passes 3/3, backends reachable) — expect exit 0,
   matched=3/backend_deleted=0. Sanctioned mutating litmus on TokenWorld. Re-persists a fresh artifact.
2. **B2 — p93-partial-failure-recovery-real-confluence (P0). Diagnosed, NOT a regression.** The
   verifier hard-pins TokenWorld (it MUTATES), but the owner's `.env` sets
   `REPOSIX_CONFLUENCE_SPACE=REPOSIX`, so the sanctioned-space guard rejects → exit 1. HARNESS FIX
   (small /gsd-quick): make `quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh`
   FORCE `REPOSIX_CONFLUENCE_SPACE=TokenWorld` internally for this pinned row (instead of rejecting
   when it sees REPOSIX), so the probe self-targets correctly regardless of ambient .env. Then
   re-run WITH creds + re-persist the fresh artifact (current on-disk artifact is a stale 2026-07-06
   env-missing/space-guard hybrid).
3. **B3 — attach-sync-real-backend (P1). Stale artifact; likely fixed by B1.** Its FAIL was
   "either downstream of the same stale-mirror drift as B1 or independent." B1 is now fixed, so
   re-run WITH creds (space guard tolerates REPOSIX, so no space issue) + re-persist. Root-cause
   against the fresh transcript if it still fails.
4. **Item 4 — tag sequence (only after 1-3 green):** `python3 quality/runners/run.py --cadence
   pre-release-real-backend --persist` WITH creds → need honest exit 0 (all 6 rows PASS/NOT a P0/P1
   FAIL). Then re-mint `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` (was RED at 563095f)
   + unbiased verifier ratification, author the tag script (pattern
   `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh`), STOP at READY-TO-TAG. Manager pushes the tag.

## 4. v0.13.0 pre-tag actions (item 6) — untouched; queued behind the v0.14.0 tag.

## 5. Constraints (unchanged)
sim-first for code; real backends only via REPOSIX_ALLOWED_ORIGINS; sanctioned mutation targets
ONLY (TokenWorld / reubenjohn-reposix issues / JIRA TEST); NO tag push ever; never over a red main;
ONE cargo invocation machine-wide; leaf test setup in /tmp clones (cd in SAME bash call); relief
~100k soft / ~150k hard absolute → replace THIS file, commit+push, end turn. To resume an agent:
SendMessage it, never fork.

## 6. Serialization
Single tree-writer at a time (ORCHESTRATION §2). Read-only recon may fan out in parallel; the B1
read-only diagnosis ran parallel to the B4/B5 writer safely this session. Watch context budget
closely — the predecessor blew past 150k mid-lane (large subagent results are heavy).

---
History lives in git — `git log` / `git show`, not restated here.
