# Architecture observations + Naming consistency + Closing note

← [index](./index.md)

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
