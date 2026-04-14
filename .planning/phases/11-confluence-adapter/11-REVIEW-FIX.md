---
phase: 11-confluence-adapter
fixed_at: 2026-04-13T00:00:00Z
review_path: .planning/phases/11-confluence-adapter/11-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 11: Code Review Fix Report

**Fixed at:** 2026-04-13
**Source review:** `.planning/phases/11-confluence-adapter/11-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (2 MEDIUM + 3 LOW)
- Fixed: 5
- Skipped: 0

## Per-finding disposition

| ID    | Severity | Commit    | Status                                                                     |
|-------|----------|-----------|----------------------------------------------------------------------------|
| WR-01 | Medium   | `dc1df3b` | FIXED — space_key percent-encoded via `url::Url::query_pairs_mut`          |
| WR-02 | Medium   | `ba25541` | FIXED — server-returned space_id validated as `^[0-9]+$` before URL splice |
| IN-01 | Low      | `fd8649a` | FIXED — comment corrected from `archived→InProgress` to `archived→Done`    |
| IN-02 | Low      | `b9cf11c` | FIXED — unused `rusqlite` dev-dep removed from Cargo.toml                  |
| IN-03 | Low      | `f9f61f3` | FIXED — unused `thiserror` prod-dep removed from Cargo.toml                |

## Fixed Issues

### WR-01: `space_key` interpolated into URL without percent-encoding

**Files modified:** `crates/reposix-confluence/Cargo.toml`, `crates/reposix-confluence/src/lib.rs`, `Cargo.lock`
**Commit:** `dc1df3b`
**Applied fix:** Replaced `format!("{}/wiki/api/v2/spaces?keys={}", base, space_key)` with `url::Url::parse(...)` + `query_pairs_mut().append_pair("keys", space_key)`. Added `url = { workspace = true }` to the crate's `[dependencies]` (was already a transitive via reposix-core, now explicit). Added unit test `space_key_is_percent_encoded_in_query_string` that feeds an adversarial key `"A&limit=1#frag ZZ"` and asserts wiremock sees the full literal value back (proves `&`, `=`, `#`, and space are all percent-encoded rather than splintering the query string).

### WR-02: Server-controlled `space_id` used in URL path without validation

**Files modified:** `crates/reposix-confluence/src/lib.rs`
**Commit:** `ba25541`
**Applied fix:** After extracting `id` from the space-lookup response, reject anything that isn't strictly `[0-9]+` with `Error::Other("malformed space id from server: ...")`. This runs before the id is interpolated into the `/wiki/api/v2/spaces/{id}/pages` URL path, so a malicious tenant returning e.g. `"12345/../admin"` is stopped before any second-round HTTP call. Added unit test `list_rejects_non_numeric_space_id` that wires wiremock to return the path-traversal payload and asserts the adapter errors out cleanly.

### IN-01: Misleading comment in contract test (wrong status label)

**Files modified:** `crates/reposix-confluence/tests/contract.rs`
**Commit:** `fd8649a`
**Applied fix:** Changed comment `archived→InProgress` to `archived→Done`. No behaviour change; the test assertion was already correct (invariant 5 only checks for a valid enum variant), this is purely a readability fix.

### IN-02: Unused `rusqlite` dev-dependency

**Files modified:** `crates/reposix-confluence/Cargo.toml`, `Cargo.lock`
**Commit:** `b9cf11c`
**Applied fix:** Removed `rusqlite = { workspace = true }` from `[dev-dependencies]`. No imports referenced it; test compilation and execution unchanged.

### IN-03: Unused `thiserror` production dependency

**Files modified:** `crates/reposix-confluence/Cargo.toml`, `Cargo.lock`
**Commit:** `f9f61f3`
**Applied fix:** Removed `thiserror = { workspace = true }` from `[dependencies]`. The crate delegates all error construction to `reposix_core::Error::Other(...)` and never defines its own error enum.

## Skipped Issues

None.

## Regression-guard verification

- `cargo test -p reposix-confluence --locked`: **19 unit + 2 contract passing** (baseline 17 unit, +2 new tests for WR-01 and WR-02). Live contract test (`contract_confluence_live`) remains `#[ignore]`-gated as before.
- `cargo test --workspace --locked`: **193/193 passing** (baseline 191, +2 new tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: **clean**.
- `cargo fmt --all --check`: **clean**.
- Smoke suite (`scripts/demos/smoke.sh` on release binaries): **4/4 PASS**.
- Live Confluence: `reposix list --backend confluence --project REPOSIX` with real creds returned **4 pages** (Home, issue 425985 "Demo plan", plus 2 more) — confirms neither WR-01 nor WR-02 broke the real-tenant path (space_key `"REPOSIX"` is already URL-safe and the returned space id `"360450"` is all-digits, both fast-path through the new code).

---

_Fixed: 2026-04-13_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
