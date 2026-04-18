---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: A
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-core/src/issue.rs
  - crates/reposix-core/src/backend.rs
  - crates/reposix-core/src/path.rs
  - crates/reposix-core/src/lib.rs
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "`Issue::parent_id: Option<IssueId>` exists with `#[serde(default, skip_serializing_if = \"Option::is_none\")]`"
    - "`BackendFeature::Hierarchy` variant exists on the enum"
    - "`IssueBackend::root_collection_name(&self) -> &'static str` trait method exists with default returning `\"issues\"`"
    - "`reposix_core::path::slugify_title(&str) -> String` and `slug_or_fallback(&str, IssueId) -> String` exist and are re-exported"
    - "`reposix_core::path::dedupe_siblings(Vec<(IssueId, String)>) -> Vec<(IssueId, String)>` exists and is deterministic"
    - "Every existing downstream call to `IssueBackend::supports(_)` / `list_issues` still compiles unchanged (additive extension only)"
    - "`cargo test -p reposix-core --locked` green; new tests pass for slug, dedupe, parent_id serde, BackendFeature variant"
    - "`cargo clippy --workspace --all-targets --locked -- -D warnings` green (no regressions in downstream crates from the new `parent_id` field)"
  artifacts:
    - path: "crates/reposix-core/src/issue.rs"
      provides: "`Issue` struct with new `parent_id: Option<IssueId>` field + serde attrs"
      contains: "parent_id: Option<IssueId>"
    - path: "crates/reposix-core/src/backend.rs"
      provides: "`BackendFeature::Hierarchy` + `root_collection_name()` default method"
      contains: "Hierarchy"
    - path: "crates/reposix-core/src/path.rs"
      provides: "`SLUG_MAX_BYTES` const; `slugify_title`, `slug_or_fallback`, `dedupe_siblings` public fns + 15+ unit tests"
      contains: "pub fn slugify_title"
      min_lines: 300
    - path: "crates/reposix-core/src/lib.rs"
      provides: "re-exports of new path helpers"
  key_links:
    - from: "crates/reposix-core/src/backend.rs"
      to: "`IssueBackend` trait"
      via: "default method on trait; no breaking change to existing impls"
      pattern: "fn root_collection_name"
    - from: "crates/reposix-core/src/issue.rs"
      to: "serde round-trip"
      via: "`#[serde(default, skip_serializing_if = \"Option::is_none\")]` on parent_id"
      pattern: "skip_serializing_if"
    - from: "crates/reposix-core/src/path.rs"
      to: "IssueId"
      via: "`slug_or_fallback(title, id)` formats id with `page-{:011}` fallback"
      pattern: "page-\\{:011\\}"
---

<objective>
Wave-A foundation. Extend `reposix-core` with every primitive downstream plans depend on: the new `Issue::parent_id` field, the `BackendFeature::Hierarchy` enum variant, the `IssueBackend::root_collection_name()` default trait method, and the `path::slugify_title` + `slug_or_fallback` + `dedupe_siblings` pure-function module. All additive — zero API breakage for existing impls (sim, GitHub, Confluence pre-B1).

Purpose: This plan publishes the types B1, B2, B3, and C all link against. It must land alone in Wave A so the three parallel Wave-B plans start from a green baseline.

Output: Four edits in `reposix-core/src/*.rs` plus the crate's public re-exports in `lib.rs`. All changes covered by unit tests inside the same files.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-RESEARCH.md
@CLAUDE.md
@crates/reposix-core/src/backend.rs
@crates/reposix-core/src/issue.rs
@crates/reposix-core/src/path.rs
@crates/reposix-core/src/lib.rs

<interfaces>
<!-- Existing types the executor extends. DO NOT invent new names. -->

From `crates/reposix-core/src/backend.rs` (existing trait surface — extension points):
```rust
#[non_exhaustive]
pub enum BackendFeature {
    Workflows,
    Delete,
    Transitions,
    StrongVersioning,
    BulkEdit,
    // NEW: Hierarchy — added by this plan
}

#[async_trait]
pub trait IssueBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports(&self, feature: BackendFeature) -> bool;
    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>;
    // ... create/get/update/delete — UNCHANGED ...
    // NEW: default method — does not break existing impls
    fn root_collection_name(&self) -> &'static str { "issues" }
}
```

From `crates/reposix-core/src/issue.rs` (existing Issue struct):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: IssueId,
    pub title: String,
    pub status: IssueStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub assignee: Option<String>,
    #[serde(default)] pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)] pub version: u64,
    #[serde(default)] pub body: String,
    // NEW: parent_id — added by this plan, between `body` and struct close
}
```

From `crates/reposix-core/src/lib.rs` (re-exports; extend but do not rename):
```rust
pub use path::{slugify_title, slug_or_fallback, dedupe_siblings, SLUG_MAX_BYTES, /* existing re-exports */};
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend `Issue::parent_id` + `BackendFeature::Hierarchy` + `root_collection_name`</name>
  <files>
    crates/reposix-core/src/issue.rs,
    crates/reposix-core/src/backend.rs
  </files>
  <behavior>
    - `Issue { parent_id: Some(IssueId(42)), .. }` serializes with a `parent_id: 42` YAML field.
    - `Issue { parent_id: None, .. }` serializes WITHOUT any `parent_id` line (skip-if-none).
    - Round-tripping an old pre-parent_id YAML (no `parent_id` field at all) deserializes with `parent_id == None` (serde default).
    - `BackendFeature::Hierarchy` is a valid enum variant; matching on the enum still compiles in downstream crates because the enum is `#[non_exhaustive]`.
    - `IssueBackend::root_collection_name()` default returns the static str `"issues"`.
    - A blanket test implements a trivial `IssueBackend` impl and asserts `backend.root_collection_name() == "issues"` without overriding the method (proves the default works).
    - `backend.supports(BackendFeature::Hierarchy) == false` by default on unmodified impls.
  </behavior>
  <action>
    **`crates/reposix-core/src/issue.rs`** — add `parent_id` as the LAST field of `Issue`:
    ```rust
    /// Parent in a hierarchy-supporting backend (currently Confluence only).
    /// Always `None` for sim and GitHub. When `Some`, is the parent page/issue
    /// id as reported by the backend. Used by `reposix-fuse` to synthesize the
    /// `tree/` overlay (Phase 13).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<IssueId>,
    ```
    Update every constructor / literal / test that builds an `Issue { ... }` in this file to include `parent_id: None` (search the file; there are several in `#[cfg(test)] mod tests`). Downstream crates use `..Default::default()`-free literal construction, but `Issue` has NO `Default` impl — the breaking-surface is just "add one field per call-site". This task is responsible ONLY for `issue.rs` internal fixups; the sim/GitHub/Confluence call-sites fall out of compile errors and are fixed as part of this same Task via direct grep + patch (they are all of the form `Issue { id, title, ... body }` literals). Enumerate all such sites with:
    ```bash
    grep -rIn "Issue {" crates/ --include='*.rs' | grep -v "parent_id"
    ```
    Patch each to add `parent_id: None,` before the closing `}` (or `parent_id: page.parent_id, // placeholder for B1` in `reposix-confluence` — but leave Confluence wiring to plan B1; in this plan just add `parent_id: None,` everywhere).

    **`crates/reposix-core/src/backend.rs`**:
    1. Add variant `Hierarchy` to `BackendFeature` with a doc comment: `/// Backend exposes a parent/child hierarchy via `Issue::parent_id`. Used by FUSE to synthesize the `tree/` overlay.`
    2. Add default method to `IssueBackend`:
    ```rust
    /// The top-level directory name under which this backend's canonical
    /// `<padded-id>.md` files are mounted. Default `"issues"`. Backends with
    /// a domain-specific vocabulary (e.g. Confluence → `"pages"`) override.
    ///
    /// The return value MUST be a valid single POSIX pathname component:
    /// no `/`, no `..`, non-empty, ASCII.
    fn root_collection_name(&self) -> &'static str { "issues" }
    ```
    3. Add tests at the bottom of the `#[cfg(test)] mod tests` block:
    ```rust
    #[test]
    fn backend_feature_hierarchy_is_a_variant() {
        // trivially compiles iff the variant exists
        let _ = BackendFeature::Hierarchy;
    }

    #[test]
    fn default_root_collection_name_is_issues() {
        struct Stub;
        #[async_trait] impl IssueBackend for Stub {
            fn name(&self) -> &'static str { "stub" }
            fn supports(&self, _: BackendFeature) -> bool { false }
            async fn list_issues(&self, _: &str) -> Result<Vec<Issue>> { Ok(vec![]) }
            async fn get_issue(&self, _: &str, _: IssueId) -> Result<Issue> { unimplemented!() }
            async fn create_issue(&self, _: &str, _: Untainted<Issue>) -> Result<Issue> { unimplemented!() }
            async fn update_issue(&self, _: &str, _: IssueId, _: Untainted<Issue>, _: Option<u64>) -> Result<Issue> { unimplemented!() }
            async fn delete_or_close(&self, _: &str, _: IssueId, _: DeleteReason) -> Result<()> { Ok(()) }
        }
        assert_eq!(Stub.root_collection_name(), "issues");
    }
    ```
    Add tests in `issue.rs` `#[cfg(test)] mod tests`:
    ```rust
    #[test]
    fn parent_id_roundtrips_through_json_when_some() {
        let iss = Issue { /* ...all fields..., */ parent_id: Some(IssueId(42)) };
        let json = serde_json::to_string(&iss).unwrap();
        assert!(json.contains("\"parent_id\":42"));
        let back: Issue = serde_json::from_str(&json).unwrap();
        assert_eq!(back.parent_id, Some(IssueId(42)));
    }

    #[test]
    fn parent_id_omitted_when_none() {
        let iss = Issue { /* ...all fields..., */ parent_id: None };
        let json = serde_json::to_string(&iss).unwrap();
        assert!(!json.contains("parent_id"), "parent_id should be omitted when None, got: {json}");
    }

    #[test]
    fn parent_id_default_on_missing_field() {
        // Old JSON payload, no parent_id field at all — must deserialize with None.
        let json = r#"{"id":1,"title":"t","status":"open","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#;
        let iss: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(iss.parent_id, None);
    }
    ```
  </action>
  <verify>
    <automated>cargo build --workspace --locked &amp;&amp; cargo test -p reposix-core --locked parent_id &amp;&amp; cargo test -p reposix-core --locked default_root_collection_name &amp;&amp; cargo test -p reposix-core --locked backend_feature_hierarchy</automated>
  </verify>
  <done>
    All three new `Issue` tests pass; `BackendFeature::Hierarchy` compiles; `root_collection_name` default returns `"issues"`; no other crate in the workspace fails to compile. Commit: `feat(13-A-1): add Issue::parent_id + BackendFeature::Hierarchy + root_collection_name default`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement `path::slugify_title` + `slug_or_fallback` + `dedupe_siblings`</name>
  <files>
    crates/reposix-core/src/path.rs,
    crates/reposix-core/src/lib.rs
  </files>
  <behavior>
    All of the following must hold as unit tests in `path::tests`:
    - `slugify_title("Hello, World!") == "hello-world"`
    - `slugify_title("  leading and trailing  ") == "leading-and-trailing"`
    - `slugify_title("multiple   spaces") == "multiple-spaces"`
    - `slugify_title("Welcome to reposix") == "welcome-to-reposix"`
    - `slugify_title("") == ""`
    - `slugify_title("日本語") == ""` (multi-byte strip → empty)
    - `slugify_title("🚀 Rocket") == "rocket"` (emoji stripped, rest preserved)
    - `slugify_title("---") == ""` (all-dashes stripped)
    - `slugify_title(".") == ""` (reserved char stripped)
    - `slugify_title("..") == ""`
    - `slugify_title("A".repeat(100).as_str()).len() <= SLUG_MAX_BYTES` and `SLUG_MAX_BYTES == 60`
    - `slugify_title("a-\u{00e9}-b") == "a-b"` (non-ASCII alphanumeric becomes separator, run collapses to single `-`, trailing dashes trimmed)
    - Truncation is char-boundary safe: no panic on `"A".repeat(100)`-style input
    - `slug_or_fallback("", IssueId(42)) == "page-00000000042"` (11-digit padding matching existing `<padded-id>.md` convention)
    - `slug_or_fallback("---", IssueId(7)) == "page-00000000007"`
    - `slug_or_fallback("..", IssueId(3)) == "page-00000000003"`
    - `slug_or_fallback("日本語", IssueId(100)) == "page-00000000100"`
    - `slug_or_fallback("Welcome", IssueId(1)) == "welcome"` (non-fallback path)
    - `dedupe_siblings(vec![(IssueId(5),"foo".into()),(IssueId(3),"foo".into()),(IssueId(4),"bar".into())])` sorted by IssueId returns `[(IssueId(3),"foo"),(IssueId(4),"bar"),(IssueId(5),"foo-2")]`
    - `dedupe_siblings` with 3 colliders: first keeps bare, second gets `-2`, third gets `-3`. Ascending IssueId is the tie-breaker.
    - `dedupe_siblings(vec![])` returns `vec![]`
    - Output preserves ALL inputs (no dropped entries).
  </behavior>
  <action>
    Implement per 13-RESEARCH.md §"Slug helper (zero-dep)" and §"Sibling collision dedupe" — use the code examples verbatim as a starting point, then add the expanded test suite above.

    **Public API in `crates/reposix-core/src/path.rs`:**
    ```rust
    /// Maximum slug length in bytes. Chosen to fit ext4 NAME_MAX (255) with
    /// ~190 bytes of headroom for collision suffixes (`-NN`) and for the
    /// kernel's own bookkeeping.
    pub const SLUG_MAX_BYTES: usize = 60;

    /// Convert a free-form title to a filesystem-safe slug.
    ///
    /// Algorithm (locked by 13-CONTEXT.md §slug-algorithm):
    /// 1. Unicode-lowercase.
    /// 2. Replace every run of non-`[a-z0-9]` ASCII (including multi-byte UTF-8
    ///    bytes) with a single `-`.
    /// 3. Trim leading/trailing `-`.
    /// 4. Byte-truncate to `SLUG_MAX_BYTES` on a UTF-8 char boundary.
    /// 5. Re-trim trailing `-` after truncation.
    ///
    /// This function is pure and `#[must_use]`. It does NOT fall back on empty
    /// results — callers that need a guaranteed-nonempty slug use
    /// [`slug_or_fallback`].
    ///
    /// # Security
    /// Mitigates T-13-01 + T-13-02 from the Phase 13 threat model: the output
    /// cannot contain `/`, `.`, `..`, `\0`, shell metacharacters, or any byte
    /// outside `[a-z0-9-]`. Proven by the `slug_is_ascii_alnum_dash_only`
    /// property test.
    #[must_use]
    pub fn slugify_title(title: &str) -> String { /* ... */ }

    /// [`slugify_title`] with a guaranteed non-empty, non-reserved result.
    /// Falls back to `page-<11-digit-padded-id>` when the slug would be empty,
    /// all dashes, `.`, or `..`. The 11-digit padding matches the existing
    /// `<padded-id>.md` convention in `pages/` and `issues/`.
    #[must_use]
    pub fn slug_or_fallback(title: &str, id: IssueId) -> String { /* ... */ }

    /// Group siblings under one parent by slug, then assign unique suffixes.
    ///
    /// - Input: vec of `(IssueId, slug)` pairs. Order of input is ignored.
    /// - Output: same length as input. For each group of colliding slugs,
    ///   entries are sorted ascending by `IssueId`; the first keeps the bare
    ///   slug, the Nth (N ≥ 2) gets suffix `-N`.
    /// - Deterministic: same input always produces same output across mounts.
    /// - Preserves all input entries (no silent drops).
    #[must_use]
    pub fn dedupe_siblings(siblings: Vec<(IssueId, String)>) -> Vec<(IssueId, String)> { /* ... */ }
    ```

    **Extra property-style test for T-13-01 invariant** (deterministic, no proptest dep needed):
    ```rust
    #[test]
    fn slug_is_ascii_alnum_dash_only_over_adversarial_inputs() {
        let adversarial = [
            "../../../etc/passwd",
            "foo/bar",
            "foo\0bar",
            "$(rm -rf /)",
            "`whoami`",
            "hello;ls",
            "\u{202e}reverse",  // Unicode right-to-left override
            "tab\there",
        ];
        for input in adversarial {
            let s = slugify_title(input);
            assert!(s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "slugify_title({input:?}) = {s:?} contains forbidden char");
            assert!(s != "." && s != "..", "slug must not be '.' or '..'; got {s:?} from {input:?}");
            assert!(!s.contains('/'), "slug must not contain '/'; got {s:?}");
            assert!(!s.contains('\0'), "slug must not contain NUL; got {s:?}");
        }
    }
    ```

    **Re-exports in `crates/reposix-core/src/lib.rs`:**
    After the existing `pub use path::{...};` line, extend to include `slugify_title, slug_or_fallback, dedupe_siblings, SLUG_MAX_BYTES`.

    Run `cargo test -p reposix-core --locked path::tests` and `cargo clippy -p reposix-core --all-targets --locked -- -D warnings` locally until both are green.
  </action>
  <verify>
    <automated>cargo test -p reposix-core --locked path::tests &amp;&amp; cargo clippy -p reposix-core --all-targets --locked -- -D warnings</automated>
  </verify>
  <done>
    All 20+ `path::tests` pass. `slug_is_ascii_alnum_dash_only_over_adversarial_inputs` passes — T-13-01 + T-13-02 mitigations proven. `slug_or_fallback` and `dedupe_siblings` match the locked algorithm exactly. Re-exports are in `reposix_core::` top-level. Commit: `feat(13-A-2): add reposix_core::path::{slugify_title,slug_or_fallback,dedupe_siblings}`.
  </done>
</task>

<task type="auto">
  <name>Task 3: Workspace-wide green check</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    Run the full workspace quality gate to confirm Wave A didn't break any downstream crate:
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    ```
    Any Issue-literal call-site missed in Task 1 surfaces here as a compile error. Fix in the appropriate crate by adding `parent_id: None,`. If fmt reports diffs, `cargo fmt --all` and amend the most recent commit.

    Specific grep to audit:
    ```bash
    grep -rIn "Issue {" crates/ --include='*.rs' | grep -v "parent_id\|pub struct Issue {"
    ```
    This should return zero lines after fixups.
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked &amp;&amp; [ $(grep -rIn "Issue {" crates/ --include='*.rs' | grep -v "parent_id\|pub struct Issue {" | wc -l) -eq 0 ]</automated>
  </verify>
  <done>
    Workspace fmt/clippy/test all green. No Issue-literal anywhere in `crates/` lacks `parent_id: None,`. Test count ≥ prior baseline + new Wave-A tests (should be at least +6: 3 in issue.rs, 2 in backend.rs, 1+ property test in path.rs plus the granular slug/dedupe tests).
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Tainted title bytes → `slugify_title` | Confluence page titles (attacker-influenced storage) feed slug generation. Output must be FS-safe. |
| Tainted parent_id → `Issue::parent_id` | Confluence `parentId` is a backend-provided u64. No interpretation here; it's just plumbed through. Cycle-detection is B2's job. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-01 | Tampering | `slugify_title` output | mitigate | Byte-level filter reduces input to `[a-z0-9-]`; test `slug_is_ascii_alnum_dash_only_over_adversarial_inputs` proves invariant against `../`, `$(...)`, backticks, null bytes, RTL-override Unicode, tabs. |
| T-13-02 | Tampering | slug resolving to `.`/`..`/empty | mitigate | `slug_or_fallback` explicitly tests `s == "."`, `s == ".."`, `s.is_empty()`, `s.chars().all(|c| c == '-')` and returns `page-<padded-id>` fallback. Three test cases cover each trigger. |
| T-13-06 | Tampering | Unicode NFC non-normalization | accept | Two titles that differ only in NFC form (e.g. `é` composed vs decomposed) will slug differently or collide unexpectedly. Documented as known limitation in D2 ADR-003. No dependency budget for a full ICU normalizer in v0.4. |

**Block-on-high:** T-13-01 and T-13-02 mitigations are tested in-task (Task 2) and MUST be green before commit.
</threat_model>

<verification>
Nyquist coverage:
- **Unit:** ≥15 tests for `slugify_title` (ASCII, Unicode strip, emoji, truncation, char-boundary safety, reserved names); ≥5 for `slug_or_fallback`; ≥3 for `dedupe_siblings`; 3 for `Issue::parent_id` serde; 2 for `BackendFeature::Hierarchy` + `root_collection_name`.
- **Workspace-wide:** `cargo test --workspace --locked` green — proves no downstream crate broke from the additive changes.
- **Clippy:** Workspace-level `-D warnings` green.
- **Adversarial:** The single `slug_is_ascii_alnum_dash_only_over_adversarial_inputs` test runs 8 adversarial inputs through `slugify_title` and enforces the output invariant.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `grep -q 'parent_id: Option<IssueId>' crates/reposix-core/src/issue.rs` exits 0.
2. `grep -qE '#\[serde\(default, skip_serializing_if = "Option::is_none"\)\]' crates/reposix-core/src/issue.rs && grep -B1 'parent_id:' crates/reposix-core/src/issue.rs | grep -q skip_serializing_if` exits 0.
3. `grep -qE 'Hierarchy\s*,' crates/reposix-core/src/backend.rs` exits 0.
4. `grep -qE 'fn root_collection_name\(&self\) -> &' crates/reposix-core/src/backend.rs` exits 0.
5. `grep -q '"issues"' crates/reposix-core/src/backend.rs` exits 0.
6. `grep -qE 'pub fn slugify_title' crates/reposix-core/src/path.rs` exits 0.
7. `grep -qE 'pub fn slug_or_fallback' crates/reposix-core/src/path.rs` exits 0.
8. `grep -qE 'pub fn dedupe_siblings' crates/reposix-core/src/path.rs` exits 0.
9. `grep -q 'pub const SLUG_MAX_BYTES: usize = 60' crates/reposix-core/src/path.rs` exits 0.
10. `grep -qE '(slugify_title|slug_or_fallback|dedupe_siblings)' crates/reposix-core/src/lib.rs` exits 0 (re-exported).
11. `cargo build --workspace --locked` exits 0.
12. `cargo test -p reposix-core --locked 2>&1 | grep -cE '(slug|dedupe|parent_id|root_collection_name|backend_feature_hierarchy)' | head -1` returns ≥ 8 matching test names.
13. `cargo test --workspace --locked` exits 0.
14. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
15. `grep -rIn "Issue {" crates/ --include='*.rs' | grep -v "parent_id\|pub struct Issue {" | wc -l` returns 0.
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-A-SUMMARY.md` documenting:
- Final `path::tests` count
- Any Issue-literal call-sites that needed patching (list them — this informs D1's sweep)
- T-13-01 + T-13-02 adversarial test outputs (copy the test's asserted-pass log line)
- Any unexpected clippy findings in downstream crates and how they were resolved
</output>
