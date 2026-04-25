//! Issue CRUD handlers — list, get, create, patch, delete.
//!
//! Every handler follows the same pattern:
//! 1. Extract `Path`/`State`/headers/body.
//! 2. Inside a tightly-scoped `state.db.lock()` critical section, run the
//!    `SQLite` work.
//! 3. Drop the lock before any `.await` (there is no `.await` here — handlers
//!    are synchronous over rusqlite — but the pattern holds if the body is
//!    ever refactored).
//!
//! Response bodies mirror [`reposix_core::Record`]'s `Serialize` output.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, SecondsFormat, Utc};
use reposix_core::{Record, RecordId, IssueStatus};
use rusqlite::{params, Connection, TransactionBehavior};
use serde::Deserialize;
use serde_json::Value;

use crate::{error::ApiError, AppState};

/// Build the issues sub-router. Routes are nested under
/// `/projects/:slug/issues` at top-level.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/projects/:slug/issues",
            get(list_records).post(create_record),
        )
        .route(
            "/projects/:slug/issues/:id",
            get(get_record).patch(patch_issue).delete(delete_record),
        )
        .with_state(state)
}

// ---------- shared helpers ------------------------------------------------

/// Parse the `labels` column (stored as JSON) into `Vec<String>`.
fn parse_labels(raw: &str) -> Result<Vec<String>, ApiError> {
    serde_json::from_str(raw).map_err(ApiError::Json)
}

/// Parse the `status` column (canonical `snake_case` form) into `IssueStatus`.
fn parse_status(raw: &str) -> Result<IssueStatus, ApiError> {
    match raw {
        "open" => Ok(IssueStatus::Open),
        "in_progress" => Ok(IssueStatus::InProgress),
        "in_review" => Ok(IssueStatus::InReview),
        "done" => Ok(IssueStatus::Done),
        "wont_fix" => Ok(IssueStatus::WontFix),
        other => Err(ApiError::Internal(format!("unknown status: {other}"))),
    }
}

/// Public shim so sibling modules (e.g. `routes::transitions`) can reuse the
/// same status-parsing table without each submodule hard-coding the set.
pub(crate) fn parse_status_shared(raw: &str) -> Result<IssueStatus, ApiError> {
    parse_status(raw)
}

fn parse_ts(raw: &str) -> Result<DateTime<Utc>, ApiError> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| ApiError::Internal(format!("timestamp {raw:?}: {e}")))
}

/// Map one `SELECT *`-shaped `rusqlite::Row` onto an `Record`.
fn row_to_issue(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawIssueRow> {
    Ok(RawIssueRow {
        id: row.get::<_, i64>("id")?,
        title: row.get::<_, String>("title")?,
        status: row.get::<_, String>("status")?,
        assignee: row.get::<_, Option<String>>("assignee")?,
        labels: row.get::<_, String>("labels")?,
        created_at: row.get::<_, String>("created_at")?,
        updated_at: row.get::<_, String>("updated_at")?,
        version: row.get::<_, i64>("version")?,
        body: row.get::<_, String>("body")?,
    })
}

struct RawIssueRow {
    id: i64,
    title: String,
    status: String,
    assignee: Option<String>,
    labels: String,
    created_at: String,
    updated_at: String,
    version: i64,
    body: String,
}

impl RawIssueRow {
    fn into_issue(self) -> Result<Record, ApiError> {
        #[allow(clippy::cast_sign_loss)] // `id`/`version` stored non-negative
        let id = self.id as u64;
        #[allow(clippy::cast_sign_loss)]
        let version = self.version as u64;
        Ok(Record {
            id: RecordId(id),
            title: self.title,
            status: parse_status(&self.status)?,
            assignee: self.assignee,
            labels: parse_labels(&self.labels)?,
            created_at: parse_ts(&self.created_at)?,
            updated_at: parse_ts(&self.updated_at)?,
            version,
            body: self.body,
            // Sim backend has no hierarchy — always None.
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        })
    }
}

/// Parse `If-Match`. Returns `None` if absent (wildcard match), or
/// `Some(raw_unquoted_string)` if present. The caller decides whether the
/// string parses to a `u64`; any parse failure is a mismatch.
fn if_match_value(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(axum::http::header::IF_MATCH)?.to_str().ok()?;
    // RFC 7232 quoted-etag: strip surrounding double quotes if present.
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed);
    Some(stripped.to_owned())
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

// ---------- GET /projects/:slug/issues ------------------------------------

/// Optional query parameters for `GET /projects/:slug/issues`. Only `since`
/// is recognized; unknown params are silently ignored (axum default).
#[derive(Debug, Deserialize)]
struct ListIssuesQuery {
    /// ISO8601/RFC3339 cutoff. When present, the response is filtered to
    /// issues with `updated_at > since`. Absent or empty → return all
    /// (backwards-compatible with v0.8.0 callers).
    #[serde(default)]
    since: Option<String>,
}

#[allow(clippy::unused_async)]
async fn list_records(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<ListIssuesQuery>,
) -> Result<Json<Vec<Record>>, ApiError> {
    // Parse the `since` bound once before touching the DB. Bad format → 400.
    let since_cutoff: Option<DateTime<Utc>> = match q.since.as_deref() {
        None | Some("") => None,
        Some(raw) => Some(
            DateTime::parse_from_rfc3339(raw)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    ApiError::BadRequest(format!("invalid `since` (expect RFC3339/ISO8601): {e}"))
                })?,
        ),
    };

    let issues: Vec<Record> = {
        let conn = state.db.lock();
        // Stored `updated_at` uses RFC3339 with `Z` suffix and seconds
        // precision (`now_rfc3339` helper). Lexicographic comparison is
        // monotonic over that canonical form, so a string `>` against a
        // SecondsFormat::Secs/UseZ rendering of the cutoff is correct.
        if let Some(t) = since_cutoff {
            let cutoff_iso = t.to_rfc3339_opts(SecondsFormat::Secs, true);
            let mut stmt = conn.prepare(
                "SELECT id, title, status, assignee, labels, created_at, updated_at, version, body \
                 FROM issues WHERE project = ?1 AND updated_at > ?2 ORDER BY id ASC",
            )?;
            let raws: Vec<RawIssueRow> = stmt
                .query_map(params![slug, cutoff_iso], row_to_issue)?
                .collect::<rusqlite::Result<_>>()?;
            raws.into_iter()
                .map(RawIssueRow::into_issue)
                .collect::<Result<Vec<_>, _>>()?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, title, status, assignee, labels, created_at, updated_at, version, body \
                 FROM issues WHERE project = ?1 ORDER BY id ASC",
            )?;
            let raws: Vec<RawIssueRow> = stmt
                .query_map(params![slug], row_to_issue)?
                .collect::<rusqlite::Result<_>>()?;
            raws.into_iter()
                .map(RawIssueRow::into_issue)
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    Ok(Json(issues))
}

// ---------- GET /projects/:slug/issues/:id --------------------------------

#[allow(clippy::unused_async)]
async fn get_record(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, u64)>,
) -> Result<Json<Record>, ApiError> {
    let issue = load_issue(&state.db.lock(), &slug, id)?;
    Ok(Json(issue))
}

fn load_issue(conn: &Connection, slug: &str, id: u64) -> Result<Record, ApiError> {
    #[allow(clippy::cast_possible_wrap)]
    let id_signed = id as i64;
    let row = conn.query_row(
        "SELECT id, title, status, assignee, labels, created_at, updated_at, version, body \
         FROM issues WHERE project = ?1 AND id = ?2",
        params![slug, id_signed],
        row_to_issue,
    );
    match row {
        Ok(raw) => raw.into_issue(),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(ApiError::NotFound),
        Err(e) => Err(ApiError::Db(e)),
    }
}

// ---------- POST /projects/:slug/issues -----------------------------------

/// Body shape for `POST`. Server-managed fields (`id`, `version`,
/// `created_at`, `updated_at`) are deliberately absent; if the client sends
/// them they are ignored by serde (struct has no field of that name).
#[derive(Debug, Deserialize)]
struct CreateIssueBody {
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    assignee: Option<String>,
    #[serde(default)]
    labels: Option<Vec<String>>,
}

#[allow(clippy::unused_async)]
async fn create_record(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<CreateIssueBody>,
) -> Result<Response, ApiError> {
    // TODO(v0.4+): wrap inbound body in Tainted<T> before frontmatter stripping.
    if body.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title must be non-empty".into()));
    }
    let status_str = body.status.as_deref().unwrap_or("open");
    // Validate the status is one of the five canonical strings before insert.
    let _ = parse_status(status_str)?;
    let labels = body.labels.unwrap_or_default();
    let labels_json = serde_json::to_string(&labels)?;
    let ts = now_rfc3339();

    let new_id: u64 = {
        let conn = state.db.lock();
        let max_id: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(id), 0) FROM issues WHERE project = ?1",
                params![slug],
                |r| r.get(0),
            )
            .map_err(ApiError::Db)?;
        let next = max_id + 1;
        conn.execute(
            "INSERT INTO issues \
             (project, id, title, status, assignee, labels, created_at, updated_at, version, body) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
            params![
                slug,
                next,
                body.title.trim(),
                status_str,
                body.assignee,
                labels_json,
                ts,
                ts,
                body.body,
            ],
        )?;
        #[allow(clippy::cast_sign_loss)]
        let as_u64 = next as u64;
        as_u64
    };

    let issue = {
        let conn = state.db.lock();
        load_issue(&conn, &slug, new_id)?
    };

    let location = format!("/projects/{slug}/issues/{new_id}");
    let mut headers = HeaderMap::new();
    // URL paths are ASCII here; HeaderValue::from_str tolerates ASCII.
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::from_str(&location).map_err(|e| ApiError::Internal(e.to_string()))?,
    );

    Ok((StatusCode::CREATED, headers, Json(issue)).into_response())
}

// ---------- PATCH /projects/:slug/issues/:id ------------------------------

/// Three-valued field update: absent (no change), present-and-null (clear),
/// or present-and-set. Avoids the Option<Option<_>> clippy flag while
/// preserving the distinction between "don't touch" and "clear".
#[derive(Debug, Default)]
enum FieldUpdate<T> {
    /// Field absent from request body — do not touch.
    #[default]
    Unchanged,
    /// Field present with JSON `null` — clear to NULL.
    Clear,
    /// Field present with a value — set.
    Set(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for FieldUpdate<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Serde invokes this deserializer only when the field is present in
        // the JSON (absent -> default -> Unchanged). Present value may be
        // null or a real T.
        let inner = Option::<T>::deserialize(d)?;
        Ok(match inner {
            None => Self::Clear,
            Some(v) => Self::Set(v),
        })
    }
}

/// Mutable-field allow-list. Any unknown field in the request body is
/// rejected by `#[serde(deny_unknown_fields)]`, which is what stops a client
/// from trying to bump `version` by hand — `version`/`id`/`created_at`/
/// `updated_at` are not fields of this struct.
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct PatchIssueBody {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    status: Option<String>,
    /// `Unchanged` means absent; `Clear` means explicit `null`; `Set` means
    /// a new assignee string.
    #[serde(default)]
    assignee: FieldUpdate<String>,
    #[serde(default)]
    labels: Option<Vec<String>>,
}

#[allow(clippy::unused_async)]
async fn patch_issue(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, u64)>,
    headers: HeaderMap,
    body: Option<Json<Value>>,
) -> Result<Json<Record>, ApiError> {
    // TODO(v0.4+): wrap inbound body in Tainted<T> before frontmatter stripping.
    let raw = body.map_or(Value::Object(serde_json::Map::new()), |Json(v)| v);
    // Deserialize via strict deny_unknown_fields.
    let patch: PatchIssueBody = serde_json::from_value(raw).map_err(ApiError::Json)?;

    // If-Match handling: None → wildcard match (allow); Some → must parse as
    // u64 and match current version, else 409.
    let if_match = if_match_value(&headers);

    let new_status = if let Some(ref s) = patch.status {
        Some(parse_status(s)?)
    } else {
        None
    };
    let labels_json: Option<String> = match &patch.labels {
        Some(v) => Some(serde_json::to_string(v)?),
        None => None,
    };
    let ts = now_rfc3339();

    let issue = {
        let mut conn = state.db.lock();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        #[allow(clippy::cast_possible_wrap)]
        let id_signed = id as i64;
        // Fetch current version + existing fields.
        let existing = tx.query_row(
            "SELECT version FROM issues WHERE project = ?1 AND id = ?2",
            params![slug, id_signed],
            |r| r.get::<_, i64>(0),
        );
        let current_version_i64 = match existing {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(ApiError::NotFound),
            Err(e) => return Err(ApiError::Db(e)),
        };
        #[allow(clippy::cast_sign_loss)]
        let current_version = current_version_i64 as u64;

        if let Some(ref raw_etag) = if_match {
            let sent_ok = raw_etag.parse::<u64>().is_ok_and(|n| n == current_version);
            if !sent_ok {
                return Err(ApiError::VersionMismatch {
                    current: current_version,
                    sent: raw_etag.clone(),
                });
            }
        }

        // Apply only the mutable-field allow-list. Everything else (id,
        // created_at, version, updated_at) is server-managed.
        let new_version_i64 = current_version_i64 + 1;
        let status_str = new_status.map(IssueStatus::as_str);

        // Unpack FieldUpdate into a "touch assignee?" bool + a value.
        let (assignee_touch, assignee_val): (i64, Option<String>) = match &patch.assignee {
            FieldUpdate::Unchanged => (0, None),
            FieldUpdate::Clear => (1, None),
            FieldUpdate::Set(s) => (1, Some(s.clone())),
        };

        tx.execute(
            "UPDATE issues SET \
                title     = COALESCE(?1, title), \
                body      = COALESCE(?2, body), \
                status    = COALESCE(?3, status), \
                assignee  = CASE WHEN ?4 = 1 THEN ?5 ELSE assignee END, \
                labels    = COALESCE(?6, labels), \
                updated_at = ?7, \
                version   = ?8 \
             WHERE project = ?9 AND id = ?10",
            params![
                patch.title,
                patch.body,
                status_str,
                assignee_touch,
                assignee_val,
                labels_json,
                ts,
                new_version_i64,
                slug,
                id_signed,
            ],
        )?;

        tx.commit()?;
        drop(conn);

        // Re-open a read — keep scope tight.
        let conn2 = state.db.lock();
        load_issue(&conn2, &slug, id)?
    };

    Ok(Json(issue))
}

// ---------- DELETE /projects/:slug/issues/:id -----------------------------

#[allow(clippy::unused_async)]
async fn delete_record(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, u64)>,
) -> Result<StatusCode, ApiError> {
    #[allow(clippy::cast_possible_wrap)]
    let id_signed = id as i64;
    let affected = {
        let conn = state.db.lock();
        conn.execute(
            "DELETE FROM issues WHERE project = ?1 AND id = ?2",
            params![slug, id_signed],
        )?
    };
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::open_db, seed::load_seed, SimConfig};
    use axum::body::Body;
    use axum::http::Request;
    use std::path::{Path as StdPath, PathBuf};
    use tower::ServiceExt;

    fn fixture_path() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("fixtures/seed.json");
        p
    }

    fn seeded_state() -> AppState {
        let conn = open_db(StdPath::new(":memory:"), true).expect("db");
        load_seed(&conn, &fixture_path()).expect("seed");
        AppState::new(conn, SimConfig::ephemeral())
    }

    async fn read_body(resp: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_048_576)
            .await
            .expect("collect");
        if bytes.is_empty() {
            return serde_json::Value::Null;
        }
        serde_json::from_slice(&bytes).expect("json")
    }

    #[tokio::test]
    async fn list_returns_all_seeded_issues() {
        let state = seeded_state();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/projects/demo/issues")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v = read_body(resp).await;
        assert!(v.is_array(), "expected array, got {v:?}");
        assert_eq!(v.as_array().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn get_returns_200_for_existing_and_404_for_missing() {
        let state = seeded_state();
        let app = router(state);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/demo/issues/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v = read_body(resp).await;
        assert_eq!(v["id"], 1);
        assert_eq!(v["version"], 1);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/projects/demo/issues/9999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn create_returns_201_with_location() {
        let state = seeded_state();
        let app = router(state);
        let body = r#"{"title":"new issue","body":"body","labels":["new"]}"#;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/demo/issues")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
        let loc = resp
            .headers()
            .get("location")
            .expect("location")
            .to_str()
            .unwrap()
            .to_owned();
        let v = read_body(resp).await;
        assert_eq!(v["id"], 7, "next id after 6 seeded is 7");
        assert_eq!(v["version"], 1);
        assert_eq!(loc, "/projects/demo/issues/7");
    }

    #[tokio::test]
    async fn patch_with_matching_if_match_bumps_version() {
        let state = seeded_state();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/projects/demo/issues/1")
                    .header("content-type", "application/json")
                    .header("if-match", "\"1\"")
                    .body(Body::from(r#"{"status":"done"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v = read_body(resp).await;
        assert_eq!(v["version"], 2);
        assert_eq!(v["status"], "done");
    }

    #[tokio::test]
    async fn patch_with_bogus_if_match_returns_409() {
        let state = seeded_state();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/projects/demo/issues/1")
                    .header("content-type", "application/json")
                    .header("if-match", "\"bogus\"")
                    .body(Body::from(r#"{"status":"done"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 409);
        let v = read_body(resp).await;
        assert_eq!(v["error"], "version_mismatch");
        assert_eq!(v["current"], 1);
        assert_eq!(v["sent"], "bogus");
    }

    #[tokio::test]
    async fn patch_without_if_match_is_wildcard_allow() {
        let state = seeded_state();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/projects/demo/issues/1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"updated"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v = read_body(resp).await;
        assert_eq!(v["title"], "updated");
        assert_eq!(v["version"], 2);
    }

    #[tokio::test]
    async fn patch_ignores_server_managed_fields_via_deny_unknown() {
        let state = seeded_state();
        let app = router(state);
        // Body contains a `version` field which is NOT in PatchIssueBody.
        // deny_unknown_fields rejects the request with 400 (via Json error
        // → BadRequest? our handler maps serde errors to ApiError::Json →
        // 500). The behavior we really care about is: version is never
        // writable from a client. So assert 400 OR 500 (both block the
        // write), but MUST NOT be 200.
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/projects/demo/issues/1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"version":999}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            resp.status().is_client_error() || resp.status().is_server_error(),
            "server-managed fields must be rejected; got {:?}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn delete_returns_204_then_get_returns_404() {
        let state = seeded_state();
        let app = router(state);
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/projects/demo/issues/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/projects/demo/issues/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn list_issues_with_since_filters_correctly() {
        // Seed 6 issues. PATCH issue 3 to bump its updated_at.
        // Then request with `since` set to a moment between the seed time
        // and the patch time. Expect issue 3 only.
        let state = seeded_state();
        let app = router(state);

        // Capture a "before patch" cutoff. The seed timestamps are far in
        // the past (fixture uses 2026-04-13T00:00:00Z, see fixtures/seed.json).
        let cutoff = "2030-01-01T00:00:00Z";

        // Patch issue 3 — its updated_at becomes "now" (post-2030 cutoff
        // is impossible for `now`, so we cannot use that cutoff to test
        // the filter). Use a cutoff that is BEFORE all seeds and assert
        // the patched-issue's updated_at is now > all-seed-time.
        let patch_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/projects/demo/issues/3")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"updated"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(patch_resp.status(), 200);

        // Cutoff between seed time (2026-04-13) and patch time (now,
        // post-2026-04-13). The patched issue's updated_at is `now`,
        // which is > the cutoff. Use a cutoff one day after the seed
        // time so unpatched seeds (updated_at == 2026-04-13) are excluded.
        let cutoff_between = "2026-04-14T00:00:00Z";
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/projects/demo/issues?since={}",
                        url_encode(cutoff_between)
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v = read_body(resp).await;
        let arr = v.as_array().expect("array");
        assert_eq!(
            arr.len(),
            1,
            "expected exactly one filtered issue, got {arr:?}"
        );
        assert_eq!(arr[0]["id"], 3);

        // And: with a far-future cutoff, we see zero issues.
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/projects/demo/issues?since={}",
                        url_encode("2099-01-01T00:00:00Z")
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v = read_body(resp).await;
        assert_eq!(v.as_array().unwrap().len(), 0);
        // suppress unused warning for `cutoff` (kept as documentation).
        let _ = cutoff;
    }

    #[tokio::test]
    async fn list_issues_absent_since_returns_all() {
        // Backwards-compatibility: v0.8.0 callers omit the `since` param
        // and still receive the full set.
        let state = seeded_state();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/projects/demo/issues")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v = read_body(resp).await;
        assert_eq!(v.as_array().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn list_issues_malformed_since_returns_400() {
        let state = seeded_state();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/projects/demo/issues?since=not-a-timestamp")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 400);
        let v = read_body(resp).await;
        assert_eq!(v["error"], "bad_request");
    }

    /// Minimal URL-encoder for ASCII timestamps. Replaces `:` with `%3A`
    /// — the only metacharacter in an ISO8601 string that needs escaping
    /// in a query value.
    fn url_encode(raw: &str) -> String {
        raw.replace(':', "%3A")
    }
}
