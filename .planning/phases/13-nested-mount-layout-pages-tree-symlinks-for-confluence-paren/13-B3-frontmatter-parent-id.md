---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: B3
type: execute
wave: 2
depends_on: [A]
files_modified:
  - crates/reposix-core/src/issue.rs
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "The `Frontmatter` struct inside `pub mod frontmatter { ... }` gains `parent_id: Option<IssueId>` with `#[serde(default, skip_serializing_if = \"Option::is_none\")]`"
    - "`frontmatter::render(issue)` with `issue.parent_id = Some(IssueId(42))` produces YAML containing exactly `parent_id: 42`"
    - "`frontmatter::render(issue)` with `issue.parent_id = None` produces YAML with NO `parent_id` line"
    - "`frontmatter::parse(text)` where `text` contains `parent_id: 42` produces `Issue { parent_id: Some(IssueId(42)), ... }`"
    - "`frontmatter::parse(text)` on a legacy frontmatter WITHOUT `parent_id` line produces `Issue { parent_id: None, ... }` (backward compat)"
    - "Roundtrip: `parse(render(issue)) == issue` for both `parent_id = Some(_)` and `parent_id = None` fixtures"
    - "All existing frontmatter tests still pass (additive extension only)"
    - "`cargo test -p reposix-core frontmatter::` green with ≥ 3 new tests added"
  artifacts:
    - path: "crates/reposix-core/src/issue.rs"
      provides: "Updated `Frontmatter` struct in `pub mod frontmatter` with parent_id field + new roundtrip tests"
      contains: "parent_id: Option<IssueId>"
  key_links:
    - from: "crates/reposix-core/src/issue.rs (frontmatter submodule)"
      to: "Issue::parent_id (Wave A)"
      via: "Frontmatter::from(&Issue) and Frontmatter::into_issue(&self, body)"
      pattern: "parent_id"
---

<objective>
Wave-B3. Extend the `Frontmatter` struct inside `reposix_core::issue::frontmatter` to carry the new `parent_id` field across the Markdown-YAML serialization boundary. Small, surgical change: one struct field added, two conversion sites updated, three new roundtrip tests. Additive and backward-compatible with legacy frontmatter files (parent_id absent → `None`).

Purpose: Wave A added `Issue::parent_id` but the `Frontmatter` substruct in `pub mod frontmatter` mirrors `Issue` for YAML marshaling. Without B3, `render(issue_with_parent)` would drop the parent_id on the floor because `Frontmatter` doesn't know about it. B3 closes that gap.

Output: Edits to the `frontmatter` submodule of `crates/reposix-core/src/issue.rs` — the struct, the `from(&Issue)` impl, the `into_issue(body)` impl, plus tests. Zero other file changes.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-A-core-foundations.md
@crates/reposix-core/src/issue.rs

<interfaces>
<!-- After Wave A ships, Issue.parent_id exists. Frontmatter submodule must mirror: -->

```rust
// crates/reposix-core/src/issue.rs — existing (approximate) structure
pub mod frontmatter {
    #[derive(Debug, Serialize, Deserialize)]
    struct Frontmatter {
        id: super::IssueId,
        title: String,
        status: super::IssueStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")] assignee: Option<String>,
        #[serde(default)] labels: Vec<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        #[serde(default)] version: u64,
        // ADD: parent_id (Wave-B3 adds this)
    }

    pub fn render(issue: &Issue) -> Result<String> { /* ... */ }
    pub fn parse(text: &str) -> Result<Issue> { /* ... */ }
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add `parent_id` to `Frontmatter` + roundtrip tests</name>
  <files>
    crates/reposix-core/src/issue.rs
  </files>
  <behavior>
    New tests in the existing `#[cfg(test)] mod tests` block under the `frontmatter` submodule (or adjacent; follow whatever convention the existing tests use):

    - `frontmatter_renders_parent_id_when_some`: render an Issue with `parent_id = Some(IssueId(42))`, assert the output string contains the substring `"parent_id: 42\n"` (YAML line).
    - `frontmatter_omits_parent_id_when_none`: render an Issue with `parent_id = None`, assert the output string does NOT contain `"parent_id"` anywhere.
    - `frontmatter_parses_parent_id_when_present`: parse a frontmatter block that contains `parent_id: 42`, assert the returned Issue has `parent_id == Some(IssueId(42))`.
    - `frontmatter_parses_legacy_without_parent_id`: parse a frontmatter block that does NOT mention parent_id (simulates a pre-Phase-13 file on disk), assert `parent_id == None` (no error, no panic).
    - `frontmatter_roundtrip_with_parent`: build Issue with parent, render, parse, assert equality (deep equality on the Issue struct — this catches any drift between Frontmatter and Issue).
    - `frontmatter_roundtrip_without_parent`: same with `parent_id = None`.
    - (Existing tests for other fields must continue to pass — do NOT edit them.)
  </behavior>
  <action>
    Open `crates/reposix-core/src/issue.rs`, find the `pub mod frontmatter { ... }` block. Inside:

    1. **Add field to `Frontmatter`** (keep serde attrs consistent with the top-level `Issue::parent_id`):
    ```rust
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent_id: Option<super::IssueId>,
    ```
    Place it after `version` (mirror the order in `Issue`).

    2. **Update the conversion from `&Issue`** (there's a `From<&Issue> for Frontmatter` or a manual constructor inside `render`; locate it and add `parent_id: issue.parent_id,`).

    3. **Update the conversion to `Issue`** inside `parse` (locate the `Issue { ... }` literal that's built from the deserialized `Frontmatter` + body; add `parent_id: fm.parent_id,`).

    4. **Write the 6 tests** listed in the behavior section. Use the existing `render`/`parse` public functions — don't reach into the `Frontmatter` struct internals from tests (it's private).

    Example fixture for the legacy-parse test:
    ```yaml
    ---
    id: 1
    title: Legacy issue
    status: open
    created_at: 2025-01-01T00:00:00Z
    updated_at: 2025-01-01T00:00:00Z
    version: 1
    ---
    Body goes here.
    ```
    This is a pre-Phase-13 file. Parsing it must yield `parent_id: None` without error.

    Run `cargo test -p reposix-core --locked frontmatter::` and `cargo clippy -p reposix-core --all-targets --locked -- -D warnings` locally until both are green.
  </action>
  <verify>
    <automated>cargo test -p reposix-core --locked frontmatter:: &amp;&amp; cargo clippy -p reposix-core --all-targets --locked -- -D warnings &amp;&amp; cargo test -p reposix-core --locked 2>&amp;1 | grep -cE 'frontmatter.*parent_id' | head -1 | awk '{ exit ($1 &gt;= 3 ? 0 : 1) }'</automated>
  </verify>
  <done>
    Frontmatter struct has the new field with correct serde attrs. Six new roundtrip tests pass. No existing frontmatter test broke. Commit: `feat(13-B3): carry Issue::parent_id through frontmatter serialization`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Workspace-wide green check</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    ```
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked</automated>
  </verify>
  <done>
    Workspace green. B3 didn't regress anything.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| User-authored frontmatter on disk → Frontmatter deserialization | A user or malicious actor with write access to the mount could craft a frontmatter with a `parent_id: 999999999` pointing at a non-existent page. This is handled at tree-build time (Wave B2) as "orphan parent → tree root", so no blast radius here. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-FM1 | Tampering | User-authored frontmatter `parent_id` collides with real pages | accept | Frontmatter is user-controlled by design (that's the entire value of the mount). The backend's own parent_id always takes precedence because `list_issues` is the source of truth, not disk state. The FUSE read path re-fetches from backend; the user's on-disk edits don't feed back into tree construction. |
| T-13-FM2 | Tampering | Malformed YAML (`parent_id: "abc"`) | mitigate | serde's `Deserialize` for `Option<IssueId>` where `IssueId = u64` will fail on a string. `parse()` already returns `Result` — a malformed parent_id surfaces as `Err`. No new risk. |
</threat_model>

<verification>
Nyquist coverage:
- **Unit:** 6 new frontmatter tests (3 render, 3 parse/roundtrip).
- **Backward compat:** `frontmatter_parses_legacy_without_parent_id` exercises the serde(default) path — proves old files on disk still parse.
- **Workspace:** `cargo test --workspace --locked` green.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `grep -q 'parent_id: Option<super::IssueId>' crates/reposix-core/src/issue.rs` exits 0 (or `parent_id: Option<IssueId>` depending on how the existing Frontmatter refers to IssueId — match the existing pattern).
2. `cargo test -p reposix-core --locked frontmatter::` exits 0.
3. `cargo test -p reposix-core --locked 2>&1 | grep -cE 'frontmatter.*parent_id|parent_id.*roundtrip|parses_legacy'` returns ≥ 3.
4. `cargo test --workspace --locked` exits 0.
5. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B3-SUMMARY.md` documenting:
- Final frontmatter test count (before/after)
- Confirmation that the legacy-parse test fixture actually represents a real pre-Phase-13 frontmatter shape (copy-paste from an existing `.md` file if possible, e.g. a Phase-11 Confluence demo output)
</output>
