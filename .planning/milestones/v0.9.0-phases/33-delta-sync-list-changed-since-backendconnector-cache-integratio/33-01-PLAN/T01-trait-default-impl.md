[index](./index.md)

# Task 01-T01 — Add `list_changed_since` to `BackendConnector` trait with default impl

<read_first>
- `.planning/phases/33-delta-sync-list-changed-since-backendconnector-cache-integratio/33-CONTEXT.md`
- `crates/reposix-core/src/backend.rs` (entire file)
- `crates/reposix-core/src/issue.rs` — confirm `IssueId` is `Copy + Send + Sync`
</read_first>

<action>
Edit `crates/reposix-core/src/backend.rs`:

1. At the top of the `#[async_trait] pub trait BackendConnector` block, after the `list_issues` method doc/signature, add a new method:

```rust
/// List issue IDs whose `updated_at` is strictly greater than `since`.
///
/// The default implementation calls [`list_issues`] and filters in
/// memory — safe for any backend but inefficient. Backends with a
/// native incremental query (`?since=` on GitHub, JQL `updated >=` on
/// JIRA, CQL `lastModified >` on Confluence, `?since=` on the sim)
/// MUST override to send the filter over the wire.
///
/// Returns IDs only; callers materialize full `Issue` objects on
/// demand via [`get_issue`]. This mirrors the Phase 31 lazy-blob
/// design: metadata (IDs) is cheap to ship, bodies are not.
///
/// # Errors
/// Same as [`list_issues`] — transport errors, egress-allowlist
/// denial (`Error::InvalidOrigin`), or backend-specific error shapes
/// surfacing as `Error::Other`.
async fn list_changed_since(
    &self,
    project: &str,
    since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<IssueId>> {
    let all = self.list_issues(project).await?;
    Ok(all
        .into_iter()
        .filter(|i| i.updated_at > since)
        .map(|i| i.id)
        .collect())
}
```

2. Add `chrono` re-export or dep. Check `crates/reposix-core/Cargo.toml` — `chrono` is already a dep (see existing `DateTime<Utc>` usage via `issue.rs`). If not exported at root, don't add a new export — the method signature references it by fully-qualified `chrono::DateTime<chrono::Utc>` so downstream crates just need their own `chrono` dep, which they already have.

3. Extend the `#[cfg(test)] mod tests` `Stub` impl in the same file: add a stub `list_changed_since` arm returning `Ok(vec![])`. **Actually — the `async fn` default impl means the `Stub` does NOT need to add this method; leave `Stub` alone.** Verify by reading the default impl — it delegates to `list_issues` which `Stub` already provides.

4. Add a NEW test at the bottom of `mod tests`:

```rust
#[tokio::test]
async fn default_list_changed_since_filters_via_list_issues() {
    use crate::issue::{Issue, IssueStatus};
    use chrono::{TimeZone, Utc};

    struct TwoIssues;
    #[async_trait]
    impl BackendConnector for TwoIssues {
        fn name(&self) -> &'static str { "two" }
        fn supports(&self, _: BackendFeature) -> bool { false }
        async fn list_issues(&self, _: &str) -> Result<Vec<Issue>> {
            let t1 = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
            let t2 = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
            Ok(vec![
                Issue {
                    id: IssueId(1), title: "old".into(), status: IssueStatus::Open,
                    assignee: None, labels: vec![], created_at: t1, updated_at: t1,
                    version: 1, body: String::new(), parent_id: None,
                    extensions: std::collections::BTreeMap::new(),
                },
                Issue {
                    id: IssueId(2), title: "new".into(), status: IssueStatus::Open,
                    assignee: None, labels: vec![], created_at: t1, updated_at: t2,
                    version: 1, body: String::new(), parent_id: None,
                    extensions: std::collections::BTreeMap::new(),
                },
            ])
        }
        async fn get_issue(&self, _: &str, _: IssueId) -> Result<Issue> { unimplemented!() }
        async fn create_issue(&self, _: &str, _: crate::taint::Untainted<Issue>) -> Result<Issue> { unimplemented!() }
        async fn update_issue(&self, _: &str, _: IssueId, _: crate::taint::Untainted<Issue>, _: Option<u64>) -> Result<Issue> { unimplemented!() }
        async fn delete_or_close(&self, _: &str, _: IssueId, _: DeleteReason) -> Result<()> { Ok(()) }
    }

    let backend = TwoIssues;
    let cutoff = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
    let got = backend.list_changed_since("demo", cutoff).await.unwrap();
    assert_eq!(got, vec![IssueId(2)]);
}
```
</action>

<acceptance_criteria>
- `grep -n 'async fn list_changed_since' crates/reposix-core/src/backend.rs` matches once inside the trait definition.
- `cargo build -p reposix-core` exits 0.
- `cargo test -p reposix-core backend::tests::default_list_changed_since_filters_via_list_issues` exits 0.
- `cargo test -p reposix-core -- _assert_dyn_compatible` still compiles — proves method stayed dyn-compatible.
</acceptance_criteria>

<threat_model>
New trait method inherits the `Send + Sync` + egress-allowlist discipline from `list_issues`; any adapter override still goes through `reposix_core::http::client()` which enforces `REPOSIX_ALLOWED_ORIGINS`. No new data source, no new exfil vector.
</threat_model>
