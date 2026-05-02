# DELETE

← [index](./index.md)

### One-shot launch artifacts (4 files) — repo root pollution
| Path | Why |
|---|---|
| `/home/reuben/workspace/reposix/MORNING-WALKTHROUGH-2026-04-25.md` (in `.planning/`) | Dated session journal. Useful for owner that morning, not for cold readers. CHANGELOG covers what shipped; STATE.md covers cursor. 32 KB of "we shipped X overnight". |
| `/home/reuben/workspace/reposix/RELEASE-NOTES-v0.10.0.md` | Duplicates CHANGELOG `[v0.10.0]` body. Industry convention is one source of truth for release notes. Either link CHANGELOG from GitHub release page (recommended), or move to `docs/release-history/v0.10.0.md`. |
| `/home/reuben/workspace/reposix/RELEASE-NOTES-v0.11.0-PREVIEW.md` | Preview notes for an unreleased version. Belongs in `.planning/notes/v0.11.0-doc-polish-backlog.md`-adjacent location, not the repo root. Or just defer until v0.11.0 actually ships. |
| `/home/reuben/workspace/reposix/PUBLIC-LAUNCH-CHECKLIST.md` | One-off pre-flight checklist; the launch is happening today. Once flown, this file is dead weight. Move to `.planning/archive/launch-checklist-2026-04-25.md` or delete. |

### FUSE-era demo scripts (10 files) — replaced by `dark-factory-test.sh`
| Path | Why |
|---|---|
| `/home/reuben/workspace/reposix/scripts/demo.sh` | 7-line shim execs `scripts/demos/full.sh` which was deleted in v0.9.0. **Broken on first invocation.** |
| `/home/reuben/workspace/reposix/scripts/demos/01-edit-and-push.sh` | `require reposix-fuse` (line 27); calls `setup_mount` from `_lib.sh`. Replaced by `examples/01-shell-loop/run.sh` + `dark-factory-test.sh`. |
| `/home/reuben/workspace/reposix/scripts/demos/02-guardrails.sh` | Uses `reposix mount` — deleted subcommand. |
| `/home/reuben/workspace/reposix/scripts/demos/03-conflict-resolution.sh` | Uses `reposix mount`. The conflict scenario lives in `examples/04-conflict-resolve/run.sh` now. |
| `/home/reuben/workspace/reposix/scripts/demos/08-full-backend-showcase.sh` | FUSE; superseded by `dark-factory-test.sh`. |
| `/home/reuben/workspace/reposix/scripts/demos/_lib.sh` | The `setup_mount`/`spawn reposix-fuse` helper library. Pure FUSE. |
| `/home/reuben/workspace/reposix/scripts/demos/_record.sh` | `script(1)` wrapper used only by the FUSE demos above. |
| `/home/reuben/workspace/reposix/scripts/demos/assert.sh` | Smoke-asserter for `_lib.sh` flows. |
| `/home/reuben/workspace/reposix/scripts/demos/parity.sh`, `parity-confluence.sh` | Demo set; FUSE-rooted. |
| `/home/reuben/workspace/reposix/scripts/demos/smoke.sh` | "Prereq: release binaries (reposix-sim, reposix-fuse...)" header. |
| `/home/reuben/workspace/reposix/scripts/demos/swarm.sh` | Swarm demo; FUSE-rooted. |

(Reviewer note: `04-token-economy.sh` is FUSE-neutral — keep that one and rehome to `scripts/bench/`.)

### FUSE-era recordings (8 files) — orphan from live nav
| Path | Why |
|---|---|
| `/home/reuben/workspace/reposix/docs/demo.transcript.txt` | `reposix-cli  reposix-core  reposix-fuse  reposix-remote  reposix-sim` literal in body. Only inbound link is `docs/demos/index.md` (also dead). |
| `/home/reuben/workspace/reposix/docs/demo.typescript` | Same. |
| `/home/reuben/workspace/reposix/docs/demos/index.md` | The Tier 1/2/3 narrative. Tier 2 row links to `scripts/demos/full.sh` (deleted). Already in `not_in_nav`. **Delete the file**; replace with `docs/tutorials/first-run.md` + `examples/`. |
| `/home/reuben/workspace/reposix/docs/demos/recordings/01-edit-and-push.{transcript.txt,typescript}` | FUSE-era. |
| `/home/reuben/workspace/reposix/docs/demos/recordings/02-guardrails.{transcript.txt,typescript}` | FUSE-era. |
| `/home/reuben/workspace/reposix/docs/demos/recordings/03-conflict-resolution.{transcript.txt,typescript}` | FUSE-era. |

(Keep `recordings/04-token-economy.{...}`, `parity.{...}`, `swarm.{...}` — they're FUSE-neutral.)

### Stale dev scripts (2 files)
| Path | Why |
|---|---|
| `/home/reuben/workspace/reposix/scripts/dev/probe-confluence.sh` | Header comment "running `reposix mount --backend confluence`". `reposix mount` was deleted; the script itself just hits the Confluence REST API directly so it's actually still functional, but the comment is a lie. **REVIEW alternative:** rewrite the comment + keep the script. |
| `/home/reuben/workspace/reposix/scripts/__pycache__/*.pyc` (2 files) | `__pycache__/` should be `.gitignore`d, not tracked. Delete and add to `.gitignore`. |

### Orphan assets (5 files)
| Path | Why |
|---|---|
| `/home/reuben/workspace/reposix/docs/screenshots/gh-pages-home-v0.2.png` | No inbound reference anywhere except `.planning/phases/30-.../30-PATTERNS.md` (planning doc). |
| `/home/reuben/workspace/reposix/docs/screenshots/gh-pages-home.png` | Same. |
| `/home/reuben/workspace/reposix/docs/screenshots/gh-pages-why-real-github.png` | Same. v0.2-era. |
| `/home/reuben/workspace/reposix/docs/screenshots/site-architecture.png` | No inbound ref. v0.7-era pre-pivot. |
| `/home/reuben/workspace/reposix/docs/github-readme-top.png` | No inbound ref anywhere. |

(Keep `site-home.png`, `site-security.png` if they're in CHANGELOG; otherwise delete those too. None are referenced from live docs.)

### Stale superseded planning artefacts (2 paths)
| Path | Why |
|---|---|
| `/home/reuben/workspace/reposix/.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/` (15 files) | Phase 30 was superseded by Phases 40–45 of v0.10.0. STATE.md notes "Legacy Phase 30 entry retained in ROADMAP.md as `<details>` traceability block but not executed." The CONTEXT/PLAN/RESEARCH/PATTERNS files inside the phase dir are not archived under `.planning/milestones/v0.9.0-phases/30-...` like every other phase. **Either move to `.planning/milestones/v0.9.0-phases/` for archival convention parity, OR delete after extracting any still-applicable content into v0.10.0 archived phases.** Current placement is the only off-pattern phase dir in the entire planning tree. |
| `/home/reuben/workspace/reposix/.planning/SESSION-5-RATIONALE.md`, `SESSION-7-BRIEF.md` | Owner-flagged in CATALOG-v1 as "keep, historical." On second look these are session-anchored and zero outbound links. Move to `.planning/archive/sessions/`. |
