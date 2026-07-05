//! Behavioral no-retry integration test (P92 SC5, RBF-B-04, DVCS-BUS-WRITE-04).
//!
//! Catalog row `agent-ux/bus-write-no-helper-retry` (Q3.6 RATIFIED: no
//! helper-side retry; the user retries the whole push). Prior state was a
//! SOURCE-GREP over `bus_handler.rs` (absence of retry-looking tokens) —
//! which cannot distinguish "no retry construct was ever written" from "a
//! retry construct exists but doesn't match this exact regex". This test
//! replaces that with a BEHAVIORAL assertion: fault-inject a mirror-push
//! failure (the same `common::make_failing_mirror_fixture` failing
//! `update` hook `bus_write_mirror_fail.rs` already uses) and assert the
//! helper's OWN process made EXACTLY ONE `git push` attempt against the
//! mirror remote — counted via the hook's own invocation log
//! (`common::count_mirror_hook_invocations`), not a grep over source
//! tokens.
//!
//! Donor patterns:
//! - `tests/bus_write_mirror_fail.rs` — SoT-ok/mirror-fail scaffolding
//!   (helper driver, failing-mirror wtree builder, multi-file export).
//! - `tests/common.rs::make_failing_mirror_fixture` +
//!   `count_mirror_hook_invocations` — the counted fault-injection fixture.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unnecessary_debug_formatting)]

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod test_impl {
    use std::fmt::Write as _;
    use std::io::Write;
    use std::path::Path;
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
        count_mirror_hook_invocations, make_failing_mirror_fixture, sample_issues, seed_mock,
        sim_backend, CacheDirGuard,
    };

    struct HasSinceQueryParam;
    impl Match for HasSinceQueryParam {
        fn matches(&self, req: &Request) -> bool {
            req.url.query_pairs().any(|(k, _)| k == "since")
        }
    }

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

    fn single_file_export(path: &str, blob: &str, msg: &str) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        writeln!(&mut out, "blob").unwrap();
        writeln!(&mut out, "mark :100").unwrap();
        writeln!(&mut out, "data {}", blob.len()).unwrap();
        out.extend_from_slice(blob.as_bytes());
        out.push(b'\n');
        writeln!(&mut out, "commit refs/heads/main").unwrap();
        writeln!(&mut out, "mark :1").unwrap();
        writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
        let bytes = msg.as_bytes();
        writeln!(&mut out, "data {}", bytes.len()).unwrap();
        out.extend_from_slice(bytes);
        out.push(b'\n');
        writeln!(&mut out, "M 100644 :100 {path}").unwrap();
        writeln!(&mut out, "done").unwrap();
        out
    }

    /// Build a working-tree configured against an EMPTY-but-FAILING bare
    /// mirror. Mirrors `bus_write_mirror_fail.rs::make_failing_mirror_wtree`.
    fn make_failing_mirror_wtree() -> (tempfile::TempDir, tempfile::TempDir, String) {
        let (mirror, mirror_url) = make_failing_mirror_fixture();
        let wtree = tempfile::tempdir().expect("wtree tempdir");

        run_git_in(wtree.path(), &["init", "."]);
        run_git_in(wtree.path(), &["config", "user.email", "p92-05@example"]);
        run_git_in(wtree.path(), &["config", "user.name", "P92 SC5 Test"]);
        run_git_in(wtree.path(), &["checkout", "-b", "main"]);
        std::fs::write(wtree.path().join("seed.txt"), "seed").unwrap();
        run_git_in(wtree.path(), &["add", "seed.txt"]);
        run_git_in(wtree.path(), &["commit", "-m", "seed"]);
        run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);

        (wtree, mirror, mirror_url)
    }

    pub(super) async fn run_no_retry() {
        let server = MockServer::start().await;
        let project = "demo";
        let issues = sample_issues(project, 1);

        seed_mock(&server, project, &issues).await;

        let cache_root = tempfile::tempdir().expect("cache_root");
        let _env = CacheDirGuard::new(cache_root.path());
        let backend: Arc<dyn BackendConnector> = sim_backend(&server);
        let cache = Cache::open(backend, "sim", project).expect("Cache::open");
        cache.sync().await.expect("seed sync (warm cache cursor)");
        drop(cache);

        Mock::given(method("GET"))
            .and(path_regex(format!(r"^/projects/{project}/issues$")))
            .and(HasSinceQueryParam)
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .with_priority(1)
            .mount(&server)
            .await;

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

        let (wtree, mirror_bare, mirror_url) = make_failing_mirror_wtree();
        let bus_url = format!(
            "reposix::{}/projects/{project}?mirror={}",
            server.uri(),
            mirror_url
        );

        let blob1 = render_issue_blob(1, 1, "edited body for 1\n");
        let stream = single_file_export("issues/1.md", &blob1, "edit issue 1\n");
        let stdin_data = {
            let mut buf = Vec::new();
            writeln!(&mut buf, "capabilities").unwrap();
            writeln!(&mut buf).unwrap();
            writeln!(&mut buf, "export").unwrap();
            buf.extend_from_slice(&stream);
            buf
        };

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

        // Sanity: the SoT side must still have succeeded per Q3.6 (helper
        // exits 0, SoT-success-mirror-fail is a partial-success ack, not a
        // hard failure) -- otherwise the "0 invocations" case below could
        // be masking an early bail rather than a genuine no-retry proof.
        assert!(
            out.status.success(),
            "helper must exit zero on SoT-success+mirror-fail per Q3.6; \
             stdout={stdout}, stderr={stderr}"
        );
        assert!(
            stderr.contains("SoT push succeeded; mirror push failed"),
            "expected stderr WARN naming SoT-success-mirror-fail; got: {stderr}"
        );

        // BEHAVIORAL ASSERTION (P92 SC5): the failing `update` hook logs
        // one line EVERY time git actually attempts to update the
        // mirror's `main` ref -- i.e. every time `push_mirror`
        // (bus_handler.rs) shells out `git push <mirror> main`. Exactly 1
        // proves the helper made ONE attempt and did NOT retry after the
        // rejection; 0 would mean the hook never even ran (a different,
        // also-wrong failure mode -- push_mirror never invoked git at
        // all); >1 would mean a retry construct fired.
        let invocations = count_mirror_hook_invocations(mirror_bare.path());
        assert_eq!(
            invocations, 1,
            "expected the mirror's update hook to run EXACTLY ONCE (one git-push \
             attempt, no helper-side retry per Q3.6 RATIFIED); got {invocations} \
             invocations. 0 would mean push_mirror never ran git at all; >1 would \
             mean a retry construct fired."
        );

        let _ = (wtree, &mirror_bare);
    }
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn bus_write_no_helper_retry_makes_exactly_one_push_attempt() {
    test_impl::run_no_retry().await;
}

// On non-Unix targets the test compiles to a no-op stub (D-04 RATIFIED:
// failing-update-hook + chmod 0o755 is POSIX-specific; reposix CI is
// Linux-only).
#[cfg(not(unix))]
#[test]
fn bus_write_no_helper_retry_makes_exactly_one_push_attempt() {
    eprintln!("skipped on non-unix per D-04");
}
