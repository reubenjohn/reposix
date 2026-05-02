← [back to index](./index.md)

# Threat model, verification, success criteria, and output

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Backend (sim) → Cache | Every `Issue` returned by `list_issues` is attacker-influenced (the sim's seed data is authored by test code or, in real-backend scenarios, by an untrusted remote API). The cache renders these bytes via `frontmatter::render` and commits them into a local bare repo. |
| Environment → Cache path resolution | `REPOSIX_CACHE_DIR` is read from the process env. A malicious caller could point the cache at `/etc/...` or an `NFS`-mounted network path. Plan 01 trusts the caller on this axis; the cache's consumers (Phase 35 CLI) are responsible for sanitizing user-supplied paths. |
| Filesystem → other local users | The cache directory is created with the process's default umask. SQLite hardening + mode=0o600 lands in Plan 02. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-31-01-01 | Tampering | `build_from` produces a tree that misrepresents backend state (e.g. truncates the issue list silently due to backend pagination quirks). | mitigate | Tree entry count equals `backend.list_issues(...).len()` — no silent filtering inside the builder. Test `tree_contains_all_issues` asserts the count matches the seed fixture exactly. |
| T-31-01-02 | Tampering | `gix::init_bare` picks up `~/.gitconfig`'s `init.defaultBranch = master`, causing the Phase 32 helper to look for `refs/heads/main` and find nothing. | mitigate | Explicitly write `HEAD` to `ref: refs/heads/main` after init (RESEARCH §Pitfall 5). Verified implicitly by `tree_contains_all_issues` peeling `refs/heads/main` successfully. |
| T-31-01-03 | Denial of Service | Unbounded `list_issues` on a backend with 100k issues fills memory with `Vec<Issue>`. | accept | v0.9.0 scope: tree metadata is cheap (RESEARCH §2); at 100k issues a single `Vec<Issue>` is maybe 50MB — well within agent memory budgets. Streaming is a v0.10.0 concern. |
| T-31-01-04 | Information Disclosure | Cache directory is created with process default umask (usually 0022), making `cache.git/objects/*` world-readable on a shared host. | accept (within Plan 01 scope) | Plan 02 lands `mode=0o600` on `cache.db`. Git object files are content-addressed hashes; exposure risk is limited to the bytes (which are already tainted-from-backend anyway). Plan 02 narrows the DB file; directory-level hardening is deferred to Phase 35 CLI setup. |
| T-31-01-05 | Tampering | `REPOSIX_CACHE_DIR` points at a shared path (e.g. `/tmp/attacker-writable`); an attacker writes a pre-baked malicious bare repo there; `Cache::open` silently mounts it. | accept | v0.9.0 single-user contract (CLAUDE.md OP: "no hidden state", `.env`-only credential path). Phase 33/34 will introduce the `meta` table collision check (Plan 02 scaffolds the `CacheCollision` error variant). Plan 01 neither detects nor exploits this — out of scope. |
| T-31-01-06 | Elevation of Privilege | A gix API version drift (e.g. `commit_as` signature change) causes the builder to no-op silently. | mitigate | gix pinned `=0.82.0`. Task 1's `gix_api_smoke` test compiles the exact API surface the builder uses; any drift during `cargo update` breaks the smoke test immediately. |
</threat_model>

<verification>
Phase 31 Wave 1 verification:

1. `cargo check -p reposix-cache` — exit 0.
2. `cargo clippy -p reposix-cache --all-targets -- -D warnings` — exit 0.
3. `cargo test -p reposix-cache` — all Plan 01 tests pass: `gix_api_smoke`, `path::tests::env_var_wins`, `tree_contains_all_issues`, `blobs_are_lazy`.
4. `cargo check --workspace` — exit 0 (no regression on existing crates).
5. `grep -c 'reposix-cache' Cargo.toml` — returns ≥ 1 (workspace member added).
6. `grep -c 'gix = "=0.82.0"' Cargo.toml` — returns ≥ 1 (pinned).

No manual verification step — every behavior is covered by automated tests.
</verification>

<success_criteria>
- [ ] `crates/reposix-cache/` exists as a workspace crate.
- [ ] `Cargo.toml` workspace members includes `crates/reposix-cache` and `workspace.dependencies` includes `gix = "=0.82.0"` and `dirs = "6"`.
- [ ] `reposix_cache::Cache::open(backend, backend_name, project)` resolves a deterministic cache path (env var honored), creates the parent dir, and `gix::init_bare`s the target.
- [ ] `reposix_cache::Cache::build_from().await` produces a commit on `refs/heads/main` whose tree enumerates exactly N `issues/<id>.md` entries for N seeded issues.
- [ ] After `build_from`, zero blob objects exist in `.git/objects/` (tree entries reference OIDs that are NOT persisted — the lazy invariant Phase 32's helper relies on).
- [ ] `HEAD` points at `refs/heads/main` regardless of the user's `init.defaultBranch`.
- [ ] `cargo test -p reposix-cache` green; `cargo clippy -p reposix-cache --all-targets -- -D warnings` clean.
- [ ] No regression: `cargo check --workspace` green.
</success_criteria>

<output>
After completion, create `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-01-SUMMARY.md` per the template. Include:
- Commit SHAs for the two task commits.
- The exact gix 0.82 method names used (in case any differed from the `<interfaces>` sketch — next plan's executor needs the ground truth).
- Any `// NOTE:` comments left in `builder.rs` explaining gix API departures.
- Confirmation that `tree_contains_all_issues` and `blobs_are_lazy` both pass, with the actual tree entry count for each seed size tested.
</output>
