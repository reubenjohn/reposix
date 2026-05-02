← [back to index](./index.md)

# Task 01-T05 — Workspace gate: clippy + tests + manual smoke note

<read_first>
- Every file edited in this plan.
</read_first>

<action>
Run from repo root:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If any new lint fires, fix it inline. Acceptable lint allowances (with rationale comment in code):
- `clippy::print_stderr` is already allowed at the call site (see existing `#[allow(clippy::print_stderr)]` in `main.rs::diag`).
- `clippy::needless_pass_by_value` for the new audit fn is acceptable — it mirrors `log_helper_fetch` which has `&Connection`.

If `cargo test --workspace` reveals state leaking across tests because of the `OnceLock<u32>` for `BLOB_LIMIT`, that's expected: the `parse_blob_limit` pure-function tests are the regression net, NOT the OnceLock-backed `configured_blob_limit`. Document this in a code comment above the `OnceLock` const.

Add a short note to the bottom of `crates/reposix-remote/src/stateless_connect.rs` (in the module-level doc) listing the env vars the helper consumes:

```rust
//! ## Environment variables
//!
//! - `REPOSIX_BLOB_LIMIT` — max `want` lines per `command=fetch` RPC
//!   turn (Phase 34, ARCH-09). Default 200; `0` = unlimited.
//! - `REPOSIX_ALLOWED_ORIGINS` — egress allowlist (Phase 1). Inherited
//!   via `reposix_core::http::client()`.
```
</action>

<acceptance_criteria>
- `cargo check --workspace` exits 0.
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- `cargo test --workspace` exits 0 with at least +6 new tests over the Phase 33 baseline (1 audit + 5 stateless_connect blob-limit tests).
- `grep -n "REPOSIX_BLOB_LIMIT" crates/reposix-remote/src/stateless_connect.rs` matches at least 3 times (const fn + module doc + parse helper).
</acceptance_criteria>

<threat_model>
N/A (verification task). Module-level doc lists env vars so a security reviewer can grep `REPOSIX_` and find every config knob.
</threat_model>
