# Phase 1 — Wave Structure

**Phase:** `01-core-contracts-security-guardrails`
**Plans:** 3
**Waves:** 1 (all three plans run in parallel)

## Wave map

```
Wave 1 ────────────────────────────────────────────────────────────
  │
  ├── 01-01-http-client-allowlist.md          (independent)
  │     creates: clippy.toml, src/http.rs, tests/http_allowlist.rs
  │     closes:  SG-01 + SG-07 (5s-timeout half) + ROADMAP SC #2 + SC #4
  │
  ├── 01-02-tainted-typing-and-path-validator.md   (independent)
  │     creates: src/taint.rs, src/path.rs, tests/compile_fail.rs,
  │              tests/compile-fail/tainted_into_untainted.rs (+ .stderr)
  │     closes:  SG-03 + SG-04 + SG-05 + 4 of 5 ROADMAP SC #1 named tests
  │
  └── 01-03-audit-schema-fixture.md           (independent)
        creates: fixtures/audit.sql, src/audit.rs,
                 examples/show_audit_schema.rs, tests/audit_schema.rs
        closes:  SG-06 + schema half of FC-06 + ROADMAP SC #3
```

## Parallel-safety check

All three plans write into `crates/reposix-core/` but into **disjoint files**. The only shared files are:

| Shared file | Touched by | Conflict strategy |
|-------------|------------|-------------------|
| `crates/reposix-core/Cargo.toml` | 01-01 (adds `reqwest`, `[dev-dependencies] tokio`, `wiremock`), 01-02 (adds `[dev-dependencies] trybuild`), 01-03 (adds `rusqlite`) | **Expected merge**: each plan appends to a different line/section of the TOML. Executor MUST re-read the file before its edit (standard Write-tool rule). If two executors stage simultaneously, the second one's edit will be rejected until it re-reads — the normal GSD workflow handles this. If contention occurs, serialize on a per-plan basis by pulling the Cargo.toml edit to the end of each plan's Task 1. |
| `crates/reposix-core/src/lib.rs` | 01-01 (`pub mod http;`), 01-02 (`mod taint; pub use taint::{...}; pub mod path;`), 01-03 (`pub mod audit;`) | **Expected merge**: each plan adds a distinct `mod` / `pub use` line. Same re-read rule as Cargo.toml. Lines are independent; a simple append-ordered commit sequence produces no semantic conflict. |
| `crates/reposix-core/src/error.rs` | 01-01 (`InvalidOrigin` + `Http` variants), 01-02 (`InvalidPath` variant) | **Expected merge**: three new variants added to the same enum; each plan appends at the end of the variant list. Re-read-before-edit is sufficient. 01-03 does not touch this file (it uses `Error::Other`). |

**Decision:** run all three plans as Wave 1 in parallel. Collisions on `Cargo.toml`, `lib.rs`, and `error.rs` are small-surface additive edits — the Write-tool's read-before-edit protocol handles them without per-plan serialization.

**Fallback:** if in practice the parallel executors thrash on `Cargo.toml` / `lib.rs` / `error.rs`, fall back to a trivial 3-wave-of-1 execution (01-01 → 01-02 → 01-03). Total wall-clock cost is ~3x larger but semantically identical. Do NOT reorder: 01-01 and 01-03 are fully independent; 01-02 only touches `error.rs` which 01-01 also edits, so if serialization is needed, 01-02 and 01-01 must not overlap on `error.rs` edits.

## Wave 2

**None.** Phase 1 completes when all three Wave-1 plans are green.

## Phase exit criterion

The phase is done when this composite command exits 0:

    cd /home/reuben/workspace/reposix && \
      cargo test -p reposix-core --all-features \
        egress_to_non_allowlisted_host_is_rejected \
        server_controlled_frontmatter_fields_are_stripped \
        filename_is_id_derived_not_title_derived \
        path_with_dotdot_or_nul_is_rejected \
        tainted_cannot_be_used_where_untainted_required && \
      cargo test -p reposix-core allowlist_default_and_env -- --nocapture && \
      cargo test -p reposix-core --test audit_schema && \
      cargo run -q -p reposix-core --example show_audit_schema | \
        grep -q 'CREATE TRIGGER audit_no_update BEFORE UPDATE' && \
      cargo run -q -p reposix-core --example show_audit_schema | \
        grep -q 'CREATE TRIGGER audit_no_delete BEFORE DELETE' && \
      [ "$(grep -RIn 'reqwest::Client::new\|Client::builder' crates/ \
            --include='*.rs' | \
            grep -v 'crates/reposix-core/src/http.rs' | wc -l)" = "0" ] && \
      grep -q 'reqwest::Client::new' clippy.toml && \
      cargo clippy -p reposix-core --all-targets -- -D warnings

That command is the union of ROADMAP phase-1 success-criteria #1–#5.
