# .planning/ archives — verdict per milestone

[← index](./index.md)

| Path | Files | Verdict | Rationale |
|---|---:|---|---|
| `v0.1.0-phases/` | 38 | **CONDENSE→ARCHIVE.md** | First milestone, mostly historical; 5 sub-dirs each with their own `*-CONTEXT.md` + `*-DONE.md` + `*-WAVES.md` + 2-3 plans. Keep `S-stretch-write-path-and-remote-helper/S-DONE.md` (referenced from CHANGELOG) by transcribing into ARCHIVE.md. |
| `v0.2.0-alpha-phases/` | 4 | **CONDENSE→ARCHIVE.md** | Only 2 phases (08, 09), tiny. |
| `v0.3.0-phases/` | 19 | **CONDENSE→ARCHIVE.md** | Single phase 11-confluence-adapter with A/B/C/D/E/F sub-plans — typical pattern. |
| `v0.4.0-phases/` | 24 | **CONDENSE→ARCHIVE.md** | Phase 13 only (nested mount layout — pre-pivot architecture). Has a `deferred-items.md` worth grepping into ROADMAP first. |
| `v0.5.0-phases/` | 16 | **CONDENSE→ARCHIVE.md** | Phases 14, 15. |
| `v0.6.0-phases/` | 68 | **CONDENSE→ARCHIVE.md** | Largest pre-pivot milestone — 5 phases × A/B/C waves × {PLAN,SUMMARY,REVIEW,REVIEW-FIX}. Highest condensation ROI. |
| `v0.7.0-phases/` | 63 | **CONDENSE→ARCHIVE.md** | Phases 21–25 hardening. Same structure. |
| `v0.8.0-phases/` | 31 | **CONDENSE→ARCHIVE.md** | Phases 27–29 (JIRA + rename). |
| `v0.9.0-phases/` | 56 | **KEEP** | Architecture pivot; still referenced by ADRs, CHANGELOG, `docs/how-it-works/`. CATALOG-v2 leaves these intact intentionally. |
| `v0.10.0-phases/` | 11 | **KEEP** | Already trimmed during ship (most phases only have CONTEXT + VERIFICATION). |
| `v0.11.0-phases/` | — | n/a | No phase dir created; phases ran inline per STATE.md "implementation_complete". After v0.11.0 tag push, decide whether to backfill from research/. |

`.planning/milestones/v0.10.0-ROADMAP.md` (4.5 KB) and `v0.9.0-ROADMAP.md` + `v0.8.0-ROADMAP.md` + `v0.8.0-REQUIREMENTS.md` are loose milestone docs. **CONDENSE into the per-milestone ARCHIVE.md** above (or move to `.planning/archive/milestones/`). They're read-only history.

`.planning/notes/`:
- `gsd-feedback.md` (142 lines, status `awaiting-user-review`) — **KEEP** until owner files upstream issue.
- `phase-30-narrative-vignettes.md` (459 lines) — **ARCHIVE** (see #10 above).
- `v0.11.0-doc-polish-backlog.md` (92 lines) — **KEEP** until v0.12.0 picks it up; it's actively referenced by STATE.md.

`.planning/research/`:
- `v0.1-fuse-era/` (4 files) — **KEEP**. Foundational + threat-model still cited from `SECURITY.md` and `README.md`.
- `v0.9-fuse-to-git-native/` (12 files including `poc/`) — **KEEP**. Source of truth for the architecture pivot; cited from `CLAUDE.md`.
- `v0.10.0-post-pivot/milestone-plan.md` (1 file) — **KEEP** but rename pattern is the model for v0.11.0 (see rec #6).
- `v0.11.0-*.md` (8 files, 1843 lines) — **RELOCATE** post-tag (see rec #6).

`.planning/v0.9.0-MILESTONE-AUDIT.md` + `v0.10.0-MILESTONE-AUDIT.md` (top-level) — **KEEP at top level until v0.11.0 audit ships**, then move all three under `.planning/milestones/audits/` to keep `.planning/` root clean. Currently 5 audit/session/retro markdown at the root; only `STATE.md`, `PROJECT.md`, `ROADMAP.md`, `REQUIREMENTS.md`, `MILESTONES.md`, `CATALOG.md`, `RETROSPECTIVE.md`, `config.json` need to live there.

`.planning/CATALOG.md` (529 lines, 45 KB) is itself a v0.11.0-era audit snapshot ("Total tracked files: 619"). The newer `v0.11.0-CATALOG-v2.md` is the supersedence. **MOVE `CATALOG.md` to `.planning/research/v0.11.0/CATALOG-v1.md`** (it's the prequel) — keeps both audits together and deletes the misleading top-level placement.

`.planning/phases/` directory exists but is empty — **DELETE empty dir**. (Was the staging area before milestone-archival pattern took over.)
