---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: B1
type: execute
wave: 2
depends_on: [A]
files_modified:
  - crates/reposix-confluence/src/lib.rs
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "`ConfPage` deserializes `parentId: Option<String>` and `parentType: Option<String>` with `#[serde(default)]`"
    - "`translate(ConfPage)` populates `Issue.parent_id = Some(IssueId(n))` iff `parentType == \"page\"` AND `parentId` parses as u64"
    - "`translate(ConfPage)` with `parentType ∈ {\"folder\",\"whiteboard\",\"database\",...}` sets `parent_id = None` AND emits exactly one `tracing::debug!` line per page"
    - "`translate(ConfPage)` with no `parentId` at all (top-level page) sets `parent_id = None`"
    - "`ConfluenceReadOnlyBackend::supports(BackendFeature::Hierarchy) == true`"
    - "`ConfluenceReadOnlyBackend::root_collection_name() == \"pages\"`"
    - "Every pre-existing Phase-11 test still passes (additive change only)"
    - "No HTTP is emitted by the new tests — they operate on `translate(ConfPage { ... })` synthesized structs + wiremock-backed list tests that return payloads WITH parentId/parentType fields"
  artifacts:
    - path: "crates/reposix-confluence/src/lib.rs"
      provides: "Extended ConfPage struct, updated translate(), supports(Hierarchy) override, root_collection_name override, new unit tests"
      contains: "parent_id"
  key_links:
    - from: "crates/reposix-confluence/src/lib.rs"
      to: "reposix_core::Issue::parent_id (from Wave A)"
      via: "`translate()` writes the field"
      pattern: "Issue \\{"
    - from: "crates/reposix-confluence/src/lib.rs"
      to: "reposix_core::BackendFeature::Hierarchy (from Wave A)"
      via: "`supports(BackendFeature::Hierarchy) => true` match arm"
      pattern: "BackendFeature::Hierarchy"
    - from: "crates/reposix-confluence/src/lib.rs"
      to: "IssueBackend::root_collection_name"
      via: "override returning `\"pages\"`"
      pattern: "fn root_collection_name"
---

<objective>
Wave-B1. Extend `reposix-confluence` to populate the new `Issue::parent_id` field from Confluence REST v2 `parentId` + `parentType`, report `supports(BackendFeature::Hierarchy) == true`, and override `root_collection_name()` to `"pages"`. Three code-level changes; 4+ new wiremock tests.

Purpose: This is what makes `tree/` actually appear for Confluence mounts. Without this plan, the FUSE wiring from Wave C has nothing to group by.

Output: One file edited (`crates/reposix-confluence/src/lib.rs`), ≥4 new wiremock tests added to the existing `#[cfg(test)] mod tests` block.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-RESEARCH.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-A-core-foundations.md
@crates/reposix-confluence/src/lib.rs
@crates/reposix-core/src/backend.rs
@crates/reposix-core/src/issue.rs
@docs/decisions/002-confluence-page-mapping.md

<interfaces>
<!-- After Wave A ships, these types exist in reposix-core: -->

```rust
// reposix_core::issue::Issue — new field
pub struct Issue {
    // ... all existing fields ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<IssueId>,
}

// reposix_core::backend::BackendFeature — new variant
pub enum BackendFeature { Workflows, Delete, Transitions, StrongVersioning, BulkEdit, Hierarchy }

// reposix_core::backend::IssueBackend — new default method
pub trait IssueBackend: Send + Sync {
    // ... existing methods ...
    fn root_collection_name(&self) -> &'static str { "issues" }
}
```

<!-- Existing Confluence code to extend (see 13-RESEARCH.md §"Extending Confluence deserialization"): -->

```rust
// crates/reposix-confluence/src/lib.rs — existing ConfPage struct
#[derive(Debug, Deserialize)]
struct ConfPage {
    id: String,
    status: String,
    title: String,
    #[serde(rename = "createdAt")] created_at: chrono::DateTime<chrono::Utc>,
    version: ConfVersion,
    #[serde(default, rename = "ownerId")] owner_id: Option<String>,
    #[serde(default)] body: Option<ConfPageBody>,
    // ADD: parent_id + parent_type
}

// crates/reposix-confluence/src/lib.rs — existing translate() fn
fn translate(page: ConfPage) -> Result<Issue> {
    // ... existing field mapping ...
    // ADD: parent_id derivation from parentType == "page" filter
    Ok(Issue { /* ... */, parent_id: /* derived */ })
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend `ConfPage` deserialization + `translate()` for parent_id</name>
  <files>
    crates/reposix-confluence/src/lib.rs
  </files>
  <behavior>
    New unit tests in the existing `#[cfg(test)] mod tests` block (all `#[tokio::test]` or plain `#[test]`):

    - `translate_populates_parent_id_for_page_parent`: synthesize `ConfPage { id: "99", parent_id: Some("42".into()), parent_type: Some("page".into()), ... }`, call `translate(page).unwrap()`, assert `issue.parent_id == Some(IssueId(42))`.
    - `translate_treats_folder_parent_as_orphan`: `parent_type: Some("folder".into())` → `issue.parent_id == None`.
    - `translate_treats_whiteboard_parent_as_orphan`: `parent_type: Some("whiteboard".into())` → `issue.parent_id == None`.
    - `translate_treats_missing_parent_as_orphan`: both `parent_id` and `parent_type` = `None` → `issue.parent_id == None`.
    - `translate_handles_unparseable_parent_id`: `parent_id: Some("not-a-number".into()), parent_type: Some("page".into())` → `issue.parent_id == None` (silently drop — do NOT propagate as Err; top-level pages must still list even if one page's parent is malformed).
    - `list_populates_parent_id_end_to_end`: wiremock returns a page list where one page has `"parentId": "42", "parentType": "page"` and another has no parentId. Run `backend.list_issues("KEY")`; assert one issue has `parent_id == Some(IssueId(42))` and the other has `parent_id == None`. This proves the field survives JSON → ConfPage → translate.
    - `supports_hierarchy_returns_true`: `assert!(backend.supports(BackendFeature::Hierarchy))`.
    - `supports_other_features_still_false`: `assert!(!backend.supports(BackendFeature::Workflows))` + Delete/Transitions/StrongVersioning/BulkEdit.
    - `root_collection_name_returns_pages`: `assert_eq!(backend.root_collection_name(), "pages")`.
  </behavior>
  <action>
    **Extend `ConfPage`** (add two fields; keep `#[serde(default)]` for forward-compat — Confluence may one day omit parentId entirely):
    ```rust
    #[derive(Debug, Deserialize)]
    struct ConfPage {
        // ... existing fields ...
        #[serde(default, rename = "parentId")]
        parent_id: Option<String>,
        #[serde(default, rename = "parentType")]
        parent_type: Option<String>,
    }
    ```

    **Update `translate`** (the exact logic from 13-RESEARCH.md §"Extending Confluence deserialization"):
    ```rust
    fn translate(page: ConfPage) -> Result<Issue> {
        let id = page.id.parse::<u64>()
            .map_err(|_| Error::Other(format!("confluence page id not a u64: {:?}", page.id)))?;

        let parent_id = match (page.parent_id.as_deref(), page.parent_type.as_deref()) {
            (Some(pid_str), Some("page")) => {
                // Silently drop unparseable ids (test: translate_handles_unparseable_parent_id).
                // Failing the whole list because one parent_id is malformed is worse than
                // surfacing that page as a tree root — the fallback degrades gracefully.
                match pid_str.parse::<u64>() {
                    Ok(n) => Some(IssueId(n)),
                    Err(_) => {
                        tracing::debug!(page_id = %page.id, bad_parent = %pid_str, "parent_id not parseable as u64, treating as orphan");
                        None
                    }
                }
            }
            (Some(_), Some(other)) => {
                tracing::debug!(page_id = %page.id, parent_type = %other, "non-page parent type, treating as orphan");
                None
            }
            _ => None,
        };

        Ok(Issue {
            id: IssueId(id),
            title: page.title,
            status: status_from_confluence(&page.status),
            assignee: page.owner_id,
            labels: vec![],
            created_at: page.created_at,
            updated_at: page.version.created_at,
            version: page.version.number,
            body: page.body.and_then(|b| b.storage).map(|s| s.value).unwrap_or_default(),
            parent_id,
        })
    }
    ```

    **Override trait methods** on `impl IssueBackend for ConfluenceReadOnlyBackend`:
    ```rust
    fn supports(&self, feature: BackendFeature) -> bool {
        matches!(feature, BackendFeature::Hierarchy)
    }

    fn root_collection_name(&self) -> &'static str { "pages" }
    ```

    Note: the existing `supports` impl probably returns `false` for everything. Change that single line to the `matches!(feature, BackendFeature::Hierarchy)` form above. This flips Hierarchy to true while leaving every other feature false, matching the Phase-11 capability matrix.

    **Wiremock test** — model after the existing `list_resolves_space_key_and_fetches_pages` test. The pages-list response JSON needs two additional fields per entry:
    ```json
    {
      "id": "98765",
      "status": "current",
      "title": "Child Page",
      "parentId": "360556",
      "parentType": "page",
      "createdAt": "2026-01-01T00:00:00Z",
      "version": { "number": 1, "createdAt": "2026-01-01T00:00:00Z" }
    }
    ```
    And one entry with `"parentType": "folder"` to exercise the orphan branch, and one with neither field to exercise the top-level-page branch.

    **Update the existing `supports_reports_no_features` test** (from Phase 11-A) to match the new behavior: it should now assert Hierarchy is `true` while all others remain `false`. Rename to `supports_reports_only_hierarchy` if clearer. Do NOT delete — just adapt.

    Run locally:
    ```bash
    cargo test -p reposix-confluence --locked
    cargo clippy -p reposix-confluence --all-targets --locked -- -D warnings
    ```
  </action>
  <verify>
    <automated>cargo test -p reposix-confluence --locked &amp;&amp; cargo clippy -p reposix-confluence --all-targets --locked -- -D warnings &amp;&amp; cargo test -p reposix-confluence --locked 2>&amp;1 | grep -cE 'parent_id|hierarchy|root_collection_name' | head -1 | awk '{ exit ($1 &gt;= 6 ? 0 : 1) }'</automated>
  </verify>
  <done>
    All new tests pass. Existing Phase-11 wiremock tests still pass (the phase-11 tests use pages WITHOUT parentId/parentType, so the new `#[serde(default)]` fields mean they continue to deserialize unchanged). `supports(Hierarchy) == true`. `root_collection_name() == "pages"`. Commit: `feat(13-B1): populate Issue::parent_id from Confluence parentId + supports(Hierarchy) + root_collection_name("pages")`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Workspace-wide green check</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    Confirm B1 is isolated — no downstream crate should care yet (C has not run):
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    ```
    Expected: ≥ 6 new test names containing `parent_id`/`hierarchy`/`root_collection_name` substrings.
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked</automated>
  </verify>
  <done>
    Full workspace still green. No regressions in reposix-github / reposix-sim / reposix-fuse (none of those depend on the new behavior yet).
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Confluence tenant → adapter | `parentId` is attacker-influenced: a malicious page author could set `parentId` to any u64, creating fake hierarchies. B2 handles the cycle/orphan consequences; this plan only plumbs the value. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-03 | DoS | Cycle in parent chain | transfer to B2 | `parent_id` here is just `Option<IssueId>` plumbing. Cycle detection happens in `reposix-fuse::tree::build_tree`. Test covered in plan B2. |
| T-13-PB1 | Tampering | Malformed `parentId` string (e.g. `"abc"`) | mitigate | Silently degrade to `None` + `tracing::debug!`. Test `translate_handles_unparseable_parent_id` asserts this. Alternative (propagate Err) would wedge the whole list on one bad page — unacceptable. |
| T-13-PB2 | Information disclosure | Leaking `parentId` in error messages | accept | `parentId` is a backend-side numeric id. Not a secret. Already exposed in URL of the frontend. |
</threat_model>

<verification>
Nyquist coverage:
- **Unit/wiremock:** ≥ 6 new tests — 5 for `translate` parent_id derivation branches + 1 end-to-end wiremock `list_issues`. Plus 2 for `supports` + 1 for `root_collection_name`.
- **Workspace-wide:** cargo test green, no regression in Phase-11 tests (they don't know about parent_id; serde default handles absence).
- **Clippy:** crate-level and workspace-level clean.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `grep -qE 'parent_id: Option<String>' crates/reposix-confluence/src/lib.rs` exits 0.
2. `grep -qE 'parent_type: Option<String>' crates/reposix-confluence/src/lib.rs` exits 0.
3. `grep -qE 'rename = "parentId"' crates/reposix-confluence/src/lib.rs` exits 0.
4. `grep -qE 'rename = "parentType"' crates/reposix-confluence/src/lib.rs` exits 0.
5. `grep -q 'BackendFeature::Hierarchy' crates/reposix-confluence/src/lib.rs` exits 0.
6. `grep -qE 'fn root_collection_name.*"pages"' crates/reposix-confluence/src/lib.rs` exits 0 (or across two lines — adjust grep to multiline if needed).
7. `cargo test -p reposix-confluence --locked` exits 0.
8. `cargo test -p reposix-confluence --locked 2>&1 | grep -oE 'test result: ok\. [0-9]+ passed' | head -1 | awk '{print $4}'` returns an integer ≥ 16 (Phase 11 shipped 10+; B1 adds ≥ 6).
9. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
10. `cargo test --workspace --locked` exits 0.
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B1-SUMMARY.md` documenting:
- Final `reposix-confluence` test count (before/after)
- Names of the new tests added (for traceability)
- Which `parentType` values were exercised in tests (page, folder, whiteboard, missing)
- Confirmation that no Phase 11 test broke
</output>
