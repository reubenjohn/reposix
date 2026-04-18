---
phase: 01-core-contracts-security-guardrails
plan: 02
type: execute
wave: 1
depends_on:
  - "01-00"
files_modified:
  - crates/reposix-core/Cargo.toml
  - crates/reposix-core/src/lib.rs
  - crates/reposix-core/src/taint.rs
  - crates/reposix-core/src/path.rs
  - crates/reposix-core/tests/compile_fail.rs
  - crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs
  - crates/reposix-core/tests/compile-fail/tainted_into_untainted.stderr
  - crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs
  - crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.stderr
autonomous: true
requirements:
  - SG-03
  - SG-04
  - SG-05
  - FC-02
user_setup: []

must_haves:
  truths:
    - "`Tainted::new(issue)` compiles; passing that `Tainted<Issue>` to a function expecting `Untainted<Issue>` FAILS to compile."
    - "Calling `reposix_core::Untainted::new(some_issue)` from outside the crate FAILS to compile (visibility-locked to `pub(crate)`) — proven by a second trybuild fixture (FIX 4 from plan-checker)."
    - "`sanitize(tainted_issue_with_version_999999, server_metadata)` returns an `Untainted<Issue>` whose `version` is from `server_metadata`, not from the tainted input."
    - "`sanitize` strips `id`, `created_at`, `version`, `updated_at` from the inbound `Issue` and replaces them with server-authoritative values."
    - "`validate_issue_filename(\"123.md\")` returns `Ok(IssueId(123))`."
    - "`validate_issue_filename` rejects `../123.md`, `123`, `123.md/`, `\\0.md`."
    - "`validate_path_component` rejects `.`, `..`, empty, and anything containing `/` or `\\0`."
  artifacts:
    - path: "crates/reposix-core/src/taint.rs"
      provides: "Tainted<T>/Untainted<T> newtypes + sanitize(tainted, server_meta) -> Untainted<Issue>"
      exports: ["Tainted", "Untainted", "ServerMetadata", "sanitize"]
    - path: "crates/reposix-core/src/path.rs"
      provides: "validate_issue_filename + validate_path_component"
      exports: ["validate_issue_filename", "validate_path_component"]
    - path: "crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs"
      provides: "trybuild fixture proving the type discipline (Tainted -> Untainted is not a valid coercion)"
      contains: "fn takes_untainted"
    - path: "crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs"
      provides: "trybuild fixture proving Untainted::new is pub(crate) — outside-crate construction MUST fail (FIX 4 from plan-checker)"
      contains: "Untainted::new"
    - path: "crates/reposix-core/tests/compile_fail.rs"
      provides: "test harness that invokes trybuild on both compile-fail fixtures"
      contains: "trybuild::TestCases::new"
  key_links:
    - from: "crates/reposix-core/src/taint.rs::sanitize"
      to: "crates/reposix-core/src/issue.rs::Issue"
      via: "destructure Tainted, rebuild Issue overwriting id/created_at/version/updated_at from ServerMetadata"
      pattern: "fn sanitize"
    - from: "crates/reposix-core/tests/compile_fail.rs"
      to: "crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs"
      via: "trybuild::TestCases::compile_fail()"
      pattern: "compile_fail"
    - from: "crates/reposix-core/tests/compile_fail.rs"
      to: "crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs"
      via: "trybuild::TestCases::compile_fail() — second fixture, locks Untainted::new visibility (FIX 4)"
      pattern: "untainted_new_is_not_pub"
    - from: "crates/reposix-core/src/path.rs"
      to: "crates/reposix-core/src/issue.rs::IssueId"
      via: "validate_issue_filename returns Result<IssueId, Error>"
      pattern: "IssueId"
---

<objective>
Land the type-level discipline that prevents tainted network bytes from being used where untainted values are required (SG-05), the `sanitize()` server-field stripper that closes SG-03, and the path/filename validator that Phase 3's FUSE boundary will plug into (SG-04). Also ship the `trybuild` compile-fail tests that ROADMAP success-criterion #1 names explicitly (`tainted_cannot_be_used_where_untainted_required`) **plus** the FIX 4 fixture that locks `Untainted::new` to `pub(crate)`.

Purpose: make the type system enforce what the prose promises. After this plan, no code can accidentally (a) pass a tainted issue body into a privileged operation, (b) round-trip an attacker-controlled `version: 999999` back to the simulator, (c) escape the FUSE mount via `../etc/passwd.md`, or (d) bypass `sanitize()` by calling `Untainted::new(...)` directly from a downstream crate.

**Wave-0 prerequisite:** plan 01-00 has already added `Error::InvalidPath(String)` to `crates/reposix-core/src/error.rs`. This plan does NOT touch `error.rs`.

Output:
  - `crates/reposix-core/src/taint.rs` exporting `Tainted<T>`, `Untainted<T>`, `ServerMetadata`, `sanitize`.
  - `crates/reposix-core/src/path.rs` exporting `validate_issue_filename`, `validate_path_component`.
  - `crates/reposix-core/tests/compile_fail.rs` + two fixtures under `tests/compile-fail/` (+ matching `.stderr` goldens) — the trybuild harness proving (1) the type discipline and (2) `Untainted::new` is private to the crate.
  - `trybuild` added to `[dev-dependencies]` (per 01-CONTEXT.md specifics block).
  - Unit tests colocated in each new module per CLAUDE.md ("Tests live next to the code").
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/01-core-contracts-security-guardrails/01-CONTEXT.md
@.planning/phases/01-core-contracts-security-guardrails/01-00-error-variants.md
@.planning/research/threat-model-and-critique.md
@CLAUDE.md
@Cargo.toml
@crates/reposix-core/Cargo.toml
@crates/reposix-core/src/lib.rs
@crates/reposix-core/src/error.rs
@crates/reposix-core/src/issue.rs

<interfaces>
From `crates/reposix-core/src/issue.rs` — the `Issue` and `IssueId` this plan wraps/returns:

    pub struct IssueId(pub u64);  // #[serde(transparent)], Display impl present
    pub struct Issue {
        pub id: IssueId,
        pub title: String,
        pub status: IssueStatus,
        pub assignee: Option<String>,
        pub labels: Vec<String>,
        pub created_at: DateTime<Utc>,   // server-authoritative; sanitize MUST overwrite
        pub updated_at: DateTime<Utc>,   // server-authoritative; sanitize MUST overwrite
        pub version: u64,                // server-authoritative; sanitize MUST overwrite
        pub body: String,
    }

After Wave-0 plan 01-00, `Error::InvalidPath(String)` already exists in `crates/reposix-core/src/error.rs`. This plan only consumes it.

Public surface this plan MUST expose:

    // reposix_core::Tainted<T> / Untainted<T>
    pub struct Tainted<T>(T);
    pub struct Untainted<T>(T);

    impl<T> Tainted<T> {
        pub fn new(value: T) -> Self;
        pub fn into_inner(self) -> T;
        pub fn as_ref(&self) -> &T;
    }
    impl<T> Untainted<T> {
        pub(crate) fn new(value: T) -> Self;   // pub(crate) — only sanitize constructs
        pub fn into_inner(self) -> T;
        pub fn as_ref(&self) -> &T;
    }

    // No Deref impls. No AsRef<T> trait impl. No From<Tainted<T>> for Untainted<T>.

    pub struct ServerMetadata {
        pub id: IssueId,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub version: u64,
    }

    pub fn sanitize(tainted: Tainted<Issue>, server: ServerMetadata) -> Untainted<Issue>;

    // reposix_core::path::*
    pub fn validate_issue_filename(name: &str) -> Result<IssueId, Error>;
    pub fn validate_path_component(name: &str) -> Result<&str, Error>;
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| network to reposix-core deserialization | Simulator/remote responses deserialize into `Issue`; every field is attacker-influenced until `sanitize` runs. |
| FUSE kernel to daemon path args | Kernel passes path components untrusted; `\0`, `/`, `..` all reachable. |
| operator code to privileged API | Only `Untainted<Issue>` should flow to push-shaped operations. |
| downstream crate → `Untainted::new` | A downstream author who calls `Untainted::new(tainted_value)` would bypass `sanitize`. The `pub(crate)` visibility is the gate; FIX 4's trybuild fixture is the proof. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-06 | Tampering | Attacker-controlled `version: 999999` in frontmatter round-trips back to server (A2) | mitigate | `sanitize()` overwrites `version` with `ServerMetadata.version`; unit test asserts stripped fields. |
| T-01-07 | Elevation of Privilege | Privileged push-path accepts `Tainted<Issue>` via duck-typing | mitigate | `Untainted<T>` has no `From<Tainted<T>>`; `Untainted::new` is `pub(crate)`; trybuild test proves compile error. |
| T-01-07b | Elevation of Privilege | Downstream crate bypasses `sanitize` by calling `Untainted::new(tainted)` directly | mitigate | `Untainted::new` is `pub(crate)`; **FIX 4 trybuild fixture `untainted_new_is_not_pub.rs`** asserts that an external-style call site fails to compile, locking the visibility against accidental promotion to `pub`. |
| T-01-08 | Information Disclosure | Path traversal via `../../etc/passwd.md` (B5) | mitigate | `validate_issue_filename` only accepts `[0-9]+\.md`; `validate_path_component` rejects `/`, `\0`, `.`, `..`, empty. |
| T-01-09 | Denial of Service | NUL byte corrupts the Rust-to-FUSE C-string boundary (B5) | mitigate | `validate_path_component` rejects `\0` before it crosses a CString boundary in Phase 3. |
| T-01-10 | Spoofing | Attacker titles issue `../etc/passwd` hoping filename is title-derived (B5) | mitigate | Filenames are ID-derived in `validate_issue_filename`; titles never participate. |
</threat_model>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Tainted/Untainted newtypes + sanitize(issue, server_metadata)</name>
  <files>
    crates/reposix-core/src/taint.rs
    crates/reposix-core/src/lib.rs
  </files>
  <behavior>
    - `Tainted::new(42_u64).as_ref() == &42` — read-only access works.
    - `Tainted::new(42_u64).into_inner() == 42` — explicit extraction works.
    - `Tainted<u64>` and `Untainted<u64>` implement `Debug` + `Clone` + `PartialEq` + `Eq` when `T` does.
    - `sanitize(Tainted::new(issue_with_version_999999), server_meta_with_version_5)` returns `Untainted<Issue>` whose inner `version == 5`, `id == server_meta.id`, `created_at == server_meta.created_at`, `updated_at == server_meta.updated_at`.
    - `sanitize` preserves the tainted issue's `title`, `status`, `assignee`, `labels`, `body` byte-for-byte (agent-writable per research A2).
    - Named test `server_controlled_frontmatter_fields_are_stripped` is present (ROADMAP SC #1).
  </behavior>
  <action>
    1. Create `crates/reposix-core/src/taint.rs`:
       - Module doc explaining the security contract (CaMeL split, SG-03/SG-05).
       - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct Tainted<T>(T);`
       - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct Untainted<T>(T);`
       - `impl<T> Tainted<T>` with `pub fn new(v: T) -> Self`, `pub fn into_inner(self) -> T`, `pub fn as_ref(&self) -> &T`. Every fn documented.
       - `impl<T> Untainted<T>` with `pub(crate) fn new(v: T) -> Self`, `pub fn into_inner(self) -> T`, `pub fn as_ref(&self) -> &T`. The `pub(crate)` on `new` is load-bearing — FIX 4's trybuild fixture asserts it.
       - Deliberately no `Deref`, no `AsRef<T>` trait impl, no `From<Tainted<T>> for Untainted<T>`, no `serde` derives (per 01-CONTEXT.md discretion block).
       - `#[derive(Debug, Clone)] pub struct ServerMetadata { pub id: IssueId, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>, pub version: u64 }`.
       - `pub fn sanitize(tainted: Tainted<Issue>, server: ServerMetadata) -> Untainted<Issue>` — destructure `tainted.into_inner()`, build a fresh `Issue { id: server.id, created_at: server.created_at, updated_at: server.updated_at, version: server.version, title, status, assignee, labels, body }`, wrap in `Untainted::new(...)`. Infallible.
       - `#[cfg(test)] mod tests` covering every `<behavior>` bullet. One test MUST be named exactly `server_controlled_frontmatter_fields_are_stripped`.
    2. Edit `crates/reposix-core/src/lib.rs`:
       - Add `mod taint;`.
       - Add `pub use taint::{Tainted, Untainted, ServerMetadata, sanitize};`.

    AVOID: implementing `serde` derives (callers `into_inner()` first). AVOID `Deref<Target=T>` (autoderef leaks inner value). AVOID making `Untainted::new` `pub` — FIX 4's trybuild fixture will fail and you'll have to revert.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib taint::tests &amp;&amp; cargo test -p reposix-core server_controlled_frontmatter_fields_are_stripped &amp;&amp; cargo clippy -p reposix-core --lib -- -D warnings &amp;&amp; grep -q 'pub(crate) fn new' crates/reposix-core/src/taint.rs</automated>
  </verify>
  <done>
    `sanitize` preserves agent fields and overwrites server fields; the exact-named test is green; `Untainted::new` visibility is `pub(crate)` (verified by grep, locked in by Task 3's FIX-4 fixture).
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: validate_issue_filename + validate_path_component (consumes Error::InvalidPath from Wave 0)</name>
  <files>
    crates/reposix-core/src/path.rs
    crates/reposix-core/src/lib.rs
  </files>
  <behavior>
    - `Error::InvalidPath(String)` variant exists (already added by Wave-0 plan 01-00 — this task only consumes it).
    - `validate_issue_filename("0.md")` -> `Ok(IssueId(0))`.
    - `validate_issue_filename("123.md")` -> `Ok(IssueId(123))`.
    - `validate_issue_filename("00042.md")` -> `Ok(IssueId(42))` (leading zeros OK; semantic is unsigned int).
    - Rejects: `"abc.md"`, `"..md"`, `"123"`, `"123.txt"`, `"123.md/"`, `"/123.md"`, `"\0.md"`, `""`, `".md"`, `".."`, `"."`, `"123.md\n"`, and a > `u64::MAX` digit string.
    - `validate_path_component("foo")` -> `Ok("foo")`; same for `"foo.md"`, `"issue-123"`.
    - Rejects: `""`, `"."`, `".."`, `"a/b"`, `"a\0b"`, `"/"`, `"\0"`.
    - Named tests `filename_is_id_derived_not_title_derived` and `path_with_dotdot_or_nul_is_rejected` are present (ROADMAP SC #1).
  </behavior>
  <action>
    1. DO NOT edit `crates/reposix-core/src/error.rs`. The `InvalidPath` variant already exists (added by Wave-0 plan 01-00). If you find yourself reaching for that file, stop — it's a sign Wave 0 didn't run.
    2. Create `crates/reposix-core/src/path.rs`:
       - Module doc explaining SG-04 and why we don't trust `std::path::Path::file_name` (normalizes `..` on some platforms, doesn't reject `\0`).
       - `pub fn validate_path_component(name: &str) -> Result<&str>`:
         - Reject empty, `"."`, `".."` with exact-match branches.
         - Iterate bytes; reject if any byte is `b'/'` or `0`.
         - Otherwise return `Ok(name)` unchanged.
       - `pub fn validate_issue_filename(name: &str) -> Result<IssueId>`:
         - First call `validate_path_component(name)?`.
         - Require `name.ends_with(".md")`.
         - Strip suffix; require prefix to be non-empty and `bytes().all(|b| b.is_ascii_digit())`.
         - Parse with `prefix.parse::<u64>()`; map overflow to `Error::InvalidPath(_)`.
         - Return `IssueId(n)`.
       - Every public fn has a `# Errors` doc section.
       - `#[cfg(test)] mod tests` covering every `<behavior>` bullet. Two tests MUST be named exactly `filename_is_id_derived_not_title_derived` and `path_with_dotdot_or_nul_is_rejected`. The id-derived test feeds titles-as-filenames (`"../etc/passwd.md"`, `"my bug.md"`, `"thing is broken.md"`) and asserts each fails.
    3. Edit `crates/reposix-core/src/lib.rs`: add `pub mod path;`.

    AVOID: `regex` crate (the grammar is 10 lines hand-rolled). AVOID `std::path::Path`. AVOID `.unwrap()` anywhere in the validators. AVOID editing `error.rs` (see step 1).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib path::tests &amp;&amp; cargo test -p reposix-core filename_is_id_derived_not_title_derived &amp;&amp; cargo test -p reposix-core path_with_dotdot_or_nul_is_rejected &amp;&amp; cargo clippy -p reposix-core --lib -- -D warnings</automated>
  </verify>
  <done>
    Both validators ship; both ROADMAP SC #1 named tests pass; `Error::InvalidPath` is the only error shape returned (variant supplied by Wave 0).
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: trybuild compile-fail tests — Tainted->Untainted mismatch + Untainted::new visibility lock (FIX 4)</name>
  <files>
    crates/reposix-core/Cargo.toml
    crates/reposix-core/tests/compile_fail.rs
    crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs
    crates/reposix-core/tests/compile-fail/tainted_into_untainted.stderr
    crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs
    crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.stderr
  </files>
  <behavior>
    - Running `cargo test -p reposix-core --test compile_fail` invokes trybuild against the `compile-fail/` directory and passes iff BOTH fixtures fail to compile.
    - **Fixture 1 (`tainted_into_untainted.rs`):** defines `fn takes_untainted(_: Untainted<Issue>) {}` and tries to call it with a `Tainted<Issue>` — locks the SG-05 type discipline. `.stderr` golden contains stable fragments `Untainted` and `Tainted` (regenerable via `TRYBUILD=overwrite`).
    - **Fixture 2 (`untainted_new_is_not_pub.rs`) — FIX 4 from plan-checker:** contains `let _u = reposix_core::Untainted::new(some_issue);` and a golden `.stderr` proving the constructor is private to the crate. The error must mention visibility (`private`, `pub(crate)`, or `inaccessible`).
    - The outer test functions MUST be named `tainted_cannot_be_used_where_untainted_required` (ROADMAP SC #1) and `untainted_new_is_pub_crate_only` (FIX 4 lock).
    - Both fixtures live next to each other under `tests/compile-fail/`; `compile_fail.rs` invokes both in a single `TestCases::new()` for fast trybuild reuse, but exposes them as separate `#[test]` functions so each can be run by name.
  </behavior>
  <action>
    1. Edit `crates/reposix-core/Cargo.toml`:
       - Under `[dev-dependencies]` (add the section if not already present from plan 01-01 — if both plans run in parallel, be prepared to merge), add `trybuild = "1"`.
    2. Create `crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs`:

           // Compile-fail fixture for SG-05 / ROADMAP phase-1 SC #1:
           // passing a `Tainted<Issue>` where `Untainted<Issue>` is required MUST NOT compile.
           use chrono::Utc;
           use reposix_core::{Issue, IssueId, IssueStatus, Tainted, Untainted};

           fn takes_untainted(_: Untainted<Issue>) {}

           fn main() {
               let tainted = Tainted::new(Issue {
                   id: IssueId(1),
                   title: String::new(),
                   status: IssueStatus::Open,
                   assignee: None,
                   labels: vec![],
                   created_at: Utc::now(),
                   updated_at: Utc::now(),
                   version: 0,
                   body: String::new(),
               });
               // This MUST fail to compile: Tainted<Issue> is not Untainted<Issue>,
               // there is no `From<Tainted<_>> for Untainted<_>`, no `Deref`, no
               // coercion. The only legal path is `sanitize(tainted, server_meta)`.
               takes_untainted(tainted);
           }

    3. Create `crates/reposix-core/tests/compile-fail/tainted_into_untainted.stderr`:
       - Generate on first run by setting `TRYBUILD=overwrite` then committing the generated file. Target fragments the human/CI verifies are present: the words `Untainted` and `Tainted` both appear; rustc's "mismatched types" phrasing is also expected. Keep the `.stderr` short (~3 lines) so version drift is easy to regenerate.
    4. **Create `crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs` (FIX 4):**

           // Compile-fail fixture for FIX 4 (plan-checker): proves
           // `reposix_core::Untainted::new` is `pub(crate)` — calling it from
           // outside the crate MUST NOT compile. Without this fixture, a future
           // edit promoting `pub(crate) fn new` to `pub fn new` would silently
           // bypass `sanitize()` and the prior fixture wouldn't catch it.
           use chrono::Utc;
           use reposix_core::{Issue, IssueId, IssueStatus, Untainted};

           fn main() {
               let some_issue = Issue {
                   id: IssueId(1),
                   title: String::new(),
                   status: IssueStatus::Open,
                   assignee: None,
                   labels: vec![],
                   created_at: Utc::now(),
                   updated_at: Utc::now(),
                   version: 0,
                   body: String::new(),
               };
               // MUST fail: `Untainted::new` is `pub(crate)`. The only legal
               // construction site is `reposix_core::sanitize`.
               let _u = Untainted::new(some_issue);
           }

    5. Create `crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.stderr`:
       - Generate via `TRYBUILD=overwrite cargo test -p reposix-core --test compile_fail untainted_new_is_pub_crate_only` then commit. The error MUST mention visibility — search-fragments to grep for: `private`, `inaccessible`, or `pub(crate)`. If rustc phrasing drifts, regenerate; the test failing is the lock.
    6. Create `crates/reposix-core/tests/compile_fail.rs`:

           //! Harness for the compile-fail fixtures under `tests/compile-fail/`.
           //!
           //! - `tainted_cannot_be_used_where_untainted_required` — ROADMAP
           //!   phase-1 SC #1: `Tainted<T>` does NOT coerce to `Untainted<T>`.
           //! - `untainted_new_is_pub_crate_only` — FIX 4 from plan-checker:
           //!   `Untainted::new` is private to the crate; outside-crate call
           //!   sites MUST fail to compile.

           #[test]
           fn tainted_cannot_be_used_where_untainted_required() {
               let t = trybuild::TestCases::new();
               t.compile_fail("tests/compile-fail/tainted_into_untainted.rs");
           }

           #[test]
           fn untainted_new_is_pub_crate_only() {
               let t = trybuild::TestCases::new();
               t.compile_fail("tests/compile-fail/untainted_new_is_not_pub.rs");
           }

    7. Run `TRYBUILD=overwrite cargo test -p reposix-core --test compile_fail` once to generate BOTH `.stderr` files, review them, then commit all four files (two `.rs`, two `.stderr`). Subsequent runs without `TRYBUILD=overwrite` MUST pass.

    AVOID: accepting a wildcard `.stderr` — committing the golden files is the point. AVOID using `trybuild::TestCases::pass` — we want the negative assertion only. AVOID depending on a specific rustc version in the error text; if CI vs. local drifts, regenerate. AVOID combining the two fixtures into one — they assert orthogonal properties (type mismatch vs. visibility) and need to fail independently for clear diagnostics.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --test compile_fail tainted_cannot_be_used_where_untainted_required &amp;&amp; cargo test -p reposix-core --test compile_fail untainted_new_is_pub_crate_only &amp;&amp; cargo test -p reposix-core --all-features tainted_cannot_be_used_where_untainted_required &amp;&amp; test -f crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.stderr</automated>
  </verify>
  <done>
    Both trybuild harness tests pass under their mandated names; both fixtures are committed with matching `.stderr` files; the type discipline AND the `Untainted::new` visibility (FIX 4) are mechanically enforced.
  </done>
</task>

</tasks>

<verification>
Phase-level checks this plan contributes to:

1. ROADMAP SC #1: three of the five required named tests land here — `server_controlled_frontmatter_fields_are_stripped`, `filename_is_id_derived_not_title_derived`, `path_with_dotdot_or_nul_is_rejected`, and `tainted_cannot_be_used_where_untainted_required`. (The fifth, `egress_to_non_allowlisted_host_is_rejected`, lands in plan 01-01.)
2. ROADMAP SC #5 (partial): `cargo clippy -p reposix-core --all-targets -- -D warnings` is clean for the new modules.
3. PROJECT.md SG-03 (server-authoritative fields immutable): enforced by `sanitize`.
4. PROJECT.md SG-04 (filename is `<id>.md`, path validation): enforced by the validators — Phase 3 plugs them into the FUSE boundary.
5. PROJECT.md SG-05 (tainted-content typing): type-level enforcement + mechanical compile-fail proof.
6. **Plan-checker FIX 4:** `cargo test -p reposix-core --test compile_fail untainted_new_is_pub_crate_only` passes — `Untainted::new` visibility is locked against accidental promotion.
</verification>

<success_criteria>
**Goal-backward verification** — if the orchestrator runs:

    cd /home/reuben/workspace/reposix && \
      cargo test -p reposix-core --all-features \
        server_controlled_frontmatter_fields_are_stripped \
        filename_is_id_derived_not_title_derived \
        path_with_dotdot_or_nul_is_rejected \
        tainted_cannot_be_used_where_untainted_required \
        untainted_new_is_pub_crate_only && \
      cargo clippy -p reposix-core --all-targets -- -D warnings

…then phase-1 success-criteria **#1 (four of five named tests, paired with plan 01-01's fifth)** and **#5 (partial)** pass, PROJECT.md SG-03 / SG-04 / SG-05 each have a committed artifact enforcing them, and plan-checker **FIX 4** is satisfied.
</success_criteria>

<output>
After completion, create `.planning/phases/01-core-contracts-security-guardrails/01-02-SUMMARY.md` per the summary template. Must include: the five named tests and their file:line anchors (including `untainted_new_is_pub_crate_only`), the `.stderr` fragments that were locked in for both fixtures, and any drift risks (rustc-version coupling for the compile-fail goldens).
</output>
