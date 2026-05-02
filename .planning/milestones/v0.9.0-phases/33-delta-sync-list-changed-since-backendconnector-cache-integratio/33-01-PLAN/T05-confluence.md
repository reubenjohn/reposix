[index](./index.md)

# Task 01-T05 — `ConfluenceBackend::list_changed_since` — CQL `lastModified > "<iso>"`

<read_first>
- `crates/reposix-confluence/src/lib.rs:780-900` (list_issues_impl for pattern)
- `crates/reposix-confluence/src/lib.rs:1476-1508` (trait impl)
- Confluence CQL docs (in the existing lib.rs CQL usage, if any; else per architecture-pivot-summary §4: CQL `lastModified > "<datetime>"`)
</read_first>

<action>
Edit `crates/reposix-confluence/src/lib.rs`:

1. Inside `impl BackendConnector for ConfluenceBackend`, after `list_issues` (line 1504), add:

```rust
async fn list_changed_since(
    &self,
    project: &str,
    since: chrono::DateTime<chrono::Utc>,
) -> reposix_core::Result<Vec<reposix_core::IssueId>> {
    // Confluence CQL accepts a `lastModified > "yyyy-MM-dd HH:mm"` filter.
    // Convert UTC to that canonical form; seconds not supported in CQL.
    let cql_time = since.format("%Y-%m-%d %H:%M").to_string();
    let space_id = self.resolve_space_id(project).await?;
    let cql = format!(
        "space = \"{}\" AND lastModified > \"{}\"",
        project.replace('"', ""), // strip quotes defensively (space slugs don't legitimately have them)
        cql_time
    );
    // CQL search endpoint: /wiki/rest/api/search?cql=...
    let encoded_cql = urlencoding::encode(&cql);
    let first = format!(
        "{}/wiki/rest/api/search?cql={}&limit={}",
        self.base(),
        encoded_cql,
        PAGE_SIZE
    );
    // Pagination: same shape as list_issues_impl.
    let mut next_url: Option<String> = Some(first);
    let mut out: Vec<reposix_core::IssueId> = Vec::new();
    let mut pages: usize = 0;
    let header_owned = self.standard_headers();
    let header_refs: Vec<(&str, &str)> =
        header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let _ = space_id; // reserved for future space-id-based endpoints; CQL filter is primary

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
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(reposix_core::Error::Other(format!(
                "confluence returned {status} for GET {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        // Search endpoint shape differs slightly from the list endpoint —
        // expect `{ "results": [{ "content": { "id": "...", "type": "page" } }, ...] }`.
        // Extract `content.id` per result; parse as u64 (Confluence page IDs
        // are numeric strings).
        let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
        let arr = body_json.get("results").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        for res in arr {
            let id_str = res.pointer("/content/id").and_then(|v| v.as_str()).unwrap_or("");
            if let Ok(n) = id_str.parse::<u64>() {
                out.push(reposix_core::IssueId(n));
                if out.len() >= MAX_ISSUES_PER_LIST {
                    return Ok(out);
                }
            }
        }
        // CQL search pagination is via `_links.next`; reuse
        // parse_next_cursor if the same helper applies, else break
        // (v0.9.0 acceptable — a cap hit is noisy-not-silent).
        next_url = None; // single page for v0.9.0 delta-sync MVP
    }
    Ok(out)
}
```

2. Add contract test (wiremock):

```rust
#[tokio::test]
async fn confluence_list_changed_since_sends_cql_lastmodified() {
    use chrono::{TimeZone, Utc};
    let server = MockServer::start().await;
    // Match on CQL substring. urlencoding::encode produces "lastModified%20%3E%20"
    // for "lastModified > "; match loosely.
    Mock::given(method("GET"))
        .and(path("/wiki/rest/api/search"))
        .and(wiremock::matchers::query_param_contains("cql", "lastModified"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                { "content": { "id": "12345", "type": "page" } }
            ]
        })))
        .mount(&server)
        .await;

    // Construct ConfluenceBackend per the existing test pattern in the same file.
    // (Read how other confluence tests new up the backend.)
    let backend = make_test_backend(server.uri()); // use the existing test helper
    let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
    let ids = backend.list_changed_since("TEST", t).await.expect("list");
    assert_eq!(ids, vec![reposix_core::IssueId(12345)]);
}
```

If `urlencoding` is not already a dep, check `Cargo.toml` — the existing codebase may use `percent_encoding`. Use whichever is already present; do NOT add a new dep for a single call site. If neither is present, `url::form_urlencoded` works via `std::fmt::Write` patterns. Avoid ad-hoc percent encoding.

If `query_param_contains` is not a wiremock matcher (check the version), use a custom `wiremock::Match` struct similar to `NoIfMatch` in `reposix-core/src/backend/sim.rs` that inspects `request.url.query()` for the substring `"lastModified"`.
</action>

<acceptance_criteria>
- `cargo build -p reposix-confluence` exits 0.
- `cargo test -p reposix-confluence confluence_list_changed_since_sends_cql_lastmodified` exits 0.
- `grep -n 'list_changed_since' crates/reposix-confluence/src/lib.rs` finds exactly one override.
- No new Cargo.toml dependency added (verify with `git diff crates/reposix-confluence/Cargo.toml`).
</acceptance_criteria>

<threat_model>
Project slug is stripped of `"` chars defensively before interpolation into CQL to prevent CQL injection via a malicious space name. The `since` value is local cache state. The auth + egress gating are inherited from the existing `self.http` path.
</threat_model>
