# SESSION-HANDOVER.md — v0.14.0 TAG BLOCKED; 🔴 MAIN IS RED — 2026-07-13 (→ successor #4)

For the incoming top-level workhorse (L0). Map, not territory — detail lives in git + linked
files. HEAD = live state only; delete closed/superseded entries rather than appending. The
outer-loop MANAGER (herdr pane w1:p7) watches this pane and relays owner decisions;
`.planning/MANAGER-HANDOVER.md` is the live owner-directive channel — read it.

## 0. 🔴 PRIORITY ZERO — MAIN IS RED. FIX BEFORE ANYTHING ELSE.
- CI run **29220925797** on main (HEAD `3f3d824`) concluded **FAILURE**. Failing job:
  **"integration (contract, real confluence)"** (job 86725997774) — step "Run
  reposix-confluence contract test against real Atlassian", **exit 101** (Rust test panic).
  Triage with `gh run view 29220925797 --log-failed` — **dispatch that to a subagent**, do
  NOT read the log into your own context.
- **KEY FINDING: CI runs REAL-Atlassian contract tests** (it holds the secrets) — not only
  the env-gated local `pre-release-real-backend` cadence. So Confluence backend-state churn
  turns CI red. This session churned TokenWorld heavily.
- **PRIME SUSPECT:** the phase-coordinator left **2 orphan "p93 smoke A" pages on TokenWorld**
  (from failed CREATE-recovery runs, §2 B2). A contract test asserting a known/clean page set
  would break on these. Also possible: the 2818063 v7 state or CREATE-recovery non-convergence.
  **FIRST MOVE:** sweep the orphan pages from TokenWorld, then re-run the failed job / CI and
  see if it greens — but confirm the real cause with `--log-failed` first; don't assume.
- Do NOT open the tag sequence over this red main (constraint §5).

## 1. State (verify: `git log --oneline -8`, `git rev-parse origin/main`, `git tag -l | grep v0.14`)
- HEAD = origin/main = `3f3d824` + this docs handover commit on top. Clean tree at handoff.
- **NO v0.14 tag exists** (correct — never push it; the tag is the MANAGER's, only after an
  HONEST green gate). VERDICT.md is honestly **RED** at
  `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`. B4/B5 stay CLOSED. B1/B2 re-diagnosed ↓.

## 2. Both prior "blockers" were MISCHARACTERIZED (verified this session)
- **B1 (vision-litmus):** reconcile is CLEAN (matched=3/backend_deleted=0). Litmus fails only
  at PUSH: the GitHub mirror (`reposix-tokenworld-mirror.git`) `pages/2818063.md` is stale v1
  vs backend v7 — the manager's out-of-band Confluence restore bypassed reposix, so mirror-head
  refresh never fired. Lost-update guard rejects **correctly**. NOT a reposix bug.
  - Assumed fix `reposix sync --reconcile` is **PROVEN INVALID** (heals local cache only, NOT
    the external mirror) → also a **DOC LIE** in root CLAUDE.md (filed `337b91d`). Evidence:
    `.planning/milestones/v0.14.0-phases/evidence/B1-mirror-reconcile-FINDINGS-2026-07-13.md`;
    ledger `76a5659`/`a617740`.
- **B2 (p93):** harness self-reject CLEARED (`311d7fe`, pin to TokenWorld — line 57). First-ever
  credentialed run then FAILED — p93 was a **never-green catalog-first scaffold** (last_verified
  null). Root cause: **CREATE-recovery is non-convergent against id-assigning backends**
  (Confluence assigns page ids; recovery re-CREATEs a landed page → unique-title reject). The
  sim twin is green because it models UPDATE-recovery. Diagnosis `526d697`; product gap filed
  `ffb93ba`. Its runs are the likely source of the orphan pages + possibly the red CI (§0).

## 3. TWO DECISIONS for the MANAGER/OWNER (tag blocked until BOTH resolved)
These are not the workhorse's to decide — surface to the manager (standing tag authority);
owner-consult if needed. Both carry phase-coordinator recommendations + committed evidence.
- **DECISION-1 (B1 vision-litmus = NON-WAIVABLE P0 — must be genuinely fixed, NO caveat escape).**
  Option 1 (untried): fetch backend-current v7 via the **BUS remote** (not the stale mirror
  origin), rebase, push → refreshes mirror. Caveat: a no-op push won't refresh the mirror
  (refresh fires only when the SoT changes), so Option 1 likely means baking a bus-fetch-rebase
  **self-heal into the litmus = a change to the non-waivable P0 probe** → manager should bless
  (aligned with the already-filed **GTH-V15-09** self-healing-fixture). Also fix/repair the
  `sync --reconcile` doc-lie. **RECOMMEND:** bless the litmus self-heal.
- **DECISION-2 (B2/p93 = P0, caveat-able — the manager's release-caveat call).** (a) fix the
  CREATE-recovery product gap (convergent recovery vs id-assigning backends — >1h), OR (b) bless
  the bounded **honest harness rewrite** (p93 tests UPDATE-recovery + teardown + correct the
  lying catalog assert, <1h) AND record the CREATE-recovery gap as a documented tag caveat.
  **RECOMMEND (b).**

## 4. Mechanical follow-ups (after red main is GREEN + both decisions land)
- Sweep the 2 orphan "p93 smoke A" pages from TokenWorld (likely also the §0 red-CI fix).
- Then: `set -a; source .env; set +a; python3 quality/runners/run.py --cadence
  pre-release-real-backend --persist` → **HONEST exit 0** (never soften) → re-mint VERDICT.md
  GREEN → dispatch a FRESH unbiased ratification subagent (template: `quality/PROTOCOL.md`
  § "Verifier subagent prompt template") → author `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh`
  (pattern `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh`) → STOP at READY-TO-TAG. Manager
  pushes the tag. Preflight was PASS 3/3 this session; creds in `.env` are live.

## 5. Constraints (unchanged)
sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; sanctioned mutation
targets ONLY (TokenWorld / reubenjohn-reposix issues / JIRA TEST/KAN); NO tag push ever; never
open work over a red main; ONE cargo invocation machine-wide; leaf test setup in /tmp clones
(cd in the SAME bash call). Relief ~100k soft / ~150k hard absolute → replace THIS file,
commit+push, end turn. Resume an agent via SendMessage, never fork.

## 6. Serialization + budget (heed this)
Single tree-writer at a time. The heaviest cost is **subagent-RESULT weight** — real-backend
+ cargo transcripts are huge. Predecessor #3 rotated at ~148k: one phase-coordinator lane
(returned a rich report) + the red-main discovery consumed the budget. Delegate every heavy run
to a subagent, demand compact committed-artifact reports (SHAs + paths + key numbers only), and
NEVER pull CI `--log-failed` or a transcript into your own context — dispatch it.

---
History lives in git — `git log` / `git show`, not restated here.
