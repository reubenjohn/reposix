//! Bus write fan-out fault-injection (a) integration test (DVCS-BUS-WRITE-06).
//!
//! RESEARCH.md § "Test (a)": mirror push fails between confluence-write
//! and ack. Helper must:
//!
//! - exit zero with `ok refs/heads/main` (Q3.6 SoT contract).
//! - write `helper_push_partial_fail_mirror_lag` audit row.
//! - advance `refs/mirrors/<sot>-head` (head moved).
//! - LEAVE `refs/mirrors/<sot>-synced-at` frozen (or absent on first push).
//! - leave mirror's `main` ref ABSENT (failing-update-hook rejects ref update).
//! - emit stderr WARN naming SoT-success-mirror-fail.
//!
//! Donor patterns:
//! - `tests/bus_write_happy.rs` — helper-driver scaffolding + multi-file
//!   export stream + cache-bare-locator.
//! - `tests/common.rs::make_failing_mirror_fixture` — bare mirror with
//!   failing `update` hook (P83-01 T05; gated `#[cfg(unix)]` per D-04).
//! - `tests/common.rs::count_audit_cache_rows` — audit-row helper.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)] // test-internal doc comments cite SoT/refs/audit ops verbatim
#![allow(clippy::too_many_lines)] // narrow integration-test setup; readability beats split fns
#![allow(clippy::unnecessary_debug_formatting)] // stderr/path Debug is intentional in test diagnostics

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod test_impl {
    use std::fmt::Write as _;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command as StdCommand;
    use std::sync::Arc;

    use assert_cmd::Command as AssertCommand;
    use chrono::TimeZone;
    use reposix_cache::Cache;
    use reposix_core::BackendConnector;
    use serde_json::json;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

    use super::common::{
        count_audit_cache_rows, make_failing_mirror_fixture, sample_issues, seed_mock, sim_backend,
        CacheDirGuard,
    };

    /// Custom matcher: matches requests that DO have a `since` query
    /// param. Mirrors `tests/bus_write_happy.rs::HasSinceQueryParam`.
    pub(super) struct HasSinceQueryParam;
    impl Match for HasSinceQueryParam {
        fn matches(&self, req: &Request) -> bool {
            req.url.query_pairs().any(|(k, _)| k == "since")
        }
    }

    /// Spawn `git` against a directory; assert success.
    fn run_git_in(dir: &Path, args: &[&str]) -> String {
        let out = StdCommand::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .unwrap_or_else(|e| panic!("spawn git {args:?}: {e}"));
        assert!(
            out.status.success(),
            "git {args:?} in {dir:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    }

    /// Render a Record's frontmatter+body form (mirrors bus_write_happy).
    fn render_issue_blob(id: u64, version: u64, body: &str) -> String {
        let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        let ts = t.to_rfc3339();
        let mut s = String::new();
        s.push_str("---\n");
        writeln!(&mut s, "id: {id}").unwrap();
        writeln!(&mut s, "title: issue {id} in demo").unwrap();
        s.push_str("status: open\n");
        writeln!(&mut s, "created_at: {ts}").unwrap();
        writeln!(&mut s, "updated_at: {ts}").unwrap();
        writeln!(&mut s, "version: {version}").unwrap();
        s.push_str("---\n");
        s.push_str(body);
        if !s.ends_with('\n') {
            s.push('\n');
        }
        s
    }

    /// Build a fast-export multi-file stream (mirrors bus_write_happy).
    fn multi_file_export(entries: &[(&str, String)], msg: &str) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        writeln!(&mut out, "feature done").unwrap();
        let base_mark: u64 = 100;
        for (i, (_, blob)) in entries.iter().enumerate() {
            writeln!(&mut out, "blob").unwrap();
            writeln!(&mut out, "mark :{}", base_mark + i as u64).unwrap();
            writeln!(&mut out, "data {}", blob.len()).unwrap();
            out.extend_from_slice(blob.as_bytes());
            out.push(b'\n');
        }
        writeln!(&mut out, "commit refs/heads/main").unwrap();
        writeln!(&mut out, "mark :1").unwrap();
        writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
        let bytes = msg.as_bytes();
        writeln!(&mut out, "data {}", bytes.len()).unwrap();
        out.extend_from_slice(bytes);
        out.push(b'\n');
        for (i, (path, _)) in entries.iter().enumerate() {
            writeln!(&mut out, "M 100644 :{} {path}", base_mark + i as u64).unwrap();
        }
        writeln!(&mut out, "done").unwrap();
        out
    }

    /// Locate the cache bare repo under `cache_dir`. Returns the bare
    /// directory path. (Mirrors bus_write_happy::find_cache_bare.)
    fn find_cache_bare(cache_dir: &Path) -> Option<PathBuf> {
        walkdir::WalkDir::new(cache_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .find(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"))
            .map(|e| e.path().to_path_buf())
    }

    /// Build a working-tree configured against an EMPTY-but-FAILING bare
    /// mirror. PRECHECK A's `git ls-remote` against an empty bare returns
    /// no output → `MirrorDriftOutcome::Stable` (bus_handler.rs:400-404).
    /// So we don't need a seeded `refs/remotes/<name>/main`; we only need
    /// the wtree's `main` branch to point at a real commit so the helper's
    /// terminal `git push <name> main` has something to push.
    ///
    /// Returns `(working_tree_dir, mirror_bare_dir, mirror_url)`.
    fn make_failing_mirror_wtree() -> (tempfile::TempDir, tempfile::TempDir, String) {
        let (mirror, mirror_url) = make_failing_mirror_fixture();
        let wtree = tempfile::tempdir().expect("wtree tempdir");

        run_git_in(wtree.path(), &["init", "."]);
        run_git_in(wtree.path(), &["config", "user.email", "p83-02@example"]);
        run_git_in(wtree.path(), &["config", "user.name", "P83-02 Test"]);
        run_git_in(wtree.path(), &["checkout", "-b", "main"]);
        std::fs::write(wtree.path().join("seed.txt"), "seed").unwrap();
        run_git_in(wtree.path(), &["add", "seed.txt"]);
        run_git_in(wtree.path(), &["commit", "-m", "seed"]);
        run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);

        (wtree, mirror, mirror_url)
    }

    pub(super) async fn run_mirror_fail() {
        let server = MockServer::start().await;
        let project = "demo";
        let issues = sample_issues(project, 3);

        // 1. Setup-phase mocks (priority 5): seed list + per-id GETs.
        seed_mock(&server, project, &issues).await;

        // 2. Per-test cache dir with warm cursor.
        let cache_root = tempfile::tempdir().expect("cache_root");
        let _env = CacheDirGuard::new(cache_root.path());
        let backend: Arc<dyn BackendConnector> = sim_backend(&server);
        let cache = Cache::open(backend, "sim", project).expect("Cache::open");
        cache.sync().await.expect("seed sync (warm cache cursor)");
        drop(cache);

        // 3. ASSERTION-PHASE mock: list_changed_since `?since=` returns []
        //    (PRECHECK B Stable).
        Mock::given(method("GET"))
            .and(path_regex(format!(r"^/projects/{project}/issues$")))
            .and(HasSinceQueryParam)
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .with_priority(1)
            .mount(&server)
            .await;

        // 4. PATCH for issue 1 returns 200 — SoT write succeeds.
        Mock::given(method("PATCH"))
            .and(path_regex(format!(r"^/projects/{project}/issues/1$")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1, "title": "issue 1 in demo", "status": "open",
                "assignee": null, "labels": [],
                "created_at": "2026-04-13T00:00:00Z",
                "updated_at": "2026-05-01T00:00:00Z",
                "version": 2, "body": "edited body for 1\n"
            })))
            .with_priority(1)
            .expect(1..)
            .mount(&server)
            .await;

        // 5. Build EMPTY bare mirror with failing update hook + wtree.
        let (wtree, mirror_bare, mirror_url) = make_failing_mirror_wtree();

        // 6. Bus URL.
        let bus_url = format!(
            "reposix::{}/projects/{project}?mirror={}",
            server.uri(),
            mirror_url
        );

        // 7. Fast-export with id=1's body changed (PATCH fires); ids 2+3
        //    unchanged → plan() skips them.
        let blob1 = render_issue_blob(1, 1, "edited body for 1\n");
        let blob2 = render_issue_blob(2, 1, "body of issue 2");
        let blob3 = render_issue_blob(3, 1, "body of issue 3");
        let entries: Vec<(&str, String)> =
            vec![("0001.md", blob1), ("0002.md", blob2), ("0003.md", blob3)];
        let stream = multi_file_export(&entries, "edit issue 1\n");
        let stdin_data = {
            let mut buf = Vec::new();
            writeln!(&mut buf, "capabilities").unwrap();
            writeln!(&mut buf).unwrap();
            writeln!(&mut buf, "export").unwrap();
            buf.extend_from_slice(&stream);
            buf
        };

        // 8. Drive the helper.
        let cache_path = cache_root.path().to_path_buf();
        let out = AssertCommand::cargo_bin("git-remote-reposix")
            .expect("cargo bin")
            .args(["origin", &bus_url])
            .write_stdin(stdin_data)
            .current_dir(wtree.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .timeout(std::time::Duration::from_secs(30))
            .output()
            .expect("run helper");

        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);

        // ASSERTION 1: helper exits zero (Q3.6 SoT contract).
        assert!(
            out.status.success(),
            "helper must exit zero on SoT-success+mirror-fail per Q3.6; \
             stdout={stdout}, stderr={stderr}"
        );

        // ASSERTION 2: stdout contains `ok refs/heads/main`.
        assert!(
            stdout.contains("ok refs/heads/main"),
            "missing `ok refs/heads/main` ack on partial-fail per Q3.6; \
             stdout={stdout}, stderr={stderr}"
        );

        // ASSERTION 3: stderr WARN names SoT-success-mirror-fail.
        assert!(
            stderr.contains("SoT push succeeded; mirror push failed"),
            "expected stderr WARN naming SoT-success-mirror-fail; got: {stderr}"
        );

        // ASSERTION 4: stderr's audit reason format includes `exit=` token
        //  (T-83-02 — operator-readable + bounded).
        assert!(
            stderr.contains("exit="),
            "expected stderr to include `exit=` token from audit reason format; got: {stderr}"
        );

        // 9. Locate the cache bare for ref + audit assertions.
        let cache_bare =
            find_cache_bare(cache_root.path()).expect("cache bare dir must exist after push");
        let db_path = cache_bare.join("cache.db");

        // ASSERTION 5: helper_push_partial_fail_mirror_lag count == 1.
        let partial = count_audit_cache_rows(&db_path, "helper_push_partial_fail_mirror_lag");
        assert_eq!(
            partial, 1,
            "expected 1 helper_push_partial_fail_mirror_lag row, got {partial}"
        );

        // ASSERTION 6: mirror_sync_written count == 0.
        let synced = count_audit_cache_rows(&db_path, "mirror_sync_written");
        assert_eq!(
            synced, 0,
            "expected 0 mirror_sync_written rows on partial-fail, got {synced}"
        );

        // ASSERTION 7: helper_push_accepted count == 1 (SoT side succeeded).
        let accepted = count_audit_cache_rows(&db_path, "helper_push_accepted");
        assert_eq!(
            accepted, 1,
            "expected 1 helper_push_accepted row, got {accepted}"
        );

        // ASSERTION 8: refs/mirrors/sim-head exists (SoT-side ref).
        let refs_out = StdCommand::new("git")
            .arg("-C")
            .arg(&cache_bare)
            .args(["for-each-ref", "refs/mirrors/"])
            .output()
            .expect("git for-each-ref");
        let refs_str = String::from_utf8_lossy(&refs_out.stdout);
        assert!(
            refs_str.contains("refs/mirrors/sim-head"),
            "missing refs/mirrors/sim-head; got: {refs_str}"
        );

        // ASSERTION 9: refs/mirrors/sim-synced-at is ABSENT (frozen — first
        //  push case; mirror never landed).
        assert!(
            !refs_str.contains("refs/mirrors/sim-synced-at"),
            "refs/mirrors/sim-synced-at must NOT be written on partial-fail; got: {refs_str}"
        );

        // ASSERTION 10: mirror's `main` ref is ABSENT (failing update hook
        //  rejected the ref update; bare repo still has no commits).
        let mirror_main = StdCommand::new("git")
            .arg("-C")
            .arg(mirror_bare.path())
            .args(["rev-parse", "--verify", "refs/heads/main"])
            .output()
            .expect("git rev-parse main on bare mirror");
        assert!(
            !mirror_main.status.success(),
            "mirror's main ref MUST be absent (failing-update-hook rejected push); \
             stdout={}, stderr={}",
            String::from_utf8_lossy(&mirror_main.stdout),
            String::from_utf8_lossy(&mirror_main.stderr)
        );

        // ASSERTION 11 (implicit): wiremock saw at least 1 PATCH call —
        // enforced by Mock::expect(1..) at mount time, checked at Drop.

        // Suppress unused warnings on tempdir handles (must outlive scope).
        let _ = (wtree, &mirror_bare);
    }
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn bus_write_mirror_fail_returns_ok_with_lag_audit_row() {
    test_impl::run_mirror_fail().await;
}

// On non-Unix targets the test compiles to a no-op stub (D-04 RATIFIED:
// failing-update-hook + chmod 0o755 is POSIX-specific; reposix CI is
// Linux-only).
#[cfg(not(unix))]
#[test]
fn bus_write_mirror_fail_returns_ok_with_lag_audit_row() {
    eprintln!("skipped on non-unix per D-04");
}
