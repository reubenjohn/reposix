# v0.7.0 — phases archive

**Shipped:** 2026-04-16 — hardening + Confluence expansion + docs.

**Phases:** 21, 22, 23, 24, 25.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.7.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` — milestone summary line.
- `git log --oneline v0.6.0..v0.7.0 -- crates/ docs/` — implementation commits.
- `git log --oneline v0.6.0..v0.7.0 -- .planning/milestones/v0.7.0-phases/` — planning artifact history.

**Per-phase artifacts retained in git history (not in working tree):**
- Phase 21 `op-7-hardening-bundle-contention-swarm-500-page-truncation` — OP-7 hardening bundle: contention swarm, 500-page truncation, perf knobs.
- Phase 22 `op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun` — OP-8 honest tokenizer benchmarks (drop `len / 4` heuristic for real token counts).
- Phase 23 `op-9a-confluence-comments-exposed-as-pages-id-comments-comme` — OP-9a Confluence comments exposed as `pages/{id}/comments/`.
- Phase 24 `op-9b-confluence-whiteboards-attachments-and-folders` — OP-9b Confluence whiteboards, attachments, folders.
- Phase 25 `op-11-docs-reorg-initialreport-md-and-agenticengineeringrefe` — OP-11 docs reorg: `initial-report.md` + `agentic-engineering-reference.md`.

The per-phase CONTEXT.md / PLAN.md / SUMMARY.md / VERIFICATION.md / REVIEW.md / WAVES.md / AUDIT-NOTES.md files are reachable via `git show HEAD~N:.planning/milestones/v0.7.0-phases/<subdir>/<file>` for forensic deep-dives.

---

Condensed in v0.11.1 POLISH2-21 to remove 74% of `.planning/` markdown noise (repo-org audit rec #5).
