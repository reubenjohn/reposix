//! Jira-style transitions endpoint.
//!
//! `GET /projects/:slug/issues/:id/transitions` returns the current issue
//! status and the list of other legal statuses. v0.1 best-effort: *all*
//! other statuses are reported as legal; the real workflow rule set
//! ("must pass through `in_progress`" etc.) is deferred to a future
//! version (not on the current roadmap).

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use reposix_core::RecordStatus;
use rusqlite::params;
use serde::Serialize;

use crate::{error::ApiError, routes::issues, AppState};

const ALL_STATUSES: [RecordStatus; 5] = [
    RecordStatus::Open,
    RecordStatus::InProgress,
    RecordStatus::InReview,
    RecordStatus::Done,
    RecordStatus::WontFix,
];

/// Build the transitions sub-router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/projects/:slug/issues/:id/transitions",
            get(get_transitions),
        )
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct TransitionsResponse {
    current_state: String,
    available: Vec<String>,
}

#[allow(clippy::unused_async)]
async fn get_transitions(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, u64)>,
) -> Result<Json<TransitionsResponse>, ApiError> {
    #[allow(clippy::cast_possible_wrap)]
    let id_signed = id as i64;
    let current_raw: String = {
        let conn = state.db.lock();
        match conn.query_row(
            "SELECT status FROM issues WHERE project = ?1 AND id = ?2",
            params![slug, id_signed],
            |r| r.get::<_, String>(0),
        ) {
            Ok(s) => s,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(ApiError::NotFound),
            Err(e) => return Err(ApiError::Db(e)),
        }
    };

    let current = issues::parse_status_shared(&current_raw)?;
    let available: Vec<String> = ALL_STATUSES
        .iter()
        .filter(|s| **s as u8 != current as u8)
        .map(|s| s.as_str().to_owned())
        .collect();

    Ok(Json(TransitionsResponse {
        current_state: current.as_str().to_owned(),
        available,
    }))
}
