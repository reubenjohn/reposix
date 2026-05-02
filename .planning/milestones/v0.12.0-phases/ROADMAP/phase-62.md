# Phase 62: Repo-org-gaps cleanup — close the v0.11.1 audit (v0.12.0)

**Goal:** Audit `.planning/research/v0.11.1/repo-organization-gaps.md` against current state and close out every remaining gap as either a fix + structure-dimension catalog row that prevents recurrence, or an explicit waiver with reason. The repo-organization-gaps doc is a forgotten todo list if not actioned; this phase ensures every gap becomes a tracked catalog row in the new framework (so a future gap audit doesn't have to be a manual document grep). Operating-principle hooks: **OP-4 self-improving infrastructure** — each gap that recurred under v0.11.x is evidence of a missing structure gate, which this phase backfills; **OP-6 ground truth obsession** — "fixed in CLAUDE.md but not enforced" is not a fix; only a catalog row + verifier counts.

**Requirements:** ORG-01

**Depends on:** P57 GREEN (structure dimension must exist), P61 GREEN (so the repo-org gaps can route into the now-mature dimension set, not into a half-built framework). **Gate-state precondition:** P61's subjective-gates catalog shows GREEN in `quality/reports/verdicts/p61/`. (P62 is also independent enough to slot earlier if scheduling demands it, but the natural order is "polish the framework first, then sweep gaps into it.")

**Success criteria:**
1. Every gap in `.planning/research/v0.11.1/repo-organization-gaps.md` has a status: `closed-by-catalog-row` (gap fixed + recurrence prevented by a structure-dimension catalog row), `closed-by-existing-gate` (gap fixed + already covered by an earlier P57/P58/P60 gate), or `waived` (explicit `quality/catalogs/orphan-scripts.json` or `quality/catalogs/waivers.json` row with reason + dimension_owner + RFC3339 `until`).
2. The audit results are committed under `quality/reports/verifications/repo-org-gaps/<ts>.md` with a row per gap and its closure path.
3. The `.planning/research/v0.11.1/repo-organization-gaps.md` document gets a top-banner update naming "fully audited and closed under P62; see `quality/reports/verifications/repo-org-gaps/<ts>.md` for per-gap closure" — the document is no longer a forgotten todo list.
4. New structure-dimension catalog rows added under `quality/catalogs/freshness-invariants.json` (or a new `repo-org.json`) for each gap that needed a recurrence guard.
5. **Recurring (catalog-first):** ORG-01 catalog row + the per-gap closure rows land BEFORE the audit-fix commits.
6. **Recurring (CLAUDE.md):** CLAUDE.md updated to cite the audit closure + new recurrence-guard rows + waivers (if any) in the appropriate freshness-invariant or workspace-layout sections. In the same PR.
7. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p62/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Repo-org cleanup" ORG-01, `.planning/research/v0.11.1/repo-organization-gaps.md` (the audit document being closed), existing `quality/gates/structure/` rows from P57 (the recurrence-guard infrastructure these gaps will route into).

**Plans:** 6 plans
- [ ] 62-01-PLAN.md — Wave 1 catalog-first commit: 3 structure rows + dim README delta (ORG-01 + POLISH-ORG contract)
- [ ] 62-02-PLAN.md — Wave 2 execute audit: render quality/reports/audits/repo-org-gaps.md + scripts/check_repo_org_gaps.py verifier (ORG-01)
- [ ] 62-03-PLAN.md — Wave 3 POLISH-ORG fix wave: relocate top-level audits + archive SESSION-END-STATE + purge __pycache__ + extend structure verifier
- [ ] 62-04-PLAN.md — Wave 4 SURPRISES.md rotation (302→<=200; archive P57+P58 to SURPRISES-archive-2026-Q2.md)
- [ ] 62-05-PLAN.md — Wave 5 CLAUDE.md QG-07 P62 subsection + audit-doc closure banner + STATE/REQUIREMENTS flips
- [ ] 62-06-PLAN.md — Wave 6 verifier subagent dispatch (Path A or Path B) + verdict GREEN
