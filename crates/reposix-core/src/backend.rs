//! Backend seam: [`BackendConnector`] is the trait every concrete issue-tracker
//! adapter implements. Implementors include [`sim::SimBackend`],
//! `GithubReadOnlyBackend` (`reposix-github`), `ConfluenceBackend`
//! (`reposix-confluence`), and `JiraBackend` (`reposix-jira`).
//!
//! ## Why this trait exists
//!
//! The simulator originally spoke its REST shape directly to the rest of the
//! workspace. Once real backends (GitHub, Confluence, JIRA) entered the picture,
//! a seam was needed so the cache materializer, remote helper, and CLI
//! orchestrator don't have to learn each new backend's quirks. The trait is the
//! normalization boundary: concrete adapters translate backend-specific wire
//! shapes into [`Record`](crate::Record) / [`RecordStatus`](crate::RecordStatus)
//! — and nothing more escapes.
//!
//! ## Error model
//!
//! Methods return [`Result<T>`](crate::Result). Backends that cannot satisfy a
//! given method (e.g. a read-only GitHub backend asked to `create_record`)
//! return [`Error::Other`](crate::Error::Other) with a `"not supported: ..."`
//! message. A typed `NotSupported` variant remains a possible future cleanup;
//! for now we keep the error surface additive-only so existing callers
//! (cache, remote helper) aren't forced to branch on a new enum arm.
//!
//! Read-path `not found` conditions (e.g. `GET /issues/{id}` returns 404) are
//! also surfaced as [`Error::Other`]. Callers who need to discriminate should
//! `match err { Error::Other(msg) if msg.starts_with("not found") => ... }`.
//!
//! ## Feature matrix
//!
//! Not every backend supports every operation. Callers that want to branch on
//! capability — "does this backend do strong versioning? if not, skip the
//! If-Match dance" — call [`BackendConnector::supports`]. See
//! [`BackendFeature`] for the set of capability flags.

#![allow(clippy::module_name_repetitions)]

use async_trait::async_trait;

use crate::record::{Record, RecordId};
use crate::taint::Untainted;
use crate::Result;

pub mod sim;

/// Capability flags a caller can query via [`BackendConnector::supports`].
///
/// This is a closed enum: new variants are API-breaking changes and must land
/// with a version bump. Each variant is orthogonal — a backend may support
/// any subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendFeature {
    /// Backend supports a real `DELETE` — not a close-to-delete remap.
    /// The simulator supports this; GitHub does not (it only closes).
    Delete,
    /// Backend honors `DeleteReason` variants and translates them to
    /// backend-specific state transitions (e.g. GitHub's
    /// `state_reason: completed|not_planned`).
    Transitions,
    /// Backend supports optimistic concurrency via a version/etag mechanism.
    /// Simulator: `If-Match: "<version>"`. GitHub v0.2:
    /// `If-Unmodified-Since`. Backends without this feature ignore the
    /// `expected_version` argument to `update_record`.
    StrongVersioning,
    /// Backend accepts bulk edits in a single request. Neither v0.1.5
    /// backend claims this yet; reserved for future Jira adapter.
    BulkEdit,
    /// Backend supports named workflow transitions beyond the 5-valued
    /// [`RecordStatus`](crate::RecordStatus) enum. Reserved for Jira.
    Workflows,
    /// Backend exposes a parent/child hierarchy via [`Record::parent_id`].
    /// Surfaced via the cache-materialized tree so consumers can walk the
    /// parent/child structure with ordinary `git ls-tree`.
    Hierarchy,
}

/// What a backend connector can do for an agent. Used by `reposix doctor`
/// to surface a capability-matrix row at runtime so an agent doesn't have to
/// infer support from a generic "not supported" error after the fact.
///
/// This is the agent-facing analogue of [`BackendFeature`]: where
/// `BackendFeature` is the runtime capability query an *implementation*
/// branches on, `BackendCapabilities` is the static row each backend
/// publishes alongside its connector for *discovery* (doctor, docs, etc.).
/// They are kept deliberately separate so the doctor row can describe the
/// shape of the support (e.g. `CommentSupport::SeparateApi`) without
/// inflating the runtime feature enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
// The four bools (read/create/update/delete) are intrinsic to the CRUD
// matrix shape we're surfacing in `reposix doctor`; collapsing them into
// an enum would lose the per-verb readability that's the whole point.
#[allow(clippy::struct_excessive_bools)]
pub struct BackendCapabilities {
    /// Backend can read records (`GET /<collection>/<id>`).
    pub read: bool,
    /// Backend can create new records.
    pub create: bool,
    /// Backend can update existing records.
    pub update: bool,
    /// Backend can delete or close records.
    pub delete: bool,
    /// How the backend exposes comments — see [`CommentSupport`].
    pub comments: CommentSupport,
    /// How the backend handles concurrent edits — see [`VersioningModel`].
    pub versioning: VersioningModel,
}

impl BackendCapabilities {
    /// Build a capability row. `const`-callable so each backend crate can
    /// publish a `pub const CAPABILITIES: BackendCapabilities = …` next to
    /// its connector. The struct itself is `#[non_exhaustive]`, so this
    /// constructor is the supported way to populate it from outside the
    /// `reposix-core` crate — additive future fields default to a
    /// conservative `false` / `None` shape via this constructor's signature
    /// being bumped in the same release as the new field.
    // See struct-level rationale: four positional bools mirror the CRUD
    // matrix and are the readable form of this constructor.
    #[allow(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub const fn new(
        read: bool,
        create: bool,
        update: bool,
        delete: bool,
        comments: CommentSupport,
        versioning: VersioningModel,
    ) -> Self {
        Self {
            read,
            create,
            update,
            delete,
            comments,
            versioning,
        }
    }
}

/// Whether a backend round-trips comments inline with the body, surfaces
/// them through a separate API, or doesn't support them at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommentSupport {
    /// Round-tripped in the issue/page body verbatim — visible in `git diff`.
    InBody,
    /// Routed to a separate comments API; not visible in the body diff.
    SeparateApi,
    /// Not yet supported.
    None,
}

/// How a backend exposes optimistic concurrency to writers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VersioningModel {
    /// Backend exposes a strong version (e.g. `version.number`), pushed
    /// with optimistic concurrency.
    Strong,
    /// HTTP-style `ETag` round-tripped via `If-Match`.
    Etag,
    /// Best-effort timestamp — write-after-read may overwrite concurrent
    /// edits silently.
    Timestamp,
}

/// Reason a `delete_or_close` call was issued. Backends that support real
/// `DELETE` may ignore the reason; backends that close-with-reason (GitHub)
/// translate the variant into their wire shape.
///
/// The variants mirror GitHub's `state_reason` taxonomy so the read path's
/// reverse mapping (decided in ADR-001) round-trips cleanly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DeleteReason {
    /// Work was done — maps to GitHub's `state_reason: completed`.
    Completed,
    /// Work was abandoned — maps to GitHub's `state_reason: not_planned`.
    NotPlanned,
    /// A duplicate of another issue. Reserved for backends that surface this
    /// state natively; v0.1.5 sim maps it to a plain DELETE.
    Duplicate,
    /// Generic abandonment, no specific reason. Reserved for compatibility
    /// with trackers that don't carry a reason field.
    Abandoned,
}

/// The seam every concrete issue-tracker adapter implements.
///
/// Implementors:
/// - [`sim::SimBackend`] — talks to the in-process `reposix-sim`.
/// - `GithubReadOnlyBackend` (v0.2, new crate `reposix-github`).
///
/// All methods are `async` via `#[async_trait::async_trait]`. This is
/// dyn-compatible; callers can hold `Box<dyn BackendConnector>` or `Arc<dyn
/// BackendConnector>` freely. The trait object is used by the CLI
/// (`reposix list`, `reposix refresh`) and by the cache materializer
/// (`reposix-cache`) when it builds tree objects from `list_records` /
/// `get_record` responses.
///
/// ## Thread-safety
///
/// `Send + Sync` bounds let callers spawn work across tokio tasks. All
/// implementors must honour that — e.g. [`sim::SimBackend`] uses an `Arc`-shared
/// `HttpClient` internally.
///
/// ## Error contract
///
/// - Read-path "not found" on `get_record` → `Err(Error::Other("not found: ..."))`.
///   (Typed `NotFound` variant is a v0.2 cleanup — see module docs.)
/// - Write-path "not supported" on read-only backends → `Err(Error::Other("not supported: ..."))`.
/// - Transport / backend errors propagate as [`Error::Http`](crate::Error::Http) /
///   [`Error::Json`](crate::Error::Json) / etc.
#[async_trait]
pub trait BackendConnector: Send + Sync {
    /// Stable, human-readable backend name. Used in log lines and the parity
    /// demo's output header. Examples: `"simulator"`, `"github"`.
    fn name(&self) -> &'static str;

    /// Capability query. Returns `true` iff this backend supports `feature`.
    ///
    /// Callers branch on this BEFORE attempting the operation so the error
    /// message lists the backend's own name instead of the generic "not
    /// supported". Cheap (no network) — implementors should hard-code the
    /// capability matrix.
    fn supports(&self, feature: BackendFeature) -> bool;

    /// List all issues in `project`. Order is backend-defined but stable
    /// within a single call.
    ///
    /// # Errors
    /// Propagates transport errors. Returns an empty vec (not an error) when
    /// the project exists but has no issues.
    async fn list_records(&self, project: &str) -> Result<Vec<Record>>;

    /// List issue IDs whose `updated_at` is strictly greater than `since`.
    ///
    /// The default implementation calls [`list_records`](Self::list_records)
    /// and filters in memory — safe for any backend but inefficient. Backends
    /// with a native incremental query (`?since=` on GitHub, JQL `updated >=`
    /// on JIRA, CQL `lastModified >` on Confluence, `?since=` on the sim)
    /// MUST override to send the filter over the wire.
    ///
    /// Returns IDs only; callers materialize full `Record` objects on
    /// demand via [`get_record`](Self::get_record). This mirrors the cache's
    /// lazy-blob design: metadata (IDs) is cheap to ship, bodies are not.
    ///
    /// # Errors
    /// Same as [`list_records`](Self::list_records) — transport errors,
    /// egress-allowlist denial (`Error::InvalidOrigin`), or backend-specific
    /// error shapes surfacing as `Error::Other`.
    async fn list_changed_since(
        &self,
        project: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<RecordId>> {
        let all = self.list_records(project).await?;
        Ok(all
            .into_iter()
            .filter(|i| i.updated_at > since)
            .map(|i| i.id)
            .collect())
    }

    /// Fetch a single issue by id.
    ///
    /// # Errors
    /// - Transport errors propagate.
    /// - Unknown `id` returns `Err(Error::Other("not found: ..."))`. See
    ///   module docs re: v0.2 typed `NotFound` variant.
    async fn get_record(&self, project: &str, id: RecordId) -> Result<Record>;

    /// Create a new issue. The `Untainted` wrapper enforces that server-
    /// controlled fields on `issue` (`id`, `created_at`, `version`) have been
    /// stripped — see [`sanitize`](crate::sanitize).
    ///
    /// Returns the server's canonical view of the created issue (with
    /// server-assigned id + timestamps).
    ///
    /// # Errors
    /// - Transport errors propagate.
    /// - Read-only backends return `Err(Error::Other("not supported: ..."))`.
    async fn create_record(&self, project: &str, issue: Untainted<Record>) -> Result<Record>;

    /// Update an existing issue. `patch` carries the fields to overwrite;
    /// untouched fields retain their current server value (backend decides
    /// the exact semantics of "untouched" — sim uses field-presence).
    ///
    /// `expected_version = Some(v)` opts into optimistic concurrency: if the
    /// server's current version is not `v`, the call fails. `None` means
    /// "wildcard" — overwrite whatever is there. Backends without
    /// [`BackendFeature::StrongVersioning`] ignore this argument.
    ///
    /// # Errors
    /// - Transport errors propagate.
    /// - Version mismatch returns `Err(Error::Other("version mismatch: ..."))`.
    /// - Read-only backends return `Err(Error::Other("not supported: ..."))`.
    async fn update_record(
        &self,
        project: &str,
        id: RecordId,
        patch: Untainted<Record>,
        expected_version: Option<u64>,
    ) -> Result<Record>;

    /// Delete or close an issue. Backends with [`BackendFeature::Delete`]
    /// may perform a real `DELETE`; backends without it (GitHub) translate
    /// `reason` to their close-with-reason wire shape.
    ///
    /// # Errors
    /// - Transport errors propagate.
    /// - Unknown id returns `Err(Error::Other("not found: ..."))`.
    /// - Read-only backends return `Err(Error::Other("not supported: ..."))`.
    async fn delete_or_close(
        &self,
        project: &str,
        id: RecordId,
        reason: DeleteReason,
    ) -> Result<()>;

    /// The top-level directory name under which this backend's canonical
    /// `<padded-id>.md` files are mounted. Default `"issues"`. Backends with
    /// a domain-specific vocabulary (e.g. Confluence → `"pages"`) override.
    ///
    /// The return value MUST be a valid single POSIX pathname component:
    /// no `/`, no `..`, non-empty, ASCII.
    fn root_collection_name(&self) -> &'static str {
        "issues"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time proof that `BackendConnector` is dyn-compatible. If any
    /// future method breaks object-safety (e.g. a generic `fn foo<T>`) the
    /// compiler will reject this function.
    #[allow(dead_code)]
    fn _assert_dyn_compatible(_: &dyn BackendConnector) {}

    #[test]
    fn backend_feature_is_copy() {
        let f = BackendFeature::Delete;
        // Implicit Copy: pass by value twice; if `f` were move-only the second
        // use would fail to compile.
        assert_eq!(f, BackendFeature::Delete);
        assert_eq!(f, BackendFeature::Delete);
    }

    #[test]
    fn delete_reason_is_copy() {
        let r = DeleteReason::Completed;
        assert_eq!(r, DeleteReason::Completed);
        assert_eq!(r, DeleteReason::Completed);
    }

    #[test]
    fn backend_feature_hierarchy_is_a_variant() {
        // Trivially compiles iff the `Hierarchy` variant exists on the enum.
        let f = BackendFeature::Hierarchy;
        assert_eq!(f, BackendFeature::Hierarchy);
    }

    #[test]
    fn default_root_collection_name_is_issues() {
        use crate::taint::Untainted;

        struct Stub;
        #[async_trait]
        impl BackendConnector for Stub {
            fn name(&self) -> &'static str {
                "stub"
            }
            fn supports(&self, _: BackendFeature) -> bool {
                false
            }
            async fn list_records(&self, _: &str) -> Result<Vec<Record>> {
                Ok(vec![])
            }
            async fn get_record(&self, _: &str, _: RecordId) -> Result<Record> {
                unimplemented!()
            }
            async fn create_record(&self, _: &str, _: Untainted<Record>) -> Result<Record> {
                unimplemented!()
            }
            async fn update_record(
                &self,
                _: &str,
                _: RecordId,
                _: Untainted<Record>,
                _: Option<u64>,
            ) -> Result<Record> {
                unimplemented!()
            }
            async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> Result<()> {
                Ok(())
            }
        }

        // Default method returns "issues" with no override.
        assert_eq!(Stub.root_collection_name(), "issues");
        // Default `supports` returns false for Hierarchy.
        assert!(!Stub.supports(BackendFeature::Hierarchy));
    }

    #[tokio::test]
    async fn default_list_changed_since_filters_via_list_issues() {
        use crate::record::{Record, RecordStatus};
        use crate::taint::Untainted;
        use chrono::{TimeZone, Utc};

        struct TwoIssues;
        #[async_trait]
        impl BackendConnector for TwoIssues {
            fn name(&self) -> &'static str {
                "two"
            }
            fn supports(&self, _: BackendFeature) -> bool {
                false
            }
            async fn list_records(&self, _: &str) -> Result<Vec<Record>> {
                let t1 = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
                let t2 = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
                Ok(vec![
                    Record {
                        id: RecordId(1),
                        title: "old".into(),
                        status: RecordStatus::Open,
                        assignee: None,
                        labels: vec![],
                        created_at: t1,
                        updated_at: t1,
                        version: 1,
                        body: String::new(),
                        parent_id: None,
                        extensions: std::collections::BTreeMap::new(),
                    },
                    Record {
                        id: RecordId(2),
                        title: "new".into(),
                        status: RecordStatus::Open,
                        assignee: None,
                        labels: vec![],
                        created_at: t1,
                        updated_at: t2,
                        version: 1,
                        body: String::new(),
                        parent_id: None,
                        extensions: std::collections::BTreeMap::new(),
                    },
                ])
            }
            async fn get_record(&self, _: &str, _: RecordId) -> Result<Record> {
                unimplemented!()
            }
            async fn create_record(&self, _: &str, _: Untainted<Record>) -> Result<Record> {
                unimplemented!()
            }
            async fn update_record(
                &self,
                _: &str,
                _: RecordId,
                _: Untainted<Record>,
                _: Option<u64>,
            ) -> Result<Record> {
                unimplemented!()
            }
            async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> Result<()> {
                Ok(())
            }
        }

        let backend = TwoIssues;
        let cutoff = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
        let got = backend.list_changed_since("demo", cutoff).await.unwrap();
        assert_eq!(got, vec![RecordId(2)]);
    }
}
