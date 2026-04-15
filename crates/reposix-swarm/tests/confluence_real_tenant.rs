//! Real-tenant smoke test for the confluence-direct swarm workload.
//!
//! Gated behind `#[ignore]` — only runs under
//! `cargo test -p reposix-swarm -- --ignored`. Also skips silently
//! (success) if any of the three Atlassian env vars is absent, so
//! running `--ignored` in CI without creds doesn't spuriously fail.
//!
//! Per Phase 17 locked decision: 3 clients × 10s, NOT 50 × 30s.
//! Read-only workload — no writes issued against the real tenant.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use std::time::Duration;

use reposix_confluence::ConfluenceCreds;
use reposix_swarm::confluence_direct::ConfluenceDirectWorkload;
use reposix_swarm::driver::{run_swarm, SwarmConfig};

fn env_or_skip(var: &str) -> Option<String> {
    match std::env::var(var) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires real Atlassian credentials; run with --ignored"]
async fn live_confluence_direct_smoke() {
    let Some(email) = env_or_skip("ATLASSIAN_EMAIL") else {
        eprintln!("skip: ATLASSIAN_EMAIL not set");
        return;
    };
    let Some(token) = env_or_skip("ATLASSIAN_API_KEY") else {
        eprintln!("skip: ATLASSIAN_API_KEY not set");
        return;
    };
    let Some(tenant) = env_or_skip("REPOSIX_CONFLUENCE_TENANT") else {
        eprintln!("skip: REPOSIX_CONFLUENCE_TENANT not set");
        return;
    };

    // Caller is responsible for setting REPOSIX_ALLOWED_ORIGINS to
    // include https://{tenant}.atlassian.net — this test does NOT set
    // it for the user (SG-01 fail-closed).

    let base = format!("https://{tenant}.atlassian.net");
    let creds = ConfluenceCreds {
        email,
        api_token: token,
    };
    // Space key for the smoke test. REPOSIX_CONFLUENCE_SPACE overrides
    // if the caller wants a different space; default "REPOSIX" matches
    // the project's seed space.
    let space = std::env::var("REPOSIX_CONFLUENCE_SPACE")
        .unwrap_or_else(|_| "REPOSIX".to_string());

    let cfg = SwarmConfig {
        clients: 3,
        duration: Duration::from_secs(10),
        mode: "confluence-direct",
        target: &base,
    };
    let markdown = run_swarm(cfg, |i| {
        ConfluenceDirectWorkload::new(
            base.clone(),
            creds.clone(),
            space.clone(),
            u64::try_from(i).unwrap_or(0),
        )
    })
    .await
    .expect("run_swarm returned cleanly");

    assert!(
        markdown.contains("Clients: 3"),
        "summary missing client count:\n{markdown}"
    );

    // No Other-class errors allowed even against the real tenant —
    // Conflict/RateLimited/NotFound are tolerated (rate limits expected).
    if let Some(err_section) = markdown.split("### Errors by class").nth(1) {
        assert!(
            !err_section.contains("| Other"),
            "real-tenant run produced Other-class errors:\n{markdown}"
        );
    }
}
