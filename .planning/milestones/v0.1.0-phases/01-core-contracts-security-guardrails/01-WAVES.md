# Phase 1 — Wave Structure

**Phase:** `01-core-contracts-security-guardrails`
**Plans:** 4
**Waves:** 2 (Wave 0: 01-00; Wave 1: 01-01, 01-02, 01-03 in parallel)

## Wave map

```
Wave 0 ────────────────────────────────────────────────────────────
  │
  └── 01-00-error-variants.md                  (FIX 1 — merge-collision avoidance)
        creates: error.rs (+InvalidOrigin, +InvalidPath, +Http)
                 Cargo.toml (+reqwest in [dependencies])
        purpose: lands all three new Error variants in ONE commit so
                 Wave-1 plans 01-01 and 01-02 do NOT both edit error.rs

Wave 1 ────────────────────────────────────────────────────────────  (depends on Wave 0)
  │
  ├── 01-01-http-client-allowlist.md          (depends_on: 01-00)
  │     creates: clippy.toml, src/http.rs, tests/http_allowlist.rs,
  │              scripts/check_clippy_lint_loaded.sh (FIX 3)
  │     consumes: Error::InvalidOrigin, Error::Http (from 01-00)
  │     closes:  SG-01 + SG-07 (5s-timeout half) + ROADMAP SC #2 + SC #4
  │              + plan-checker FIX 2 (redirect-target recheck test)
  │              + plan-checker FIX 3 (clippy.toml load-proof script)
  │
  ├── 01-02-tainted-typing-and-path-validator.md   (depends_on: 01-00)
  │     creates: src/taint.rs, src/path.rs, tests/compile_fail.rs,
  │              tests/compile-fail/tainted_into_untainted.rs (+ .stderr),
  │              tests/compile-fail/untainted_new_is_not_pub.rs (+ .stderr) (FIX 4)
  │     consumes: Error::InvalidPath (from 01-00)
  │     closes:  SG-03 + SG-04 + SG-05 + 4 of 5 ROADMAP SC #1 named tests
  │              + plan-checker FIX 4 (Untainted::new visibility lock)
  │
  └── 01-03-audit-schema-fixture.md           (no dependency — uses Error::Other only)
        creates: fixtures/audit.sql, src/audit.rs,
                 examples/show_audit_schema.rs, tests/audit_schema.rs
        closes:  SG-06 + schema half of FC-06 + ROADMAP SC #3
```

## Why Wave 0 exists (FIX 1 from plan-checker)

The original 3-plan layout had 01-01 and 01-02 both editing `crates/reposix-core/src/error.rs` in Wave 1 (01-01 added `Error::InvalidOrigin` + `Error::Http(#[from] reqwest::Error)`; 01-02 added `Error::InvalidPath`). Two parallel executors writing to the same file is a guaranteed merge collision regardless of Write-tool re-read protocols, because both edits land at the end of the same enum.

**Resolution:** Wave 0 is a single 1-task plan that adds all three variants in one commit. After Wave 0:

- `crates/reposix-core/src/error.rs` is **off-limits** to Wave-1 plans 01-01 and 01-02.
- `crates/reposix-core/Cargo.toml` `[dependencies]` already has `reqwest = { workspace = true }` (added by 01-00 so the `Http` variant compiles); Wave-1 plans only touch `[dev-dependencies]`.
- 01-03 is unaffected — it only uses `Error::Other`.

## Parallel-safety check (Wave 1)

After Wave 0 lands, the three Wave-1 plans write into `crates/reposix-core/` but into **disjoint files** for the security-sensitive surfaces. The remaining shared files are limited to additive `[dev-dependencies]` and `mod`/`pub use` lines:

| Shared file | Touched by | Conflict strategy |
|-------------|------------|-------------------|
| `crates/reposix-core/Cargo.toml` (`[dev-dependencies]` only) | 01-01 (adds `tokio`, `wiremock`), 01-02 (adds `trybuild`); 01-03 only adds to `[dependencies]` (`rusqlite`) | **Expected merge**: each plan appends to a different line/section. Executor MUST re-read the file before its edit (standard Write-tool rule). 01-00 already populated `[dependencies]` with `reqwest`, so neither 01-01 nor 01-02 touches `[dependencies]`. |
| `crates/reposix-core/src/lib.rs` | 01-01 (`pub mod http;`), 01-02 (`mod taint; pub use taint::{...}; pub mod path;`), 01-03 (`pub mod audit;`) | **Expected merge**: each plan adds a distinct `mod` / `pub use` line. Same re-read rule as Cargo.toml. Lines are independent; a simple append-ordered commit sequence produces no semantic conflict. |
| `crates/reposix-core/src/error.rs` | **NONE in Wave 1.** All three variants were added in Wave 0 (01-00). | **No collision possible** — Wave-1 plans are forbidden from editing `error.rs`. |

**Decision:** run all three Wave-1 plans in parallel. The remaining additive collisions on `Cargo.toml` `[dev-dependencies]` and `lib.rs` are small-surface, append-only, and handled by the Write-tool's read-before-edit protocol.

**Fallback:** if in practice the parallel executors thrash on `Cargo.toml` / `lib.rs`, fall back to a trivial 3-wave-of-1 execution (01-01 → 01-02 → 01-03) within Wave 1's slot. Total wall-clock cost is ~3x larger but semantically identical.

## Wave 2

**None.** Phase 1 completes when Wave 0 (01-00) and all three Wave-1 plans are green.

## Phase exit criterion

The phase is done when this composite command exits 0:

    cd /home/reuben/workspace/reposix && \
      cargo test -p reposix-core --all-features \
        egress_to_non_allowlisted_host_is_rejected \
        server_controlled_frontmatter_fields_are_stripped \
        filename_is_id_derived_not_title_derived \
        path_with_dotdot_or_nul_is_rejected \
        tainted_cannot_be_used_where_untainted_required \
        untainted_new_is_pub_crate_only \
        redirect_target_is_rechecked_against_allowlist && \
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
      bash scripts/check_clippy_lint_loaded.sh && \
      cargo clippy -p reposix-core --all-targets -- -D warnings

That command is the union of ROADMAP phase-1 success-criteria #1–#5 plus the four plan-checker fixes (FIX 1 implicit via the wave structure, FIX 2 via `redirect_target_is_rechecked_against_allowlist`, FIX 3 via `scripts/check_clippy_lint_loaded.sh`, FIX 4 via `untainted_new_is_pub_crate_only`).
