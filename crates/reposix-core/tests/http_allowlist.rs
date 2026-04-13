//! Integration tests for `reposix_core::http`.
//!
//! Covers:
//!
//! - ROADMAP phase-1 SC #1 (named tests): `egress_to_non_allowlisted_host_is_rejected`.
//! - ROADMAP phase-1 SC #4 (env-override): `allowlist_default_and_env`.
//! - Plan-checker FIX 2: `redirect_target_is_rechecked_against_allowlist`.
//! - Origin classes (loopback default-allow, non-loopback default-deny, env override).
//! - Redirect refusal + timeout contract.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use reposix_core::http::ClientOpts;
use reposix_core::http::{client, ALLOWLIST_ENV_VAR};
use reposix_core::Error;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

// Serialize env-var-touching tests — REPOSIX_ALLOWED_ORIGINS is a process-global.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Scope guard: sets the env var for the life of the guard, restores on drop.
struct EnvGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
    prev: Option<String>,
}

impl EnvGuard {
    fn set(value: &str) -> Self {
        let guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var(ALLOWLIST_ENV_VAR).ok();
        // SAFETY: we are holding the mutex, so no other test mutates the env concurrently.
        std::env::set_var(ALLOWLIST_ENV_VAR, value);
        Self {
            _guard: guard,
            prev,
        }
    }

    fn unset() -> Self {
        let guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var(ALLOWLIST_ENV_VAR).ok();
        std::env::remove_var(ALLOWLIST_ENV_VAR);
        Self {
            _guard: guard,
            prev,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(v) => std::env::set_var(ALLOWLIST_ENV_VAR, v),
            None => std::env::remove_var(ALLOWLIST_ENV_VAR),
        }
    }
}

#[tokio::test]
async fn egress_to_non_allowlisted_host_is_rejected() {
    let _g = EnvGuard::unset();
    let c = client(ClientOpts::default()).expect("client builds");
    let t0 = Instant::now();
    let err = c
        .request(reqwest::Method::GET, "https://evil.example/")
        .await
        .expect_err("non-allowlisted host must be rejected");
    let elapsed = t0.elapsed();
    assert!(matches!(err, Error::InvalidOrigin(_)), "got: {err:?}");
    // No DNS, no TCP — must short-circuit before I/O.
    assert!(
        elapsed < Duration::from_millis(500),
        "recheck must short-circuit; took {elapsed:?}"
    );
}

#[tokio::test]
async fn allowlist_default_and_env() {
    // Under default (env unset), loopback is allowed in-principle (we don't
    // actually connect — we assert the allowlist check passes).
    {
        let _g = EnvGuard::unset();
        let c = client(ClientOpts::default()).expect("client builds");
        // 127.0.0.1:0 — allowlisted, but no listener. Error should be a transport
        // error (Error::Http), NOT InvalidOrigin.
        let err = c
            .request(reqwest::Method::GET, "http://127.0.0.1:0/")
            .await
            .expect_err("no listener so connection must fail");
        assert!(
            !matches!(err, Error::InvalidOrigin(_)),
            "loopback must NOT be origin-rejected under default allowlist; got {err:?}"
        );
    }
    // Under env override, loopback is rejected and other.example would be allowed.
    {
        let _g = EnvGuard::set("http://other.example:8080");
        let c = client(ClientOpts::default()).expect("client builds");
        let err = c
            .request(reqwest::Method::GET, "http://127.0.0.1:7878/")
            .await
            .expect_err("loopback must be rejected under narrow env allowlist");
        assert!(
            matches!(err, Error::InvalidOrigin(_)),
            "expected InvalidOrigin, got {err:?}"
        );
    }
}

#[tokio::test]
async fn loopback_is_allowed_by_default() {
    let _g = EnvGuard::unset();
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let c = client(ClientOpts::default()).expect("client builds");
    let resp = c
        .request(reqwest::Method::GET, &server.uri())
        .await
        .expect("loopback under default allowlist must succeed");
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn non_loopback_is_denied_by_default() {
    let _g = EnvGuard::unset();
    let c = client(ClientOpts::default()).expect("client builds");
    // 93.184.216.34 is example.com's long-standing IP; using the literal
    // avoids DNS resolution entirely.
    let err = c
        .request(reqwest::Method::GET, "http://93.184.216.34/")
        .await
        .expect_err("non-loopback IP must be rejected by default");
    assert!(matches!(err, Error::InvalidOrigin(_)), "got {err:?}");
}

#[tokio::test]
async fn env_override_redefines_allowlist() {
    let _g = EnvGuard::set("http://other.example:8080");
    let c = client(ClientOpts::default()).expect("client builds");
    let err = c
        .request(reqwest::Method::GET, "http://127.0.0.1:7878/")
        .await
        .expect_err("loopback must be rejected when env narrows the allowlist");
    assert!(matches!(err, Error::InvalidOrigin(_)), "got {err:?}");
}

#[tokio::test]
async fn http_redirects_are_not_followed() {
    let _g = EnvGuard::unset();
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(302).insert_header("Location", "https://attacker.example/"),
        )
        .mount(&server)
        .await;

    let c = client(ClientOpts::default()).expect("client builds");
    let resp = c
        .request(reqwest::Method::GET, &server.uri())
        .await
        .expect("302 must surface as-is, not cause a connect error");
    assert_eq!(resp.status().as_u16(), 302);
    let loc = resp
        .headers()
        .get("Location")
        .expect("Location header present")
        .to_str()
        .expect("ASCII");
    assert_eq!(loc, "https://attacker.example/");
}

#[tokio::test]
async fn redirect_target_is_rechecked_against_allowlist() {
    let _g = EnvGuard::unset();
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(302).insert_header("Location", "https://attacker.example/"),
        )
        .mount(&server)
        .await;

    let c = client(ClientOpts::default()).expect("client builds");

    // Step 1: hit the allowlisted loopback fixture; observe the 302.
    let resp = c
        .request(reqwest::Method::GET, &server.uri())
        .await
        .expect("loopback request succeeds");
    assert_eq!(resp.status().as_u16(), 302);
    let location = resp
        .headers()
        .get("Location")
        .expect("Location header present")
        .to_str()
        .expect("Location is ASCII")
        .to_owned();
    assert_eq!(location, "https://attacker.example/");

    // Step 2: re-feed the redirect target through HttpClient::request().
    // The per-request recheck MUST reject it BEFORE any I/O to
    // attacker.example.
    let t0 = Instant::now();
    let err = c
        .request(reqwest::Method::GET, location.as_str())
        .await
        .expect_err("redirect target must be rejected by allowlist recheck");
    let elapsed = t0.elapsed();
    assert!(
        matches!(err, Error::InvalidOrigin(_)),
        "expected InvalidOrigin, got: {err:?}"
    );
    assert!(
        elapsed < Duration::from_millis(500),
        "recheck must short-circuit before I/O; took {elapsed:?}"
    );
}

#[tokio::test]
async fn request_with_headers_rechecks_allowlist() {
    // Same fast-reject invariant as `egress_to_non_allowlisted_host_is_rejected`,
    // but exercised through the header-carrying variant. Must short-circuit
    // BEFORE any header is attached and BEFORE any I/O.
    let _g = EnvGuard::unset();
    let c = client(ClientOpts::default()).expect("client builds");
    let t0 = Instant::now();
    let err = c
        .request_with_headers(
            reqwest::Method::GET,
            "https://evil.example/",
            &[("X-Reposix-Agent", "reposix-fuse-1")],
        )
        .await
        .expect_err("non-allowlisted host must be rejected");
    let elapsed = t0.elapsed();
    assert!(matches!(err, Error::InvalidOrigin(_)), "got: {err:?}");
    assert!(
        elapsed < Duration::from_millis(500),
        "recheck must short-circuit; took {elapsed:?}"
    );
}

#[tokio::test]
async fn request_with_headers_attaches_header() {
    // When the origin IS allowlisted, the provided header pair must land on
    // the outgoing request. wiremock's header() matcher enforces this: absent
    // the expected header, the mock returns its default 404.
    let _g = EnvGuard::unset();
    let server = MockServer::start().await;
    Mock::given(any())
        .and(wiremock::matchers::header(
            "X-Reposix-Agent",
            "reposix-fuse-123",
        ))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let c = client(ClientOpts::default()).expect("client builds");
    let resp = c
        .request_with_headers(
            reqwest::Method::GET,
            server.uri(),
            &[("X-Reposix-Agent", "reposix-fuse-123")],
        )
        .await
        .expect("allowlisted request with matching header succeeds");
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
#[ignore = "sleeps ~5s to exercise the timeout; run with --ignored in CI"]
async fn request_times_out_after_5_seconds() {
    let _g = EnvGuard::unset();
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
        .mount(&server)
        .await;

    let c = client(ClientOpts::default()).expect("client builds");
    let t0 = Instant::now();
    let err = c
        .request(reqwest::Method::GET, &server.uri())
        .await
        .expect_err("request must time out");
    let elapsed = t0.elapsed();
    assert!(elapsed < Duration::from_secs(6), "took {elapsed:?}");
    match err {
        Error::Http(e) => assert!(e.is_timeout(), "expected timeout, got {e:?}"),
        other => panic!("expected Error::Http(timeout), got {other:?}"),
    }
}
