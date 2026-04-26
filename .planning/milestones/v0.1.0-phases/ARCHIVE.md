# v0.1.0 — phases archive

**Shipped:** 2026-04-13 (~05:38) — initial autonomous overnight build, simulator-only MVD.

**Phases:** 1, 2, 3, 4, S (stretch).

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.1.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `git log --oneline v0.1.0 -- crates/ docs/` — implementation commits.
- `git log --oneline v0.1.0 -- .planning/milestones/v0.1.0-phases/` — planning artifact history (THIS dir, before condensation).

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 01 `core-contracts-security-guardrails` — error variants, HTTP allowlist, `Tainted<T>` typing, audit schema fixture.
- Phase 02 `simulator-audit-log` — axum simulator + SQLite append-only audit log.
- Phase 03 `readonly-fuse-mount-cli` — read-only FUSE mount and CLI (later folded into git-native pivot v0.9.0).
- Phase 04 `demo-recording-readme` — first demo recording + README.
- Phase S `stretch-write-path-and-remote-helper` — FUSE write path + first `git-remote-reposix` helper sketch (Phase S referenced from CHANGELOG).

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md files are reachable via `git show HEAD~N:.planning/milestones/v0.1.0-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
