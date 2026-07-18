//! Integration tests for `reposix explain <code>` — the RPX code-lookup
//! subcommand (Phase 121 / P121, the codified half of the Rust-compiler-grade
//! UX north star).
//!
//! CATALOG-FIRST SCAFFOLD (W0). This target is committed in the phase's FIRST
//! commit — before the `explain` subcommand (W2), the `reposix-core::codes`
//! registry (W1), or any `.code(...)` site (W3/W4) exist — so the gate row
//! `agent-ux/rpx-codes-registry` and its verifier predate the implementation.
//! It intentionally holds NO test functions yet: an assert_cmd test that shelled
//! out to `reposix explain --list` would RED on today's binary (no such
//! subcommand), and a per-code assertion would reference the not-yet-created
//! `reposix_core::codes` REGISTRY. An empty integration target compiles cleanly
//! and reports zero tests (a green, honest baseline), keeping the workspace + CI
//! green while the catalog contract is in place.
//!
//! Later waves APPEND (never rewrite) here:
//!   - W2: the anchor test (`reposix explain --list` exits 0, prints ≥1 `RPX-`
//!     line), the unknown-code-teaches test (`reposix explain RPX-9999` names
//!     `reposix explain --list`, exits non-zero, no panic), and the
//!     `rustc_parity_of_shape` test (capture `rustc --explain E0308` best-effort
//!     + `reposix explain RPX-0900`; assert the shared shape invariants on
//!     reposix's OWN output — code-header line + non-empty multi-line body + fix
//!     section).
//!   - W3/W4: per-code assertions iterating `reposix_core::codes::REGISTRY`, each
//!     asserting a non-empty cause + `Fix:` + copy-paste `Recovery:`.
//!
//! The assert_cmd harness helper (`workspace_root()` / `Command::cargo_bin`)
//! lands with the W2 anchor test — see `crates/reposix-cli/tests/cli.rs` and
//! `crates/reposix-cli/tests/errors_teach_recovery.rs` for the pattern to copy.
