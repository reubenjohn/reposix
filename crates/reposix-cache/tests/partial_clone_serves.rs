//! QL-001 fetch-path regression: the cache's bare repo must be configured
//! to SERVE the partial-clone protocol, and — once record blobs are
//! materialized — a real `git clone --filter=blob:none` + `git checkout`
//! must round-trip actual file content.
//!
//! Root cause (reproduced end-to-end below on git 2.25 via `file://`
//! transport, which drives the same `git upload-pack` the helper's
//! `stateless-connect` path spawns): `Cache::build_from` writes trees +
//! the commit but NO blob objects (lazy-blob invariant, see lib.rs). If
//! the cache repo does not advertise the object-filter capability
//! (`uploadpack.allowFilter`), git SILENTLY IGNORES a client's
//! `--filter=blob:none` ("warning: filtering not recognized by server,
//! ignoring") and `pack-objects` tries to pack every reachable blob —
//! including the unmaterialized ones — then dies with the misleading
//! "possible repository corruption on the remote side" (CI row
//! agent-ux/real-git-push-e2e on ubuntu-latest, run 28724087420). The
//! follow-up lazy blob fetch (`git checkout`) then requests blobs by
//! non-tip OID, which upload-pack rejects ("not our ref") unless
//! `uploadpack.allowAnySHA1InWant` is set.
//!
//! RED-proof against the pre-fix code:
//!
//! - `cache_open_sets_upload_pack_partial_clone_config` — keys unset.
//! - `filtered_clone_of_lazy_cache_succeeds` — the clone dies exactly as
//!   CI did ("filtering not recognized … possible repository corruption").
//!
//! `filtered_clone_then_checkout_serves_materialized_blobs` is a reality
//! check that the served config supports a genuine partial clone + lazy
//! by-OID checkout; it is not a standalone RED sentinel (pre-materialized
//! blobs let a filter-ignoring server fall back to a full clone).

mod common;

use common::CacheDirGuard;
use reposix_cache::Cache;
use std::process::Command;
use tempfile::tempdir;
use wiremock::MockServer;

/// Read a single git config value from `repo`; `None` if unset.
fn git_config(repo: &std::path::Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "--get", key])
        .output()
        .expect("spawn git config");
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// After `Cache::open`, the bare repo must advertise both partial-clone
/// upload-pack knobs. RED-provable: the pre-fix `Cache::open` never wrote
/// either key.
#[tokio::test]
async fn cache_open_sets_upload_pack_partial_clone_config() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());
    let server = MockServer::start().await;
    common::seed_mock(&server, "proj-1", &common::sample_issues("proj-1", 2)).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-1").unwrap();
    let repo = cache.repo_path();

    assert_eq!(
        git_config(repo, "uploadpack.allowFilter").as_deref(),
        Some("true"),
        "cache repo must set uploadpack.allowFilter=true so `git upload-pack` \
         honors --filter=blob:none instead of silently packing unmaterialized blobs",
    );
    assert_eq!(
        git_config(repo, "uploadpack.allowAnySHA1InWant").as_deref(),
        Some("true"),
        "cache repo must set uploadpack.allowAnySHA1InWant=true so the lazy \
         blob fetch (git checkout) can request blobs by non-tip OID",
    );
}

/// End-to-end RED sentinel that reproduces the CI failure directly: a
/// `git clone --filter=blob:none --no-checkout` of a freshly built (lazy,
/// NO blobs materialized) cache must succeed. Against the pre-fix code the
/// server ignores the filter and `pack-objects` dies trying to pack the
/// unmaterialized blobs — the "possible repository corruption on the remote
/// side" that failed agent-ux/real-git-push-e2e. `--no-checkout` keeps the
/// scope to the tip fetch (commit + trees), which the cache always has.
#[tokio::test]
async fn filtered_clone_of_lazy_cache_succeeds() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());
    let server = MockServer::start().await;
    common::seed_mock(&server, "proj-1", &common::sample_issues("proj-1", 3)).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();

    let dst = tmp.path().join("clone");
    let file_url = format!("file://{}", cache.repo_path().display());
    let clone = Command::new("git")
        .args([
            "clone",
            "--filter=blob:none",
            "--no-checkout",
            &file_url,
            dst.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&clone.stderr);
    assert!(
        clone.status.success(),
        "filtered clone of a lazy cache must succeed once uploadpack.allowFilter \
         is set; pre-fix this dies 'possible repository corruption'. stderr:\n{stderr}",
    );
    assert!(
        !stderr.contains("filtering not recognized"),
        "server must advertise the filter capability (allowFilter); stderr:\n{stderr}",
    );
}

/// Full read-path proof: with the config in place and record blobs
/// materialized (simulating the helper's want-path materialization), a
/// real `git clone --filter=blob:none` followed by `git checkout` brings
/// down actual file content. This exercises the SAME `git upload-pack`
/// the helper spawns — via `file://` transport so it runs on git < 2.34.
#[tokio::test]
async fn filtered_clone_then_checkout_serves_materialized_blobs() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());
    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-1", 3);
    common::seed_mock(&server, "proj-1", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();
    let cache_path = cache.repo_path().to_path_buf();

    // Materialize every record blob — this is what the helper's fetch path
    // now does on demand for each wanted OID (proven separately by the CI
    // git>=2.34 e2e row + parse_want_oid unit test in reposix-remote).
    let db = rusqlite::Connection::open(cache_path.join("cache.db")).unwrap();
    let oids: Vec<String> = db
        .prepare("SELECT oid FROM oid_map WHERE backend='sim' AND project='proj-1'")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    drop(db);
    assert_eq!(oids.len(), 3, "expected 3 record OIDs in oid_map");
    for oid_hex in &oids {
        let oid = gix::ObjectId::from_hex(oid_hex.as_bytes()).unwrap();
        cache.read_blob(oid).await.expect("materialize record blob");
    }

    // Real partial clone over file:// (upload-pack path).
    let dst = tmp.path().join("clone");
    let file_url = format!("file://{}", cache_path.display());
    let clone = Command::new("git")
        .args([
            "clone",
            "--filter=blob:none",
            &file_url,
            dst.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        clone.status.success(),
        "filtered clone must succeed (allowFilter honored); stderr:\n{}",
        String::from_utf8_lossy(&clone.stderr),
    );

    // The working tree must contain the real issue bodies — proving the
    // checkout's lazy blob fetch was served (allowAnySHA1InWant + blob
    // present in the object store).
    let issue_1 = std::fs::read_to_string(dst.join("issues/1.md"))
        .expect("issues/1.md must be checked out with real content");
    assert!(
        issue_1.contains("body of issue 1"),
        "checked-out blob must carry the backend body; got:\n{issue_1}",
    );
}
