# REFACTOR

← [index](./index.md)

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
