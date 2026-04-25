# reposix file catalogue (2026-04-24/25)

**Total tracked files:** 619
**Auditor:** general-purpose subagent for the overnight polish session
**Authoritative:** this file is a SNAPSHOT — git history is canonical source.

> **Scope.** Read-only audit of every git-tracked file. Decisions on each: keep / rename / move / delete / rewrite-needed / investigate. The owner is preparing reposix for a publishable state after the v0.9.0 architecture pivot from FUSE to git-native partial clone; the v0.10.0 milestone (Docs & Narrative Shine) is currently scaffolded.
>
> **Working assumption.** v0.9.0 deleted `crates/reposix-fuse/` entirely. References to FUSE / `fuser` / `fusermount3` / `reposix mount` / `reposix demo` / `reposix-fuse` binary in *active code paths*, *user-facing docs*, *current CI*, and *current CLI surface* are stale. References to those terms in `.planning/milestones/` (archived phase records), `CHANGELOG.md` historical sections, `.planning/research/v0.1-fuse-era/`, and `docs/archive/` are correctly historical and stay.

---

## Top-level disposition matrix

| Bucket | Files | Default disposition | Exceptions |
|---|---:|---|---|
| `crates/reposix-core/` | 23 | keep | `IssueId`/`Issue` rename (owner-flagged), FUSE comments in `backend.rs` + `path.rs` + `backend/sim.rs` need rewrite |
| `crates/reposix-cache/` | 25 | keep | none — clean post-pivot crate (Phase 31) |
| `crates/reposix-cli/` | 14 | keep | `refresh.rs` retains `mount_point` field name + FUSE-active guard; `cli.rs` test still uses `mount_point` arg name; FUSE-residue migration tests are appropriate |
| `crates/reposix-remote/` | 11 | keep | `main.rs` hardcodes `SimBackend` (known tech debt — see `v0.9.0-MILESTONE-AUDIT.md` §1); `fast_import.rs` mentions FUSE in comment |
| `crates/reposix-sim/` | 15 | keep | clean |
| `crates/reposix-confluence/` | 5 | keep | `lib.rs` doc comment mentions FUSE history |
| `crates/reposix-github/` | 3 | keep | clean |
| `crates/reposix-jira/` | 4 | keep | clean |
| `crates/reposix-swarm/` | 14 | rewrite-needed | `fuse_mode.rs` + `Mode::Fuse` enum variant + `FuseWorkload` use are **dead code** (FUSE crate deleted) |
| `docs/concepts/` | 2 | keep | post-pivot, clean |
| `docs/how-it-works/` | 3 | keep | post-pivot, clean |
| `docs/tutorials/` | 1 | keep | post-pivot, clean |
| `docs/guides/` | 3 | keep | post-pivot, clean |
| `docs/reference/` | 8 | rewrite-needed | `cli.md`, `crates.md`, `git-remote.md`, `confluence.md`, `jira.md`, `http-api.md` all FUSE-era; `simulator.md` + `testing-targets.md` clean |
| `docs/decisions/` | 5 | keep (annotate) | ADR-001/002/003 reference deleted FUSE layer; need scope-superseded notice; ADR-004/005 are recent |
| `docs/research/` | 2 | keep | both have explicit pre-v0.1 status banners; correctly historical |
| `docs/archive/` | 2 | keep | already labelled archived |
| `docs/development/` | 2 | rewrite-needed | `roadmap.md` stops at v0.7; `contributing.md` lists `reposix-fuse/` |
| `docs/connectors/`, `docs/architecture.md`, `docs/security.md`, `docs/why.md`, `docs/demo.md` | 5 | keep | redirect-stub pages — all already converted to "(moved)" pointers |
| `docs/index.md` | 1 | keep | post-pivot hero, clean |
| `docs/benchmarks/v0.9.0-latency.md` | 1 | keep | sim column populated; real cells `pending-secrets` |
| `docs/social/` | 17 | keep | promo assets; demo gif is FUSE-era (flagged but low priority) |
| `docs/screenshots/` | 6 | keep | mostly v0.2/landing |
| `docs/demos/` | 13 | rewrite-needed | `index.md` heavily FUSE-era; recordings are FUSE-era artefacts (delete or re-record per Phase 45) |
| `scripts/demos/*.sh` | 16 | rewrite-needed/delete | most reference deleted `reposix mount`/`reposix-fuse` binary; `dark-factory-test.sh` replaces them |
| `scripts/dev/*.sh` | 4 | delete | `test-bucket-index.sh`, `test-tree-index.sh`, `probe-confluence.sh`, `list-confluence-spaces.sh` — `bucket`/`tree` index were FUSE features |
| `scripts/migrations/*.py` | 2 | keep | one-time migrations; harmless |
| `scripts/hooks/*` | 2 | keep | pre-push credential guard still relevant |
| `scripts/tag-v0.X.0.sh` | 7 | keep | each gates its own historical release (auditable) |
| `scripts/*` other | 8 | keep mostly | `demo.sh` (top-level shim) is FUSE-era — delete or repoint to `dark-factory-test.sh` |
| `benchmarks/` | 11 | keep | token-economy benchmark; all four fixture pairs current |
| `.github/workflows/` | 3 | rewrite-needed | `release.yml` still tarballs `reposix-fuse` binary (broken); `ci.yml` recently updated; `docs.yml` clean |
| `.claude/` | 2 | keep | `settings.json`, `reposix-agent-flow/SKILL.md` (post-pivot) |
| `.planning/` (top + active phases + active milestone) | 14 | keep | living planning artefacts |
| `.planning/milestones/v0.X.0-phases/` | 307 | keep | historical phase records — archival; do not edit |
| `.planning/research/v0.1-fuse-era/` | 4 | keep | path explicitly marks era |
| `.planning/research/v0.9-fuse-to-git-native/` | 9 | keep | the design doc set for the pivot — load-bearing |
| `.planning/research/v0.10.0-post-pivot/` | 1 | keep | active milestone research |
| `.planning/notes/` | 2 | keep (annotate) | `phase-30-narrative-vignettes.md` predates pivot; banned-word list updated in REQUIREMENTS — note that |
| `.planning/archive/scripts/` | 3 | keep | historical |
| Root: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `LICENSE-{MIT,APACHE}`, `.gitignore`, `.env.example` | 9 | keep | clean |
| Root: `README.md`, `CHANGELOG.md`, `mkdocs.yml`, `CLAUDE.md` | 4 | rewrite-needed (README) / keep (others) | README has FUSE-era Quickstart sections + 27 FUSE refs |
| Root: `HANDOFF.md` | 1 | delete | v0.7-era doc; superseded by `.planning/STATE.md` + `.planning/MILESTONES.md` + `v0.9.0-MILESTONE-AUDIT.md` |

---

## Bucket-by-bucket review

### `crates/reposix-core/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean |
| `src/lib.rs` | keep | exposes the eight pub re-exports cleanly; module structure stable |
| `src/issue.rs` | rewrite-needed | **OWNER FLAGGED:** `IssueId` too narrow. See "Naming generalization" section below. Module name `issue.rs` should rename in lockstep. Also: `Issue.parent_id` doc still mentions "FUSE `tree/` overlay" history — fold into the v0.4 historical clause. |
| `src/error.rs` | keep | clean (`InvalidIssue` variant would also rename if `Issue` does) |
| `src/backend.rs` | rewrite-needed | Doc comments lines 7-13 still describe "FUSE daemon historically spoke the sim's REST shape directly" + "FUSE layer and CLI orchestrator". `BackendFeature::Hierarchy` doc says "Used by FUSE to synthesize the `tree/` overlay" — true historically; rephrase as "exposed in pre-v0.9.0 FUSE; semantically still meaningful for clients that want a tree." |
| `src/backend/sim.rs` | rewrite-needed | Line 499 comment "the FUSE write path's conflict-current log line" — drop FUSE-specific framing. `mount(...)` calls (lines 399, 415, 430, 456, 509, 549) are wiremock `mount`, not FUSE — fine. |
| `src/path.rs` | rewrite-needed | Lines 3, 13: "FUSE boundary (Phase 3)" and "FUSE `tree/` overlay" mentioned in module-level docs. The functions themselves (`validate_issue_filename`, `slug_or_fallback`, `dedupe_siblings`) are still relevant for partial-clone working tree filename hygiene; just rephrase the framing. |
| `src/project.rs` | keep | clean |
| `src/remote.rs` | keep | `RemoteSpec` parsing — load-bearing for the helper |
| `src/audit.rs` | keep | clean; `BEFORE UPDATE/DELETE` triggers + `SQLITE_DBCONFIG_DEFENSIVE` are the SG-06 keystone |
| `src/http.rs` | keep | the sealed `HttpClient` factory — workspace-wide allowlist enforcement seam |
| `src/taint.rs` | keep | `Tainted<T>` / `Untainted<T>` — load-bearing for SG-05 |
| `examples/show_audit_schema.rs` | keep | small example, clean |
| `fixtures/audit.sql` | keep | schema fixture |
| `tests/audit_schema.rs` | keep | invariant test for SG-06 |
| `tests/http_allowlist.rs` | keep | SG-01 invariant test |
| `tests/compile_fail.rs` + `tests/compile-fail/*.{rs,stderr}` (6 files) | keep | type-system invariants for `Tainted` discipline + sealed `HttpClient`; load-bearing |

### `crates/reposix-cache/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | gix=0.82, dirs=6 — pinned deliberately |
| `src/lib.rs` | keep | clean post-pivot module structure |
| `src/cache.rs` | keep | the `Cache` API — central object |
| `src/builder.rs` | keep | `build_from`/`SyncReport` |
| `src/audit.rs` | keep | append-only audit per ARCH-02; `helper_push_rejected_conflict` op vocabulary is stable |
| `src/db.rs` | keep | SQLite schema + WAL configuration |
| `src/error.rs` | keep | clean |
| `src/meta.rs` | keep | `last_fetched_at` + delta sync metadata |
| `src/path.rs` | keep | XDG cache dir resolution |
| `src/cli_compat.rs` | keep | provides `reposix_cli::cache_db::*` shim — should stay until refresh integration test imports are migrated; explicitly tagged in lib.rs as a Phase-31 Plan-02 holdover |
| `src/sink.rs` | keep | `#[doc(hidden)]` — privileged-sink stubs for compile-fail tests |
| `fixtures/cache_schema.sql` | keep | schema fixture |
| `tests/audit_is_append_only.rs` | keep | SG-06 enforcement |
| `tests/blobs_are_lazy.rs` | keep | ARCH-01 invariant |
| `tests/common/mod.rs` | keep | shared test setup |
| `tests/compile-fail/*.{rs,stderr}` (4 files) | keep | type-system invariants for `Tainted` ↔ egress sink; load-bearing |
| `tests/compile_fail.rs` | keep | trybuild driver |
| `tests/delta_sync.rs` | keep | ARCH-06/07 invariant |
| `tests/egress_denied_logs.rs` | keep | SG-01 + audit invariant |
| `tests/gix_api_smoke.rs` | keep | gix pin sanity |
| `tests/materialize_one.rs` | keep | lazy-blob invariant |
| `tests/tree_contains_all_issues.rs` | keep | tree-sync invariant |

### `crates/reposix-cli/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean |
| `src/main.rs` | rewrite-needed (minor) | `reposix init` uses a CLI arg `mount_point` for `Refresh` — should rename to `working_tree` or `path` (FUSE residue in identifier). Test `cli.rs` and tests in `refresh_integration.rs` would update in lockstep. |
| `src/lib.rs` | keep | clean |
| `src/init.rs` | keep | new in v0.9.0; clean |
| `src/list.rs` | keep | clean |
| `src/refresh.rs` | rewrite-needed (minor) | `RefreshConfig.mount_point: PathBuf` field + `is_fuse_active(&cfg.mount_point)` guard + the `FUSE mount is active` error message. The guard itself is harmless (checks for a `.reposix/fuse.pid` file that no longer exists), but the framing is misleading post-pivot. **Decision:** delete the `is_fuse_active` check entirely (the file it looks for is never created in v0.9.0+) and rename `mount_point` to `working_tree`. |
| `src/sim.rs` | keep | sim-spawn wrapper |
| `src/spaces.rs` | keep | Confluence space listing |
| `src/binpath.rs` | keep | binary discovery helper |
| `tests/cli.rs` | keep (annotate) | tests assert `mount`/`demo` are *removed* — these are intentional regression tests for the v0.9.0 breaking change; the references are correct. |
| `tests/agent_flow.rs` | keep | sim path of dark-factory regression |
| `tests/agent_flow_real.rs` | keep | real-backend gated tests (TokenWorld, github, JIRA TEST) |
| `tests/refresh_integration.rs` | rewrite-needed (minor) | `refresh_fuse_active_guard` test — once `is_fuse_active` is deleted, this whole test goes; rename surviving tests' use of `mount_point: dir.path().to_path_buf()` to `working_tree`. |
| `tests/no_truncate.rs` | keep | `--no-truncate` flag invariant |

### `crates/reposix-remote/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean |
| `src/main.rs` | rewrite-needed | **Known tech debt (audit §1):** `SimBackend` is hardcoded at lines 20, 98 with the comment "v0.9.0 sim-only phase this is hardcoded to `\"sim\"`; Phase 35 will derive it from the parsed remote URL for real backends." Phase 35 did not address it — owner-tracked work for v0.10.0/v0.11.0. |
| `src/protocol.rs` | keep | stdin/stdout pktline framing |
| `src/pktline.rs` | keep | clean |
| `src/stateless_connect.rs` | keep | the `stateless-connect` capability handler (ARCH-04) |
| `src/fast_import.rs` | rewrite-needed (cosmetic) | Comment line 5 references "FUSE read path uses". The fact that the same `frontmatter::render` is used by the helper and (historically) by FUSE is no longer true — rephrase as "the same `frontmatter::render` the cache uses on materialization". |
| `src/diff.rs` | keep | bulk-delete cap + push planner |
| `tests/protocol.rs` | keep | pktline invariants |
| `tests/stateless_connect.rs` | keep | ARCH-04 invariant |
| `tests/push_conflict.rs` | keep | ARCH-08 invariant |
| `tests/bulk_delete_cap.rs` | keep | SG-02 invariant |

### `crates/reposix-sim/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean |
| `src/main.rs` | keep | clean |
| `src/lib.rs` | keep | clean |
| `src/db.rs` | keep | clean |
| `src/error.rs` | keep | clean |
| `src/seed.rs` | keep | seed loader |
| `src/state.rs` | keep | clean |
| `src/middleware/mod.rs` | keep | clean |
| `src/middleware/audit.rs` | keep | per-request audit insertion |
| `src/middleware/rate_limit.rs` | keep | clean |
| `src/routes/mod.rs` | keep | clean |
| `src/routes/issues.rs` | keep | core sim shape |
| `src/routes/transitions.rs` | keep | JIRA-style transitions for parity tests |
| `fixtures/seed.json` | keep | demo seed; load-bearing for tutorials |
| `tests/api.rs` | keep | sim API contract |

### `crates/reposix-confluence/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean |
| `src/lib.rs` | rewrite-needed (cosmetic) | comment lines 1-7 reference "FUSE / mount path"; rephrase post-pivot. ADF logic is current. |
| `src/adf.rs` | keep | Markdown ↔ ADF converter |
| `tests/contract.rs` | keep | parameterized contract test (sim + wiremock + live) |
| `tests/roundtrip.rs` | keep | end-to-end create→fetch→delete |

### `crates/reposix-github/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean |
| `src/lib.rs` | keep | `GithubReadOnlyBackend` impl |
| `tests/contract.rs` | keep | parameterized contract test (sim + wiremock + live `octocat/Hello-World`) |

### `crates/reposix-jira/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean |
| `src/lib.rs` | keep | `JiraBackend` impl (read+write per Phase 28/29) |
| `src/adf.rs` | keep | shared with reposix-confluence — confirm no duplication once the `crates.md` rewrite happens |
| `tests/contract.rs` | keep | live + wiremock |

### `crates/reposix-swarm/`

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | rewrite-needed (cosmetic) | description still mentions "FUSE mount" |
| `src/lib.rs` | rewrite-needed | line 22 `pub mod fuse_mode;` — delete |
| `src/main.rs` | rewrite-needed | `Mode::Fuse` enum variant + `FuseWorkload` use + `--target` doc + `Mode::Fuse` match arm — DELETE every reference |
| `src/fuse_mode.rs` | **DELETE** | The whole module is "real syscalls against a FUSE mount" — **dead code** post-v0.9.0; FUSE crate is gone, so a FUSE swarm is not possible |
| `src/driver.rs` | keep | swarm driver |
| `src/workload.rs` | rewrite-needed (cosmetic) | comment line 10 references `crate::fuse_mode` — drop |
| `src/sim_direct.rs` | keep | clean |
| `src/confluence_direct.rs` | keep | clean |
| `src/contention.rs` | keep | If-Match contention test mode |
| `src/metrics.rs` | keep | HDR histograms |
| `tests/chaos_audit.rs` | keep | audit chaos invariant |
| `tests/contention_e2e.rs` | keep | clean |
| `tests/confluence_real_tenant.rs` | keep | live `#[ignore]` smoke |
| `tests/mini_e2e.rs` | keep | clean |

### `docs/` review

#### `docs/index.md`
**Disposition:** keep. Hero, three measured numbers, "Tested against" — fully post-pivot, references `v0.9.0-latency.md`. Owner of v0.10.0 Phase 40.

#### `docs/concepts/{mental-model-in-60-seconds.md, reposix-vs-mcp-and-sdks.md}`
**Disposition:** keep. Both written for the v0.10.0 milestone; clean.

#### `docs/how-it-works/{filesystem-layer.md, git-layer.md, trust-model.md}`
**Disposition:** keep. The post-pivot trio (Phase 41). Each has a mermaid diagram. `filesystem-layer.md` correctly frames the v0.1 FUSE design as superseded.

#### `docs/tutorials/first-run.md`
**Disposition:** keep. The 5-minute tutorial — load-bearing for DOCS-06.

#### `docs/guides/{write-your-own-connector.md, integrate-with-your-agent.md, troubleshooting.md}`
**Disposition:** keep. Newly written for v0.10.0 Phase 42. `troubleshooting.md` references a `[remote rejected] main -> main (fetch first)` flow that maps to ARCH-08.

#### `docs/reference/`
| File | Disposition | Notes |
|---|---|---|
| `cli.md` | rewrite-needed | Lists the deleted `mount` and `demo` subcommands, calls reposix "git-backed FUSE filesystem". Needs full rewrite for the v0.9.0 CLI (`init`, `list`, `refresh`, `sim`, `spaces`, `version`). |
| `crates.md` | rewrite-needed | Lists `reposix-fuse` as a workspace crate; missing `reposix-cache` + `reposix-jira`; references `IssueBackend` (renamed); still says "Ships in v0.2/v0.3". Needs full re-write. |
| `git-remote.md` | rewrite-needed | Capabilities listed as `import`/`export`/`refspec` only — missing `stateless-connect`. Says "the same function the FUSE daemon uses" — pre-pivot. v0.2 backlog section is outdated. |
| `confluence.md` | rewrite-needed | "FUSE daemon and reposix list" + "Mount the space as a POSIX directory of Markdown files" + "## FUSE mount layout (v0.4+)" — full rewrite to match the v0.9.0 init/clone flow. |
| `jira.md` | rewrite-needed | Section "Mount as FUSE Filesystem" — replace with `reposix init jira::TEST /tmp/repo`. Otherwise content is recent (JIRA shipped Phase 28). |
| `http-api.md` | keep (annotate) | Documents the simulator's REST shape; references "FUSE" twice in pass-through context. Two-line annotation suffices. |
| `simulator.md` | keep | Already post-pivot (talks about the cache + helper). |
| `testing-targets.md` | keep | New for v0.9.0 Phase 36; documents the three sanctioned targets. One mention of "FUSE-free transport" — fine framing. |

#### `docs/decisions/`
| File | Disposition | Notes |
|---|---|---|
| `001-github-state-mapping.md` | keep (annotate) | References "FUSE-mounted views" as the user-facing surface. Add a brief note at the top: "v0.9.0 supersedes the FUSE-rendering claim; the status mapping itself is unchanged and authoritative." |
| `002-confluence-page-mapping.md` | keep (annotate) | "Option A: flat" decision is supplanted by ADR-003; ADR-003 is supplanted by the v0.9.0 partial-clone working tree. Add a "scope superseded" header note. |
| `003-nested-mount-layout.md` | keep (annotate) | Scope: "the FUSE mount root layout emitted by `reposix-fuse`" — codebase no longer exists. Add an "obsolete; see filesystem-layer.md" header. |
| `004-backend-connector-rename.md` | keep | Recent (Phase 27); clean. |
| `005-jira-issue-mapping.md` | keep | Recent (Phase 28); 1 stale mention of "FUSE filenames" — minor sweep. |

#### `docs/research/`
| File | Disposition | Notes |
|---|---|---|
| `initial-report.md` | keep | Has explicit "pre-v0.1 design research" banner; correctly historical. |
| `agentic-engineering-reference.md` | keep | The dark-factory / lethal-trifecta reference; load-bearing. |

#### `docs/archive/{MORNING-BRIEF.md, PROJECT-STATUS.md}`
**Disposition:** keep. Both have explicit "Archived" header banners; correctly historical.

#### `docs/development/`
| File | Disposition | Notes |
|---|---|---|
| `contributing.md` | rewrite-needed | Workspace tree includes `reposix-fuse/`; the FUSE-callbacks-are-safe-Rust paragraph; "fusermount3 --version" prereq. Needs an architectural-pivot pass. |
| `roadmap.md` | rewrite-needed | Stops at v0.7. v0.8 (JIRA), v0.9 (architecture pivot), v0.10 (docs) all missing. North-star list still mentions "macFUSE" + "ProjFS / WinFsp + reposix-fuse" — neither applies post-pivot. |

#### Redirect-stub pages
| File | Disposition | Notes |
|---|---|---|
| `architecture.md` | keep | "(moved)" stub pointing to how-it-works trio. |
| `security.md` | keep | "(moved)" stub pointing to trust-model. |
| `why.md` | keep | "(moved)" stub. |
| `demo.md` | keep | "(moved)" stub pointing to tutorials/first-run.md. |
| `connectors/guide.md` | keep | "(moved)" stub pointing to guides/write-your-own-connector. |

#### `docs/demos/`
| File | Disposition | Notes |
|---|---|---|
| `index.md` | rewrite-needed | "FUSE mount + cat/sed edit + git push round-trip" — full rewrite for v0.9.0. Tier 1/2/3 demo lineup needs to be re-cast: half the demos depended on `reposix mount`. |
| `recordings/01-edit-and-push.{transcript.txt, typescript}` | keep (or re-record) | FUSE-era recordings; either keep with an "archive" annotation or re-record per v0.10.0 Phase 45. README itself flags the demo gif as "FUSE-era — Phase 45 will re-record against `reposix init`". |
| `recordings/02-guardrails.{...}` | keep / re-record | Same. |
| `recordings/03-conflict-resolution.{...}` | keep / re-record | Same. |
| `recordings/04-token-economy.{...}` | keep | benchmark recording; not FUSE-tied. |
| `recordings/parity.{...}` | keep | sim-vs-real parity recording; FUSE-neutral. |
| `recordings/swarm.{...}` | keep | swarm load-test recording; only loosely FUSE-tied. |

#### `docs/social/` (17 files)
**Disposition:** keep. LinkedIn + Twitter copy + asset builders + rendered images. The architecture diagram (`architecture.mmd`/`.png`) and demo gif are FUSE-era — owner-flagged as Phase 45 work. Builder scripts (`_build_*.py`) are reusable.

#### `docs/screenshots/` (6 PNGs)
**Disposition:** keep. Site/landing screenshots — refresh in Phase 44/45.

#### `docs/benchmarks/v0.9.0-latency.md`
**Disposition:** keep. Sim column populated; real-backend cells `pending-secrets`. Regenerated by `scripts/v0.9.0-latency.sh`.

### `.planning/` review

#### Sources of truth (active)
| File | Disposition | Notes |
|---|---|---|
| `STATE.md` | keep | The cursor. |
| `PROJECT.md` | keep | Active milestone v0.10.0 + validated v0.9.0 sections. |
| `REQUIREMENTS.md` | keep | DOCS-01..11 active; v0.9.0 ARCH-* validated. |
| `MILESTONES.md` | keep | v0.9.0 entry. |
| `ROADMAP.md` | keep | Phases 40–45 detailed. |
| `RETROSPECTIVE.md` | keep | Living doc; historical FUSE-era milestones expected. |
| `v0.9.0-MILESTONE-AUDIT.md` | keep | The "tech_debt" verdict + carry-forward items. |
| `config.json` | keep | GSD configuration. |

#### Phase records (active phase dirs in `.planning/phases/`)
- `30-docs-ia-and-narrative-overhaul-...` — Phase 30 was deferred from v0.9.0 to v0.10.0 (renumbered to Phases 40–45). The phase 30 dir contains `09 PLAN.md` files + `OVERVIEW`, `RESEARCH`, `VALIDATION`, `PATTERNS`. **Disposition:** investigate — either fold this into the milestone archive (move to `.planning/milestones/v0.9.0-phases/30-...` like the other v0.9.0 phases), or delete after extracting any still-applicable content. STATE.md notes: "Legacy Phase 30 entry retained in ROADMAP.md as `<details>` traceability block but not executed." Strong recommendation: **move to `.planning/milestones/v0.9.0-phases/`** to match the rest of the historical convention.
- `40, 41, 42, 43` — active v0.10.0 phases. Each has CONTEXT.md + VERIFICATION.md so far.

#### `.planning/notes/`
| File | Disposition | Notes |
|---|---|---|
| `phase-30-narrative-vignettes.md` | keep (annotate) | Status `ready-for-phase-30-planning` is stale; banned-word list inside is for the FUSE era. REQUIREMENTS.md already has the revised git-native banned-word list. Add a header note: "Banned-word list superseded by REQUIREMENTS.md DOCS-07; vignette V1 still applicable." |
| `gsd-feedback.md` | keep | meta-tool feedback log |

#### `.planning/research/`
| Dir / File | Disposition | Notes |
|---|---|---|
| `v0.1-fuse-era/` (4 files) | keep | Path explicitly marks era; all FUSE refs valid. |
| `v0.9-fuse-to-git-native/` (9 files + `poc/` 4 files) | keep | The pivot design corpus; load-bearing for the architectural argument. POC artefacts are reproducible by `run-poc*.sh`. |
| `v0.10.0-post-pivot/milestone-plan.md` | keep | Active milestone source-of-truth. |

#### `.planning/milestones/`
**Disposition (default):** keep. 307 archived phase records from v0.1.0 → v0.9.0. Treat as a write-once log; do not edit. Subdirs are conventional (e.g. `v0.7.0-phases/22-bench-token-economy.../...`). Two top-level files in this dir are slightly off-pattern:
- `v0.8.0-REQUIREMENTS.md`, `v0.8.0-ROADMAP.md`, `v0.9.0-ROADMAP.md` — fine; per-milestone snapshots taken at archive time.

Per-milestone dir summary:
| Milestone | Files | Notes |
|---|---:|---|
| `v0.1.0-phases/` | 38 | original sim+FUSE+helper work |
| `v0.2.0-alpha-phases/` | 4 | github read-only adapter |
| `v0.3.0-phases/` | 19 | confluence read |
| `v0.4.0-phases/` | 24 | nested mount layout |
| `v0.5.0-phases/` | 16 | bucket `_INDEX.md` |
| `v0.6.0-phases/` | 68 | confluence write + tree-recursive index + refresh + others |
| `v0.7.0-phases/` | 63 | hardening + comments + attachments + whiteboards + docs reorg |
| `v0.8.0-phases/` | 31 | JIRA |
| `v0.9.0-phases/` | 41 | architecture pivot |

#### `.planning/archive/scripts/` (3 files)
**Disposition:** keep. Pre-v0.1 phase exit scripts; historical.

#### `.planning/SESSION-*.md`
- `SESSION-5-RATIONALE.md` (Phase 14 rationale) — keep, historical
- `SESSION-7-BRIEF.md` (Phase 16+ session brief) — keep, historical

### `.github/` review

| File | Disposition | Notes |
|---|---|---|
| `workflows/ci.yml` | keep (minor) | dark-factory job + integration-contract-{conf,gh,jira}-v09 jobs are live. The `integration-contract` (legacy github octocat) test is still wired alongside the new `-v09` variants — **redundant**; consolidate. |
| `workflows/docs.yml` | keep | clean; `mkdocs build --strict` + gh-deploy. |
| `workflows/release.yml` | rewrite-needed | **Broken:** tarball-staging step (line 75) `for bin in reposix reposix-sim reposix-fuse git-remote-reposix; do cp ... ; done` will fail because `reposix-fuse` no longer builds. Drop `reposix-fuse` from the binary list. |

### `.claude/` review

| File | Disposition | Notes |
|---|---|---|
| `settings.json` | keep | minimal allow-list |
| `skills/reposix-agent-flow/SKILL.md` | keep | The dark-factory regression skill; load-bearing for OP-4 (self-improving infrastructure) per CLAUDE.md. |

### `scripts/` review

| File | Disposition | Notes |
|---|---|---|
| `bench_token_economy.py` | keep | counts tokens via Anthropic API |
| `test_bench_token_economy.py` | keep | pytest harness |
| `check_clippy_lint_loaded.sh` | keep | invariant test for `clippy.toml` |
| `check_fixtures.py` | keep | benchmark fixture validator |
| `dark-factory-test.sh` | keep | the v0.9.0 regression script |
| `demo.sh` | rewrite-needed | shim that execs `scripts/demos/full.sh` (FUSE-era). Either delete the shim or repoint to `dark-factory-test.sh`. |
| `green-gauntlet.sh` | keep (audit) | full pre-tag check; verify no FUSE refs remain inside |
| `install-hooks.sh` | keep | git hooks installer |
| `tag-v0.3.0.sh` … `tag-v0.9.0.sh` (7 scripts) | keep | release-tag scripts; auditable |
| `v0.9.0-latency.sh` | keep | latency-table regenerator |
| `dev/list-confluence-spaces.sh` | keep | dev tool |
| `dev/probe-confluence.sh` | keep | dev tool |
| `dev/test-bucket-index.sh` | **delete** | tested FUSE bucket `_INDEX.md` synthesis — feature deleted with `reposix-fuse` |
| `dev/test-tree-index.sh` | **delete** | tested FUSE `tree/` overlay — feature deleted with `reposix-fuse` |
| `migrations/fix_demos_index_links.py` | keep | one-shot migration |
| `migrations/mermaid_divs_to_fences.py` | keep | one-shot migration |
| `hooks/pre-push` | keep | credential-leak guard |
| `hooks/test-pre-push.sh` | keep | hook self-test (wired into CI) |
| `demos/01-edit-and-push.sh` | rewrite-needed | uses `reposix mount` + `reposix-fuse` |
| `demos/02-guardrails.sh` | rewrite-needed | uses `reposix mount` |
| `demos/03-conflict-resolution.sh` | rewrite-needed | uses `reposix mount` (per `IssueBackend`-trace mention) |
| `demos/04-token-economy.sh` | keep (audit) | benchmark; verify it doesn't depend on the FUSE mount |
| `demos/05-mount-real-github.sh` | **delete** | mounts real GitHub via FUSE |
| `demos/06-mount-real-confluence.sh` | **delete** | mounts real Confluence via FUSE |
| `demos/07-mount-real-confluence-tree.sh` | **delete** | tree/ overlay demo for FUSE |
| `demos/08-full-backend-showcase.sh` | rewrite-needed | full-backend showcase; uses FUSE |
| `demos/_lib.sh`, `_record.sh` | keep (audit) | helpers; ensure they don't bake in FUSE assumptions |
| `demos/assert.sh` | keep | smoke-test asserter |
| `demos/full.sh` | **delete** | the 9-step Tier 2 walkthrough; entirely FUSE-based; replaced by `scripts/dark-factory-test.sh` |
| `demos/parity.sh` + `parity-confluence.sh` | keep (audit) | sim-vs-real parity comparison; should not depend on FUSE — verify |
| `demos/smoke.sh` | rewrite-needed | runs the Tier-1 demos; rewire to the surviving subset (or delete altogether) |
| `demos/swarm.sh` | keep (audit) | swarm-mode demo; verify it uses `sim-direct` not `fuse` mode |

### `benchmarks/` review

| File | Disposition | Notes |
|---|---|---|
| `README.md` | keep | clean |
| `RESULTS.md` | keep | historical results (v0.7-era 89.1% number) — current; v0.10.0 may add a v0.9.0-vintage refresh |
| `fixtures/README.md` | keep | clean |
| `fixtures/{github_issues, confluence_pages, mcp_jira_catalog, reposix_session}.{json/txt}` + `.tokens.json` siblings (8 files) | keep | benchmark fixtures + cached token counts |

### `requirements-bench.txt`
**Disposition:** keep. Python deps for the token-economy benchmark.

### Root files

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean; workspace v0.9.0; nine members |
| `Cargo.lock` | keep | committed |
| `rust-toolchain.toml` | keep | pins stable |
| `rustfmt.toml` | keep | clean |
| `clippy.toml` | keep | the `disallowed-methods` ban for `reqwest::Client::*` |
| `LICENSE-MIT`, `LICENSE-APACHE` | keep | dual license |
| `.gitignore` | keep | clean |
| `.env.example` | keep | clean |
| `mkdocs.yml` | keep | nav fully reflects post-pivot site (Concepts / How it works / Tutorials / Guides / Reference / Benchmarks / Decisions / Research). Redirect stubs are excluded via `not_in_nav`. |
| `README.md` | rewrite-needed | 27 FUSE refs; "Quickstart (v0.7.x — pre-FUSE-deletion)" section, the "Folder-structure mount" section, the swarm-tree mention, the demo gif annotation, the prebuilt-binary list — all need v0.9.0-shaped updates. **Owner-flagged for Phase 45.** |
| `CHANGELOG.md` | keep | post-pivot v0.9.0 entry is authoritative; older FUSE-era entries are correctly historical. |
| `CLAUDE.md` | keep | rewritten in Phase 36 to steady-state git-native; one transitional reference to `crates/reposix-fuse` "BEING DELETED in v0.9.0" remaining — minor cleanup once everyone agrees the deletion has fully landed. |
| `HANDOFF.md` | **delete** | v0.7-era doc; superseded by `.planning/STATE.md` + `.planning/MILESTONES.md` + `.planning/v0.9.0-MILESTONE-AUDIT.md`. Contains 26 FUSE refs, `OP-1..OP-11` (all closed), Phase 27+ direction (now shipped). Anything still load-bearing should migrate to `STATE.md` or to a `docs/development/` page first. |

---

## Cleanup decisions — prioritized work list

### Must do before public launch

1. **Fix release.yml** — `.github/workflows/release.yml:75` lists `reposix-fuse` in the binary tarball; this will break the next `git push v0.10.0`. One-line fix.
2. **Delete `crates/reposix-swarm/src/fuse_mode.rs` + `Mode::Fuse` enum + `FuseWorkload` references** — dead code that imports the deleted `reposix-fuse` crate is a ticking compile bomb. (Currently survives because the swarm crate doesn't actually depend on `reposix-fuse`; it `std::fs::*`s a path. Still: dead code.)
3. **Delete `scripts/demos/full.sh`, `scripts/demos/05-mount-real-github.sh`, `scripts/demos/06-mount-real-confluence.sh`, `scripts/demos/07-mount-real-confluence-tree.sh`, `scripts/dev/test-bucket-index.sh`, `scripts/dev/test-tree-index.sh`** — all reference the deleted `reposix-fuse` binary; if a contributor `bash`-runs any of them the error will be confusing.
4. **Rewrite `docs/reference/cli.md`** — currently advertises the deleted `mount` and `demo` subcommands. This is the single most user-misleading doc page in the repo.
5. **Rewrite `docs/reference/crates.md`** — lists `reposix-fuse` as a workspace crate; missing `reposix-cache` + `reposix-jira`; uses old `IssueBackend` name. This is the discoverability page for the Rust API surface.
6. **Delete `HANDOFF.md`** — v0.7-era stub doc with stale OP-1..OP-11 closed-items table that pretends to be the source of truth (the actual source is `.planning/STATE.md`). Risk: a new contributor reads HANDOFF and thinks the project is at v0.7.
7. **Rewrite `README.md` Quickstart section** — already partially flagged as "v0.10.0 Phase 45". The "Quickstart (v0.7.x — pre-FUSE-deletion)" section should be removed or moved to a History block; the new five-line quickstart for `reposix init sim::demo` should be the first runnable thing.

### Should do this milestone (v0.10.0 — Docs & Narrative Shine)

1. **Annotate ADR-001/002/003** with "scope-superseded by v0.9.0 partial-clone" headers. The mappings (status, page-id) are still authoritative; only the rendering layer changed.
2. **Update `docs/development/contributing.md`** — workspace tree listing + non-negotiable invariants (`#1 references FUSE callbacks`).
3. **Update `docs/development/roadmap.md`** — extend through v0.8 / v0.9 / v0.10 (currently stops at v0.7); revise long-term north stars (macFUSE / ProjFS no longer applicable).
4. **Rewrite `docs/reference/git-remote.md`** — add `stateless-connect` capability; remove FUSE references; refresh the v0.2 backlog section.
5. **Rewrite `docs/reference/{confluence,jira}.md`** — replace "Mount as FUSE" sections with `reposix init` flows. Drop the `reposix mount` CLI line.
6. **Rewrite `docs/demos/index.md`** — recast the demo lineup around `reposix init` + git workflow; the FUSE-era recordings can stay as archived assets but the index page must lead with the post-pivot UX.
7. **Move `.planning/phases/30-docs-ia-and-narrative-overhaul-...` into `.planning/milestones/v0.9.0-phases/`** — it's a deferred-then-superseded phase; living in `phases/` next to the active 40–45 dirs is confusing.
8. **Rewrite `scripts/demos/01-edit-and-push.sh`, `02-guardrails.sh`, `03-conflict-resolution.sh`, `08-full-backend-showcase.sh`** — into the post-pivot equivalents (`reposix init` + git-push round-trip). The `dark-factory-test.sh` is the template.
9. **Rewrite `scripts/demo.sh`** — currently a shim to `demos/full.sh`; either delete or repoint to `dark-factory-test.sh`.
10. **Resolve helper-hardcodes-SimBackend** (audit §1, audit-v0.9.0). Tracked for v0.10.0/v0.11.0 per STATE.md.

### Nice-to-have backlog

1. **`IssueId` → `RecordId` rename.** Owner-flagged. ~304 call sites; biggest type-rename in the project. See "Naming generalization" section. Plan as a dedicated phase; coordinate with frontmatter compat (the YAML field name is `id` so on-disk format is unaffected).
2. **`Issue` → `Record` rename** (or keep `Issue` as one valid record kind; promote `Record` as alias). Pre-1.0 hard rename is acceptable per the project's history (cf. ADR-004 `IssueBackend → BackendConnector`).
3. **Consolidate `integration-contract` and `integration-contract-github-v09`** — both hit GitHub. Drop the legacy job once the v09 variant is proven stable.
4. **`crates/reposix-cli/src/refresh.rs`** — rename `mount_point` field to `working_tree`; delete `is_fuse_active` guard (looks for a never-created `.reposix/fuse.pid` file). Lock-step rename in `cli.rs` + `refresh_integration.rs`.
5. **Re-record demo gifs.** README itself flags `docs/social/assets/demo.gif` as "FUSE-era — Phase 45 will re-record". Same for `architecture.mmd`/`.png`.
6. **Annotate `.planning/notes/phase-30-narrative-vignettes.md`** with "banned-word list updated in REQUIREMENTS.md DOCS-07 for git-native; vignette V1 still applicable".
7. **`docs/research/initial-report.md` + `agentic-engineering-reference.md`** are stylistically dense and academic. Owner consideration: they're cited from CLAUDE.md as "the architectural argument"; some readers will bounce off them. Lower priority than user-facing surface.
8. **Sweep `crates/reposix-{core,confluence,remote}/src/*.rs` doc comments** for "FUSE" / "mount" framing in module-level docs. Cosmetic but signals the post-pivot maturity. ~10 lines total.

---

## Stale references audit

A grep pass for terms that should be 0 or near-0 in tracked code post-v0.9.0 (counts exclude `.planning/milestones/`, `.planning/research/v0.1-fuse-era/`, `docs/archive/`, and `CHANGELOG.md` historical sections, which are correctly historical):

| Term | Count (approx) | Where it lives |
|---|---:|---|
| `fuser` (cargo dependency) | 0 | crate purged |
| `fusermount` | ~16 | `scripts/demos/*.sh`, `scripts/dev/test-*.sh`, `docs/reference/{cli,crates}.md`, `docs/development/contributing.md`, `docs/decisions/{001,002,003}.md`, `docs/demo.md` (stub — false positive), `README.md` |
| `FUSE` (case-insensitive) | ~70 | `README.md` (27), `docs/reference/{cli,crates,git-remote,confluence,jira,http-api,testing-targets}.md` (~25), `docs/decisions/{001,002,003,005}.md` (~10), `docs/development/{contributing,roadmap}.md`, `crates/reposix-{core,confluence,remote,swarm}/**` doc comments (~10), `crates/reposix-cli/{src,tests}` (~8 — most are valid migration tests) |
| `reposix mount` | ~25 | `scripts/demos/*.sh` (most), `docs/reference/{cli,confluence,crates}.md`, `README.md`, `HANDOFF.md`, `docs/decisions/002`, `docs/demos/index.md` |
| `reposix-fuse` (binary or crate name) | ~12 | `.github/workflows/release.yml` (broken), `scripts/demos/*.sh`, `docs/reference/crates.md`, `docs/decisions/{002,003,004}.md`, `docs/development/{contributing,roadmap}.md`, `HANDOFF.md` |
| `IssueId` | 304 | active code (load-bearing rename target — see below) |
| `IssueBackend` | ~25 docs / 0 active code | `docs/reference/crates.md`, `docs/connectors/guide.md` (stub), `docs/decisions/{001,002}.md`, `docs/why.md` (stub — false), `docs/demos/index.md`, `docs/reference/confluence.md`, `docs/development/roadmap.md`, `docs/security.md` (stub — false), `scripts/demos/{08, parity, parity-confluence, 05}.sh`, `scripts/tag-v0.8.0.sh`, `docs/archive/{...}` (correctly historical). `crates/` source has 0 — Phase 27 rename is clean. |
| `IssueBackend` in active Rust code | 0 | The trait is `BackendConnector`. References in `crates/` `.rs` files are all in *comments* like "FUSE daemon and CLI orchestrator" — those are doc-comment cleanups, not code. |

---

## Naming generalization candidates

> **OWNER FLAGGED:** `IssueId` is too narrow ("could be a doc, issue, note, etc"). The architecture is now backend-agnostic (sim, GitHub Issues, Confluence pages, JIRA tickets) but the type name still privileges one shape. The frontmatter field name (`id`) is already generic; only the Rust type names lag.

| Current name | Proposed | Scope | Risk | Notes |
|---|---|---|---|---|
| `IssueId` | `RecordId` | workspace-wide; ~304 call sites across 13 src files + tests | Medium — typed-error variants reference it; `Issue.parent_id: Option<IssueId>` cross-references; Confluence/JIRA tests use `IssueId(N)` as constructor. Mechanical rename via cargo + `rust-analyzer` — no semantic change. | YAML serialization is `#[serde(transparent)]` — on-disk format `id: 42` is unaffected. Same precedent as ADR-004 `IssueBackend → BackendConnector`. |
| `Issue` | `Record` | workspace-wide; load-bearing struct; ~600+ call sites | High — frontmatter YAML `Frontmatter` DTO references it; test fixtures named `sample()` return `Issue`; `Issue.title`/`.body`/`.status` field names; `IssueStatus` enum. **Decision recommendation: rename `Issue` → `Record` AND `IssueStatus` → `RecordStatus`** (the GitHub-specific status terms `Open`/`InProgress`/`Done` are already overly specific; consider also widening the enum if doing this rename). | YAML round-trip safe (`Frontmatter` DTO is internal). |
| `IssueStatus` | `RecordStatus` (or `WorkflowState`) | workspace-wide; ~50 call sites | Medium | Tied to the `Issue → Record` decision. The JIRA-flavored enum (`Open/InProgress/InReview/Done/WontFix`) is already a "Jira-flavored superset" per the doc comment — fine to keep semantically; only the type name moves. |
| `Error::InvalidIssue` | `Error::InvalidRecord` | workspace-wide; ~10 call sites | Low | Tied to `Issue` rename. |
| `error.rs::Issue` body / file format violation message | (text update) | `crates/reposix-core/src/error.rs:16` | Low | Trivial. |
| `path::validate_issue_filename` | `path::validate_record_filename` | ~6 call sites | Low | Tied to `Issue` rename. |
| `slug_or_fallback(title, id: IssueId)` | (just take `id: RecordId`) | already semantically generic | Low | Function purpose is generic; only the type rename lands. |
| `ProjectSlug` | (keep) | — | — | Already generic. |
| `BackendConnector` | (keep) | — | — | Already renamed Phase 27 (ADR-004). Good. |
| `bucket` term in code/test naming | (keep) | — | — | Already generic — refers to `issues/` vs `pages/` collection at the working-tree root. Survived the FUSE deletion intact. |

**Other narrow terms to consider sweeping in the same rename pass:**

- `update_issue` / `create_issue` / `delete_or_close` (`BackendConnector` methods) → `update_record` / `create_record` / `close_record`. ~50 call sites; mechanical. Tied to the `Issue` rename to keep the API surface coherent.
- `list_issues` / `list_changed_since` (returns `Vec<IssueId>`) → `list_records` / `list_changed_since` (returns `Vec<RecordId>`). Same pass.
- Module name `crates/reposix-core/src/issue.rs` → `record.rs` (and re-export `pub use record::*`).
- `crates/reposix-core/src/backend/sim.rs::SimBackend` is fine (a backend not a record).

**Suggested sequencing for the rename:**
1. Plan as a single coordinated phase (call it `RENAME-02` per the existing ADR-004 → ADR-005 numbering convention).
2. Hard rename, no backward-compat aliases (precedent: Phase 27 / ADR-004).
3. Tooling: `cargo check --workspace` + `rust-analyzer rename` + a tracking script under `scripts/check_naming_generic.sh` that fails CI on regressions.
4. Run `bash scripts/dark-factory-test.sh sim` after the rename — agent UX is "pure git", so the rename should be invisible to the dark-factory regression by construction. That's the cleanest possible end-to-end check.

---

## Recommended sequencing

**Tonight (overnight polish session).** Land the must-do items 1–6: fix `release.yml` (one line), delete the dead FUSE swarm code + the FUSE demo scripts (file deletions only), rewrite `docs/reference/cli.md` + `docs/reference/crates.md` (highest user-visible misleading docs), and delete `HANDOFF.md` (after verifying nothing in `.planning/STATE.md` or `MILESTONES.md` is missing). These are all low-risk, high-leverage, and keep the v0.10.0 milestone scaffolding plan intact. Item 7 (README rewrite) is owner-tagged for Phase 45 — leave it to that phase rather than pre-empting; just delete the explicit "v0.7.x — pre-FUSE-deletion" Quickstart section as a strict subset of the Phase 45 work.

**v0.10.0 milestone (already-planned Phases 40–45).** The v0.10.0 plan-of-record handles most of the docs work organically (Phase 41 = how-it-works trio, Phase 42 = tutorials/guides, Phase 43 = nav + theme, Phase 44 = clarity-review gate, Phase 45 = README + tag). The "should-do this milestone" items above slot into Phase 41 (`docs/reference/git-remote.md`, `confluence.md`, `jira.md` rewrites are reference-page work that aligns with the trio), Phase 42 (rewrite the demo scripts), Phase 43 (banned-words linter would catch any future regressions), and Phase 44 (the gate — running doc-clarity-review against everything). Add a Phase 43.1 or 44.1 explicitly for "scope-superseded ADR annotations" since those are decision-records edits and not nav work. Item 10 (helper-hardcodes-SimBackend) is the v0.9.0 milestone-audit carry-forward; STATE.md correctly schedules it before any v0.11.0 benchmarks.

**v0.11.0 ("Performance & Sales Assets" per the audit) and beyond.** The `IssueId/Issue/IssueStatus` rename is a v0.11.0 candidate — it's a code-level change that should not happen during a docs milestone (would invalidate every example in the freshly-written docs), and not before v0.10.0 ships (locks the new docs to a not-yet-existent type vocabulary). Plan it as the first phase of v0.11.0 with the dark-factory regression as its acceptance gate. The "re-record demo gifs" and "consolidate redundant CI jobs" items are nice-to-have backlog and don't block any milestone; both are appropriate to land opportunistically as their work surfaces (Phase 45 will already need fresh recordings, for example).
