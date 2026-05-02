← [back to index](./index.md)

# Bucket-by-bucket review — `crates/`

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
