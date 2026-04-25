---
phase: 31
plan: 03
status: complete
completed_at: 2026-04-24
---

# Phase 31 Plan 03 — Summary

## Objective achieved

Landed the ARCH-02 type-level guard: two `trybuild` compile-fail
fixtures mechanically lock the `Tainted<T>` vs `Untainted<T>`
discipline so a future contributor cannot silently break it.

1. **`tainted_blob_into_egress.rs`** — tries to pass
   `Tainted<Vec<u8>>` (the actual return type of `Cache::read_blob`)
   to a privileged sink expecting `Untainted<Vec<u8>>`. The compiler
   rejects with `E0308: mismatched types`.
2. **`untainted_new_is_pub_crate.rs`** — tries to call
   `reposix_core::Untainted::new` from `reposix-cache`. The compiler
   rejects with `E0624: associated function 'new' is private`.

## Tasks completed

- **Task 1** — `src/sink.rs` (stub `sink_egress`), trybuild driver
  `tests/compile_fail.rs`, fixture +
  `.stderr` for Tainted→Untainted.
- **Task 2** — second fixture + `.stderr` for the `pub(crate)`
  privacy lock.

Merged into one commit (both tasks are tiny and thematically
inseparable).

## Commits

- `72e5cd4` feat(31-03): trybuild compile-fail fixtures lock Tainted discipline

## `.stderr` regeneration status

Both `.stderr` files were regenerated from scratch via
`TRYBUILD=overwrite` on the first run (expected for a new fixture)
and committed verbatim. **rustc used:** stable 1.94 (matches
project `rust-toolchain.toml`).

First two lines of each:

**`tainted_blob_into_egress.stderr`:**
```
error[E0308]: mismatched types
  --> tests/compile-fail/tainted_blob_into_egress.rs:21:17
```

**`untainted_new_is_pub_crate.stderr`:**
```
error[E0624]: associated function `new` is private
  --> tests/compile-fail/untainted_new_is_pub_crate.rs:12:34
```

If these drift after a rustc upgrade, the procedural control is
spelled out in `tests/compile_fail.rs`'s module docstring:
regenerate via `TRYBUILD=overwrite`, then **review the diff** in
`git status` before committing — a silent overwrite could mask an
actual discipline regression.

## Tests added

- `tests/compile_fail.rs` — 2 `#[test]` functions, each invoking
  one `trybuild::TestCases::new().compile_fail(...)`.
- `tests/compile-fail/tainted_blob_into_egress.{rs,stderr}`.
- `tests/compile-fail/untainted_new_is_pub_crate.{rs,stderr}`.

Total tests added by Plan 03: **2 trybuild fixtures**.

Full `cargo test -p reposix-cache`: 9 runtime tests + 2 trybuild =
**11 green**. Workspace: **452 tests still green, 0 regressions.**

## Seed point for Phase 34

Phase 34 implementers: `sink_egress` in
`crates/reposix-cache/src/sink.rs` is the seed for the real
egress-sanitize function. The signature already enforces
`Untainted<Vec<u8>>` input; Phase 34 replaces the no-op body with
the push path. The trybuild fixture guarantees that every Phase 34
consumer must acquire an `Untainted<Vec<u8>>` via a future
`sanitize_blob` helper (not yet written — the only existing
`sanitize` is `reposix_core::sanitize(Tainted<Issue>, ServerMetadata)
-> Untainted<Issue>`, which is not applicable to raw bytes). A
`sanitize_vec_u8` helper will need to live somewhere; the obvious
home is `reposix_core::taint` so the `pub(crate)` `Untainted::new`
remains respected.

## Deviations from the plan sketch

1. **Plan sketch** suggested the fixture `.stderr` would likely
   match the seed-text on first try. In practice the rustc 1.94
   output includes a note line pointing at `$WORKSPACE/.../taint.rs`
   with the actual `pub(crate) fn new` signature, which is slightly
   richer than the seed — trybuild captured this on overwrite and
   both fixtures now pass in the non-overwrite mode.
2. **`sink.rs`** is `#[doc(hidden)]` via an attribute on both the
   module declaration (in `lib.rs`) and the function itself — belt
   and suspenders so rustdoc does not surface it anywhere.
3. **No cfg-gating of `sink.rs`.** The plan considered gating behind
   a `compile-fail-fixtures` feature but the module is already
   effectively unreachable: `sink_egress` is `doc(hidden)` and its
   body is a no-op. Phase 34 will either gate the real impl behind
   a feature or promote the module to a proper public API with a
   documented privileged-sink contract.

## Acceptance status

All 10 acceptance criteria across Task 1 + Task 2 satisfied.
`cargo test -p reposix-cache --test compile_fail` exits 0 with both
fixtures confirming expected failure. `cargo clippy -p reposix-cache
--all-targets -- -D warnings` clean. `cargo check --workspace`
clean.

## Phase 31 overall

- **3 plans, all complete.**
- **14 atomic commits** (7 feat + 3 test + 2 refactor/chore + 1 fix
  + 3 docs/SUMMARY).
- **Zero regressions:** 452 workspace tests green.
- **ARCH-01, ARCH-02, ARCH-03 all landed** with automated coverage.
- **Cache API public surface:** `reposix_cache::{Cache, Error,
  Result, resolve_cache_path, CACHE_DIR_ENV}` + `sink` (doc-hidden).
- **Substrate ready for Phase 32.**
