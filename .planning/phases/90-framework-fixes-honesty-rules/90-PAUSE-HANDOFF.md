# ⏸ P90 SESSION PAUSE — 2026-07-04 (owner restarting session with new permissions)

RESUME PROTOCOL for the next P90 coordinator (fable, full GSD cycle,
report-once-at-close — same charter as the paused session):

1. **Ground truth first:** `git -C /home/reuben/workspace/reposix log --oneline -8`
   + `git status --porcelain`. At pause: HEAD `fdc6bbc`, tree CLEAN (except
   this handoff file, committed right after writing). Nothing pushed this
   phase — origin/main = `29cc497`; local is 4 commits ahead (`603024e`
   Wave A catalog-first; `91bec9a` 90-02a/b/e/f validator helpers;
   `859f14d` 90-02c/d/e/f runner branch edits; `fdc6bbc` 90-02g wrapper +
   schema docs + GOOD-TO-HAVES). Wave-B1 (90-02, opus) landed all its
   commits but was stopped by the pause directive BEFORE delivering its
   final report — its unittest-count / zero-false-RED-sweep evidence was
   NOT reported back. **Resume step 0: independently re-verify B1's work**:
   `python3 -m unittest discover -s quality/runners -p 'test_*.py'` and
   `python3 quality/runners/run.py --cadence pre-commit`; read the three
   commit bodies for claimed coverage; spot-check 90-02f per-assert
   congruence against the p86 F6 fixture and 90-02d
   fail-closed-with-history against D90-04(AMENDED)/D90-12. Do not take
   the commits' word for green.

2. **Where we are in the P90 cycle:**
   - DONE: research (R1+R2), plan authoring, plan-check (GO-WITH-FIXES, all
     3 mandatory fixes applied + verified on disk), decisions ratified
     (D90-01..12), Wave A catalog-first `603024e` (9 rows NOT-VERIFIED,
     minted_at, pre-commit exit 0), Wave B1 = 90-02 commits landed (verify
     per step 1 before trusting).
   - NOT STARTED: 90-03 (test-name-vs-asserts gate + subagent-graded
     migration — NOTE plan-check fix #3: the dispatcher lives at
     .claude/skills/reposix-quality-review/dispatch.sh, NOT
     quality/gates/agent-ux/), 90-04 (templates + PROTOCOL + verdict.py
     adversarial hook — verdict.py was deliberately left untouched by
     90-02), 90-05 (RAISE LIST + waiver-cliff triage), 90-06 (cargo wave:
     5 MISSING_TEST tests — the ONLY cargo wave), 90-07 (intake drain +
     ROADMAP P91 amendment + CLAUDE.md), close ritual (push, CI,
     zero-context verifier, verdict at quality/reports/verdicts/p90/,
     STATE.md).

3. **Binding artifacts (all committed in `603024e`), read in order:**
   `90-DECISIONS.md` (D90-01..12 — includes QL-001→P91 routing, amended
   skip semantics, plan-check dispositions) → `90-PLAN-OVERVIEW.md` (wave
   DAG, SC1-8 traceability) → `90-01..90-07-PLAN.md` → research files as
   the per-wave briefs cite them.

4. **Operating constraints (unchanged):** ONE tree-writer at a time (Wave B
   was deliberately run SEQUENTIAL 90-02→90-03→90-04 despite the plan's
   parallel option — keep that); ONE cargo invocation machine-wide (only
   90-06 needs cargo); no --no-verify; push only at honestly-green points
   (expected: single terminal push at 90-07/close — new NOT-VERIFIED rows
   block pre-push until implementations flip them); commit trailers:
   `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` + Claude-Session
   line; model tiering per OD-4 (executors sonnet, complex lanes opus,
   verifier opus zero-context).

5. **Close requirements (unchanged from charter):** verdict at
   quality/reports/verdicts/p90/, CI green via `gh run watch --exit-status`,
   RAISE LIST at quality/reports/raise-list-p90.md must be decision-ready
   (seeds P92/P94/P95), SURPRISES-INTAKE dispositions (4 cross-AI entries +
   QL-001 ROUTED-P91 + underscore-typo RESOLVED c0d5459), ROADMAP P91
   amendment (QL-001 sharpened criteria + sanctioned-target litmus-body
   criterion per D90-06), STATE.md cursor advance, single final report
   (verdict, commits, RAISE LIST, QL-001 disposition, intake, deviations,
   NOTICED aggregate, P91 handoff). Never touch PR #61. Do NOT start P91.

6. **Waiver clock awareness:** 12 catalog waivers expire 2026-07-26;
   `agent-ux/real-git-push-e2e` expires 2026-07-31 (QL-001 backstop —
   deliberately NOT renewed per D90-01); 5 docs-repro waivers already
   expired 2026-05-12. 90-05 owns the triage; do not let the cliff hit
   undrained.
