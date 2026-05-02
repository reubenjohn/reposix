[index](./index.md)

# Task 01-T02 — Sim route: accept `?since=<ISO8601>` query parameter

<read_first>
- `crates/reposix-sim/src/routes/issues.rs` (lines 1–200)
- `crates/reposix-sim/src/error.rs` — confirm `ApiError` variants and `BadRequest`-equivalent
</read_first>

<action>
Edit `crates/reposix-sim/src/routes/issues.rs`:

1. Add a `Query` extractor import: `use axum::extract::Query;`.

2. Define a new struct near the top of the file (after the `parse_ts` helper):

```rust
#[derive(Debug, Deserialize)]
struct ListIssuesQuery {
    /// Optional ISO8601 cutoff. Returns only issues with
    /// `updated_at > since`. Absent = return all (backwards compatible).
    #[serde(default)]
    since: Option<String>,
}
```

3. Replace the signature of `list_issues` (line 145) with:

```rust
async fn list_issues(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<ListIssuesQuery>,
) -> Result<Json<Vec<Issue>>, ApiError> {
    // Parse the `since` bound once, before touching the DB. Bad format → 400.
    let since_cutoff: Option<DateTime<Utc>> = match q.since.as_deref() {
        None | Some("") => None,
        Some(raw) => Some(
            DateTime::parse_from_rfc3339(raw)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| ApiError::BadRequest(format!(
                    "invalid `since` (expect RFC3339/ISO8601): {e}"
                )))?,
        ),
    };

    let issues: Vec<Issue> = {
        let conn = state.db.lock();
        let (sql, bind_since): (&str, Option<String>) = if let Some(t) = since_cutoff {
            (
                "SELECT id, title, status, assignee, labels, created_at, updated_at, version, body \
                 FROM issues WHERE project = ?1 AND updated_at > ?2 ORDER BY id ASC",
                Some(t.to_rfc3339_opts(SecondsFormat::Secs, true)),
            )
        } else {
            (
                "SELECT id, title, status, assignee, labels, created_at, updated_at, version, body \
                 FROM issues WHERE project = ?1 ORDER BY id ASC",
                None,
            )
        };
        let mut stmt = conn.prepare(sql)?;
        let raws: Vec<RawIssueRow> = match bind_since.as_deref() {
            Some(t) => stmt.query_map(params![slug, t], row_to_issue)?.collect::<rusqlite::Result<_>>()?,
            None => stmt.query_map(params![slug], row_to_issue)?.collect::<rusqlite::Result<_>>()?,
        };
        raws.into_iter()
            .map(RawIssueRow::into_issue)
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(Json(issues))
}
```

Note: string-compare on ISO8601 RFC3339 is lexicographically monotonic when both values are UTC with `Z` suffix and same second precision. The stored `updated_at` in the sim DB uses `.to_rfc3339()` form (see Phase 14 route handlers) — confirm by reading a few lines of `create_issue` / `patch_issue` in the same file. If the stored form differs, parse-in-SQL via `datetime(col) > datetime(?)` instead.

4. Confirm `ApiError::BadRequest(String)` exists. If not, add:
```rust
// in crates/reposix-sim/src/error.rs
BadRequest(String),
```
plus an `IntoResponse` arm returning `(StatusCode::BAD_REQUEST, body)`. If a similar "validation" variant already exists under a different name, reuse it.

5. Add test in `routes/issues.rs` `mod tests` at the bottom:

```rust
#[tokio::test]
async fn list_issues_with_since_filters_correctly() {
    // Seed 3 issues with staggered updated_at. Request with since pointing at the 2nd's ts.
    // Expect exactly the 3rd returned.
    // Use the existing in-process test scaffold (see sibling test `list_returns_all_seeded_issues`).
    // Re-use that test's setup helpers.
    // (Concrete setup: inspect how `list_returns_all_seeded_issues` bootstraps `AppState`.)
    todo!("see sibling list_returns_all_seeded_issues for the in-process harness")
}

#[tokio::test]
async fn list_issues_absent_since_returns_all() {
    // Backwards-compatibility check — the v0.8.0 caller (no `since` param)
    // still receives the full set.
    todo!("see sibling list_returns_all_seeded_issues")
}

#[tokio::test]
async fn list_issues_malformed_since_returns_400() {
    // `?since=not-a-timestamp` → HTTP 400.
    todo!("in-process axum test with TestClient")
}
```

Replace each `todo!` with the concrete test body patterned on `list_returns_all_seeded_issues` in the same file (line 482). Do not leave any `todo!` in the final commit.
</action>

<acceptance_criteria>
- `cargo build -p reposix-sim` exits 0.
- `cargo test -p reposix-sim list_issues_with_since_filters_correctly` exits 0.
- `cargo test -p reposix-sim list_issues_absent_since_returns_all` exits 0.
- `cargo test -p reposix-sim list_issues_malformed_since_returns_400` exits 0.
- `cargo test -p reposix-sim list_returns_all_seeded_issues` still passes (regression).
</acceptance_criteria>

<threat_model>
`since` is a caller-supplied timestamp string parsed with `chrono::DateTime::parse_from_rfc3339`; parse failure becomes a 400 response with a fixed error message template (no reflection of the raw input beyond the chrono error body, which does not echo the user string as control characters). No SQL injection surface — bound via `params![]`.
</threat_model>
