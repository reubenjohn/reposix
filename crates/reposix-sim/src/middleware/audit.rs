//! Audit middleware — writes one `audit_events` row per HTTP request.
//!
//! # Column mapping (EXACTLY matches `crates/reposix-core/fixtures/audit.sql`)
//!
//! | Column            | Source                                               |
//! |-------------------|------------------------------------------------------|
//! | `ts`              | `chrono::Utc::now().to_rfc3339_opts(Secs, true)`     |
//! | `agent_id`        | header `X-Reposix-Agent`, default `"anonymous"`      |
//! | `method`          | request method (`GET`, `POST`, …)                    |
//! | `path`            | URI path                                             |
//! | `status`          | final response status code                           |
//! | `request_body`    | first 256 chars of the body (UTF-8 lossy)            |
//! | `response_summary`| `"{status}:{sha256_hex_prefix_16}"`                  |
//!
//! # `parts` discipline (axum 0.7)
//!
//! `http::request::Parts` is **not** `Clone`. The middleware captures
//! `method` / `uri` / the agent header into local vars BEFORE passing
//! `parts` into `Request::from_parts(parts, …)`, which consumes it.
//! Attempting `parts.clone()` is a compile error and would block the
//! whole plan — do not try to "fix" that with a `.clone()`.

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::StatusCode,
    middleware::{from_fn_with_state, Next},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::AppState;

/// 1 MiB body cap. Bodies larger than this return 413 and DO write an audit
/// row (the 413 response itself is audited because it's produced inside
/// this middleware, before the downstream handler runs).
pub const BODY_LIMIT_BYTES: usize = 1_048_576;

/// Attach the audit middleware to an axum `Router`.
///
/// The concrete layer type returned by `from_fn_with_state` is unnameable in
/// stable Rust without `type_alias_impl_trait`; `attach` hides it behind a
/// generic `Router -> Router` transform that callers can use directly.
pub fn attach<S>(router: axum::Router<S>, state: AppState) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(from_fn_with_state(state, audit_middleware))
}

/// The middleware function itself.
///
/// # Errors
/// Never returns an `Err`; failures in the body-buffering step are mapped to
/// 413 responses, and DB-insert failures are logged and swallowed so the
/// downstream response is never masked.
pub async fn audit_middleware(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // 1. Decompose request.
    let (parts, body) = req.into_parts();

    // 2. Capture fields we need BEFORE moving `parts` into `from_parts`.
    //    http::request::Parts is NOT Clone in axum 0.7; individual fields are.
    let method = parts.method.clone();
    let uri = parts.uri.clone();
    let agent_id = parts
        .headers
        .get("X-Reposix-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_owned();

    // 3. Buffer the body to bytes. 1 MiB cap — oversize returns 413 *and*
    //    gets audited (we still fall through to the DB insert).
    let (bytes, oversize) = match to_bytes(body, BODY_LIMIT_BYTES).await {
        Ok(b) => (b, false),
        Err(_) => (axum::body::Bytes::new(), true),
    };

    // 4. Lossy UTF-8 view for the `request_body` column (first 256 chars).
    let body_string_lossy = String::from_utf8_lossy(&bytes);
    let truncated: String = body_string_lossy.as_ref().chars().take(256).collect();

    // 5. Full-body SHA-256 (prefix 16 hex chars goes into response_summary).
    let sha_hex = hex::encode(Sha256::digest(&bytes));

    // 6. Rebuild the request and run downstream (or short-circuit with 413).
    let response: Response = if oversize {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "error": "body_too_large",
                "limit": BODY_LIMIT_BYTES,
            })),
        )
            .into_response()
    } else {
        // `parts` is consumed here. method/uri/agent_id were captured above.
        let rebuilt = Request::from_parts(parts, Body::from(bytes.clone()));
        next.run(rebuilt).await
    };

    // 7. Capture response status.
    let status_u16 = response.status().as_u16();

    // 8. Build audit row values.
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let path_str = uri.path().to_owned();
    let method_str = method.as_str().to_owned();
    let response_summary = format!("{status_u16}:{}", &sha_hex[..16]);

    // 9. Insert, sync-scoped, no .await held across the lock.
    //    Log-and-swallow on Err so the downstream response is never masked.
    {
        let conn = state.db.lock();
        if let Err(e) = conn.execute(
            "INSERT INTO audit_events \
             (ts, agent_id, method, path, status, request_body, response_summary) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                ts,
                agent_id,
                method_str,
                path_str,
                i64::from(status_u16),
                truncated,
                response_summary,
            ],
        ) {
            tracing::error!(error = %e, "audit insert failed");
        }
    }

    // TODO(phase-3): wrap captured request_body in Tainted<String> before
    // any future egress use.

    response
}

#[cfg(test)]
mod tests {
    use super::{attach, BODY_LIMIT_BYTES};
    use crate::{db::open_db, seed::load_seed, AppState, SimConfig};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, post},
        Router,
    };
    use std::path::{Path, PathBuf};
    use tower::ServiceExt;

    fn fixture_path() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("fixtures/seed.json");
        p
    }

    fn seeded_state() -> AppState {
        let conn = open_db(Path::new(":memory:"), true).expect("db");
        load_seed(&conn, &fixture_path()).expect("seed");
        AppState::new(conn, SimConfig::ephemeral())
    }

    fn mini_router(state: AppState) -> Router {
        let app: Router<AppState> = Router::new()
            .route("/echo", post(echo))
            .route("/z", get(zed));
        attach(app.with_state(state.clone()), state)
    }

    async fn echo(body: String) -> String {
        body
    }

    async fn zed() -> StatusCode {
        StatusCode::NO_CONTENT
    }

    fn count_rows(state: &AppState) -> i64 {
        state
            .db
            .lock()
            .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
            .unwrap()
    }

    #[tokio::test]
    async fn audit_row_shape() {
        let state = seeded_state();
        let app = mini_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/echo")
                    .header("content-type", "text/plain")
                    .body(Body::from("hello"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        assert_eq!(count_rows(&state), 1);

        // Inspect the single row.
        let (method, path, status, body, summary, agent): (
            String,
            String,
            i64,
            String,
            String,
            String,
        ) = {
            let conn = state.db.lock();
            conn.query_row(
                "SELECT method, path, status, request_body, response_summary, agent_id \
                 FROM audit_events ORDER BY id DESC LIMIT 1",
                [],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                    ))
                },
            )
            .unwrap()
        };
        assert_eq!(method, "POST");
        assert_eq!(path, "/echo");
        assert_eq!(status, 200);
        assert_eq!(body, "hello");
        assert_eq!(agent, "anonymous");
        // response_summary = "200:<16-hex>"
        assert!(
            summary.starts_with("200:"),
            "summary must start with 200:, got {summary:?}"
        );
        let hex_part = &summary[4..];
        assert_eq!(
            hex_part.len(),
            16,
            "sha prefix must be 16 hex chars, got {hex_part:?} (len {})",
            hex_part.len()
        );
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn agent_id_header() {
        let state = seeded_state();
        let app = mini_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/z")
                    .header("X-Reposix-Agent", "alpha")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);
        let agent: String = state
            .db
            .lock()
            .query_row(
                "SELECT agent_id FROM audit_events ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(agent, "alpha");
    }

    #[tokio::test]
    async fn trigger_blocks_update() {
        let state = seeded_state();
        let app = mini_router(state.clone());
        // Create at least one row.
        let _ = app
            .oneshot(Request::builder().uri("/z").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(count_rows(&state) >= 1);

        let err = {
            let conn = state.db.lock();
            conn.execute(
                "UPDATE audit_events SET path='x' WHERE id = (SELECT MAX(id) FROM audit_events)",
                [],
            )
            .unwrap_err()
        };
        let msg = err.to_string();
        assert!(
            msg.contains("append-only"),
            "trigger error must mention `append-only` literal; got {msg:?}"
        );
    }

    #[tokio::test]
    async fn oversized_body_returns_413_and_audits() {
        let state = seeded_state();
        let app = mini_router(state.clone());
        let big = vec![b'x'; BODY_LIMIT_BYTES + 1];
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/echo")
                    .body(Body::from(big))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 413);
        // The 413 itself is audited — invariant.
        let status: i64 = state
            .db
            .lock()
            .query_row(
                "SELECT status FROM audit_events ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, 413);
    }
}
