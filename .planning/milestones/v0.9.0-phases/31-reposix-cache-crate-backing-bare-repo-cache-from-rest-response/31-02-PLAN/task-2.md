← [back to index](./index.md)

# Task 2: Implement `Cache::read_blob` with Tainted return + egress-denial audit + materialize-row test

**Files:**
```
crates/reposix-cache/src/builder.rs,
crates/reposix-cache/src/cache.rs,
crates/reposix-cache/src/lib.rs,
crates/reposix-cache/tests/materialize_one.rs,
crates/reposix-cache/tests/egress_denied_logs.rs
```

**Read first:**
```
crates/reposix-cache/src/builder.rs,
crates/reposix-cache/src/cache.rs,
crates/reposix-cache/src/audit.rs,
crates/reposix-cache/src/meta.rs,
crates/reposix-core/src/taint.rs,
crates/reposix-core/src/http.rs,
crates/reposix-core/src/error.rs,
crates/reposix-sim/src/lib.rs,
.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md
```

## Behavior

- `Cache::read_blob(oid)` signature: `pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>>`.
- On success: looks up `oid_map` for `issue_id`, calls `backend.get_issue(&project, IssueId)`, renders bytes via `frontmatter::render`, persists via `gix::Repository::write_blob(&bytes)`, asserts the returned OID equals the requested OID (else `Error::OidDrift`), writes one `op='materialize'` audit row via `log_materialize`, returns `Tainted::new(bytes)`.
- On `Error::UnknownOid` (oid not in `oid_map`): returns immediately without backend call.
- On backend error whose inner variant is `reposix_core::Error::InvalidOrigin(_)`: fires `log_egress_denied` BEFORE returning `Error::Egress(origin_str)`. Detection must work even when the backend wraps the core error in its own `Error::Other(String)` — use a fallback substring check (`"InvalidOrigin" || "origin" || "allowlist"`) as a best-effort match. Comment: "per CONTEXT.md §Open Question A5 — downcast path for `InvalidOrigin` is imperfect; substring match is the pragmatic v0.9.0 fallback. Phase 33 will tighten via typed error refactor."
- Integration test `materialize_one.rs`: seeded sim with 5 issues, build_from, read_blob on the OID of issue 1, assert (a) return type is `Tainted<Vec<u8>>`, (b) rendered bytes match `frontmatter::render(&issue_1)`, (c) exactly one `.git/objects/` blob exists after, (d) exactly one row exists in `audit_events_cache` with `op='materialize'`, (e) calling `read_blob(oid_1)` a second time does NOT create a duplicate blob (gix `write_blob` is content-addressed and idempotent), though it DOES create a second audit row.
- Integration test `egress_denied_logs.rs`: set `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:65535` (a port we know the sim is not on), create a `GithubBackend` (or a stub backend) whose base URL is outside that allowlist, call `Cache::read_blob` on a known OID, assert (a) return is `Err(Error::Egress(_))`, (b) `audit_events_cache` has exactly one row with `op='egress_denied'` populated. (If `GithubBackend` construction is too heavy, use the `SimBackend` pointed at a port not in the allowlist — the test just needs a backend whose URL triggers the allowlist gate.)

## Action

Step 1 — Extend `crates/reposix-cache/src/builder.rs`:
```rust
// Add at the top (if not already imported):
use reposix_core::issue::IssueId;
use reposix_core::taint::Tainted;

impl Cache {
    /// Materialize a blob by OID. Writes the blob object to `.git/objects/`
    /// and returns its bytes wrapped in [`Tainted`].
    ///
    /// # Errors
    /// - [`Error::UnknownOid`] — the OID has no entry in `oid_map`.
    /// - [`Error::Egress`] — the backend's origin is not in the
    ///   `REPOSIX_ALLOWED_ORIGINS` allowlist (audit row fired first).
    /// - [`Error::Backend`] — any other backend failure.
    /// - [`Error::OidDrift`] — backend returned bytes that hash to a
    ///   different OID than requested (eventual-consistency race).
    /// - [`Error::Render`] — frontmatter rendering failed.
    /// - [`Error::Git`] — gix write_blob failed.
    pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>> {
        let oid_hex = oid.to_hex().to_string();

        // Look up issue_id without holding the lock across the await.
        let issue_id_str = {
            let db = self.db.lock().expect("cache db mutex poisoned");
            crate::meta::get_issue_for_oid(&db, &oid_hex)?
                .ok_or_else(|| Error::UnknownOid(oid_hex.clone()))?
        };

        // Parse back to IssueId (simulator stores numeric).
        let issue_num: u64 = issue_id_str.parse().map_err(|_| {
            Error::Backend(format!("oid_map issue_id {issue_id_str} is not numeric"))
        })?;

        // Call backend. On InvalidOrigin, fire egress_denied audit row THEN return Egress.
        let issue_res = self.backend.get_issue(&self.project, IssueId(issue_num)).await;
        let issue = match issue_res {
            Ok(i) => i,
            Err(e) => {
                let emsg = e.to_string();
                // Pragmatic detection: core::Error::InvalidOrigin renders as
                // "invalid origin: <url>"; backend adapters may wrap in
                // Error::Other. Substring match catches both.
                let is_egress =
                    matches!(&e, reposix_core::Error::InvalidOrigin(_))
                    || emsg.contains("invalid origin")
                    || emsg.contains("allowlist");
                if is_egress {
                    let db = self.db.lock().expect("cache db mutex poisoned");
                    crate::audit::log_egress_denied(
                        &db,
                        &self.backend_name,
                        &self.project,
                        Some(&issue_id_str),
                        &emsg,
                    );
                    return Err(Error::Egress(emsg));
                }
                return Err(Error::Backend(emsg));
            }
        };

        // Render and write the blob.
        let rendered = reposix_core::issue::frontmatter::render(&issue)?;
        let bytes = rendered.into_bytes();
        let written_oid = self
            .repo
            .write_blob(&bytes)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Consistency check.
        if written_oid != oid {
            return Err(Error::OidDrift {
                requested: oid_hex,
                actual: written_oid.to_hex().to_string(),
                issue_id: issue_id_str,
            });
        }

        // Audit. Best-effort.
        {
            let db = self.db.lock().expect("cache db mutex poisoned");
            crate::audit::log_materialize(
                &db,
                &self.backend_name,
                &self.project,
                &issue_id_str,
                &oid_hex,
                bytes.len(),
            );
        }

        Ok(Tainted::new(bytes))
    }
}
```

Step 2 — Create `crates/reposix-cache/tests/materialize_one.rs`:
```rust
//! ARCH-02: one read_blob = one blob in .git/objects + one materialize audit row;
//! return type is Tainted<Vec<u8>>.

use std::sync::Arc;

use reposix_cache::Cache;
use reposix_core::issue::frontmatter;
use reposix_sim::SimBackend;
use tempfile::tempdir;

#[tokio::test]
async fn read_blob_materializes_exactly_one_and_audits() {
    let tmp = tempdir().unwrap();
    let prev = std::env::var(reposix_cache::CACHE_DIR_ENV).ok();
    std::env::set_var(reposix_cache::CACHE_DIR_ENV, tmp.path());

    let sim = Arc::new(SimBackend::new_seeded("proj-1", 5));
    let cache = Cache::open(sim.clone(), "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();

    // Pick issue 1's OID from the oid_map. We query it directly — the
    // plan does not yet expose a public helper, so read via rusqlite.
    let cache_dir_for_db = cache.repo_path();
    let db = rusqlite::Connection::open(cache_dir_for_db.join("cache.db")).unwrap();
    let oid_hex: String = db.query_row(
        "SELECT oid FROM oid_map WHERE issue_id = '1' AND backend = 'sim' AND project = 'proj-1'",
        [],
        |r| r.get(0),
    ).unwrap();
    let oid = gix::ObjectId::from_hex(oid_hex.as_bytes()).unwrap();

    // Count objects before.
    let repo = gix::open(cache.repo_path()).unwrap();
    let blob_before: usize = repo.objects.iter().unwrap().filter_map(|r| r.ok())
        .filter(|o| repo.objects.header(*o).unwrap().kind == gix::object::Kind::Blob)
        .count();
    assert_eq!(blob_before, 0, "Plan 01 invariant: no blobs before read_blob");

    // Materialize.
    let tainted = cache.read_blob(oid).await.expect("read_blob succeeds");

    // Type is Tainted<Vec<u8>>. Compile-time: the compiler already checked.
    // Runtime sanity: bytes match frontmatter::render of issue 1.
    let inner = tainted.into_inner();
    let issue_1 = sim.get_issue("proj-1", reposix_core::IssueId(1)).await.unwrap();
    let expected = frontmatter::render(&issue_1).unwrap();
    assert_eq!(inner, expected.into_bytes());

    // One blob object now exists.
    let repo = gix::open(cache.repo_path()).unwrap();
    let blob_after: usize = repo.objects.iter().unwrap().filter_map(|r| r.ok())
        .filter(|o| repo.objects.header(*o).unwrap().kind == gix::object::Kind::Blob)
        .count();
    assert_eq!(blob_after, 1, "exactly one blob after read_blob");

    // Exactly one materialize audit row.
    let db = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let mat_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'materialize'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(mat_count, 1);

    // Second read_blob on same OID: blob count stays at 1 (content-addressed);
    // audit count goes to 2.
    let _ = cache.read_blob(oid).await.unwrap();
    let mat_count2: i64 = db.query_row(
        "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'materialize'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(mat_count2, 2, "second read_blob fires a second materialize audit row");

    match prev {
        Some(v) => std::env::set_var(reposix_cache::CACHE_DIR_ENV, v),
        None => std::env::remove_var(reposix_cache::CACHE_DIR_ENV),
    }
}
```

Step 3 — Create `crates/reposix-cache/tests/egress_denied_logs.rs`:
```rust
//! ARCH-03: non-allowlisted backend origin -> Error::Egress + op=egress_denied audit row.
//!
//! We exercise the allowlist gate through `reposix_core::http::client()`.
//! The cleanest way is to call the allowlist-aware HTTP client directly
//! against a non-allowlisted origin and confirm Error::InvalidOrigin
//! surfaces — then assert the cache's read_blob path translates that
//! to Error::Egress + audit row. Since constructing a real backend
//! whose origin is outside the default localhost allowlist requires
//! either GithubBackend (heavy) or a custom mock, we use a bespoke
//! stub BackendConnector that simulates the allowlist rejection.

use std::sync::Arc;

use async_trait::async_trait;
use reposix_cache::Cache;
use reposix_core::backend::{BackendConnector, DeleteReason};
use reposix_core::issue::{Issue, IssueId};
use reposix_core::taint::Untainted;
use reposix_core::Result as CoreResult;
use reposix_core::Error as CoreError;
use reposix_sim::SimBackend;
use tempfile::tempdir;

/// Stub backend whose `get_issue` always returns `Error::InvalidOrigin`,
/// simulating the allowlist gate firing. `list_issues` delegates to an
/// inner `SimBackend` so Cache::build_from can seed oid_map.
struct EgressRejectingBackend {
    inner: Arc<SimBackend>,
}

#[async_trait]
impl BackendConnector for EgressRejectingBackend {
    async fn list_issues(&self, project: &str) -> CoreResult<Vec<Issue>> {
        self.inner.list_issues(project).await
    }
    async fn get_issue(&self, _project: &str, _id: IssueId) -> CoreResult<Issue> {
        Err(CoreError::InvalidOrigin("https://evil.example:443/".into()))
    }
    async fn create_issue(&self, _: &str, _: Untainted<Issue>) -> CoreResult<Issue> {
        Err(CoreError::Other("unsupported in stub".into()))
    }
    async fn update_issue(
        &self,
        _project: &str,
        _id: IssueId,
        _issue: Untainted<Issue>,
        _expected_version: Option<u64>,
    ) -> CoreResult<Issue> {
        Err(CoreError::Other("unsupported in stub".into()))
    }
    async fn delete_or_close(&self, _: &str, _: IssueId, _: DeleteReason) -> CoreResult<()> {
        Err(CoreError::Other("unsupported in stub".into()))
    }
}

#[tokio::test]
async fn egress_denied_writes_audit_row_and_returns_egress_error() {
    let tmp = tempdir().unwrap();
    let prev = std::env::var(reposix_cache::CACHE_DIR_ENV).ok();
    std::env::set_var(reposix_cache::CACHE_DIR_ENV, tmp.path());

    let sim = Arc::new(SimBackend::new_seeded("proj-1", 3));
    let backend = Arc::new(EgressRejectingBackend { inner: sim.clone() });
    let cache = Cache::open(backend, "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();

    // Pick any oid from the map.
    let db = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let oid_hex: String = db.query_row(
        "SELECT oid FROM oid_map LIMIT 1", [], |r| r.get(0)
    ).unwrap();
    let oid = gix::ObjectId::from_hex(oid_hex.as_bytes()).unwrap();

    // read_blob MUST return Error::Egress.
    let res = cache.read_blob(oid).await;
    match res {
        Err(reposix_cache::Error::Egress(_)) => {}
        other => panic!("expected Error::Egress, got {other:?}"),
    }

    // Audit row must exist with op='egress_denied'.
    let denied_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'egress_denied'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(denied_count, 1);

    match prev {
        Some(v) => std::env::set_var(reposix_cache::CACHE_DIR_ENV, v),
        None => std::env::remove_var(reposix_cache::CACHE_DIR_ENV),
    }
}
```

Step 4 — Verify zero `reqwest::Client` constructors in `crates/reposix-cache/src/`:
```bash
# Should return empty (no matches).
grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new|Client::)" crates/reposix-cache/src/
```

If any match is found, remove it; the cache must use only `BackendConnector` methods which route HTTP through `reposix_core::http::client()`.

Step 5 — Run:
```bash
cargo test -p reposix-cache
cargo clippy -p reposix-cache --all-targets -- -D warnings
```

## Acceptance Criteria

- `grep -qE "pub async fn read_blob\s*\(\s*&self,\s*oid: gix::ObjectId\s*\)\s*->\s*Result<Tainted<Vec<u8>>>" crates/reposix-cache/src/builder.rs` returns 0 (modulo formatting; the literal `Tainted<Vec<u8>>` substring must be present in the signature region).
- `grep -q "log_materialize" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "log_egress_denied" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "Tainted::new" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "OidDrift" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new)" crates/reposix-cache/src/` returns empty (no matches — enforced).
- `cargo test -p reposix-cache --test materialize_one` exits 0.
- `cargo test -p reposix-cache --test egress_denied_logs` exits 0.
- `cargo test -p reposix-cache` (full crate) exits 0.
- `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.

## Verify

```
cargo test -p reposix-cache && cargo clippy -p reposix-cache --all-targets -- -D warnings && ! grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new)" crates/reposix-cache/src/
```

## Done

`Cache::read_blob` materializes exactly one blob per call, audits it, and wraps the result in `Tainted<Vec<u8>>`. Egress denial is detected, audited with `op='egress_denied'`, and returned as `Error::Egress`. Zero `reqwest::Client` constructors in the new crate.
