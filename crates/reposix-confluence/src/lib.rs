//! [`ConfluenceBackend`] — read/write [`BackendConnector`] adapter for
//! Atlassian Confluence Cloud REST v2.
//!
//! # Scope
//!
//! v0.6 ships the full read+write path: `list_records` + `get_record` work
//! against a real Atlassian tenant (once credentials are configured);
//! `create_record`, `update_record`, and `delete_or_close` are implemented
//! against `POST /wiki/api/v2/pages`, `PUT /wiki/api/v2/pages/{id}`, and
//! `DELETE /wiki/api/v2/pages/{id}` respectively. Audit log rows are added in
//! Wave C (v0.6 Phase 16).
//!
//! # Page → Issue mapping (Option A flattening, per ADR-002)
//!
//! | Issue field  | Confluence source                             |
//! |--------------|-----------------------------------------------|
//! | `id`         | `parse_u64(page.id)` — Confluence page IDs are numeric strings |
//! | `title`      | `page.title`                                  |
//! | `status`     | `"current" | "draft"` → `Open`; `"archived" | "trashed" | "deleted"` → `Done` |
//! | `body`       | `page.body.storage.value` (raw storage HTML)  |
//! | `created_at` | `page.createdAt`                              |
//! | `updated_at` | `page.version.createdAt`                      |
//! | `version`    | `page.version.number`                         |
//! | `assignee`   | `page.ownerId` (Atlassian accountId)          |
//! | `labels`     | `vec![]` (deferred — labels live on a separate endpoint) |
//!
//! Parent/child hierarchy, space metadata, comments, attachments, and
//! restrictions are out of scope for v0.3 and documented as lost metadata
//! in ADR-002.
//!
//! # Pagination
//!
//! Confluence v2 uses cursor-based pagination via `_links.next` (relative
//! path). This differs from GitHub's `Link: rel="next"` header. The adapter
//! prepends `self.base()` to the relative cursor path; attacker-controlled
//! absolute URLs would still be caught by the SG-01 allowlist gate inside
//! [`HttpClient`], but the construction-by-relative-prepend shape defeats
//! that class of attack by construction.
//!
//! # Rate limiting
//!
//! On HTTP 429 (or when `x-ratelimit-remaining: 0` is reported), the adapter
//! arms a shared rate-limit gate on an `Instant` derived from the
//! `Retry-After` header (seconds). The next outbound call parks until the
//! gate elapses, capped at `MAX_RATE_LIMIT_SLEEP`.
//!
//! # Security
//!
//! - **SG-01:** every HTTP call goes through `reposix-core`'s sealed
//!   [`HttpClient`], which re-checks every target URL against
//!   `REPOSIX_ALLOWED_ORIGINS` before any socket I/O. Callers MUST set the
//!   env var to include `https://{tenant}.atlassian.net` at runtime.
//! - **SG-05:** every decoded page is wrapped through [`Tainted::new`] before
//!   translation, documenting the "came from untrusted network" origin of
//!   every byte.
//! - **T-11-01 (creds leak):** [`ConfluenceCreds`] has a manual `Debug` impl
//!   that prints `api_token: "<redacted>"`. Same redaction on the backend
//!   struct.
//! - **T-11-02 (SSRF via tenant injection):** [`ConfluenceBackend::new`]
//!   validates `tenant` against DNS-label rules (`^[a-z0-9][a-z0-9-]{0,62}$`)
//!   before URL construction.
//!
//! # Module layout
//!
//! Implementation is split alongside three sibling modules so each concern
//! reads in isolation:
//!
//! - [`types`] — [`ConfluenceCreds`] + wire-format response shapes + comment
//!   + attachment + whiteboard public types.
//! - [`translate`] — pure DTO → [`Record`] translation + [`validate_tenant`]
//!   + [`basic_auth_header`] + [`parse_next_cursor`] + redaction helpers.
//! - [`client`] — [`ConfluenceBackend`] struct, HTTP plumbing, audit hooks,
//!   rate-limit gate.
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

pub use client::ConfluenceBackend;
pub use translate::{basic_auth_header, parse_next_cursor, status_from_confluence};
pub use types::{
    CommentKind, ConfAttachment, ConfBodyAdf, ConfBodyStorage, ConfComment, ConfCommentVersion,
    ConfPageBody, ConfSpaceSummary, ConfWhiteboard, ConfluenceCreds, CAPABILITIES,
    DEFAULT_BASE_URL_FORMAT,
};

use crate::translate::{redact_url, translate};
use crate::types::{ConfPage, MAX_ISSUES_PER_LIST, PAGE_SIZE};

// ─── BackendConnector impl ────────────────────────────────────────────────────

#[async_trait]
impl BackendConnector for ConfluenceBackend {
    fn name(&self) -> &'static str {
        "confluence"
    }

    fn supports(&self, feature: BackendFeature) -> bool {
        // v0.6: write path is now implemented.
        // - `Hierarchy`: Confluence is the only current backend that exposes a
        //   parent/child tree, used by FUSE for the `tree/` overlay.
        // - `Delete`: `DELETE /wiki/api/v2/pages/{id}` moves pages to trash.
        // - `StrongVersioning`: PUT body carries `version.number = current + 1`;
        //   Confluence returns 409 on concurrent edits (optimistic locking).
        // `Workflows` and `BulkEdit` remain false: Confluence has no in-flight
        // state labels analogous to GitHub's status/* and no batch-edit API.
        matches!(
            feature,
            BackendFeature::Hierarchy | BackendFeature::Delete | BackendFeature::StrongVersioning
        )
    }

    fn root_collection_name(&self) -> &'static str {
        // Confluence-native vocabulary: pages, not issues. The default
        // `"issues"` stays correct for sim + GitHub; Confluence overrides
        // here so mounts surface as `pages/<padded-id>.md` — the layout
        // locked in Phase 13 CONTEXT.md and ADR-003.
        "pages"
    }

    async fn list_records(&self, project: &str) -> Result<Vec<Record>> {
        self.list_issues_impl(project, false).await
    }

    async fn list_changed_since(
        &self,
        project: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<RecordId>> {
        // Confluence CQL accepts `lastModified > "yyyy-MM-dd HH:mm"` —
        // seconds-precision is not supported by CQL.
        // Strip `"` from the project slug defensively to defeat CQL injection
        // via a malicious space key (legitimate keys never contain quotes).
        let cql_time = since.format("%Y-%m-%d %H:%M").to_string();
        let safe_project = project.replace('"', "");
        let cql = format!("space = \"{safe_project}\" AND lastModified > \"{cql_time}\"");
        // URL-encode via `url::Url::query_pairs_mut` — same pattern as
        // `resolve_space_id` (avoids adding a new dep just for one call site).
        let mut url = url::Url::parse(&format!("{}/wiki/rest/api/search", self.base()))
            .map_err(|e| Error::Other(format!("bad base url: {e}")))?;
        url.query_pairs_mut()
            .append_pair("cql", &cql)
            .append_pair("limit", &PAGE_SIZE.to_string());
        let url = url.to_string();

        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::GET, url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for GET {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        // Search endpoint shape: `{ "results": [{ "content": { "id": "<numeric>" } }, ...] }`.
        // Extract content.id, parse as u64. Single-page MVP for v0.9.0 — pagination
        // adds complexity (CQL search uses `_links.next`); a delta exceeding
        // PAGE_SIZE is rare enough that surfacing the cap loudly via a future
        // strict mode is acceptable. (Tests in Phase 35 will validate against
        // real backends.)
        let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
        let arr = body_json
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut out: Vec<RecordId> = Vec::with_capacity(arr.len());
        for res in arr {
            if let Some(id_str) = res.pointer("/content/id").and_then(|v| v.as_str()) {
                if let Ok(n) = id_str.parse::<u64>() {
                    out.push(RecordId(n));
                    if out.len() >= MAX_ISSUES_PER_LIST {
                        break;
                    }
                }
            }
        }
        Ok(out)
    }

    async fn get_record(&self, _project: &str, id: RecordId) -> Result<Record> {
        // First attempt: request ADF body format for lossless Markdown conversion.
        let url_adf = format!(
            "{}/wiki/api/v2/pages/{}?body-format=atlas_doc_format",
            self.base(),
            id.0
        );
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::GET, url_adf.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {}", redact_url(&url_adf))));
        }
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for GET {}: {}",
                redact_url(&url_adf),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: ConfPage = serde_json::from_slice(&bytes)?;
        // Check if the ADF body is non-empty (null ADF means pre-ADF page).
        let adf_present = page
            .body
            .as_ref()
            .and_then(|b| b.adf.as_ref())
            .is_some_and(|a| !a.value.is_null());
        if adf_present {
            // SG-05: Tainted::new wraps ingress before translation.
            let tainted = Tainted::new(page);
            return translate(tainted.into_inner());
        }
        // Fallback: ADF body was absent/null (pre-ADF page) — re-fetch with
        // storage format so callers still get the raw storage HTML.
        let url_storage = format!(
            "{}/wiki/api/v2/pages/{}?body-format=storage",
            self.base(),
            id.0
        );
        self.await_rate_limit_gate().await;
        let resp2 = self
            .http
            .request_with_headers(Method::GET, url_storage.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp2);
        let status2 = resp2.status();
        let bytes2 = resp2.bytes().await?;
        if status2 == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!(
                "not found: {}",
                redact_url(&url_storage)
            )));
        }
        if !status2.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status2} for GET {}: {}",
                redact_url(&url_storage),
                String::from_utf8_lossy(&bytes2)
            )));
        }
        let page2: ConfPage = serde_json::from_slice(&bytes2)?;
        // SG-05: Tainted::new wraps ingress before translation.
        let tainted = Tainted::new(page2);
        translate(tainted.into_inner())
    }

    async fn create_record(&self, project: &str, issue: Untainted<Record>) -> Result<Record> {
        let space_id = self.resolve_space_id(project).await?;
        // Convert the Markdown body to Confluence storage XHTML.
        let storage_xhtml = crate::adf::markdown_to_storage(&issue.inner_ref().body)?;
        let post_body = serde_json::json!({
            "spaceId": space_id,
            "status": "current",
            "title": issue.inner_ref().title,
            "parentId": issue.inner_ref().parent_id.map(|id| id.0.to_string()),
            "body": {
                "representation": "storage",
                "value": storage_xhtml,
            },
        });
        let post_body_bytes = serde_json::to_vec(&post_body)?;
        let url = format!("{}/wiki/api/v2/pages", self.base());
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
        // T-16-C-04: audit title only (first 256 chars), never body content.
        let req_summary: String = issue.inner_ref().title.chars().take(256).collect();
        self.audit_write(
            "POST",
            "/wiki/api/v2/pages",
            status_u16,
            &req_summary,
            &bytes,
        );
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for POST {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: ConfPage = serde_json::from_slice(&bytes)?;
        // SG-05: wrap ingress bytes as Tainted before translating.
        let tainted = Tainted::new(page);
        translate(tainted.into_inner())
    }

    async fn update_record(
        &self,
        _project: &str,
        id: RecordId,
        patch: Untainted<Record>,
        expected_version: Option<u64>,
    ) -> Result<Record> {
        // Pre-flight version resolution: if caller supplied an expected version
        // trust it; otherwise do a GET to discover the current version number.
        let current_version = match expected_version {
            Some(v) => v,
            None => self.fetch_current_version(id).await?,
        };
        // Convert the Markdown body to Confluence storage XHTML.
        let storage_xhtml = crate::adf::markdown_to_storage(&patch.inner_ref().body)?;
        let put_body = serde_json::json!({
            "id": id.0.to_string(),
            "status": "current",
            "title": patch.inner_ref().title,
            "version": { "number": current_version + 1 },
            "body": {
                "representation": "storage",
                "value": storage_xhtml,
            },
        });
        let put_body_bytes = serde_json::to_vec(&put_body)?;
        let url = format!("{}/wiki/api/v2/pages/{}", self.base(), id.0);
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
        // T-16-C-04: audit title only (first 256 chars), never body content.
        let req_summary: String = patch.inner_ref().title.chars().take(256).collect();
        let audit_path = format!("/wiki/api/v2/pages/{}", id.0);
        self.audit_write("PUT", &audit_path, status_u16, &req_summary, &bytes);
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {}", redact_url(&url))));
        }
        if status == StatusCode::CONFLICT {
            let body_preview: String = String::from_utf8_lossy(&bytes).chars().take(256).collect();
            return Err(Error::Other(format!(
                "confluence version conflict for PUT {}: {body_preview}",
                redact_url(&url),
            )));
        }
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for PUT {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: ConfPage = serde_json::from_slice(&bytes)?;
        // SG-05: wrap ingress bytes as Tainted before translating.
        let tainted = Tainted::new(page);
        translate(tainted.into_inner())
    }

    async fn delete_or_close(
        &self,
        _project: &str,
        id: RecordId,
        _reason: DeleteReason,
    ) -> Result<()> {
        // Note: `reason` is intentionally ignored — Confluence has no reason
        // field on DELETE. The DELETE moves the page to trash (status becomes
        // "trashed"), which the read path already maps to `RecordStatus::Done`.
        // A future "purge" would require `?purge=true`; that is out of scope
        // for v0.6.
        let url = format!("{}/wiki/api/v2/pages/{}", self.base(), id.0);
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::DELETE, url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let status_u16 = status.as_u16();
        let audit_path = format!("/wiki/api/v2/pages/{}", id.0);
        if status == StatusCode::NO_CONTENT {
            // 204: no body — audit with empty bytes, empty request summary.
            self.audit_write("DELETE", &audit_path, status_u16, "", &[]);
            return Ok(());
        }
        let bytes = resp.bytes().await?;
        // Audit on all non-204 paths (failures).
        self.audit_write("DELETE", &audit_path, status_u16, "", &bytes);
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {}", redact_url(&url))));
        }
        Err(Error::Other(format!(
            "confluence returned {status} for DELETE {}: {}",
            redact_url(&url),
            String::from_utf8_lossy(&bytes)
        )))
    }
}
