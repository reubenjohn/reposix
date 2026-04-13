---
phase: 01-core-contracts-security-guardrails
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-core/Cargo.toml
  - crates/reposix-core/src/lib.rs
  - crates/reposix-core/src/taint.rs
  - crates/reposix-core/src/path.rs
  - crates/reposix-core/src/error.rs
  - crates/reposix-core/tests/compile_fail.rs
  - crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs
  - crates/reposix-core/tests/compile-fail/tainted_into_untainted.stderr
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
      provides: "trybuild fixture proving the type discipline"
      contains: "fn takes_untainted"
    - path: "crates/reposix-core/tests/compile_fail.rs"
      provides: "test harness that invokes trybuild on the compile-fail fixture"
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
    - from: "crates/reposix-core/src/path.rs"
      to: "crates/reposix-core/src/issue.rs::IssueId"
      via: "validate_issue_filename returns Result<IssueId, Error>"
      pattern: "IssueId"
---

<objective>
Land the type-level discipline that prevents tainted network bytes from being used where untainted values are required (SG-05), the `sanitize()` server-field stripper that closes SG-03, and the path/filename validator that Phase 3's FUSE boundary will plug into (SG-04). Also ship the `trybuild` compile-fail test that ROADMAP success-criterion #1 names explicitly (`tainted_cannot_be_used_where_untainted_required`).

Purpose: make the type system enforce what the prose promises. After this plan, no code can accidentally (a) pass a tainted issue body into a privileged operation, (b) round-trip an attacker-controlled `version: 999999` back to the simulator, or (c) escape the FUSE mount via `../etc/passwd.md`.

Output:
  - `crates/reposix-core/src/taint.rs` exporting `Tainted<T>`, `Untainted<T>`, `ServerMetadata`, `sanitize`.
  - `crates/reposix-core/src/path.rs` exporting `validate_issue_filename`, `validate_path_component`; plus `Error::InvalidPath(String)` variant.
  - `crates/reposix-core/tests/compile_fail.rs` + `tests/compile-fail/tainted_into_untainted.rs` (+ `.stderr`) — the trybuild harness proving the type discipline.
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

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-06 | Tampering | Attacker-controlled `version: 999999` in frontmatter round-trips back to server (A2) | mitigate | `sanitize()` overwrites `version` with `ServerMetadata.version`; unit test asserts stripped fields. |
| T-01-07 | Elevation of Privilege | Privileged push-path accepts `Tainted<Issue>` via duck-typing | mitigate | `Untainted<T>` has no `From<Tainted<T>>`; `Untainted::new` is `pub(crate)`; trybuild test proves compile error. |
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
       - `impl<T> Untainted<T>` with `pub(crate) fn new(v: T) -> Self`, `pub fn into_inner(self) -> T`, `pub fn as_ref(&self) -> &T`.
       - Deliberately no `Deref`, no `AsRef<T>` trait impl, no `From<Tainted<T>> for Untainted<T>`, no `serde` derives (per 01-CONTEXT.md discretion block).
       - `#[derive(Debug, Clone)] pub struct ServerMetadata { pub id: IssueId, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>, pub version: u64 }`.
       - `pub fn sanitize(tainted: Tainted<Issue>, server: ServerMetadata) -> Untainted<Issue>` — destructure `tainted.into_inner()`, build a fresh `Issue { id: server.id, created_at: server.created_at, updated_at: server.updated_at, version: server.version, title, status, assignee, labels, body }`, wrap in `Untainted::new(...)`. Infallible.
       - `#[cfg(test)] mod tests` covering every `<behavior>` bullet. One test MUST be named exactly `server_controlled_frontmatter_fields_are_stripped`.
    2. Edit `crates/reposix-core/src/lib.rs`:
       - Add `mod taint;`.
       - Add `pub use taint::{Tainted, Untainted, ServerMetadata, sanitize};`.

    AVOID: implementing `serde` derives (callers `into_inner()` first). AVOID `Deref<Target=T>` (autoderef leaks inner value). AVOID making `Untainted::new` `pub`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib taint::tests &amp;&amp; cargo test -p reposix-core server_controlled_frontmatter_fields_are_stripped &amp;&amp; cargo clippy -p reposix-core --lib -- -D warnings</automated>
  </verify>
  <done>
    `sanitize` preserves agent fields and overwrites server fields; the exact-named test is green; `Untainted::new` visibility is `pub(crate)`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: validate_issue_filename + validate_path_component + Error::InvalidPath</name>
  <files>
    crates/reposix-core/src/path.rs
    crates/reposix-core/src/lib.rs
    crates/reposix-core/src/error.rs
  </files>
  <behavior>
    - `Error::InvalidPath(String)` variant exists with `#[error("invalid path: {0}")]`.
    - `validate_issue_filename("0.md")` -> `Ok(IssueId(0))`.
    - `validate_issue_filename("123.md")` -> `Ok(IssueId(123))`.
    - `validate_issue_filename("00042.md")` -> `Ok(IssueId(42))` (leading zeros OK; semantic is unsigned int).
    - Rejects: `"abc.md"`, `"..md"`, `"123"`, `"123.txt"`, `"123.md/"`, `"/123.md"`, `"\0.md"`, `""`, `".md"`, `".."`, `"."`, `"123.md\n"`, and a > `u64::MAX` digit string.
    - `validate_path_component("foo")` -> `Ok("foo")`; same for `"foo.md"`, `"issue-123"`.
    - Rejects: `""`, `"."`, `".."`, `"a/b"`, `"a\0b"`, `"/"`, `"\0"`.
    - Named tests `filename_is_id_derived_not_title_derived` and `path_with_dotdot_or_nul_is_rejected` are present (ROADMAP SC #1).
  </behavior>
  <action>
    1. Edit `crates/reposix-core/src/error.rs`: add variant `InvalidPath(String)` with `#[error("invalid path: {0}")]`. Keep existing variants undisturbed.
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

    AVOID: `regex` crate (the grammar is 10 lines hand-rolled). AVOID `std::path::Path`. AVOID `.unwrap()` anywhere in the validators.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib path::tests &amp;&amp; cargo test -p reposix-core filename_is_id_derived_not_title_derived &amp;&amp; cargo test -p reposix-core path_with_dotdot_or_nul_is_rejected &amp;&amp; cargo clippy -p reposix-core --lib -- -D warnings</automated>
  </verify>
  <done>
    Both validators ship; both ROADMAP SC #1 named tests pass; the `Error::InvalidPath` variant is the only error shape returned.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: trybuild compile-fail test + named `tainted_cannot_be_used_where_untainted_required`</name>
  <files>
    crates/reposix-core/Cargo.toml
    crates/reposix-core/tests/compile_fail.rs
    crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs
    crates/reposix-core/tests/compile-fail/tainted_into_untainted.stderr
  </files>
  <behavior>
    - Running `cargo test -p reposix-core --test compile_fail` invokes trybuild against the `compile-fail/` directory and passes iff the fixture fails to compile.
    - The fixture MUST define `fn takes_untainted(_: Untainted<Issue>) {}` and then try to call it with a `Tainted<Issue>` — exactly the SG-05 discipline.
    - The `.stderr` golden file MUST contain a stable fragment of the error message (`expected struct` / `Untainted` / `Tainted`) — tolerant to rustc's exact phrasing by using trybuild's default behavior (update with `TRYBUILD=overwrite`).
    - The outer test function MUST be named `tainted_cannot_be_used_where_untainted_required` — the exact name ROADMAP SC #1 mandates.
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
       - Generate on first run by setting `TRYBUILD=overwrite` then committing the generated file. Target fragment the human/CI verifies is present: the word `Untainted` and the word `Tainted` both appear in the expected error; rustc's "mismatched types" phrasing is also expected. If exact rustc-version coupling is a concern, keep the `.stderr` short (~3 lines) so version drift is easy to regenerate.
    4. Create `crates/reposix-core/tests/compile_fail.rs`:

           //! Harness for the compile-fail fixtures under `tests/compile-fail/`.
           //! This is ROADMAP phase-1 SC #1: `tainted_cannot_be_used_where_untainted_required`.

           #[test]
           fn tainted_cannot_be_used_where_untainted_required() {
               let t = trybuild::TestCases::new();
               t.compile_fail("tests/compile-fail/tainted_into_untainted.rs");
           }

    5. Run `TRYBUILD=overwrite cargo test -p reposix-core --test compile_fail tainted_cannot_be_used_where_untainted_required` once to generate the `.stderr`, review it, then commit both `.rs` and `.stderr`. Subsequent runs without `TRYBUILD=overwrite` MUST pass.

    AVOID: accepting a wildcard `.stderr` — committing the golden file is the point. AVOID using `trybuild::TestCases::pass` — we want the negative assertion only. AVOID depending on a specific rustc version in the error text; if CI vs. local drifts, regenerate.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --test compile_fail tainted_cannot_be_used_where_untainted_required &amp;&amp; cargo test -p reposix-core --all-features tainted_cannot_be_used_where_untainted_required</automated>
  </verify>
  <done>
    The trybuild harness test passes under its mandated name; the fixture is committed with a matching `.stderr`; the discipline is mechanically enforced.
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
</verification>

<success_criteria>
**Goal-backward verification** — if the orchestrator runs:

    cd /home/reuben/workspace/reposix && \
      cargo test -p reposix-core --all-features \
        server_controlled_frontmatter_fields_are_stripped \
        filename_is_id_derived_not_title_derived \
        path_with_dotdot_or_nul_is_rejected \
        tainted_cannot_be_used_where_untainted_required && \
      cargo clippy -p reposix-core --all-targets -- -D warnings

…then phase-1 success-criteria **#1 (four of five named tests, paired with plan 01-01's fifth)** and **#5 (partial)** pass, and PROJECT.md SG-03 / SG-04 / SG-05 each have a committed artifact enforcing them.
</success_criteria>

<output>
After completion, create `.planning/phases/01-core-contracts-security-guardrails/01-02-SUMMARY.md` per the summary template. Must include: the four named tests and their file:line anchors, the `.stderr` fragment that was locked in, and any drift risks (rustc-version coupling for the compile-fail golden).
</output>
