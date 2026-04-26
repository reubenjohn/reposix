# v0.4.0 — phases archive

**Shipped:** 2026-04-14 — nested mount layout `pages/+tree/` for Confluence parent-child page hierarchy.

**Phases:** 13.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.4.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `git log --oneline v0.3.0..v0.4.0 -- crates/ docs/` — implementation commits.
- `git log --oneline v0.3.0..v0.4.0 -- .planning/milestones/v0.4.0-phases/` — planning artifact history.

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 13 `nested-mount-layout-pages-tree-symlinks-for-confluence-parent-child` — `pages/+tree/` layout with symlinks for Confluence parent-child hierarchy.

**Deferred items snapshot (now superseded by ROADMAP / v0.5+ phases):**
- `contract_github` test required `REPOSIX_ALLOWED_ORIGINS` env on `--ignored` runs (later swept by `skip_if_no_env!` macro adoption).
- `reposix-remote` emitted `{:04}.md` paths while FUSE mount used 11-digit padding (resolved by Wave D1 BREAKING migration sweep, then made moot by the v0.9.0 git-native pivot which deleted the FUSE mount entirely).

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md / deferred-items.md files are reachable via `git show HEAD~N:.planning/milestones/v0.4.0-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
