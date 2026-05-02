# ROADMAP.md, notes/, research/, and phases/ issues

← [back to index](./index.md)

## ROADMAP.md issues

### RM1 — v0.11.0 milestone bullet underspecifies the just-landed work (line 15) [P1]

**Current text:**
```
- 📋 **v0.11.0 Performance & Sales Assets** — planning (helper multi-backend dispatch fix prereq; doc-polish backlog at `.planning/notes/v0.11.0-doc-polish-backlog.md`)
```

The `helper multi-backend dispatch fix prereq` is closed. Also missing: launch blog post, latency benchmark, screenshots, real-backend coverage.

**Recommended replacement:**
```
- 📋 **v0.11.0 Performance & Sales Assets** — planning (latency benchmark with real-backend cells; doc-polish backlog `.planning/notes/v0.11.0-doc-polish-backlog.md`; playwright screenshots deferred from v0.10.0; launch blog at `docs/blog/2026-04-25-reposix-launch.md`)
```

### RM2 — Phase 30 detailed entry (lines 309–331) still shows up [P2]

The "Legacy Phase 30 — superseded by Phases 40–45 above (retained for traceability)" `<details>` block (lines 309–331) is correct prose but the corresponding phase dir is still in `.planning/phases/` (see "phases/ vs milestones/" below). Consistency: the `<details>` block says "NOT executed" but a future agent who runs `find .planning/phases/` will see the dir and try to execute it.

**Recommendation:** keep the `<details>` block; move the phase dir (see PV1 below) into archived milestone — the `<details>` is the right historical record, but the directory living next to active phases creates confusion.

### RM3 — Plan-row-30-09 references `CHANGELOG v0.9.0` (line 330) [P3]

**Current text (line 330):** "30-09-PLAN.md — Wave 4: ... CHANGELOG v0.9.0 + SUMMARY (DOCS-01..09)"

This was Phase 30's original v0.9.0 framing; the work landed under v0.10.0 (CHANGELOG `[v0.10.0]`). Cosmetic, traceability-only.

**Recommendation:** prefix the legacy plan list with a one-line note "Plan rows below predate the v0.9.0/v0.10.0 split; CHANGELOG references mean v0.10.0 in current vocabulary." OR leave as-is since the whole block is `<details>`-collapsed.

### RM4 — Backlog Phase 999.1 (lines 395–413) is appropriate as-is [—]

This block — "Resolve plans that ran without producing summaries" — is a legitimate backlog item that survived multiple milestones. Keep.

---

## notes/ inventory

| File | Status | Recommendation |
|------|--------|----------------|
| `phase-30-narrative-vignettes.md` | Stale frontmatter (`status: ready-for-phase-30-planning`); banned-word list pre-pivot. P1/P2 framing principles still load-bearing for v0.10.0 phase descriptions. | Keep. Add a one-line header note: "**2026-04-25 update:** Phase 30 superseded by v0.10.0 Phases 40–45 (shipped). Banned-word list updated for git-native in REQUIREMENTS.md DOCS-07. Vignette V1 still load-bearing — referenced by Phase 40 `40-CONTEXT.md`." |
| `gsd-feedback.md` | `status: awaiting-user-review` from 2026-04-19 (6 days old). | Keep. The four issues are GSD-tool concerns, not project state; appropriate to remain in notes/ until user files them. Optionally bump frontmatter `captured` date to reflect the freeze date. |
| `v0.11.0-doc-polish-backlog.md` | Active. New 2026-04-25; matches MILESTONES.md `v0.11.0-doc-polish-backlog` reference. | Keep. Authoritative for DOCS-NN follow-ups. |

**No notes/ files to delete.**

---

## research/ inventory

| File / Dir | Status | Recommendation |
|------------|--------|----------------|
| `v0.1-fuse-era/` (4 files) | Path explicitly era-tagged. Correctly historical. | Keep as-is. |
| `v0.9-fuse-to-git-native/` (9 files + `poc/` 4 files) | The pivot design corpus. Load-bearing. | Keep as-is. |
| `v0.10.0-post-pivot/milestone-plan.md` | Source-of-truth for v0.10.0 — now SHIPPED. | Move to `.planning/milestones/v0.10.0-research/milestone-plan.md` (parallel to `v0.9.0-ROADMAP.md` archive convention) OR keep in research/ as historical. **Recommendation:** keep in `research/` — the file's status header "Research draft. Owner reviews and promotes" is still accurate, and removing it would break ROADMAP/REQUIREMENTS cross-references. Optionally add a top-line header banner "STATUS: shipped — see `.planning/milestones/v0.10.0-ROADMAP.md`." |
| `v0.11.0-vision-and-innovations.md` | Active research. | Keep. Active. |
| `v0.11.0-latency-benchmark-plan.md` | Active research. | Keep. Active. |

**No research/ files to delete.** All correctly era-tagged or active.

---

## phases/ vs milestones/

`find .planning/phases/ -mindepth 1 -maxdepth 1 -type d` returns **one** directory:

```
.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p
```

### PV1 — `phases/30-...` should be archived [P0]

This dir is the deferred-then-superseded Phase 30 work (planning artifacts only — never executed; PLAN files are dated 2026-04-17 and not committed against). CATALOG.md line 312 already flagged this:
> "Strong recommendation: **move to `.planning/milestones/v0.9.0-phases/`** to match the rest of the historical convention."

**Recommendation:** `git mv .planning/phases/30-... .planning/milestones/v0.9.0-phases/30-docs-ia-deferred-superseded/` (rename the dir to make its status visible). Note that `MILESTONES.md` v0.9.0 entry already does NOT mention Phase 30 as a v0.9.0 phase, so the move is consistent with the milestone narrative. ROADMAP.md `<details>` traceability block for the legacy phase 30 (lines 309–331) is unaffected.

After the move, `find .planning/phases/ -mindepth 1 -maxdepth 1 -type d` will return empty — the desired clean-slate state for v0.11.0 planning.

### PV2 — milestones/v0.10.0-phases/ archive is healthy [—]

`milestones/v0.10.0-phases/` contains Phases 40–45 dirs with CONTEXT/VERIFICATION pairs. CATALOG.md and the audit confirm these were archived during the lifecycle close. Nothing to do.

### PV3 — milestones/v0.10.0-ROADMAP.md is healthy [—]

This is the ROADMAP archive snapshot per v0.9.0 precedent. Cross-referenced from ROADMAP.md line 14 (`[archive](milestones/v0.10.0-ROADMAP.md)`). The link uses a relative path that, given `ROADMAP.md` lives in `.planning/`, resolves to `.planning/milestones/v0.10.0-ROADMAP.md` — file exists. OK.
