[index](./index.md)

# Task 01-T03 — `SimBackend::list_changed_since` — pass `?since=` on the wire

<read_first>
- `crates/reposix-core/src/backend/sim.rs` (entire file)
- `crates/reposix-sim/src/routes/issues.rs` — confirm query param name is exactly `since`
</read_first>

<action>
Edit `crates/reposix-core/src/backend/sim.rs`:

1. Inside the `#[async_trait] impl BackendConnector for SimBackend { ... }` block, after `list_issues` (line 224), add:

```rust
async fn list_changed_since(
    &self,
    project: &str,
    since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<IssueId>> {
    // URL-encode via `format!` on an RFC3339 UTC string — contains no
    // ambiguous chars (only digits, `T`, `-`, `:`, `Z`) so percent-encoding
    // is a no-op. If future callers pass non-UTC or fractional seconds
    // that widen the charset, move to `url::form_urlencoded`.
    let since_iso = since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let url = format!(
        "{}/projects/{}/issues?since={}",
        self.base(),
        project,
        since_iso
    );
    let resp = self
        .http
        .request_with_headers(Method::GET, &url, &self.agent_only())
        .await?;
    let issues = decode_issues(resp, &url).await?;
    Ok(issues.into_iter().map(|i| i.id).collect())
}
```

2. Add wiremock tests in the existing `mod tests` at the bottom:

```rust
#[tokio::test]
async fn list_changed_since_sends_since_query_param() {
    use chrono::{TimeZone, Utc};
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/issues"))
        .and(wiremock::matchers::query_param("since", "2026-04-24T00:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            sample_issue_json(42)
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let backend = SimBackend::new(server.uri()).expect("backend");
    let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
    let ids = backend.list_changed_since("demo", t).await.expect("list_changed");
    assert_eq!(ids, vec![IssueId(42)]);
}

#[tokio::test]
async fn list_changed_since_returns_ids_only() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            sample_issue_json(1),
            sample_issue_json(2),
            sample_issue_json(3),
        ])))
        .mount(&server)
        .await;

    let backend = SimBackend::new(server.uri()).expect("backend");
    let t = chrono::Utc::now();
    let ids = backend.list_changed_since("demo", t).await.expect("list_changed");
    assert_eq!(ids, vec![IssueId(1), IssueId(2), IssueId(3)]);
}
```
</action>

<acceptance_criteria>
- `cargo build -p reposix-core` exits 0.
- `cargo test -p reposix-core list_changed_since_sends_since_query_param` exits 0 (proves `?since=<iso>` hits the wire).
- `cargo test -p reposix-core list_changed_since_returns_ids_only` exits 0.
- `grep -n 'list_changed_since' crates/reposix-core/src/backend/sim.rs` matches the override.
</acceptance_criteria>

<threat_model>
Same egress discipline as `list_issues` (allowlist enforced by `http::client`). The `since` value is derived from `cache.db` meta (trusted local state), never attacker-controlled.
</threat_model>
