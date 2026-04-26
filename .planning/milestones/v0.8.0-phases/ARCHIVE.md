# v0.8.0 — phases archive

**Shipped:** 2026-04-16 — JIRA Cloud integration.

**Phases:** 27, 28, 29.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.8.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `.planning/milestones/v0.8.0-ROADMAP.md` + `.planning/milestones/v0.8.0-REQUIREMENTS.md` — milestone-level scoping (NOT condensed; kept intact).
- `git log --oneline v0.7.0..v0.8.0 -- crates/ docs/` — implementation commits.
- `git log --oneline v0.7.0..v0.8.0 -- .planning/milestones/v0.8.0-phases/` — planning artifact history.

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 27 `foundation-issuebackend-backendconnector-rename-issue-extensions-field-v0-8-0` — foundation: `IssueBackend` → `BackendConnector` rename, `issue.extensions` field.
- Phase 28 `jira-cloud-read-only-adapter-reposix-jira-v0-8-0` — `reposix-jira` read-only adapter.
- Phase 29 `jira-write-path-create-update-delete-or-close-via-transitions-api` — JIRA write path (create / update / delete / close via transitions API).

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md files are reachable via `git show HEAD~N:.planning/milestones/v0.8.0-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
