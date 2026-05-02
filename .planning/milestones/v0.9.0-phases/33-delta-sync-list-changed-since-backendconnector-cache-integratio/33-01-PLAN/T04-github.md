[index](./index.md)

# Task 01-T04 — `GithubReadOnlyBackend::list_changed_since` — native `?since=` param

<read_first>
- `crates/reposix-github/src/lib.rs` (esp. lines 340–410 for trait impl + `list_issues`)
- GitHub docs — the `GET /repos/{owner}/{repo}/issues` endpoint natively accepts `since` (ISO8601). No extra plumbing needed beyond appending the query param.
</read_first>

<action>
Edit `crates/reposix-github/src/lib.rs`:

1. Inside the `impl BackendConnector for GithubReadOnlyBackend` block, after `get_issue` (line 412) but still within the impl, add:

```rust
async fn list_changed_since(
    &self,
    project: &str,
    since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<reposix_core::IssueId>> {
    let since_iso = since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let first = format!(
        "{}/repos/{}/issues?state=all&per_page={}&since={}",
        self.base(),
        project,
        PAGE_SIZE,
        since_iso
    );
    // Reuse the same pagination loop shape as list_issues. Extract it
    // into a helper if two copies drift — but for clarity keep it
    // inline on v0.9.0.
    let mut next_url: Option<String> = Some(first);
    let mut out: Vec<reposix_core::IssueId> = Vec::new();
    let mut pages: usize = 0;
    let header_owned = self.standard_headers();
    let header_refs: Vec<(&str, &str)> =
        header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();

    while let Some(url) = next_url.take() {
        pages += 1;
        if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
            tracing::warn!(pages, "reached MAX_ISSUES_PER_LIST cap; stopping pagination");
            break;
        }
        self.await_rate_limit_gate().await;
        let resp = self.http.request_with_headers(Method::GET, url.as_str(), &header_refs).await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let link_hdr = resp.headers().get("link").and_then(|v| v.to_str().ok()).map(std::string::ToString::to_string);
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Other(format!(
                "github returned {status} for GET {url}: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: Vec<GhIssue> = serde_json::from_slice(&bytes)?;
        for gh in page {
            let issue = translate(gh);
            out.push(issue.id);
            if out.len() >= MAX_ISSUES_PER_LIST {
                return Ok(out);
            }
        }
        next_url = link_hdr.as_deref().and_then(parse_next_link);
    }
    Ok(out)
}
```

2. Add wiremock contract test (not gated `#[ignore]` — it's offline-safe):

```rust
#[tokio::test]
async fn github_list_changed_since_sends_since_param_and_returns_ids() {
    use chrono::{TimeZone, Utc};
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/octo/r/issues"))
        .and(wiremock::matchers::query_param("since", "2026-04-24T00:00:00Z"))
        .and(wiremock::matchers::query_param("state", "all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "number": 7, "title": "x", "state": "open", "created_at": "2026-04-24T01:00:00Z",
              "updated_at": "2026-04-24T02:00:00Z", "body": "", "labels": [] }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new(server.uri(), "TOKEN".into()).expect("backend");
    let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
    let ids = backend.list_changed_since("octo/r", t).await.expect("list");
    assert_eq!(ids, vec![reposix_core::IssueId(7)]);
}
```

Model after existing github tests in the same file; match the constructor signature whatever it is (`GithubReadOnlyBackend::new`, maybe different params — read the file).
</action>

<acceptance_criteria>
- `cargo build -p reposix-github` exits 0.
- `cargo test -p reposix-github github_list_changed_since_sends_since_param_and_returns_ids` exits 0.
- `grep -n 'list_changed_since' crates/reposix-github/src/lib.rs` finds exactly one override (not the default, which lives in reposix-core).
</acceptance_criteria>

<threat_model>
Reuses `standard_headers()` (PAT-bearing) and `self.http` (egress-allowlist gated). The `since` parameter is local cache state; we emit an ISO8601 string only — no reflection of attacker bytes into the URL.
</threat_model>
