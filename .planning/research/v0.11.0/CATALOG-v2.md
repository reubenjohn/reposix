# v0.11.0 CATALOG-v2 (cold review)

> **Cold-eyed re-audit, 2026-04-25 morning.** Read-only walk of all 679 git-tracked files. The previous overnight `.planning/CATALOG.md` is taken as input but every claim was re-verified against the current tree; several CATALOG-v1 statements are now stale (FUSE-era release.yml line 75 is fixed; `crates/reposix-swarm/src/fuse_mode.rs` is already deleted; `Mode::Fuse` is gone). This catalog supersedes v1 for the v0.11.0 cleanup pass.

## Headline

- **679** files audited (vs CATALOG-v1's 619 — an additional 60 net-new files landed overnight: 3 new ADRs, 3 new CLI subcommands and tests, 5 examples, blog post, RELEASE-NOTES, asciinema script, etc.).
- **DELETE: 24** files. Mostly FUSE-era demo scripts/recordings, broken stubs, orphan screenshots, archival walkthrough.
- **REFACTOR: 38** files. Tech debt clusters around (1) FUSE residue in code/doc comments, (2) duplicated `cache_path_from_worktree`/`backend_slug_from_origin`/`git_config_get` triplet across 4 CLI modules, (3) `refresh.rs`'s dead FUSE guard.
- **REVIEW: 11** files. Owner judgement calls (blog post wiring, screenshots, release-notes location, tag-script duplication).
- **KEEP: ~605** files including all of `.planning/milestones/` (write-once log).

## Top recommendations (the 10 highest-value moves)

The owner explicitly invited cold-hard decisions. These are ordered by ratio of "improves new-reader grasp" to "lines of work":

1. **Bump `Cargo.toml` workspace `version` from `0.9.0` to `0.10.0`** (and follow with `0.11.0-pre`). v0.10.0 shipped + tagged but the package metadata still says 0.9.0. `cargo run -p reposix-cli -- --version` lies. **One-line fix; high embarrassment-on-discovery cost.**
2. **Delete `scripts/demo.sh`.** It's a shim execing `scripts/demos/full.sh`, which was deleted in the v0.9.0 pivot. A new contributor running `bash scripts/demo.sh` gets a confusing `No such file or directory`. The CHANGELOG link from v0.2.0-alpha is the only reason to keep it; cite history in the changelog instead.
3. **Delete the entire FUSE-era demo set under `scripts/demos/`** (`01-edit-and-push.sh`, `02-guardrails.sh`, `03-conflict-resolution.sh`, `08-full-backend-showcase.sh`, `_lib.sh`, `_record.sh`, `assert.sh`, `parity.sh`, `parity-confluence.sh`, `smoke.sh`, `swarm.sh`, `04-token-economy.sh` keep). All except 04 invoke `reposix-fuse` / `reposix mount`; replaced by `scripts/dark-factory-test.sh` + the agent-flow skill. Six scripts can be deleted outright; replace `_lib.sh`/`assert.sh` if you need them for `04-token-economy.sh`.
4. **Delete the FUSE-era recordings under `docs/demos/recordings/`** plus the orphaned `docs/demo.transcript.txt` + `docs/demo.typescript` at root. Each recording lists `reposix-fuse` as a workspace crate. They have ZERO inbound links from the live mkdocs nav (`demos/index.md` is in `not_in_nav`). Cold reader who finds them will be confused. Re-record after v0.11.0 if needed; until then, delete.
5. **Delete `docs/demos/index.md`.** Tier 1/2/3 demo lineup explicitly references the deleted `scripts/demos/full.sh`. Already in `not_in_nav`. The new equivalent is `docs/tutorials/first-run.md` + the `examples/` directory. One stale page, four dead links.
6. **Consolidate the four `cache_path_from_worktree` / `backend_slug_from_origin` / `git_config_get` triplets** into `crates/reposix-cli/src/worktree.rs`. They're verbatim-copied across `doctor.rs`, `history.rs`, `gc.rs`, `tokens.rs`. Each duplication is ~50 lines including tests; net saving is ~150 lines and one source of truth for how the CLI maps a working tree → cache path. **Single highest-leverage code refactor.**
7. **Extract / inline `crates/reposix-cache/src/cli_compat.rs`.** It's a 252-line v0.8 `refresh_meta` schema clearly tagged "Phase-31 Plan-02 holdover; refresh subcommand will migrate in Phase 35." Phase 35 shipped without the migration (the cache crate has BOTH a `cli_compat` module and a `db` module, two SQLite schemas). Either (a) migrate `refresh.rs` to use `cache::db` and delete `cli_compat.rs`, or (b) move `cli_compat` back into `reposix-cli` as `cache_db.rs` (where it lived pre-Phase-31). Current placement contaminates the cache crate's public API.
8. **Delete `refresh.rs`'s `is_fuse_active` guard + `mount_point` field name.** It checks for `.reposix/fuse.pid` which is never created in v0.9.0+. The check IS harmless (always returns `false`) but the doc comment, error message, and field name are pure FUSE residue that confuses any reader of `cargo doc`. Rename `mount_point: PathBuf` → `working_tree: PathBuf` in lockstep. Catalog v1 noted this; it remains undone.
9. **Delete `MORNING-WALKTHROUGH-2026-04-25.md`, `RELEASE-NOTES-v0.10.0.md`, `RELEASE-NOTES-v0.11.0-PREVIEW.md`, `PUBLIC-LAUNCH-CHECKLIST.md` from the repo root.** They were cathartic to write but they are session journals and one-off launch checklists. CHANGELOG.md already covers the release notes. The walkthrough is dated and obsolete. Move to `.planning/archive/2026-04-25-launch/` if archival value is desired; otherwise just delete (git history preserves them).
10. **Sweep the ~12 FUSE-era doc comments out of source files in one mechanical pass.** `crates/reposix-core/src/lib.rs` (line 3 "FUSE daemon"), `core/src/path.rs` (lines 3,13), `core/src/backend.rs` (lines 7,9,22,72,106), `core/src/backend/sim.rs` (lines 499,629-632,909), `core/src/project.rs` (line 8), `confluence/src/lib.rs` (lines 433, 1484), `cli/src/refresh.rs` (lines 55, 63, 75, 77, 248-275, 368-383), `remote/src/fast_import.rs` (line 5). All cosmetic but they're the first thing a new reader sees in `cargo doc`. Single-PR ~30-line diff.

---

## DELETE

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

---

## REFACTOR

### Code-refactor cluster A — CLI worktree-context helpers (4 files, ~150 LOC)
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/crates/reposix-cli/src/doctor.rs` | Defines `git_config_get` (line 705), `backend_slug_from_origin` (line 741) plus inline `cache_path_from_worktree` logic at line 264. | Extract the trio into a new `crates/reposix-cli/src/worktree.rs` module. |
| `/home/reuben/workspace/reposix/crates/reposix-cli/src/history.rs` | Defines `cache_path_from_worktree` (line 25), `backend_slug_from_origin` (line 43), `git_config_get` (line 53). **Verbatim copy.** | Use `crate::worktree::*`. |
| `/home/reuben/workspace/reposix/crates/reposix-cli/src/gc.rs` | Defines `cache_path_from_worktree` (line 146), `backend_slug_from_origin` (line 186), `git_config_get` (line 196). **Verbatim copy.** | Use `crate::worktree::*`. |
| `/home/reuben/workspace/reposix/crates/reposix-cli/src/tokens.rs` | Defines `cache_path_from_worktree` (line 273), `backend_slug_from_origin` (line 297), `git_config_get` (line 307). **Verbatim copy.** | Use `crate::worktree::*`. |

Justification: 4× duplication is the single most violating-DRY pattern in the codebase. The four modules accreted these helpers as they were added in sequence overnight.

### Code-refactor cluster B — `parse_remote_url` is split between `core` and `remote`
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/crates/reposix-core/src/remote.rs` | `pub fn parse_remote_url(url) -> Result<RemoteSpec>` returning `{origin, project}`. Used by 4 CLI subcommands (doctor, history, gc, tokens). | Keep — this is the canonical core URL parser. |
| `/home/reuben/workspace/reposix/crates/reposix-remote/src/backend_dispatch.rs` | `pub fn parse_remote_url(url) -> Result<ParsedRemote>` returning `{kind, origin, project}` with extra Confluence/JIRA `/jira/`,`/confluence/` segment marker handling. | **Refactor to wrap core's `parse_remote_url`** and add only the `BackendKind` resolution on top. Currently this is a separate ~50-line parser that re-implements the prefix-strip and `/projects/` split. Keeping two URL parsers in two crates with the same name is a footgun. |

### Code-refactor cluster C — `crates/reposix-cache/src/cli_compat.rs`
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/crates/reposix-cache/src/cli_compat.rs` | 252-line module providing the v0.8 `refresh_meta` SQLite schema for the `reposix refresh` subcommand. Self-doc says "this is intentionally SEPARATE from the new `reposix_cache::db` module which owns the v0.9.0 `cache_schema.sql`. The CLI's refresh subcommand will migrate to the new schema in Phase 35 (CLI pivot); until then, the two coexist." Phase 35 shipped without the migration. | **Decision required:** either (a) migrate `refresh.rs` to use `cache::db` and delete `cli_compat.rs` (removes anyhow dep from the cache crate; cache becomes single-schema), OR (b) move `cli_compat.rs` back into `reposix-cli` as `crates/reposix-cli/src/cache_db.rs` and update the `reposix_cli::cache_db` re-export shim accordingly. Either path beats the current state where the cache crate is contaminated by a CLI-layer schema with a different SQLite layout. |

### Code-refactor cluster D — `refresh.rs` FUSE residue (1 file)
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/crates/reposix-cli/src/refresh.rs` | `RefreshConfig.mount_point: PathBuf`; `pub fn is_fuse_active(mount: &Path) -> Result<bool>` (lines 248-275); `if is_fuse_active(...)` guard (line 75); error message "FUSE mount is active at {} run `reposix unmount` first"; `.gitignore` includes `fuse.pid` line; doc comments lines 55, 63 reference FUSE; tests in `#[cfg(test)]` block depend on `is_fuse_active`. | Delete `is_fuse_active` and its 3 unit tests (`fuse_active_with_live_pid`, `fuse_inactive_no_pid_file`, `fuse_inactive_dead_pid`). Rename `mount_point` → `working_tree` in `RefreshConfig` and update the 8 call sites in `tests/refresh_integration.rs` + `tests/cli.rs`. Drop `fuse.pid` from the `.gitignore` writeline. |

### FUSE-residue doc comments (cosmetic; ~12 files; 30 LOC total)
| Path | Specifically |
|---|---|
| `/home/reuben/workspace/reposix/crates/reposix-core/src/lib.rs` | line 3-4 "the FUSE daemon" — replace with "the cache materializer". |
| `/home/reuben/workspace/reposix/crates/reposix-core/src/path.rs` | lines 3, 13 — FUSE boundary / FUSE `tree/` overlay framing. |
| `/home/reuben/workspace/reposix/crates/reposix-core/src/backend.rs` | lines 7, 9, 22, 72, 106 — historical FUSE references; rephrase as "v0.1 FUSE / v0.9 partial-clone working tree". |
| `/home/reuben/workspace/reposix/crates/reposix-core/src/backend/sim.rs` | line 499 + lines 629-632 + 909 — FUSE-write-path framing. |
| `/home/reuben/workspace/reposix/crates/reposix-core/src/project.rs` | line 8 — "FUSE mount" in doc comment. |
| `/home/reuben/workspace/reposix/crates/reposix-confluence/src/lib.rs` | lines 433, 1484 — "FUSE `read()`" + "tree/ overlay". |
| `/home/reuben/workspace/reposix/crates/reposix-remote/src/fast_import.rs` | line 5 — "FUSE read path" reference. |
| `/home/reuben/workspace/reposix/crates/reposix-cli/src/main.rs` | lines 4-12 — top-level subcommand docstring lists 6 commands; the actual `Cmd` enum has at least 9 (doctor, gc, history, tokens added overnight). |

### FUSE-era docs (5 files) — owner-flagged but still present
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/docs/development/contributing.md` | Workspace tree includes `reposix-fuse/`; lists `fusermount3 --version` as a prereq. Already in `not_in_nav`. | Either rewrite for v0.9.0 or **delete entirely**. The repo-root `CONTRIBUTING.md` (148 lines, FUSE-free) is the live one. **Recommendation: delete** — having two contributing docs is more confusing than having only one. |
| `/home/reuben/workspace/reposix/docs/development/roadmap.md` | Stops at v0.7. Mentions macFUSE / ProjFS / WinFsp. Already in `not_in_nav`. | **Delete.** `.planning/ROADMAP.md` is the live roadmap. The `docs/development/` sub-tree should be abolished. |
| `/home/reuben/workspace/reposix/docs/reference/cli.md` | Lists deleted `mount` and `demo` subcommands. Already in `not_in_nav`. | **Delete or rewrite.** A new contributor opens reference/ and the live pages link to `simulator.md`, `testing-targets.md`, `jira.md`, `confluence.md`. Missing: `cli.md` for the live `reposix init/list/refresh/sim/spaces/version/doctor/gc/history/tokens` subcommand surface. **Recommendation: rewrite as the v0.9.0+ CLI reference and add to nav.** |
| `/home/reuben/workspace/reposix/docs/reference/crates.md` | Lists `reposix-fuse`. Missing `reposix-cache`, `reposix-jira`. Already in `not_in_nav`. | **Rewrite or delete.** With 9 workspace crates, a one-line-each crates index is genuinely useful for new contributors. |
| `/home/reuben/workspace/reposix/docs/reference/git-remote.md` | Capabilities listed are `import`/`export`/`refspec`; missing `stateless-connect`. References "the FUSE daemon". Already in `not_in_nav`. | **Rewrite for v0.9.0.** The git remote helper is the architectural keystone; not having it in `reference/` is a gap. |
| `/home/reuben/workspace/reposix/docs/reference/http-api.md` | Two FUSE references in passing. Already in `not_in_nav`. | Two-line annotation; cheap fix; bring it back into nav. |

### Decision-record annotations (3 files)
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/docs/decisions/001-github-state-mapping.md` | References "FUSE-mounted views"; mapping itself unchanged. | Add header: "v0.9.0 supersedes the FUSE-rendering claim; the status mapping is unchanged and authoritative." |
| `/home/reuben/workspace/reposix/docs/decisions/002-confluence-page-mapping.md` | "Option A: flat" supplanted by ADR-003; ADR-003 supplanted by partial-clone working tree. | Add "scope superseded by v0.9.0 working tree layout" header. |
| `/home/reuben/workspace/reposix/docs/decisions/003-nested-mount-layout.md` | Scope: "the FUSE mount root layout emitted by reposix-fuse". | Add "obsolete; see how-it-works/filesystem-layer.md" header. |

### `mkdocs.yml` nav (1 file)
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/mkdocs.yml` | ADR-008 (helper-backend-dispatch) exists in `docs/decisions/` but is NOT in the `nav:` block under "Decisions". `docs/blog/2026-04-25-reposix-launch.md` is in the tree but not in `nav` and not in `not_in_nav`. **`mkdocs build --strict` will fail until ADR-008 is added.** | Add `- ADR-008 Helper backend dispatch: decisions/008-helper-backend-dispatch.md` to the Decisions list. Wire `docs/blog/` via the Material blog plugin OR add to `not_in_nav`. |

### Tag scripts — promotion candidate (7 files)
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/scripts/tag-v0.3.0.sh`, `tag-v0.4.0.sh`, `tag-v0.4.1.sh`, `tag-v0.5.0.sh`, `tag-v0.6.0.sh`, `tag-v0.8.0.sh`, `tag-v0.9.0.sh`, `tag-v0.10.0.sh` | 8 near-identical scripts, each ~120 lines, each enforcing "current branch = main / tree clean / tag doesn't already exist locally or upstream / CHANGELOG section exists / Cargo.toml version matches / cargo test green". The diff between any two is just the version literal and the milestone description. | **Promote to a single parametrized `scripts/tag-release.sh <version>` script** that derives all the version-specific pieces. Per CLAUDE.md OP #4: "Ad-hoc bash is a missing-tool signal." 8 near-clones is the textbook signal. Keep the historical tag scripts as committed artifacts (audit trail per CLAUDE.md OP #6) but stop generating new ones. **REVIEW (not auto-DELETE)** because they form an audit trail. |

### `.github/workflows/ci.yml` (1 file)
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/.github/workflows/ci.yml` | `integration-contract` (legacy GitHub octocat contract test) and `integration-contract-github-v09` (gated agent_flow_real on reubenjohn/reposix) both run on every push. They overlap conceptually — both prove "GitHub backend contract works against real API". `integration-contract-confluence` and `integration-contract-confluence-v09` similarly overlap. Comment line 103 references `HANDOFF.md` which was deleted. | Either consolidate the legacy `integration-contract*` jobs (4 jobs → 0) since the v09 jobs cover the same surface, OR clearly justify why both. Update the HANDOFF.md reference. |

### Social-asset builders — hardcoded paths (5 files)
| Path | Current state | Proposed change |
|---|---|---|
| `/home/reuben/workspace/reposix/docs/social/assets/_build_benchmark.py`, `_build_combined.py`, `_build_demo_gif.py`, `_build_hero_filebrowser.py`, `_build_workflow.py` | Every script hardcodes `/home/reuben/workspace/reposix/...`. Non-portable; would fail on any other contributor's machine. | Replace hardcoded paths with `Path(__file__).resolve().parent`. Pure mechanical fix; one-line per file. |

---

## REVIEW

| Path | Owner judgement needed because… |
|---|---|
| `/home/reuben/workspace/reposix/docs/blog/2026-04-25-reposix-launch.md` | Owner explicitly flagged: "owner may not want a blog directory at all." It's a 357-line launch post. NOT wired into mkdocs nav. Either (a) wire it in via Material blog plugin (full treatment), (b) move to `docs/release-history/v0.10.0-launch.md` (release-anchored), or (c) move to a personal blog repo and just link from CHANGELOG. |
| `/home/reuben/workspace/reposix/docs/demos/asciinema-script.md` | A 200-line shell-paste-ready screenplay for recording the launch screencast. It's a one-shot script masquerading as docs. Once the screencast is recorded and uploaded, this file's value drops to zero. **Decision:** keep until launch+1d, then delete or move to `.planning/archive/asciinema-launch.md`. |
| `/home/reuben/workspace/reposix/docs/screenshots/site-home.png` and `site-security.png` | Catalog-v1 says "keep" (refresh in Phase 44/45). I find no inbound link from any live page. If they're not promoted into README/index.md by v0.11.0, **delete**. |
| `/home/reuben/workspace/reposix/docs/social/twitter.md`, `linkedin.md` | Promo copy. Useful for launch only. After launch, **either move to `.planning/archive/social-2026-04-25/` or delete**. |
| `/home/reuben/workspace/reposix/scripts/take-screenshots.sh` | Stub that names a contract for screenshots that "deferred (cairo system libs unavailable)" per STATE.md. If never executed, **delete**; otherwise document the cairo prereq in `docs/development/` (or its successor). |
| `/home/reuben/workspace/reposix/.planning/MILESTONES.md`, `RETROSPECTIVE.md` | Both are living docs; no timestamping. Owner judgement: are these still tools the orchestrator + GSD agents read, or are they superseded by `STATE.md` + per-milestone audits? |
| `/home/reuben/workspace/reposix/scripts/v0.9.0-latency.sh` | One-shot latency-table regenerator for `docs/benchmarks/v0.9.0-latency.md`. As of v0.10.0, the table has `pending-secrets` cells; will it be re-run for v0.11.0 with real-backend numbers? If yes, **rename to `scripts/bench-latency.sh`** (parameterize over version). If no, **delete after the table is finalized**. |
| `/home/reuben/workspace/reposix/scripts/dev/list-confluence-spaces.sh` | Still functional, but is it ever run in practice? Could be replaced by a `reposix spaces` invocation. If `reposix spaces` covers it, **delete**. |
| `/home/reuben/workspace/reposix/.planning/v0.10.0-MILESTONE-AUDIT.md`, `v0.9.0-MILESTONE-AUDIT.md` | These are auditable artifacts but they sit at `.planning/` root; per pattern they should probably be under `.planning/milestones/v0.X.0-MILESTONE-AUDIT.md`. Owner-judgement on archival convention. |
| `/home/reuben/workspace/reposix/.planning/notes/phase-30-narrative-vignettes.md` | Status `ready-for-phase-30-planning` is stale; banned-word list inside is FUSE-era. CATALOG-v1 says "keep (annotate)" but the document is inert. **Either delete or rewrite to inherit from REQUIREMENTS.md DOCS-07.** |

---

## KEEP (summary, not exhaustive)

| Area | File count | Notes |
|---|---:|---|
| `crates/reposix-core/` (src + tests) | 20 | Clean public API; load-bearing compile-fail tests for SG-01/04/05. |
| `crates/reposix-cache/` (excl. cli_compat.rs) | 23 | Phase-31 baby; `gc.rs`, `sync_tag.rs`, `meta.rs` v0.11-vision-anchored. |
| `crates/reposix-cli/` (excl. FUSE residue + 4-way duplication) | 21 | Well-documented; `agent_flow_real.rs` is dark-factory keystone. |
| `crates/reposix-sim/` | 16 | Clean. |
| `crates/reposix-remote/` | 11 | `backend_dispatch.rs` closes the v0.9.0 carry-forward debt. |
| `crates/reposix-{confluence,github,jira}/` | 12 | Clean apart from cosmetic FUSE comments; ADF logic current. |
| `crates/reposix-swarm/` | 14 | `fuse_mode.rs` deletion already done (CATALOG-v1 outdated). |
| `docs/{index,concepts,how-it-works,tutorials,guides}/` | 11 | Post-v0.10.0 narrative; clean. |
| `docs/decisions/004..008/`, `docs/research/`, `docs/reference/{simulator,testing-targets,jira,confluence}.md`, `docs/benchmarks/v0.9.0-latency.md`, `docs/archive/` | 14 | Live or correctly historical. |
| `examples/` | 12 | Five working examples plus README; clean. |
| `benchmarks/` | 11 | Token-economy benchmark + fixture pairs; clean. |
| `.planning/milestones/` | 307 | Write-once log. |
| `.planning/research/` | 18 | Living research corpus. |
| `.github/` | 11 | `audit.yml`, `docs.yml`, `release.yml` clean (CATALOG-v1's release.yml broken-tarball claim is stale; verified line 77 lists `reposix reposix-sim git-remote-reposix` only — no reposix-fuse). |
| `.claude/` | 3 | Settings + 2 agent skills. Clean. |
| Root: Cargo.toml + 9 config files | 10 | Clean apart from Cargo.toml version-bump miss (top recommendation #1). |
| Root: README/CHANGELOG/CONTRIBUTING/SECURITY/COC/CLAUDE/mkdocs/pre-commit | 8 | Live; clean post-v0.10.0 (apart from `mkdocs.yml` ADR-008 nav-miss). |
| `scripts/` (banned-words-lint, bench_token_economy, check_*, dark-factory-test, green-gauntlet, install-hooks, hooks/, migrations/) | 12 | Live tooling. |

---

## Architecture observations

### Cross-cutting refactor opportunities

1. **CLI worktree-context module is missing.** Four CLI subcommands (`doctor`, `history`, `gc`, `tokens`) duplicate `cache_path_from_worktree`/`backend_slug_from_origin`/`git_config_get` verbatim. One `crates/reposix-cli/src/worktree.rs` module eliminates ~150 lines.

2. **URL parsing split between `core` and `remote`.** `remote::backend_dispatch::parse_remote_url` re-implements the prefix-strip, `/projects/` split, and slug validation rather than calling `core::parse_remote_url`. Have it wrap core's parser and layer `BackendKind` on top.

3. **Two SQLite schemas in one cache crate.** `reposix-cache` exposes `db::open_cache_db` (v0.9.0) AND `cli_compat::open_cache_db` (v0.8 refresh-meta). Same function name, different file path, different schema. Unify or relocate.

4. **No shared ADF helpers between Confluence and JIRA.** `confluence/src/adf.rs` (747 LOC) and `jira/src/adf.rs` (548 LOC) both implement ADF↔Markdown conversion with similar AST visitors. Not blocking for v0.11.0; next ROI cliff after worktree-helper extraction.

5. **No top-level binary surface document.** README hero mentions `reposix init`; CLAUDE.md mentions four binaries; nothing in `docs/` lists them. The `docs/reference/cli.md` rewrite (REFACTOR) closes this.

### Test-coverage observations (positive)

The compile-fail test pattern (`Tainted` ↔ egress-sink, `HttpClient::new` privacy) is a defensive crown jewel — load-bearing for SG-05; never delete. The `agent_flow_real.rs` gated-by-secret pattern is the right shape.

---

## Naming consistency

### Drift inventory

1. **`mount_point` vs `working_tree` in CLI.** `RefreshConfig.mount_point: PathBuf` is the only remaining FUSE-era field name. All v0.9.0+ subcommands use `working_tree` or `path`. Single-rename PR closes it.

2. **`Backend` vs `Connector` in trait names.** The trait is `BackendConnector` (post-Phase-27 rename from `IssueBackend`, ADR-004). But:
   - The concrete impls are named `SimBackend`, `GithubReadOnlyBackend`, `ConfluenceBackend`, `JiraBackend` (no `Connector` suffix).
   - The CLI uses `ListBackend` enum.
   - `BackendKind` enum in `backend_dispatch`.
   - The user-facing concept is "backend" (per CLAUDE.md, README, mkdocs).
   - Pattern: trait name has `Connector`, concrete-type names have `Backend`. **Inconsistent and confusing.** ADR-004 explicitly chose `BackendConnector` for the trait but didn't rename the impls. **Recommendation:** either rename the trait to `Backend` (would break `parse_remote_url` API and is a workspace breaking-change) or rename the impls to `SimConnector`/`GithubConnector`/etc. **Strong preference:** rename trait to `Backend` because (a) all user-facing nomenclature uses "backend", (b) `BackendConnector` is jargon, (c) it un-violates the "single concept = single name" rule.

3. **CLI module privacy is asymmetric.** `crates/reposix-cli/src/lib.rs` exposes `list`, `refresh`, `spaces` as `pub mod` (because integration tests need them) but `init`, `doctor`, `gc`, `history`, `sim`, `tokens`, `binpath` are private to `main.rs`. The split is "module needed by integration tests" → `lib.rs`; "module local to main" → `main.rs`. **It works** but the asymmetry is a tripwire for future contributors. **Recommendation:** declare ALL modules in `lib.rs` and have `main.rs` just contain the CLI dispatch. Existing convention in many Rust binaries.

4. **File extensions on demo recordings.** Some files are `.transcript.txt`, others are `.typescript`. `.typescript` is `script(1)` output (binary-ish, not TypeScript). Use `.script.log` or `.cast` (asciinema). Cosmetic but currently a rude surprise for anyone who opens `.typescript` expecting TypeScript code.

5. **ADR file numbering.** `001..008` is sequential, good. But `004-backend-connector-rename.md`, `006-issueid-to-recordid-rename.md`, `008-helper-backend-dispatch.md` are all renames/refactors; the others (`001`, `002`, `003`, `005`, `007`) are domain decisions. Worth distinguishing in titles? E.g. `006-rename-issueid-to-recordid.md`. Minor.

6. **Workspace crate-name plurality.** All crates are `reposix-<thing>` with `<thing>` singular (`core`, `cache`, `remote`, `cli`, `sim`, `confluence`, `jira`, `github`, `swarm`). Consistent — keep it.

7. **`scripts/demos/` vs `examples/`.** Both directories exist. `examples/` is the v0.10.0 canonical location for "shell-runnable agent loops"; `scripts/demos/` is the v0.1-era demo suite. After the deletions in DELETE, `scripts/demos/` should be empty (or contain only `04-token-economy.sh` until that's rehomed to `scripts/bench/`). **Then delete the directory.** A new reader who sees both `examples/` and `scripts/demos/` is confused which to look at.

8. **`docs/development/` vs root `CONTRIBUTING.md`.** Already covered in REFACTOR — pick one. Recommendation: delete `docs/development/`.

---

## Closing note

The codebase is 80%+ clean. The big-ticket items are:

- One Cargo.toml version-bump (item #1).
- One large mechanical sweep of FUSE-residue scripts/docs/recordings (items #2-5, #8-10).
- One small but high-leverage code refactor (item #6 — extract worktree-helper module).
- One architectural decision call (item #7 — `cli_compat.rs` direction).

After these four buckets land, a new reader cloning the repo cold should be able to read `README.md` → `docs/index.md` → `docs/tutorials/first-run.md` → `examples/01-shell-loop/run.sh` end-to-end without tripping over a single FUSE reference, dead link, or broken script. That's the bar.

The owner explicitly invited cold-hard decisions; the cold call is: **the v0.10.0 launch shipped clean enough; the v0.11.0 cleanup is real but bounded.** Two days of focused work closes everything in this catalog.
