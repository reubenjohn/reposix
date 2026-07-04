# ⏸ P90 SESSION PAUSE — 2026-07-04 (owner restarting session with new permissions)

RESUME PROTOCOL for the next P90 coordinator (fable, full GSD cycle,
report-once-at-close — same charter as the paused session):

1. **Ground truth first:** `git -C /home/reuben/workspace/reposix log --oneline -8`
   + `git status --porcelain`. At pause: HEAD `fdc6bbc`, tree CLEAN (except
   this handoff file, committed right after writing). Nothing pushed this
   phase — origin/main = `29cc497`; local is 4 commits ahead (`603024e`
   Wave A catalog-first; `91bec9a` 90-02a/b/e/f validator helpers;
   `859f14d` 90-02c/d/e/f runner branch edits; `fdc6bbc` 90-02g wrapper +
   schema docs + GOOD-TO-HAVES). Wave-B1 (90-02, opus) completed all
   subtasks a–g; its final report arrived post-pause. **B1 evidence
   (received):** 93 unit tests OK (`unittest discover -s quality/runners`);
   pre-commit exit 0; catalog load sweep 12/12 clean;
   `git diff --stat quality/catalogs/` empty (zero status flips); gated
   zero-false-RED sweep = 0; p86 F6 fixture (9-vs-17, per-pair ≤1 shared
   token) correctly REDs. run.py now 459 lines (its ≤350 cap-breach filed
   as GOOD-TO-HAVES-06); verdict.py untouched at 367.
   **B1 deviations the resumed session MUST know:**
   (1) 90-02f congruence is GATED ON `minted_at` (new-regime rows only),
   NOT "both lists non-empty" — legacy prose asserts (e.g.
   docs-repro/example-03) would false-RED under any token threshold; the
   pure helper `asserts_congruent` stays threshold-based (≥2 shared
   significant tokens; ≥1 for ≤2-token asserts) so the F6 fixture fires.
   Congruence is therefore armed-but-dormant on legacy rows — honest, per
   D90-05's new-vs-legacy split.
   (2) artifact `skip_reason` is the machine marker `"env-missing"`; human
   text lives in a new `skip_detail` field (no existing consumer broke).
   (3) 90-02g wrapper uses the minimal sibling pattern (run suite + echo
   PASS; runner synthesizes artifact) per claim-vs-assertion-audit
   precedent.
   (4) verdict.py does NOT yet render FW-07a's artifact `error` marker —
   deliberately homed in 90-04 (its verdict.py task), do not lose it.
   Resume step 0 (cheap spot-check, evidence already reported): re-run the
   two verification commands above before building on the runner.

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
