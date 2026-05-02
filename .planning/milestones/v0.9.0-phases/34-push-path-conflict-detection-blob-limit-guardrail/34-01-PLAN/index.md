---
phase: 34
plan: 01
title: "Blob limit guardrail — REPOSIX_BLOB_LIMIT enforcement in stateless-connect"
wave: 1
depends_on: []
requirements: [ARCH-09]
files_modified:
  - crates/reposix-cache/fixtures/cache_schema.sql
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-remote/src/stateless_connect.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/Cargo.toml
autonomous: true
mode: standard
---

# Phase 34 Plan 01 — Blob-limit guardrail (`REPOSIX_BLOB_LIMIT`)

<objective>
Make the helper refuse a `command=fetch` RPC turn whose `want` count exceeds
`REPOSIX_BLOB_LIMIT` (default 200, env-configurable, `0` = unlimited). Refusal
writes a verbatim self-teaching stderr message that names `git sparse-checkout`
literally so an unprompted agent recovers without prompt engineering. One audit
row per refusal (`op='blob_limit_exceeded'`). Implements ARCH-09.
</objective>

<must_haves>
- Env var read once at helper startup, cached in a `OnceLock<u32>`.
- Default 200 if unset/empty/non-numeric (warn-log + fall back).
- `0` means "unlimited" (explicit opt-out, documented in code comment).
- Limit enforcement runs INSIDE `proxy_one_rpc` AFTER request frames are
  consumed (so `want_count` is final) but BEFORE `git upload-pack` is spawned.
- Refusal stderr is the verbatim const string from `BLOB_LIMIT_EXCEEDED_FMT`,
  with `<N>` and `<M>` substituted; literal backticks preserved around
  `git sparse-checkout set <pathspec>`.
- Refusal returns `Err(...)` from `handle_stateless_connect` so `main()` exits
  non-zero (exit code 2 via the `e => 2` arm).
- `cache_schema.sql` CHECK list extended with `'blob_limit_exceeded'`. Triggers
  (`audit_cache_no_update`, `audit_cache_no_delete`) untouched.
- New audit helper `log_blob_limit_exceeded(conn, backend, project, want_count, limit)`
  in `reposix-cache/src/audit.rs`; thin wrapper method `Cache::log_blob_limit_exceeded`.
- All HTTP factory paths and existing audit invariants (append-only, DEFENSIVE)
  preserved.
</must_haves>

<canonical_refs>
- `.planning/phases/34-push-path-conflict-detection-blob-limit-guardrail/34-CONTEXT.md` §Blob-limit guardrail (locked).
- `.planning/REQUIREMENTS.md` ARCH-09 (verbatim stderr message).
- `.planning/ROADMAP.md` Phase 34 success criterion 4.
- `crates/reposix-remote/src/stateless_connect.rs:184-298` — `proxy_one_rpc` fn (limit check inserts here).
- `crates/reposix-remote/src/pktline.rs:117` — `is_want_line` already counts wants.
- `crates/reposix-cache/src/audit.rs` — pattern for new helper.
- `crates/reposix-cache/fixtures/cache_schema.sql` — CHECK list to extend.
- `crates/reposix-cache/src/cache.rs:115-163` — `log_helper_*` methods (pattern for `log_blob_limit_exceeded`).
</canonical_refs>

---

## Chapters

- [T01 — Extend `cache_schema.sql` CHECK constraint](./T01-cache-schema.md)
- [T02 — Add `log_blob_limit_exceeded` audit helper + Cache method](./T02-audit-helper.md)
- [T03 — Read `REPOSIX_BLOB_LIMIT` into `OnceLock` + define verbatim message constant](./T03-blob-limit-const.md)
- [T04 — Enforce limit inside `proxy_one_rpc` BEFORE upload-pack spawn](./T04-enforce-limit.md)
- [T05 — Workspace gate: clippy + tests + manual smoke note](./T05-workspace-gate.md)
