---
phase: 31
plan: 03
type: execute
wave: 3
depends_on:
  - "31-01"
  - "31-02"
files_modified:
  - crates/reposix-cache/tests/compile_fail.rs
  - crates/reposix-cache/tests/compile-fail/tainted_blob_into_egress.rs
  - crates/reposix-cache/tests/compile-fail/tainted_blob_into_egress.stderr
  - crates/reposix-cache/tests/compile-fail/untainted_new_is_pub_crate.rs
  - crates/reposix-cache/tests/compile-fail/untainted_new_is_pub_crate.stderr
  - crates/reposix-cache/src/sink.rs
  - crates/reposix-cache/src/lib.rs
autonomous: true
requirements:
  - ARCH-02
tags:
  - rust
  - trybuild
  - compile-fail
  - taint
  - type-safety
user_setup: []

must_haves:
  truths:
    - "A `trybuild` compile-fail fixture exists that fails to compile if a caller passes a `Tainted<Vec<u8>>` to a privileged sink expecting `Untainted<Vec<u8>>`."
    - "A second compile-fail fixture confirms `reposix_core::Untainted::new` is NOT callable from outside `reposix-core` (pub(crate) discipline intact)."
    - "`cargo test -p reposix-cache --test compile_fail` exits 0 — trybuild reports both expected compile failures matched their captured `.stderr` fingerprints."
    - "`reposix-cache` ships a module `sink` demonstrating a privileged-sink signature `fn sink_egress(_: Untainted<Vec<u8>>)` that represents the shape Phase 34's push path will take — the sink exists only for compile-fail-fixture consumption, guarded by a `doc(hidden)` + cfg gate OR lives under `#[cfg(any(test, feature = \"compile-fail-fixtures\"))]`."
  artifacts:
    - path: "crates/reposix-cache/tests/compile_fail.rs"
      provides: "trybuild driver test that invokes both compile-fail fixtures."
      contains: "trybuild::TestCases::new"
    - path: "crates/reposix-cache/tests/compile-fail/tainted_blob_into_egress.rs"
      provides: "A Rust file designed to fail to compile: tries to pass Tainted<Vec<u8>> to a sink expecting Untainted<Vec<u8>>."
      contains: "Tainted::new"
    - path: "crates/reposix-cache/tests/compile-fail/tainted_blob_into_egress.stderr"
      provides: "Captured compiler error signature — trybuild asserts the error output matches this file."
      contains: "mismatched types"
    - path: "crates/reposix-cache/tests/compile-fail/untainted_new_is_pub_crate.rs"
      provides: "A Rust file designed to fail to compile: tries to call reposix_core::Untainted::new from outside reposix-core."
      contains: "Untainted::new"
    - path: "crates/reposix-cache/tests/compile-fail/untainted_new_is_pub_crate.stderr"
      provides: "Captured privacy-violation error."
      contains: "private"
    - path: "crates/reposix-cache/src/sink.rs"
      provides: "Privileged-sink stub `fn sink_egress(_: Untainted<Vec<u8>>)` used by the compile-fail fixture. Gated so it does not appear in production API."
      contains: "pub fn sink_egress"
  key_links:
    - from: "crates/reposix-cache/tests/compile_fail.rs"
      to: "crates/reposix-cache/tests/compile-fail/tainted_blob_into_egress.rs"
      via: "trybuild::TestCases::compile_fail"
      pattern: "compile_fail"
    - from: "crates/reposix-cache/tests/compile-fail/tainted_blob_into_egress.rs"
      to: "crates/reposix-cache/src/sink.rs"
      via: "use reposix_cache::sink::sink_egress"
      pattern: "sink_egress"
---

# Phase 31-03 — Compile-fail fixtures (ARCH-02 taint discipline)

<objective>
Ship the type-level guard that Phase 31 ARCH-02 requires: a `trybuild` compile-fail fixture proving `reposix_core::Tainted<Vec<u8>>` cannot be passed to a privileged sink expecting `Untainted<Vec<u8>>`. Pair it with a second compile-fail fixture that locks `reposix_core::Untainted::new`'s `pub(crate)` discipline (prevents downstream crates from constructing an `Untainted<T>` directly).

Purpose: ARCH-02 (mechanical type-system enforcement — code review cannot miss it). RESEARCH §Pattern 3 "Tainted/Untainted boundary" and §Code Example 2.
Output: Two compile-fail fixtures + a trybuild driver test + a minimal privileged-sink stub in `sink.rs` (gated so it does not leak into production APIs). When the fixtures fail to compile with the expected error patterns, trybuild reports PASS.

Scope narrowness: this plan does NOT refactor `Tainted`/`Untainted` — those exist already in `reposix-core::taint` and are not modified. This plan does NOT implement the push path — that is Phase 34. This plan exists solely to land the trybuild discipline alongside the type it constrains.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-CONTEXT.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-01-SUMMARY.md

@crates/reposix-cache/Cargo.toml
@crates/reposix-cache/src/lib.rs
@crates/reposix-core/src/taint.rs
@crates/reposix-core/src/lib.rs

<interfaces>
<!-- Load-bearing types Plan 03's fixtures reference. -->

From `crates/reposix-core/src/taint.rs`:
```rust
pub struct Tainted<T>(T);
impl<T> Tainted<T> {
    pub fn new(value: T) -> Self;                // pub — downstream can wrap
    pub fn into_inner(self) -> T;                // pub — explicit escape hatch
    pub fn inner_ref(&self) -> &T;               // pub — read-only borrow
}

pub struct Untainted<T>(T);
impl<T> Untainted<T> {
    pub(crate) fn new(value: T) -> Self;         // PUB(CRATE) — only reposix-core can construct
    pub fn into_inner(self) -> T;
    pub fn inner_ref(&self) -> &T;
}

// The ONLY public path from Tainted<Issue> to Untainted<Issue>:
pub fn sanitize(tainted: Tainted<Issue>, server: ServerMetadata) -> Untainted<Issue>;
// Note: sanitize only handles Issue. There is NO sanitize for Vec<u8>
// in v0.9.0 — this is intentional; Phase 34's push path will add one.
```

Critical deliberate omissions in taint.rs — the compile-fail fixture will exploit these:
- No `impl<T> From<Tainted<T>> for Untainted<T>` — you cannot `.into()` from tainted to untainted.
- No `impl<T> Deref for Tainted<T>` — you cannot auto-coerce via `&*tainted`.
- No `impl<T> AsRef<Untainted<T>> for Tainted<T>` — no transitive upgrade.
- `Untainted::new` is `pub(crate)` — `use reposix_core::Untainted; Untainted::new(x)` from an external crate is a privacy error.

Plan 03 stubs a privileged-sink `fn sink_egress(_: Untainted<Vec<u8>>)` in `reposix-cache`. When the fixture tries `sink_egress(tainted_vec_u8)`, the compiler yields `mismatched types: expected Untainted<Vec<u8>>, found Tainted<Vec<u8>>`. That exact diagnostic is captured as `.stderr` for trybuild to match.

From `crates/reposix-cache/Cargo.toml` (already configured in Plan 01):
```toml
[dev-dependencies]
trybuild = "1"
```

Confirmed available in workspace. Plan 03 does NOT add new dev-deps.
</interfaces>
</context>

## Chapters

- **[T01 — Privileged-sink stub + Tainted→Untainted compile-fail fixture](./T01.md)**
  Creates `sink.rs` with the `sink_egress` stub, the first compile-fail fixture (`tainted_blob_into_egress.rs`), its `.stderr` fingerprint, and the trybuild driver. Acceptance: `cargo test -p reposix-cache --test compile_fail` green for this fixture.

- **[T02 — Compile-fail fixture for `Untainted::new` pub(crate) discipline](./T02.md)**
  Adds the second fixture (`untainted_new_is_pub_crate.rs`) proving callers outside `reposix-core` cannot call `Untainted::new` directly. Extends the driver. Also contains the `<threat_model>`, `<verification>`, `<success_criteria>`, and `<output>` sections.
