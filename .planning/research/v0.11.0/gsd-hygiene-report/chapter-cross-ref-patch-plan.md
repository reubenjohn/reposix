# Cross-reference brittleness, Patch plan, and Recommended execution order

← [back to index](./index.md)

## Cross-reference brittleness

I verified each cited path. Findings:

| Reference | Cited from | File exists? | Notes |
|-----------|------------|---|---|
| `scripts/tag-v0.10.0.sh` | PROJECT.md line 131, ROADMAP.md line 28 | ✓ | Lives at repo root `scripts/tag-v0.10.0.sh`. STATE/PROJECT/MILESTONES say "not yet authored — owner gate deferred" but the file actually exists (commit `5ffdc4e`). **Inconsistency:** stop saying "not yet authored." Owner gate is the push, not the script. |
| `scripts/tag-v0.9.0.sh` | ROADMAP.md line 145, MILESTONES.md (implicit) | ✓ | Exists. Owner gate is push. |
| `docs/benchmarks/v0.9.0-latency.md` | ROADMAP.md / PROJECT.md / REQUIREMENTS.md many | ✓ | Sim column populated; real-backend pending. |
| `docs/reference/testing-targets.md` | ROADMAP.md, REQUIREMENTS.md ARCH-18 | ✓ | Exists. |
| `.planning/v0.9.0-MILESTONE-AUDIT.md` | PROJECT.md line 160, ROADMAP.md line 28 | ✓ | Verdict already flipped to `passed` (line 24). |
| `.planning/v0.10.0-MILESTONE-AUDIT.md` | ROADMAP.md line 143 | ✓ | Verdict tech_debt with item 2 already struck-through (lines 25–32). |
| `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` | every section | ✓ | Exists. |
| `.planning/research/v0.10.0-post-pivot/milestone-plan.md` | every section | ✓ | Exists. |
| `.planning/notes/v0.11.0-doc-polish-backlog.md` | STATE.md / ROADMAP.md / PROJECT.md | ✓ | Exists. |
| `AgenticEngineeringReference.md` / `InitialReport.md` (root stubs) | STATE.md lines 31–33 (historical roadmap-evolution) | ✗ at root — correctly deleted in Phase 26 | The text is historical narrative ("Phase 26 SHIPPED — deleted ... root stubs"), so non-existence is correct. Not a broken cross-ref. |
| `HANDOFF.md` | CATALOG.md line 432 says "**delete**" | ✗ at root | Confirmed deleted (no HANDOFF.md, MORNING-BRIEF.md, or PROJECT-STATUS.md at root). CATALOG recommendation already executed. |

**No file paths that resolve to nothing.** The grounding is intact.

The brittleness is in *implication*, not *paths*: phrases like "tag script not yet authored" while the file exists, "carry-forward still open" while the audit struck it out, "ARCH-01..18 status: planning" while PROJECT.md says ✓ Validated.

---

## Patch plan (prioritized)

### P0 — Must fix before v0.11.0 phase scoping (data quality, contradicts reality)

1. **STATE.md S2:** Replace "Current Position" block (lines 64–75) with v0.11.0 cursor.
2. **STATE.md S5:** Delete the Pending Todos / Blockers / Concerns block (lines 197–212) — every entry is v0.1.0 MVD ghost data. Replace with the three real v0.11.0 carry-forwards.
3. **PROJECT.md P1:** Delete or `<details>`-fold the v0.1.0 Active functional-core list (lines 56–80) which advertises deleted commands (`reposix mount`, `reposix demo`).
4. **REQUIREMENTS.md R1:** Move v0.10.0 DOCS-01..11 section out of Active (lines 8–56) into a "Validated" sub-section, parallel to the existing v0.9.0 sub-section.
5. **REQUIREMENTS.md R4:** Update header and "Active milestone" line from v0.10.0 to v0.11.0.

### P1 — Should fix before any new phase planning (correctness drift)

6. **STATE.md S6:** Replace Session Continuity block (lines 214–222) with v0.10.0 close-out checkpoint and v0.11.0 cursor.
7. **STATE.md S7:** Delete Phase 13/14 historical session blocks (lines 224–245).
8. **STATE.md S1:** Reset Performance Metrics block (lines 77–125) — either zero out, or hydrate from MILESTONES.md.
9. **PROJECT.md P4:** Rewrite v0.11.0 goal block (lines 124–135) to remove three closed carry-forwards (helper dispatch, Record rename, doctor) and add real open items (latency benchmark, launch).
10. **PROJECT.md P2:** Sweep "Constraints" block (lines 99–106) — strike 2026-04-13 timeline and T+3h decision deadline.
11. **REQUIREMENTS.md R2:** Flip ARCH-01..18 traceability rows from `planning` to `shipped`.
12. **ROADMAP.md RM1:** Rewrite v0.11.0 milestone bullet (line 15) to enumerate latency benchmark + screenshots + launch blog instead of just helper-dispatch (closed) + doc-polish.
13. **PV1:** `git mv .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p .planning/milestones/v0.9.0-phases/30-docs-ia-deferred-superseded` to clear `phases/`.

### P2 — Nice-to-have hygiene (readability, ledger consistency)

14. **STATE.md S3 + S4:** Wrap historical Roadmap-Evolution rows (older than 2026-04-22) and Phase-tagged Decisions in collapsed `<details>` to keep the active band visible.
15. **PROJECT.md P3:** Sweep Key-Decisions table Outcome column (lines 110–123) — replace `— Pending` with `Validated`.
16. **PROJECT.md P5:** Strike v0.9.0 tech-debt carry-forward language (line 188 area) since v0.9.0 audit verdict flipped to `passed`.
17. **REQUIREMENTS.md R3:** Reconcile traceability table with explanatory note (line 205) once R2 is done.
18. **notes/phase-30-narrative-vignettes.md:** Add header note pointing at v0.10.0 phases as the actual landing site.
19. **research/v0.10.0-post-pivot/milestone-plan.md:** Add "STATUS: shipped" header banner.
20. **PROJECT.md / STATE.md / MILESTONES.md:** Replace "not yet authored" claims about `scripts/tag-v0.10.0.sh` with "exists; owner-gate is the push (`bash scripts/tag-v0.10.0.sh`)".

### P3 — Polish (cosmetic, low priority)

21. **ROADMAP.md RM3:** Annotate Phase 30 `<details>` legacy-plan list with the v0.9.0/v0.10.0 vocabulary disambiguation.
22. **STATE.md S8:** Decide and document `progress` frontmatter convention (per-milestone or cumulative).

---

## Recommended execution order

A single GSD scrub sub-phase, three waves:

1. **Wave A — P0 fixes (5 edits):** STATE.md S2 + S5; PROJECT.md P1; REQUIREMENTS.md R1 + R4. These are the items that make the next agent confused or wrong. Single commit each.
2. **Wave B — P1 fixes (8 edits) + the phase-dir move (PV1):** state-cleanup + correctness pass. One commit per file.
3. **Wave C — P2 + P3 polish:** wrap-historical-in-`<details>`, header notes, key-decisions sweep. Single rollup commit acceptable.

Total estimated: ~30 distinct edits across 4 files + 1 directory move. No code changes, no test impact. After landing, `.planning/STATE.md` should be the single coherent v0.11.0 cursor that an agent picking up the project tomorrow can read top-to-bottom without context-switching across two stale milestones.

End of report. (Word count ~2 850 — under 3 000 cap.)
