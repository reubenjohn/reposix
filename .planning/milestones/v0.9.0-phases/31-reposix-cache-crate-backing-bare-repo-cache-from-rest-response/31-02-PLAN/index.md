---
phase: 31
plan: 02
type: execute
wave: 2
depends_on:
  - "31-01"
files_modified:
  - crates/reposix-cache/Cargo.toml
  - crates/reposix-cache/src/lib.rs
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/meta.rs
  - crates/reposix-cache/src/db.rs
  - crates/reposix-cache/src/error.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-cache/src/builder.rs
  - crates/reposix-cache/fixtures/cache_schema.sql
  - crates/reposix-cache/tests/materialize_one.rs
  - crates/reposix-cache/tests/audit_is_append_only.rs
  - crates/reposix-cache/tests/egress_denied_logs.rs
  - crates/reposix-cli/src/cache_db.rs
  - crates/reposix-cli/src/lib.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/src/refresh.rs
  - crates/reposix-cli/Cargo.toml
autonomous: true
requirements:
  - ARCH-02
  - ARCH-03
tags:
  - rust
  - sqlite
  - audit
  - egress
  - allowlist
  - cache
user_setup: []
---

# Phase 31 Plan 02: Cache operating-principle hooks + lazy blob materialization

Wire the cache to its three non-negotiable operating-principle hooks: append-only SQLite audit, egress allowlist denial that is audited BEFORE the typed error returns, and `Tainted<Vec<u8>>` public return for `read_blob`. Also implement the actual lazy-blob materialization path (`Cache::read_blob`) that Plan 01 deliberately left unimplemented.

**Purpose:** ARCH-02 (audit row per materialize + Tainted return + append-only triggers) and ARCH-03 (egress allowlist reuse of `reposix_core::http::client` + `EgressDenied` error + audit row).

**Output:** `audit_events_cache` + `meta` + `oid_map` tables with append-only triggers, `read_blob` returning `Tainted<Vec<u8>>`, three integration tests (`materialize_one`, `audit_is_append_only`, `egress_denied_logs`), and `cache_db.rs` lifted from `reposix-cli` into `reposix-cache`.

## Chapters

- **[Objective and Context](./objective-and-context.md)** — Full plan execution context with dependencies and reference files.
- **[Task 1: Schema + Audit + Metadata](./task-1-schema.md)** — Land `cache_schema.sql`, `db.rs`, `audit.rs`, `meta.rs`, and append-only trigger test.
- **[Task 2: Lazy Blob Materialization](./task-2-read-blob.md)** — Implement `Cache::read_blob` with Tainted return, egress-denial audit, and materialize-row test.
- **[Task 3: Lift cache_db.rs](./task-3-lift-cache-db.md)** — Move `cache_db.rs` from `reposix-cli` to `reposix-cache::cli_compat` for schema unification.
- **[Trust Boundaries and Threats](./trust-and-threats.md)** — STRIDE threat register and mitigation for each component.
- **[Verification and Success Criteria](./verification.md)** — Acceptance criteria, automated verifiers, and testing strategy.

## Essential readings before starting

Read in this order to understand the full execution:

1. `./objective-and-context.md` — execution context, dependencies, interfaces.
2. One of the task chapters corresponding to your assignment.
3. `./trust-and-threats.md` — if you're concerned about security implications of your changes.
4. `./verification.md` — acceptance criteria for sign-off.

Refer back to `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md` and `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-CONTEXT.md` for deep dives on architecture and decision rationale.
