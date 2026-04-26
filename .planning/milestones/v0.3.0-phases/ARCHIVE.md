# v0.3.0 — phases archive

**Shipped:** 2026-04-14 — the "real Confluence" cut: Confluence Cloud read-only adapter.

**Phases:** 11.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.3.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `git log --oneline v0.2.0-alpha..v0.3.0 -- crates/ docs/` — implementation commits.
- `git log --oneline v0.2.0-alpha..v0.3.0 -- .planning/milestones/v0.3.0-phases/` — planning artifact history.

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 11 `confluence-adapter` — first `reposix-confluence` `BackendConnector` (read path); pages exposed as issues.

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md files are reachable via `git show HEAD~N:.planning/milestones/v0.3.0-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
