---
phase: 30
doc_type: plan-overview
purpose: High-altitude review surface so a reviewer can sanity-check 9 plans without reading each one.
---

# Phase 30 — Plan Overview

## 1. Scope one-liner

Rewrite the landing page and restructure the MkDocs IA so reposix's value prop lands in 10 seconds, with the technical architecture progressively revealed in a "How it works" section instead of leaking above the fold.

## 2. Wave diagram

```
Wave 0 (tooling + skeletons, parallel) ─────────────────────────────
  ├─ 30-01  Vale + hooks + screenshot/mermaid scripts + CI install
  └─ 30-02  14 page skeletons (H1 + locked H2s + placeholder mermaid)
              │
Wave 1 (authoring + nav, parallel) ──────────────────────────────────
  ├─ 30-03  Hero + mental-model + vs-mcp-sdks copy
  ├─ 30-04  mkdocs.yml nav restructure + theme + social plugin
  └─ 30-06  Tutorial (5-min, runnable against sim) + test runner
              │
Wave 2 (carve existing content, parallel) ──────────────────────────
  ├─ 30-05  How-it-works carve (filesystem/git/trust-model)
  └─ 30-07  Guides (connector/agent/troubleshoot) + reference/simulator
              │
Wave 3 (cleanup, serial) ────────────────────────────────────────────
  └─ 30-08  Grep-audit + delete obsolete files + README + nav trim
              │
Wave 4 (gate, human checkpoint) ─────────────────────────────────────
  └─ 30-09  Full suite (mkdocs --strict + vale + playwright +
             doc-clarity-review LANDED) + CHANGELOG + SUMMARY
```

## 3. Per-plan summary table

| ID | Wave | Objective (one line) | Files | Requirements |
|----|------|----------------------|-------|--------------|
| 30-01 | 0 | Vale, hooks, screenshot/mermaid scripts, structure check, CI install | 11 | DOCS-09 |
| 30-02 | 0 | 14 page skeletons so nav and link checks don't dangle later | 16 | DOCS-02, -03, -04, -05, -06 |
| 30-03 | 1 | Author hero (V1 Jira vignette), mental-model, vs-mcp-sdks copy | 3 | DOCS-01, -03 |
| 30-04 | 1 | `mkdocs.yml` nav to IA sketch + theme tuning + social plugin | 1 | DOCS-07, -08 |
| 30-05 | 2 | Carve `architecture.md`/`security.md` into three how-it-works pages | 6 | DOCS-02 |
| 30-06 | 1 | 4-step tutorial against simulator + end-to-end test runner | 2 | DOCS-06 |
| 30-07 | 2 | Fill guides (move connector, author agent + troubleshooting) + reference/simulator | 5 | DOCS-04, -05 |
| 30-08 | 3 | Grep-audit, delete obsolete files, update README + nav trim | 9 | DOCS-07 |
| 30-09 | 4 | Full-suite gate + SUMMARY + CHANGELOG | 3 | ALL DOCS-01..09 |

## 4. Cross-cutting constraints

- **P1 (complement, not replace).** "replace" banned in hero/value-prop copy (Vale `NoReplace` scoped to `docs/index.md`). 30-03 enforces.
- **P2 (progressive disclosure).** FUSE/inode/daemon/helper/kernel/mount/syscall banned above Layer 3 (Vale `ProgressiveDisclosure`). Per-glob opt-outs for `how-it-works/`, `reference/`, `decisions/`, `research/`, `development/`. `mental-model.md` has a per-file exception for its locked H2 `mount = git working tree`.
- **Simulator-first tutorial.** 30-06 runs against `reposix-sim`, never a real backend (enforced by `scripts/test_phase_30_tutorial.sh`).
- **Wave 0 gates everything.** 30-01 ships Vale + hooks + scripts before any copy merges.
- **Deletion only in Wave 3.** 30-08 deletes after grep-audit; `mkdocs.yml` parks deprecated files in `not_in_nav` during the transition.
- **Wave 4 is a real gate.** `mkdocs build --strict` + `vale` + 14 playwright screenshots + `doc-clarity-review` verdict `LANDED` all required; `PARTIAL`/`MISSED` triggers revision.

## 5. Deferred / out-of-scope

- No new features, CLI surface, or backend connectors.
- No changes to `docs/reference/` or `docs/decisions/` (Phase 26 fixed those).
- No `REQUIREMENTS.md` or MILESTONES.md-level work beyond Phase 30's 9 DOCS-NN requirements.
- Per-backend guides (`connect-{github,jira,confluence}.md`) ship as thin redirect stubs — full how-to content deferred.
- No Jinja home-template override (markdown-native per RESEARCH.md).
- Custom social-card color deferred to post-launch.
- Troubleshooting ships with 3 symptom/cause/fix entries — explicit stub that grows post-launch.

## 6. What to challenge (reviewer checklist)

1. **Hero vignette representativeness.** 30-03 Task 1 uses V1 "Close a Jira ticket" verbatim from source-of-truth lines 114–177. Is the 5-curl "before" the right hero, or will platform engineers call it a strawman?
2. **Nav placement of `docs/vs-mcp-sdks.md`.** 30-04 puts it top-level. Sub-page under Home would reduce top-bar clutter — which wins?
3. **Deletion set in Wave 3.** 30-08 deletes `architecture.md`, `security.md`, `demo.md`, `demos/`, `connectors/`. Grep-audit is repo-internal only — external inbound links (blogs, HN, README badges pointing at `docs/security.md`) are NOT checked. Redirect shim needed?
4. **Simulator move to Reference (DOCS-05).** Assumes no external bookmarks to the old URL. No `mkdocs-redirects` plugin planned.
5. **`doc-clarity-review` prompt (30-09 Task 3).** A cold reader might name the tech ("POSIX adapter for issue trackers") rather than the payoff ("edit issues as files, git push to sync") and trip a false PARTIAL. Is the rubric calibrated?
6. **Tutorial first-run UX.** 30-06 assumes `fuse3` is installed. A macOS/WSL2 reader fails at step 2 with no escape hatch. Docker fast-path before the native path?
7. **Troubleshooting ship-or-defer.** 30-07 ships only 3 symptom/cause/fix entries — may read as abandoned. Defer until post-launch?
8. **Banned-word list completeness (P2).** Current tokens miss `kernel-space`, `userspace`, `VFS`, `fusermount`, `/dev/fuse`. Extend before 30-01 locks the Vale rule?
9. **Creative-license enforcement.** Source-of-truth bans `empower / revolutionize / next-generation / feature-grid check-marks / stock photos` — no Vale rule enforces these. Add `MarketingBans.yml` in 30-01?
