---
phase: 11-confluence-adapter
plan: C
type: execute
wave: 2
depends_on:
  - A
files_modified:
  - crates/reposix-confluence/tests/contract.rs
  - crates/reposix-confluence/Cargo.toml
autonomous: true
requirements:
  - FC-01
  - FC-03
  - SG-01
user_setup: []

must_haves:
  truths:
    - "`cargo test -p reposix-confluence` runs the contract test against SimBackend AND a wiremock-backed ConfluenceReadOnlyBackend, both pass"
    - "`cargo test -p reposix-confluence -- --ignored` runs the live-Atlassian half; skips cleanly (returns early, test passes) if any of ATLASSIAN_API_KEY / ATLASSIAN_EMAIL / REPOSIX_CONFLUENCE_TENANT / REPOSIX_CONFLUENCE_SPACE are unset"
    - "The same 5 IssueBackend invariants are exercised against all three backends via a shared `assert_contract` helper (copied from reposix-github/tests/contract.rs)"
    - "Compiles even when env vars are unset — no compile-time conditional test gating"
  artifacts:
    - path: "crates/reposix-confluence/tests/contract.rs"
      provides: "`assert_contract` helper; `contract_sim`, `contract_confluence_wiremock` (always run), `contract_confluence_live` (`#[ignore]`-gated), `skip_if_no_env!` macro"
      min_lines: 200
  key_links:
    - from: "crates/reposix-confluence/tests/contract.rs `contract_confluence_wiremock`"
      to: "`reposix_confluence::ConfluenceReadOnlyBackend::new_with_base_url`"
      via: "mock server backing the 3 endpoints (spaces by key, list pages, get page)"
      pattern: "ConfluenceReadOnlyBackend::new_with_base_url"
    - from: ".github/workflows/ci.yml `integration-contract-confluence`"
      to: "`contract_confluence_live`"
      via: "`cargo test -p reposix-confluence -- --ignored`"
      pattern: "contract_confluence_live"
---

<objective>
Port `crates/reposix-github/tests/contract.rs` to `crates/reposix-confluence/tests/contract.rs`, parameterizing the same five IssueBackend invariants over: (1) `SimBackend` (always runs, no env needed); (2) wiremock-backed `ConfluenceReadOnlyBackend` (always runs, no env needed); (3) live `ConfluenceReadOnlyBackend` (`#[ignore]`-gated + skips if env unset). Introduce a `skip_if_no_env!` helper macro so the live test's env-var check is readable and consistent.

Purpose: The IssueBackend trait's value is that it's contract-testable. 11-C is the proof that the Phase 11 adapter upholds the contract — without this file, we're shipping a library with no guarantee that it behaves like a backend. Runs in Wave 2 because it consumes `ConfluenceReadOnlyBackend` (needs 11-A fully landed, not just scaffolded).

Output: One new test file at `crates/reposix-confluence/tests/contract.rs`. Minor `[dev-dependencies]` additions if needed.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/11-confluence-adapter/11-CONTEXT.md
@.planning/phases/11-confluence-adapter/11-RESEARCH.md
@CLAUDE.md
@crates/reposix-github/tests/contract.rs
@crates/reposix-confluence/src/lib.rs
@crates/reposix-confluence/Cargo.toml

<interfaces>
From `reposix-confluence/src/lib.rs` (produced by 11-A):
```rust
pub struct ConfluenceCreds { pub email: String, pub api_token: String }
pub struct ConfluenceReadOnlyBackend { /* ... */ }
impl ConfluenceReadOnlyBackend {
    pub fn new(creds: ConfluenceCreds, tenant: &str) -> Result<Self>;
    pub fn new_with_base_url(creds: ConfluenceCreds, base_url: String) -> Result<Self>;
}
```

From `reposix-github/tests/contract.rs` — COPY THIS STRUCTURE:
- `async fn assert_contract<B: IssueBackend>(backend: &B, project: &str, known_issue_id: IssueId)` with the 5 invariants.
- `async fn spawn_sim() -> (String, tempfile::NamedTempFile, tokio::task::JoinHandle<()>)` — re-use verbatim.
- `fn sim_seed_fixture() -> PathBuf` — adjust path by one extra `..` because we're deeper. Actually no — `env!("CARGO_MANIFEST_DIR")` points to `crates/reposix-confluence/`, so `../reposix-sim/fixtures/seed.json` is identical. Copy verbatim.
- `#[tokio::test(flavor = "multi_thread", worker_threads = 2)] async fn contract_sim()` — unchanged.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Port contract-test file with sim + wiremock-confluence test arms</name>
  <files>
    crates/reposix-confluence/tests/contract.rs,
    crates/reposix-confluence/Cargo.toml
  </files>
  <action>
    Create `crates/reposix-confluence/tests/contract.rs`. Structure:

    ```rust
    //! Contract test — the same 5 invariants hold for SimBackend,
    //! a wiremock-backed ConfluenceReadOnlyBackend, and (when
    //! `#[ignore]`-unlocked + env configured) a live Atlassian tenant.
    //!
    //! Mirrors `crates/reposix-github/tests/contract.rs` exactly in spirit —
    //! the assert_contract helper is identical, only the fixture-plumbing
    //! differs (wiremock mounts for confluence, real sim for sim).

    use std::path::PathBuf;

    use reposix_confluence::{ConfluenceCreds, ConfluenceReadOnlyBackend};
    use reposix_core::backend::sim::SimBackend;
    use reposix_core::backend::IssueBackend;
    use reposix_core::{IssueId, IssueStatus};
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Skip the test if any listed env var is unset or empty. Prints a SKIP
    /// line to stderr (visible in `cargo test -- --nocapture`) and returns
    /// from the outer fn. Use only in `#[ignore]`-gated live-wire tests.
    macro_rules! skip_if_no_env {
        ($($var:literal),+ $(,)?) => {{
            let mut missing: Vec<&'static str> = Vec::new();
            $(
                if std::env::var($var).map_or(true, |v| v.is_empty()) {
                    missing.push($var);
                }
            )+
            if !missing.is_empty() {
                eprintln!("SKIP: env vars unset: {}", missing.join(", "));
                return;
            }
        }};
    }

    async fn assert_contract<B: IssueBackend>(backend: &B, project: &str, known_issue_id: IssueId) {
        // … copied VERBATIM from reposix-github/tests/contract.rs …
    }

    // ------------------------------------------------ sim fixture + spawn_sim
    // … copied VERBATIM, including sim_seed_fixture (path still works because
    //   CARGO_MANIFEST_DIR is a sibling of reposix-sim, not a descendant).

    // ------------------------------------------------ SimBackend test
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn contract_sim() {
        let (origin, _db, handle) = spawn_sim().await;
        let backend = SimBackend::new(origin).expect("SimBackend::new");
        assert_contract(&backend, "demo", IssueId(1)).await;
        handle.abort();
    }

    // ------------------------------------------------ wiremock-Confluence test
    /// Always runs. Mounts the three endpoints the contract will hit
    /// (resolve_space_id → list pages → get single page → 404 on u64::MAX)
    /// and drives assert_contract through them. This proves the adapter
    /// implements the contract correctly against a synthetic upstream, which
    /// is stronger than unit tests in lib.rs because it exercises the full
    /// `list_issues → get_issue → get_issue(u64::MAX)` sequence through the
    /// trait seam, not through private helpers.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn contract_confluence_wiremock() {
        let server = MockServer::start().await;

        // 1. resolve space key "REPOSIX" → space id "12345"
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .and(query_param("keys", "REPOSIX"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{"id": "12345", "key": "REPOSIX"}]
            })))
            .mount(&server)
            .await;

        // 2. list pages (single-page response, no _links.next)
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/12345/pages"))
            .and(query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    {
                        "id": "1",
                        "status": "current",
                        "title": "Home",
                        "createdAt": "2024-01-15T10:30:00.000Z",
                        "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
                        "ownerId": null,
                        "body": {}
                    },
                    {
                        "id": "2",
                        "status": "archived",
                        "title": "Old Page",
                        "createdAt": "2024-01-15T10:30:00.000Z",
                        "version": {"number": 3, "createdAt": "2024-02-20T14:00:00.000Z"},
                        "ownerId": "557058:abc",
                        "body": {}
                    }
                ]
            })))
            .mount(&server)
            .await;

        // 3. get_issue(IssueId(1)) — single page with storage body
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/1"))
            .and(query_param("body-format", "storage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "1",
                "status": "current",
                "title": "Home",
                "createdAt": "2024-01-15T10:30:00.000Z",
                "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
                "ownerId": null,
                "body": {"storage": {"value": "<p>home</p>", "representation": "storage"}}
            })))
            .mount(&server)
            .await;

        // 4. get_issue(IssueId(u64::MAX)) — 404
        // We match by path only; wiremock resolves most-specific-first so the
        // id=1 mount above still wins for page 1.
        Mock::given(method("GET"))
            .and(path(format!("/wiki/api/v2/pages/{}", u64::MAX)))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({"statusCode":404,"message":"Not found"})))
            .mount(&server)
            .await;

        let creds = ConfluenceCreds {
            email: "ci@example.com".into(),
            api_token: "dummy".into(),
        };
        let backend = ConfluenceReadOnlyBackend::new_with_base_url(creds, server.uri())
            .expect("backend");

        assert_contract(&backend, "REPOSIX", IssueId(1)).await;
    }

    // ------------------------------------------------ live-Atlassian test
    /// Hits a real Atlassian tenant. `#[ignore]`-gated + `skip_if_no_env!`-
    /// guarded so a fresh clone's CI can be green without any secrets set.
    ///
    /// Required env vars:
    /// - `ATLASSIAN_API_KEY` — API token from id.atlassian.com
    /// - `ATLASSIAN_EMAIL`   — account email that issued the token
    /// - `REPOSIX_CONFLUENCE_TENANT` — your `<tenant>.atlassian.net` subdomain
    /// - `REPOSIX_CONFLUENCE_SPACE`  — a space key that exists in the tenant
    /// - `REPOSIX_ALLOWED_ORIGINS`   — must contain `https://<tenant>.atlassian.net`
    ///
    /// The test passes trivially (SKIP) if any of the first four are missing.
    /// The allowlist one is HttpClient-enforced and surfaces as a real test
    /// failure if mis-set — that's the correct behavior because it means the
    /// invoker EXPECTED a live run (they unlocked `--ignored`) but the env is
    /// misconfigured.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    async fn contract_confluence_live() {
        skip_if_no_env!(
            "ATLASSIAN_API_KEY",
            "ATLASSIAN_EMAIL",
            "REPOSIX_CONFLUENCE_TENANT",
            "REPOSIX_CONFLUENCE_SPACE",
        );

        let origins = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
        let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap();
        let expected = format!("{tenant}.atlassian.net");
        assert!(
            origins.contains(&expected),
            "contract_confluence_live requires REPOSIX_ALLOWED_ORIGINS to include \
             https://{expected}; got {origins:?}"
        );

        let creds = ConfluenceCreds {
            email: std::env::var("ATLASSIAN_EMAIL").unwrap(),
            api_token: std::env::var("ATLASSIAN_API_KEY").unwrap(),
        };
        let space = std::env::var("REPOSIX_CONFLUENCE_SPACE").unwrap();
        let backend = ConfluenceReadOnlyBackend::new(creds, &tenant).expect("backend");

        // list_issues must succeed and return ≥1 page. To pick a known id we
        // take the first element of that list — we cannot hardcode an id
        // because real Confluence spaces don't have a canonical "issue 1"
        // like octocat/Hello-World does.
        let issues = backend.list_issues(&space).await
            .unwrap_or_else(|e| panic!("list_issues({space}) failed: {e:?}"));
        assert!(!issues.is_empty(), "live Confluence space {space} has zero pages");

        let known_id = issues[0].id;
        assert_contract(&backend, &space, known_id).await;
    }
    ```

    Verify `Cargo.toml` dev-deps already cover this (from 11-A): `wiremock`, `tokio` with `macros + rt-multi-thread`, `reposix-sim`, `tempfile`, `rusqlite`, plus `serde_json`. If `serde_json` is not in dev-deps, add it (it's already a regular dep from 11-A, so `cargo-features` just works).

    Run `cargo test -p reposix-confluence --locked` locally — expect both `contract_sim` and `contract_confluence_wiremock` to pass, `contract_confluence_live` to not run (ignored). Then run `cargo test -p reposix-confluence -- --ignored` with NO env vars set — expect `contract_confluence_live` to print `SKIP: env vars unset: ...` and pass (empty assertion after early return is fine; `#[tokio::test]` returning from the body is a success).
  </action>
  <verify>
    <automated>cargo test -p reposix-confluence --locked 2>&amp;1 | grep -E 'test (tests::)?contract_(sim|confluence_wiremock) \.\.\. ok' | wc -l | grep -q '^2$' &amp;&amp; env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE cargo test -p reposix-confluence --locked -- --ignored --nocapture 2>&amp;1 | grep -q 'SKIP: env vars unset'</automated>
  </verify>
  <done>
    Contract test file exists. Both always-on tests pass. Live test compiles and skips cleanly when env vars unset. Commit: `test(11-C-1): contract test parameterized over sim + wiremock-confluence + live-confluence`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Verify full workspace stays green</name>
  <files>
    (validation only)
  </files>
  <action>
    Gate check after the contract test lands:
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    bash scripts/demos/smoke.sh
    ```
    Expected test count: baseline + new contract tests (2 always-on = `contract_sim` + `contract_confluence_wiremock`). The `--ignored` test doesn't count in default runs.
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked &amp;&amp; bash scripts/demos/smoke.sh</automated>
  </verify>
  <done>
    Full workspace green. Smoke demos 4/4 green. No new commits unless `cargo fmt` caught something.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| test runner → env vars | Contract test reads three authenticators at runtime; must never persist them into test output or the repository. |
| live tenant → test assertions | A compromised tenant could serve crafted payloads; the test must not deserialize them into anything side-effectful. |
| wiremock server → test process | Loopback only by construction (`MockServer::start` binds 127.0.0.1 ephemeral). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11C-01 | Information disclosure | test output leaking `ATLASSIAN_API_KEY` | mitigate | `skip_if_no_env!` prints variable NAMES only, never VALUES. `assert_contract` panics with `{error:?}` on failure — confirm via code review that `Error::Other(msg)` Debug does not include request headers. reposix-confluence's manual Debug redact (T-11-01) covers the backend struct; `Error` comes from reposix-core and is a plain String, which carries URL but not Authorization header. |
| T-11C-02 | Tampering | live tenant returning crafted JSON to trigger parse panics | mitigate | `serde_json::from_slice::<ConfPage>(...)` returns `Err(...)` on unexpected shapes, caught by the `?` operator in list/get — surfaces as `assert_contract` panic with a message, which is a loud test failure, not an RCE. |
| T-11C-03 | DoS | live test against a malicious/slow tenant blocking CI | mitigate | HttpClient's 5-second timeout (SG-07) + CI job's `timeout-minutes: 5` bound the test runtime. The live test only ever hits the tenant the invoker has explicitly configured, so a malicious tenant is a self-inflicted problem. |
| T-11C-04 | Repudiation | no audit row for test HTTP | accept | See T-11-05 in 11-A: read-only audit coverage is v0.4 scope. |

Block-on-high: none; all mitigations are mechanical and apparent from code review.
</threat_model>

<verification>
Nyquist coverage:
- **Contract (sim):** `contract_sim` — proves the 5 invariants hold for SimBackend end-to-end. Always runs.
- **Contract (wiremock-confluence):** `contract_confluence_wiremock` — proves the adapter satisfies the contract against a synthetic upstream. Always runs.
- **Contract (live-confluence):** `contract_confluence_live` — proves the adapter satisfies the contract against real Atlassian. `#[ignore]`-gated + env-gated; runs only in the CI `integration-contract-confluence` job (11-B Task 3) and in developer on-demand invocations.
- **Compile-time:** file compiles even with zero env vars set.
- **Skip behavior:** `SKIP: env vars unset` line appears in output when env incomplete. Demonstrated in Task 1's verify command.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -f crates/reposix-confluence/tests/contract.rs` returns 0.
2. `grep -q 'async fn assert_contract' crates/reposix-confluence/tests/contract.rs` returns 0.
3. `grep -q 'async fn contract_sim' crates/reposix-confluence/tests/contract.rs` returns 0.
4. `grep -q 'async fn contract_confluence_wiremock' crates/reposix-confluence/tests/contract.rs` returns 0.
5. `grep -q 'async fn contract_confluence_live' crates/reposix-confluence/tests/contract.rs` returns 0.
6. `grep -q 'macro_rules! skip_if_no_env' crates/reposix-confluence/tests/contract.rs` returns 0.
7. `grep -q '#\[ignore\]' crates/reposix-confluence/tests/contract.rs` returns 0.
8. `cargo test -p reposix-confluence --locked 2>&1 | grep -E 'contract_(sim|confluence_wiremock) \.\.\. ok' | wc -l` returns 2.
9. `env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE cargo test -p reposix-confluence --locked -- --ignored --nocapture 2>&1 | grep -q 'SKIP: env vars unset'` returns 0.
10. `cargo test --workspace --locked` exits 0.
11. `bash scripts/demos/smoke.sh` exits 0.
12. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
</success_criteria>

<rollback_plan>
If `contract_confluence_wiremock` is flaky due to mock-ordering issues:
1. Ensure the `pages/{u64::MAX}` mock is mounted BEFORE the `pages/{id}` catch-all (wiremock matches most-recently-mounted-first).
2. If needed, make the id=1 mock more specific with an explicit `path("/wiki/api/v2/pages/1")` (exact) vs a regex.

If the live-wire test panics in CI when secrets ARE set (suggesting the adapter is actually broken against real Atlassian):
1. That's a genuine bug — route to 11-A for a fix, not rollback here.
2. The `contract_confluence_live` failing is the EXPECTED diagnostic surface — do not `#[ignore]`-the-test-to-hide-it (violates CLAUDE.md invariant #9).

If `cargo fmt` objects to the macro:
1. `rustfmt` sometimes mis-formats macro arms. Add `#[rustfmt::skip]` above the `macro_rules!` if necessary.
</rollback_plan>

<output>
After completion, create `.planning/phases/11-confluence-adapter/11-C-SUMMARY.md` with:
- Final test count delta (before / after).
- Confirmation of the two always-on tests passing.
- Confirmation the live test skips cleanly.
- Note any choice about macro placement or mock ordering worth preserving.
</output>
