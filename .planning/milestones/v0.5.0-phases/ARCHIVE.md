# v0.5.0 — phases archive

**Shipped:** 2026-04-14 — `IssueBackend` decoupling + bucket `_INDEX.md`.

**Phases:** 14, 15.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.5.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `git log --oneline v0.4.1..v0.5.0 -- crates/ docs/` — implementation commits.
- `git log --oneline v0.4.1..v0.5.0 -- .planning/milestones/v0.5.0-phases/` — planning artifact history.

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 14 `decouple-sim-rest-shape-from-fuse-write-path-and-git-remote` — split simulator REST shape from FUSE write path and git-remote helper.
- Phase 15 `dynamic-index-md-synthesized-in-fuse-bucket-directory-op-2-partial` — synthesized `_INDEX.md` in each FUSE bucket directory (OP-2 partial).

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md files are reachable via `git show HEAD~N:.planning/milestones/v0.5.0-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
