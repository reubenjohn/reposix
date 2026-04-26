# v0.2.0-alpha — phases archive

**Shipped:** 2026-04-13 (post-noon) — the "real GitHub" cut: first real-backend read path.

**Phases:** 8, 9.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.2.0-alpha]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `git log --oneline v0.1.0..v0.2.0-alpha -- crates/ docs/` — implementation commits.
- `git log --oneline v0.1.0..v0.2.0-alpha -- .planning/milestones/v0.2.0-alpha-phases/` — planning artifact history.

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 08 `demos-and-real-backend` — GitHub Issues read-only `BackendConnector` + first real-backend demo.
- Phase 09 `swarm-harness` — multi-agent contention/swarm test harness scaffold.

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md files are reachable via `git show HEAD~N:.planning/milestones/v0.2.0-alpha-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
