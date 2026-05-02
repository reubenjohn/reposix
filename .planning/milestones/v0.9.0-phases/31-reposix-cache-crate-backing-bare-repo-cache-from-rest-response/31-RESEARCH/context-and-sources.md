← [back to index](./index.md)

# State of the Art, Assumptions, Open Questions, Environment, Constraints, Sources, and Metadata

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| FUSE virtual FS, every read = REST hop | Bare-repo cache + git partial clone | v0.9.0 (this milestone) | This phase ships the substrate; full pivot completes in Phase 36 |
| `git2` (libgit2 bindings) for Rust git ops | `gix` pure-Rust (since gix matured to 0.7+ in 2023, now 0.82 in 2026) | gix issue #470 tracks 1.0 readiness; current state "production-ready for object writing, iterating on higher-level porcelain" | Avoids `pkg-config` / `libgit2-sys` build dep — matches workspace's "no system C deps" stance |
| Hand-rolled SQLite append-only enforcement | `BEFORE UPDATE/DELETE RAISE(ABORT)` triggers + `SQLITE_DBCONFIG_DEFENSIVE` flag | Established in `reposix-core` Phase 1 audit fixture (M-04, H-02) | Lift the pattern; no novel work |

**Deprecated/outdated:**
- The pre-v0.9.0 `crates/reposix-cli/src/cache_db.rs` `refresh_meta` table will be superseded once `reposix-cache` ships its own `meta` table. CONTEXT.md specifies "possibly lifts the code into reposix-cache" — Wave A should decide: either (a) dual-source `last_fetched_at` (CLI writes to its own, cache writes to its own — a Phase 33 / 35 reconciliation issue) or (b) move the CLI module into the new crate now. Recommend (b) — fewer reconciliation foot-guns. But this is out of CONTEXT.md's locked scope; flag it as an open question for the planner to either time-box into Wave A or kick to Phase 33.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | gix 0.82's `commit_as` / `commit` method takes `(reference_name, message, tree_oid, parents)` in that order | Pattern 1 code sketch | LOW — Wave A smoke test will catch any signature mismatch before the rest of the builder is written. Mitigated by including a "verify gix API surface" task. |
| A2 | `gix::init_bare` defaults HEAD to `refs/heads/main` when no `init.defaultBranch` is configured globally | Pitfall 5 | MEDIUM — depends on user's `~/.gitconfig`. The mitigation (explicitly setting HEAD post-init) makes this assumption irrelevant. |
| A3 | `frontmatter::render` is fully deterministic given an Issue (no time, no random) | Pitfall 1 | LOW — the function is in `reposix-core` and the existing test `frontmatter_roundtrips` implicitly covers determinism, but adding an explicit "render twice, compare bytes" test is cheap insurance. |
| A4 | `gix::Repository::write_blob` returns the OID it computed (allowing OID-drift detection) | Pattern 2 | LOW — this is the standard contract for any git object writer. Verifiable in 30 seconds via `cargo doc`. |
| A5 | `reposix_core::http::client()` already returns `Error::InvalidOrigin` for non-allowlisted backends, and that error type is `downcast`-able from `BackendConnector` errors | Pattern 2 read_blob sketch | MEDIUM — needs verification. The `BackendConnector` trait error contract is `Error::Other` for many cases. Wave B may need a small refactor to surface `Error::InvalidOrigin` distinctly from `Error::Other`, OR the cache can match on the error message string. Flag for planner. |
| A6 | The single-row `meta` table from `cache_db.rs` is conceptually replaced by this crate's `meta` table — pre-existing CLI callers either move with it or get refactored in Phase 35 | State of the Art | MEDIUM — coordination with Phase 35 (CLI pivot). Flag for planner: either lift now or document the divergence. |
| A7 | `gix` 0.82 has the `tree-editor` cargo feature enabled by default | Pattern 1 | LOW — verified via docs.rs feature listing. If wrong, add `features = ["tree-editor"]` to Cargo.toml dep. |

## Open Questions for the Planner

1. **Lift `cache_db.rs` from `reposix-cli` into `reposix-cache` now, or in Phase 35?**
   - What we know: Both crates have a `meta` table conceptually. CONTEXT.md says "possibly lifts the code".
   - What's unclear: Whether dual-sourcing through v0.9.0 is OK or causes Phase 33 friction.
   - Recommendation: Lift in Wave B of this phase. The CLI's existing `refresh` subcommand is being deleted in Phase 35 anyway (replaced by `reposix init`), so leaving `cache_db.rs` orphaned in `reposix-cli` for one phase risks divergence.

2. **Should `Cache::read_blob` return `Tainted<Vec<u8>>` or `Tainted<Bytes>` (i.e., `bytes::Bytes`)?**
   - What we know: `Vec<u8>` is the simplest return type. `Bytes` enables zero-copy when piping to git's stdout (Phase 32).
   - What's unclear: Whether the protocol-v2 send path in Phase 32 will be a serious bytes-copy hot spot.
   - Recommendation: Start with `Vec<u8>`. Convert to `Bytes` only if Phase 32 perf benchmarks show it matters. Premature optimization risk is real and `Bytes` adds a workspace dep.

3. **Is the `Error::EgressDenied` variant new, or does it reuse `reposix_core::Error::InvalidOrigin`?**
   - What we know: `reposix_core::Error::InvalidOrigin` already exists and is what `reposix_core::http::client` returns.
   - What's unclear: Whether the cache should re-export that variant, define its own that wraps it, or just propagate it.
   - Recommendation: Define `reposix_cache::Error::Egress(reposix_core::Error)` and pattern-match on `InvalidOrigin` in `read_blob` to fire the `op=egress_denied` audit row. This keeps the cache's error space tight (one `Egress` variant) while preserving the underlying detail.

4. **What does the cache's `Cache::open` do if the bare-repo dir already exists with content from a different `(backend, project)` combo?**
   - What we know: The cache path is deterministic from `(backend, project)`, so this can only happen if a user manually moves files or `REPOSIX_CACHE_DIR` is shared across projects.
   - What's unclear: Whether to error, refuse, or silently overwrite.
   - Recommendation: Read the `meta` table's `backend` and `project` keys at open. If present and mismatching, return `Error::CacheCollision { expected, found }`. Defensive, cheap.

5. **Does the planner want a Wave 0 for test-infra, separate from Wave A's crate scaffold?**
   - What we know: Per CONTEXT.md, "trivial small" gates apply — and CONTEXT.md token-budget note suggests "2–3 plans max: (A) crate scaffold + types + bare-repo builder, (B) audit log + tainted + egress + SQLite, (C) tests including compile-fail."
   - What's unclear: Whether the `trybuild` fixture must land BEFORE the type it constrains (Wave 0) or alongside (Wave C).
   - Recommendation: Land alongside (Wave C). The type itself is `reposix_core::Tainted<Vec<u8>>` and already exists — no Wave 0 needed for that. The compile-fail fixture is a *test* of the type's existing behavior, not a precondition.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` (Rust toolchain ≥ 1.82) | All crates | ✓ | per `rust-toolchain.toml` | — |
| Git (`git` binary) | Optional smoke-test of cache against `git fsck` | ✓ (assumed) | 2.34+ per CLAUDE.md transition section | If absent, skip the fsck smoke test (degrade to Rust-only verification) |
| `gix` 0.82 | `reposix-cache` git ops | ✓ on crates.io | 0.82.0 (2026-04-24) | git2 0.20.4 (NOT recommended; pulls libgit2-sys) |
| SQLite (bundled via rusqlite) | Audit + meta DB | ✓ | rusqlite 0.32 `bundled` feature already in workspace | None needed — bundled |
| `trybuild` | Compile-fail fixture | ✓ on crates.io | 1.0.116 (2026-02-12) | Hand-rolled `cargo build --release 2>&1 | grep "expected"` (DO NOT — trybuild is already proven in `reposix-core`) |

**Missing dependencies with no fallback:** None — every dep is workspace-available or one `cargo add` away.

**Missing dependencies with fallback:** None.

## Project Constraints (from CLAUDE.md)

- **`#![forbid(unsafe_code)]`** at every crate root — `reposix-cache/src/lib.rs` MUST include this.
- **`#![warn(clippy::pedantic)]`** at every crate root — same as above.
- **`thiserror`** for typed errors in libraries; **`anyhow`** only at binary boundaries — this crate is a library, so it's pure `thiserror`.
- **All `Result`-returning functions have a `# Errors` doc section** — `Cache::open`, `Cache::build_from`, `Cache::read_blob` each need this.
- **Tests live next to the code** (`#[cfg(test)] mod tests`) for unit tests; integration tests in `tests/`.
- **YAML frontmatter via `serde_yaml` 0.9** — handled by `reposix_core::issue::frontmatter::render`; the cache calls into it, never re-implements.
- **Times are `chrono::DateTime<Utc>`** — for audit `ts` and meta `updated_at` (serialize as RFC 3339).
- **Workspace `clippy.toml` `disallowed_methods`** already lists `reqwest::Client::new`, `reqwest::Client::builder`, `reqwest::ClientBuilder::new` — DO NOT modify; the cache must NOT add to this list either (it has zero reqwest call sites).
- **Subagent delegation rules** — `gsd-planner` writes the plan; `gsd-executor` writes the code; `gsd-code-reviewer` reviews. Per CLAUDE.md, "use `gsd-phase-researcher` for any 'how do I build a bare git repo from raw blobs in Rust' question — non-trivial, easy to over-research in the orchestrator." This research file is the answer.
- **GSD entry rule:** Never edit code outside a GSD-tracked phase. This phase is GSD-tracked (Phase 31).

## Sources

### Primary (HIGH confidence)
- **In-repo source code (load-bearing):**
  - `crates/reposix-core/src/backend.rs` — `BackendConnector` trait (verified Phase 27 rename in place)
  - `crates/reposix-core/src/http.rs` — `HttpClient` + `client()` factory + allowlist (verified Phase 1 Hardening)
  - `crates/reposix-core/src/taint.rs` — `Tainted<T>` / `Untainted<T>` + `sanitize` (verified, no `From` / `Deref`)
  - `crates/reposix-core/src/audit.rs` + `crates/reposix-core/fixtures/audit.sql` — append-only schema pattern
  - `crates/reposix-cli/src/cache_db.rs` — single-row `refresh_meta` table + WAL+EXCLUSIVE pattern
  - `clippy.toml` — workspace `disallowed-methods` config
  - `Cargo.toml` (workspace) — verified workspace deps and dep versions
- **External authoritative docs:**
  - [docs.rs/gix Repository methods](https://docs.rs/gix/latest/gix/struct.Repository.html) — `write_blob`, `write_object`, `edit_tree`, `commit`, `commit_as`, `edit_reference` confirmed
  - [docs.rs/gix init_bare](https://docs.rs/gix/latest/gix/fn.init_bare.html) — `pub fn init_bare(directory) -> Result<Repository, Error>` confirmed
  - [git-scm.com/docs/partial-clone](https://git-scm.com/docs/partial-clone) — promisor remote semantics, `extensions.partialClone` set on consumer not promisor
- **crates.io API (version verification, 2026-04-24):**
  - gix 0.82.0 — published 2026-04-24
  - git2 0.20.4 — published 2026-02-02 (alternative)
  - trybuild 1.0.116 — published 2026-02-12
  - dirs 6.0.0 — published 2025-01-12
  - etcetera 0.11.0 — published 2025-10-28 (alternative)

### Secondary (MEDIUM confidence)
- [GitoxideLabs/gitoxide crate-status.md](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md) — high-level capability status (no per-method stability labels)
- [gix issue #470 toward 1.0](https://github.com/GitoxideLabs/gitoxide/issues/470) — gix is pre-1.0; minor-version churn possible (mitigation: pin `=0.82.0`)
- [GitLab partial clone overview](https://about.gitlab.com/blog/partial-clone-for-massive-repositories/) — confirms 50% faster / 70% less data pattern
- [git fast-import / fast-export docs (gitprotocol-v2)](https://git-scm.com/docs/gitprotocol-v2) — relevant to Phase 32 consumer, not directly this phase

### Tertiary (LOW confidence)
- WebSearch results referencing 2025 dates around gix object-mutation examples — verbose summaries, no concrete code snippets confirming method signatures. **Not used as authoritative source**; the docs.rs link above is the authoritative reference and Wave A's first task is a direct API smoke test.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — gix 0.82, trybuild 1.0.116, dirs 6.0.0 all verified via crates.io API; rusqlite/chrono/thiserror already in workspace
- Architecture: HIGH — substantially traced from existing `reposix_core::audit`, `reposix_core::taint`, `reposix_core::http` patterns; no novel infra
- Pitfalls: MEDIUM — pitfalls 1-6 are based on familiar git/SQLite/Rust failure modes; pitfall 4 (gix API stability) mitigated by version pin; the rest are operational diligence
- Threat model: HIGH — extends existing well-understood threat model; no new attack surface that isn't already mitigated by the existing allowlist + Tainted discipline

**Research date:** 2026-04-24
**Valid until:** 2026-05-24 (30 days — gix is fast-moving, pin recommended)

## RESEARCH COMPLETE
