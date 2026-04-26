# v0.6.0 — phases archive

**Shipped:** 2026-04-14 — write path + full sitemap.

**Phases:** 16, 17, 18, 19, 20, 26.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.6.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `git log --oneline v0.5.0..v0.6.0 -- crates/ docs/` — implementation commits.
- `git log --oneline v0.5.0..v0.6.0 -- .planning/milestones/v0.6.0-phases/` — planning artifact history.

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 16 `confluence-write-path-update-create-delete-or-close` — Confluence write path (update / create / delete / close) with ADF converter.
- Phase 17 `swarm-confluence-direct-mode-add-mode-confluence-direct-to-r` — swarm harness mode `confluence-direct` for contention testing.
- Phase 18 `op-2-remainder-tree-recursive-and-mount-root-index-md-synthe` — OP-2 remainder: recursive `+tree/` plus mount-root `_INDEX.md`.
- Phase 19 `op-1-remainder-labels-and-spaces-directory-views-as-read-onl` — OP-1 remainder: labels and spaces directory views (read-only).
- Phase 20 `op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount` — `reposix refresh` subcommand and `git diff` cache for mount.
- Phase 26 `docs-clarity-overhaul-unbiased-subagent-review-of-readme-mkd` — docs clarity overhaul via unbiased subagent review of README + mkdocs.

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md / RESEARCH.md files are reachable via `git show HEAD~N:.planning/milestones/v0.6.0-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
