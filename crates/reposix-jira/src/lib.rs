//! [`JiraBackend`] — read/write [`BackendConnector`] adapter for
//! Atlassian JIRA Cloud REST v3.
//!
//! # Scope
//!
//! Phase 28 ships the read path: `list_records` (POST `/rest/api/3/search/jql`
//! with cursor pagination) and `get_record` (GET `/rest/api/3/issue/{id}`).
//! Phase 29 ships the full write path: `create_record`, `update_record`,
//! and `delete_or_close` (via transitions API with DELETE fallback).
//!
//! # Issue → Issue mapping
//!
//! | Issue field     | JIRA source                                                  |
//! |-----------------|--------------------------------------------------------------|
//! | `id`            | `fields.id` (numeric string → u64)                          |
//! | `title`         | `fields.summary`                                             |
//! | `status`        | Two-field mapping on `statusCategory.key` + resolution name  |
//! | `body`          | `fields.description` (ADF → plain text; null → "")           |
//! | `created_at`    | `fields.created`                                             |
//! | `updated_at`    | `fields.updated`                                             |
//! | `version`       | `fields.updated` as Unix-milliseconds u64                    |
//! | `assignee`      | `fields.assignee.displayName`                                |
//! | `labels`        | `fields.labels`                                              |
//! | `parent_id`     | `fields.parent.id` (numeric string → u64)                    |
//! | `extensions`    | `jira_key`, `issue_type`, `priority`, `status_name`, `hierarchy_level` |
//!
//! # Pagination
//!
//! Uses `POST /rest/api/3/search/jql` with cursor-based pagination via
//! `nextPageToken` + `isLast: true` as the terminator. The old `GET /search`
//! endpoint was retired August 2025 and is not used here.
//!
//! # Rate limiting
//!
//! On HTTP 429 the adapter honors the `Retry-After` header (seconds) and
//! parks the rate-limit gate. If the header is absent, exponential backoff
//! with jitter is applied (max 4 attempts, base 1 s, cap 60 s).
//!
//! # Security
//!
//! - **SG-01:** every HTTP call goes through `reposix-core`'s sealed
//!   [`HttpClient`], which re-checks every target URL against
//!   `REPOSIX_ALLOWED_ORIGINS` before any socket I/O. Callers MUST set the
//!   env var to include `https://{tenant}.atlassian.net` at runtime.
//! - **SG-05:** every decoded JIRA issue is wrapped in [`Tainted::new`] before
//!   translation, documenting the "came from untrusted network" origin.
//! - **T-28-01 (creds leak):** [`JiraCreds`] has a manual `Debug` impl that
//!   prints `api_token: "<redacted>"`. Same redaction on the backend struct.
//! - **T-28-02 (SSRF via tenant injection):** [`JiraBackend::new`] validates
//!   `tenant` against DNS-label rules before URL construction.
//!
//! # Module layout
//!
//! Implementation is split alongside three sibling modules so each concern
//! reads in isolation:
//!
//! - [`types`] — [`JiraCreds`] + wire-format response shapes.
//! - [`translate`] — pure DTO → [`Record`] translation + [`validate_tenant`].
//! - [`client`] — [`JiraBackend`] struct, HTTP plumbing, audit hooks, rate-limit gate.
//!
//! This file holds the [`BackendConnector`] trait impl, which adapts the
//! plumbing in [`client`] to the canonical reposix surface.
//!
//! [`HttpClient`]: reposix_core::http::HttpClient

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod adf;
pub mod client;
pub mod translate;
pub mod types;

use async_trait::async_trait;
use reqwest::{Method, StatusCode};

use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
use reposix_core::{Error, Record, RecordId, Result, Tainted, Untainted};

pub use client::JiraBackend;
pub use translate::{basic_auth_header, validate_tenant};
pub use types::{JiraCreds, CAPABILITIES, DEFAULT_BASE_URL_FORMAT};

use crate::translate::translate;
use crate::types::{JiraSearchResponse, JIRA_FIELDS, MAX_ISSUES_PER_LIST, PAGE_SIZE};

// ─── BackendConnector impl ────────────────────────────────────────────────────

#[async_trait]
#[allow(clippy::too_many_lines)] // write path: create_record + update_record + delete_or_close each need ~50 lines
impl BackendConnector for JiraBackend {
    fn name(&self) -> &'static str {
        "jira"
    }

    fn supports(&self, feature: BackendFeature) -> bool {
        matches!(
            feature,
            BackendFeature::Hierarchy | BackendFeature::Delete | BackendFeature::Transitions
        )
    }

    async fn list_records(&self, project: &str) -> Result<Vec<Record>> {
        self.list_issues_impl(project, false).await
    }

    async fn list_changed_since(
        &self,
        project: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<RecordId>> {
        // JQL: `updated >= "yyyy-MM-dd HH:mm"`. JQL does not accept full
        // ISO8601 with timezone — use the canonical two-field form.
        let jql_time = since.format("%Y-%m-%d %H:%M").to_string();
        // Strip quotes from project slug defensively before interpolation.
        let safe_project = project.replace('"', "");
        let url = format!("{}/rest/api/3/search/jql", self.base());
        let fields: Vec<String> = JIRA_FIELDS.iter().map(|s| (*s).to_owned()).collect();
        let mut request_body = serde_json::json!({
            "jql": format!("project = \"{safe_project}\" AND updated >= \"{jql_time}\" ORDER BY id ASC"),
            "fields": fields,
            "maxResults": PAGE_SIZE,
        });

        let mut out: Vec<RecordId> = Vec::new();
        let mut pages: usize = 0;

        let header_owned = self.write_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();

        loop {
            pages += 1;
            if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) + 1 {
                tracing::warn!(
                    pages,
                    "reached MAX_ISSUES_PER_LIST cap; stopping pagination"
                );
                break;
            }

            self.await_rate_limit_gate().await;
            let body_bytes = serde_json::to_vec(&request_body)?;
            let resp = self
                .http
                .request_with_headers_and_body(
                    Method::POST,
                    url.as_str(),
                    &header_refs,
                    Some(body_bytes),
                )
                .await?;
            self.ingest_rate_limit(&resp);
            let status = resp.status();
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "JIRA returned {status} for POST /rest/api/3/search/jql: {}",
                    String::from_utf8_lossy(&bytes)
                )));
            }
            let search_resp: JiraSearchResponse = serde_json::from_slice(&bytes)?;
            let is_last = search_resp.is_last.unwrap_or(true);
            let next_token = search_resp.next_page_token.clone();

            for issue in search_resp.issues {
                // SG-05: wrap as Tainted before translating, then keep only
                // the RecordId. Full-Issue translation is needed because
                // JIRA's payload encodes the id deep in the fields tree.
                let tainted = Tainted::new(issue);
                let translated = translate(tainted.into_inner())?;
                out.push(translated.id);
                if out.len() >= MAX_ISSUES_PER_LIST {
                    return Ok(out);
                }
            }

            if is_last {
                break;
            }
            if let Some(token) = next_token {
                request_body["nextPageToken"] = serde_json::Value::String(token);
            } else {
                break;
            }
        }

        Ok(out)
    }

    async fn get_record(&self, _project: &str, id: RecordId) -> Result<Record> {
        self.await_rate_limit_gate().await;
        self.get_issue_inner(id).await
    }

    async fn create_record(&self, project: &str, issue: Untainted<Record>) -> Result<Record> {
        // Response struct declared before statements to satisfy clippy::items_after_statements.
        #[derive(serde::Deserialize)]
        struct CreateResp {
            id: String,
        }

        // Get or initialize issue type cache.
        let issue_types = if let Some(cached) = self.issue_type_cache.get() {
            cached
        } else {
            let fetched = self.fetch_issue_types(project).await?;
            // Ignore error if another concurrent call beat us to it.
            let _ = self.issue_type_cache.set(fetched);
            self.issue_type_cache.get().expect("just set")
        };
        let chosen_type = issue_types
            .iter()
            .find(|t| t.eq_ignore_ascii_case("Task"))
            .or_else(|| issue_types.first())
            .cloned()
            .unwrap_or_else(|| "Task".to_owned());

        let issue_ref = issue.inner_ref();
        let post_body = serde_json::json!({
            "fields": {
                "project": {"key": project},
                "summary": issue_ref.title,
                "issuetype": {"name": chosen_type},
                "description": crate::adf::adf_paragraph_wrap(&issue_ref.body),
                "labels": issue_ref.labels,
            }
        });
        let post_body_bytes = serde_json::to_vec(&post_body)?;
        let url = format!("{}/rest/api/3/issue", self.base());
        let header_owned = self.write_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers_and_body(
                Method::POST,
                url.as_str(),
                &header_refs,
                Some(post_body_bytes),
            )
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let status_u16 = status.as_u16();
        // T-16-C-04 pattern: audit title only (max 256 chars), never body.
        let req_summary: String = issue_ref.title.chars().take(256).collect();
        self.audit_event(
            "POST",
            "/rest/api/3/issue",
            status_u16,
            &req_summary,
            &bytes,
        );
        if !status.is_success() {
            return Err(Error::Other(format!(
                "jira returned {status} for POST /rest/api/3/issue: {}",
                String::from_utf8_lossy(&bytes)
                    .chars()
                    .take(200)
                    .collect::<String>()
            )));
        }
        // Response: {"id": "10001", "key": "PROJ-1", "self": "..."}
        let created: CreateResp = serde_json::from_slice(&bytes)?;
        let new_id: u64 = created.id.parse().map_err(|_| {
            Error::Other(format!(
                "jira create returned non-numeric id: {}",
                created.id
            ))
        })?;
        // Hydrate full Issue via GET.
        self.get_issue_inner(RecordId(new_id)).await
    }

    async fn update_record(
        &self,
        _project: &str,
        id: RecordId,
        patch: Untainted<Record>,
        _expected_version: Option<u64>,
    ) -> Result<Record> {
        // JIRA has no ETag — expected_version is silently ignored.
        // Status changes are NOT allowed via PUT (require transitions).
        let patch_ref = patch.inner_ref();
        let put_body = serde_json::json!({
            "fields": {
                "summary": patch_ref.title,
                "description": crate::adf::adf_paragraph_wrap(&patch_ref.body),
                "labels": patch_ref.labels,
            }
        });
        let put_body_bytes = serde_json::to_vec(&put_body)?;
        let issue_path = format!("/rest/api/3/issue/{}", id.0);
        let url = format!("{}{}", self.base(), issue_path);
        let header_owned = self.write_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers_and_body(
                Method::PUT,
                url.as_str(),
                &header_refs,
                Some(put_body_bytes),
            )
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let status_u16 = status.as_u16();
        let req_summary: String = patch_ref.title.chars().take(256).collect();
        self.audit_event("PUT", &issue_path, status_u16, &req_summary, &bytes);
        // JIRA PUT returns 204 No Content on success.
        if status == StatusCode::NO_CONTENT {
            return self.get_issue_inner(id).await;
        }
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {}", id.0)));
        }
        Err(Error::Other(format!(
            "jira returned {status} for PUT {issue_path}: {}",
            String::from_utf8_lossy(&bytes)
                .chars()
                .take(200)
                .collect::<String>()
        )))
    }

    async fn delete_or_close(
        &self,
        _project: &str,
        id: RecordId,
        reason: DeleteReason,
    ) -> Result<()> {
        // Struct declarations hoisted before statements (clippy::items_after_statements).
        #[derive(serde::Deserialize)]
        struct TransitionTo {
            #[serde(rename = "statusCategory")]
            status_category: TransitionCategory,
        }
        #[derive(serde::Deserialize)]
        struct TransitionCategory {
            key: String,
        }
        #[derive(serde::Deserialize)]
        struct Transition {
            id: String,
            name: String,
            to: TransitionTo,
        }
        #[derive(serde::Deserialize)]
        struct TransitionsResp {
            transitions: Vec<Transition>,
        }

        let transitions_path = format!("/rest/api/3/issue/{}/transitions", id.0);
        let transitions_url = format!("{}{}", self.base(), transitions_path);

        // Step 1: GET available transitions.
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::GET, transitions_url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let get_status = resp.status();
        let bytes = resp.bytes().await?;
        if !get_status.is_success() {
            self.audit_event("GET", &transitions_path, get_status.as_u16(), "", &bytes);
            return Err(Error::Other(format!(
                "jira transitions GET returned {get_status} for issue {}",
                id.0
            )));
        }

        let parsed: TransitionsResp = serde_json::from_slice(&bytes).unwrap_or(TransitionsResp {
            transitions: vec![],
        });

        // Step 2: filter to "done" category transitions.
        let done: Vec<&Transition> = parsed
            .transitions
            .iter()
            .filter(|t| t.to.status_category.key == "done")
            .collect();

        if done.is_empty() {
            // Fallback: DELETE /rest/api/3/issue/{id}
            tracing::warn!(
                issue_id = id.0,
                "jira: no done transitions found, falling back to DELETE"
            );
            let delete_path = format!("/rest/api/3/issue/{}", id.0);
            let delete_url = format!("{}{}", self.base(), delete_path);
            let del_header_owned = self.standard_headers();
            let del_header_refs: Vec<(&str, &str)> = del_header_owned
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            self.await_rate_limit_gate().await;
            let del_resp = self
                .http
                .request_with_headers(Method::DELETE, delete_url.as_str(), &del_header_refs)
                .await?;
            self.ingest_rate_limit(&del_resp);
            let del_status = del_resp.status();
            let del_bytes = del_resp.bytes().await?;
            self.audit_event("DELETE", &delete_path, del_status.as_u16(), "", &del_bytes);
            if del_status == StatusCode::NO_CONTENT {
                return Ok(());
            }
            return Err(Error::Other(format!(
                "jira DELETE fallback returned {del_status} for issue {}",
                id.0
            )));
        }

        // Step 3: select transition by reason preference.
        // NotPlanned/Duplicate map to "Won't Fix"-style transitions where available.
        let prefer_wontfix = matches!(reason, DeleteReason::NotPlanned | DeleteReason::Duplicate);
        let chosen = if prefer_wontfix {
            done.iter()
                .find(|t| {
                    let lower = t.name.to_lowercase();
                    lower.contains("won't")
                        || lower.contains("wont")
                        || lower.contains("reject")
                        || lower.contains("not planned")
                        || lower.contains("invalid")
                        || lower.contains("duplicate")
                })
                .or_else(|| done.first())
        } else {
            done.first()
        }
        .expect("done is non-empty — checked above");

        // Step 4: POST transition.
        let transition_body = serde_json::json!({"transition": {"id": chosen.id}});
        let post_bytes = serde_json::to_vec(&transition_body)?;
        let write_owned = self.write_headers();
        let write_refs: Vec<(&str, &str)> =
            write_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let post_resp = self
            .http
            .request_with_headers_and_body(
                Method::POST,
                transitions_url.as_str(),
                &write_refs,
                Some(post_bytes),
            )
            .await?;
        self.ingest_rate_limit(&post_resp);
        let post_status = post_resp.status();
        let post_body_bytes = post_resp.bytes().await?;

        // JIRA may require resolution field on 400 — retry with it.
        if post_status == StatusCode::BAD_REQUEST {
            let retry_body = serde_json::json!({
                "transition": {"id": chosen.id},
                "fields": {"resolution": {"name": "Done"}},
            });
            let retry_bytes = serde_json::to_vec(&retry_body)?;
            self.await_rate_limit_gate().await;
            let retry_resp = self
                .http
                .request_with_headers_and_body(
                    Method::POST,
                    transitions_url.as_str(),
                    &write_refs,
                    Some(retry_bytes),
                )
                .await?;
            self.ingest_rate_limit(&retry_resp);
            let retry_status = retry_resp.status();
            let retry_body_bytes = retry_resp.bytes().await?;
            self.audit_event(
                "POST",
                &transitions_path,
                retry_status.as_u16(),
                &format!("transition:{}", chosen.id),
                &retry_body_bytes,
            );
            if retry_status == StatusCode::NO_CONTENT || retry_status.is_success() {
                return Ok(());
            }
            return Err(Error::Other(format!(
                "jira transition POST retry returned {retry_status} for issue {}",
                id.0
            )));
        }

        self.audit_event(
            "POST",
            &transitions_path,
            post_status.as_u16(),
            &format!("transition:{}", chosen.id),
            &post_body_bytes,
        );
        if post_status == StatusCode::NO_CONTENT || post_status.is_success() {
            return Ok(());
        }
        Err(Error::Other(format!(
            "jira transition POST returned {post_status} for issue {}",
            id.0
        )))
    }
}
