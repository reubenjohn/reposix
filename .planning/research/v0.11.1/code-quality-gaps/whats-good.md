← [back to index](./index.md)

# What's actually GOOD

A senior reviewer should also call out the parts that are well-built — they're not the squeaky wheel, but they're the load-bearing assets you don't want to disturb:

1. **`Tainted<T>` / `Untainted<T>` discipline** in `crates/reposix-core/src/taint.rs`. Newtype with no `Deref`, `Untainted::new` is `pub(crate)`, the *only* legal upcast is `sanitize`. Compile-fail fixtures (`crates/reposix-cache/tests/compile_fail.rs:22`) lock the invariant. Genuinely well-designed for a security-critical seam.

2. **HTTP allowlist gate** (`crates/reposix-core/src/http.rs`). The `HttpClient` newtype hides `reqwest::Client` behind a private field. The workspace `clippy.toml` (`disallowed-methods = ["reqwest::Client::new", "reqwest::Client::builder", "reqwest::ClientBuilder::new"]`) enforces this at compile time. Defence-in-depth done right.

3. **`reposix-cache::error::Error`** (`crates/reposix-cache/src/error.rs`). Eight typed variants with no `Other(String)` escape hatch. `CacheCollision { expected, found }` and `OidDrift { requested, actual, issue_id }` carry structured payload — exactly the pattern `reposix-core::Error` should adopt.

4. **Audit append-only trigger** (`crates/reposix-core/src/audit.rs:35`, schema in `crates/reposix-core/fixtures/audit.sql`). Schema-level `BEFORE UPDATE/DELETE` triggers raise `RAISE(ABORT, …)`. Tested at `crates/reposix-core/tests/audit_schema.rs:128`. The runtime guarantee matches the docstring.

5. **`BackendConnector` trait dyn-compatibility proof** (`crates/reposix-core/src/backend.rs:247`). One `#[allow(dead_code)] fn _assert_dyn_compatible(_: &dyn BackendConnector) {}` line costs nothing and breaks the build immediately on any future violation. Idiomatic Rust pattern.

6. **`SyncTag` time-travel** (`crates/reposix-cache/src/sync_tag.rs`) — `parse_sync_tag_timestamp`, `format_sync_tag_slug`, deterministic ordering, fully tested at `crates/reposix-cache/tests/sync_tags.rs`. Replayable artefact, no implicit state.

7. **Doctor's check-isolation pattern** (`crates/reposix-cli/src/doctor.rs:336-940`). Each `check_*` function returns `DoctorFinding` and is independently testable. `tests/doctor.rs` exercises ~16 of the 18 checks separately. Adding a check is one new function; it never explodes the call-site complexity.

8. **`#![deny(clippy::print_stdout)]` in `reposix-remote/src/main.rs:10`**. The remote helper's protocol depends on stdout being only protocol-frames; an accidental future `println!` is now a compile error rather than a corrupted protocol stream. This is exactly the right tool for the right invariant.

9. **`gix::ObjectId`-typed throughout the cache crate** (`crates/reposix-cache/src/builder.rs:22,353`). Never stringly-typed except at the SQL boundary (where it's `oid.to_hex().to_string()` once). The SHA1 vs SHA256 future-proofing falls out of `repo.object_hash()`.

10. **`#[non_exhaustive]` on `BackendFeature` and `DeleteReason`** (`crates/reposix-core/src/backend.rs:51,83`). Adding a variant is non-breaking. Future-proof at zero cost.

---

*Compiled by reading every `crates/*/src/**/*.rs` and spot-checking `tests/cli.rs`, `tests/agent_flow.rs`, `tests/contract.rs`. No `cargo` or `rustc` invocations per CLAUDE.md RAM rule.*
