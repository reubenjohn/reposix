---
phase: 17
plan: B
wave: 2
slug: tests-and-docs
type: execute
serial: true
depends_on: [A]
depends_on_waves: [1]
blocks_waves: []
estimated_wall_clock: 20m
executor_role: gsd-executor
autonomous: true
files_modified:
  - crates/reposix-swarm/Cargo.toml
  - crates/reposix-swarm/tests/mini_e2e.rs
  - crates/reposix-swarm/tests/confluence_real_tenant.rs
  - CHANGELOG.md
  - .planning/STATE.md
  - .planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md
requirements:
  - SWARM-02
must_haves:
  truths:
    - "A CI test proves --mode confluence-direct works end-to-end against wiremock"
    - "The real-tenant smoke test exists but is gated behind --ignored + env-var presence"
    - "Swarm summary markdown matches sim-direct format (Clients: N line, list row present)"
    - "No Other-class errors under wiremock (transport/deser bugs would show up there)"
    - "CI test does NOT assert audit_rows > 0 (read-only workload writes 0 audit rows)"
    - "Phase 17 is recorded in CHANGELOG and STATE"
  artifacts:
    - path: "crates/reposix-swarm/tests/mini_e2e.rs"
      provides: "confluence_direct_3_clients_5s wiremock test"
      contains: "confluence_direct_3_clients_5s"
    - path: "crates/reposix-swarm/tests/confluence_real_tenant.rs"
      provides: "real-tenant smoke under --ignored"
      contains: "#[ignore"
    - path: "crates/reposix-swarm/Cargo.toml"
      provides: "dev-deps: reposix-confluence + wiremock 0.6"
      contains: "wiremock"
    - path: "CHANGELOG.md"
      provides: "Phase 17 entry under v0.6.0 Unreleased"
      contains: "confluence-direct"
    - path: ".planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md"
      provides: "phase summary"
      contains: "Phase 17"
    - path: ".planning/STATE.md"
      provides: "phase 17 completion cursor"
      contains: "17"
  key_links:
    - from: "crates/reposix-swarm/tests/mini_e2e.rs"
      to: "crates/reposix-swarm/src/confluence_direct.rs"
      via: "run_swarm factory with ConfluenceDirectWorkload::new"
      pattern: "ConfluenceDirectWorkload::new"
    - from: "crates/reposix-swarm/tests/mini_e2e.rs"
      to: "wiremock MockServer"
      via: "wiremock stubs for /wiki/api/v2/spaces + /pages"
      pattern: "wiremock::MockServer"
    - from: "crates/reposix-swarm/tests/confluence_real_tenant.rs"
      to: "env vars ATLASSIAN_API_KEY / ATLASSIAN_EMAIL / REPOSIX_CONFLUENCE_TENANT"
      via: "env::var() + #[ignore] gate"
      pattern: "ATLASSIAN_API_KEY"
---

## Goal

Wave 2 closes Phase 17 by adding automated verification and release docs for
the workload built in Wave 1:

- A CI integration test using `wiremock` that runs the real
  `ConfluenceDirectWorkload` through `run_swarm` with 3 clients × 5s and
  asserts the markdown summary and error-class invariants (SWARM-02).
- A gated real-tenant smoke test under `#[ignore]` (runs only with
  `cargo test -- --ignored` and only when the three Atlassian env vars are
  present).
- Dev-dep additions to `Cargo.toml` (`reposix-confluence` path + `wiremock
  0.6`).
- `17-SUMMARY.md`, `CHANGELOG.md` entry, and STATE.md update.

**Purpose:** ship the verifiable contract for SWARM-02. Until Wave 2 is green,
Wave 1's compile-time changes aren't actually proven to run correctly.

**Output:** one new test module, one new extended test in `mini_e2e.rs`, two
new dev-deps in `Cargo.toml`, and the three release-docs artifacts.

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/CONTEXT.md
@.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-RESEARCH.md
@.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-VALIDATION.md
@.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-A-workload-and-cli-PLAN.md
@CLAUDE.md
@crates/reposix-swarm/tests/mini_e2e.rs
@crates/reposix-swarm/src/confluence_direct.rs
@crates/reposix-swarm/Cargo.toml
@crates/reposix-confluence/src/lib.rs

<interfaces>
<!-- Wiremock stub shapes required by ConfluenceBackend list_issues + get_issue. -->
<!-- Extracted from crates/reposix-confluence/src/lib.rs + tests/roundtrip.rs. -->
<!-- Risks 2/3/4 and pitfalls 1/2/3 in RESEARCH.md detail why these exact shapes. -->

1. `GET /wiki/api/v2/spaces?keys=TESTSPACE` →
   `{"results":[{"id":"9001","key":"TESTSPACE"}]}`
2. `GET /wiki/api/v2/spaces/9001/pages` (no `.expect(N)` matcher) →
   `{"results":[<minimal page>, ...], "_links":{}}`  (empty `_links` ⇒ single page)
3. `GET /wiki/api/v2/pages/{id}?body-format=atlas_doc_format` (path_regex +
   query_param matcher) → minimal ADF page JSON.

Minimal page shape required by ConfluenceBackend's `translate()` (fields
below are the MINIMUM that deserializes without Error::Other):

```json
{
  "id": "10001",
  "status": "current",
  "title": "Page 1",
  "createdAt": "2026-01-01T00:00:00Z",
  "version": {"number": 1, "createdAt": "2026-01-01T00:00:00Z"},
  "body": {"atlas_doc_format":{"value":{"type":"doc","version":1,"content":[]}}}
}
```
</interfaces>
</context>

## Tasks

<tasks>

<task type="auto" tdd="true">
  <name>Task 17-B-01: Add dev-deps + wiremock CI test confluence_direct_3_clients_5s</name>
  <files>crates/reposix-swarm/Cargo.toml, crates/reposix-swarm/tests/mini_e2e.rs</files>
  <behavior>
    Test `confluence_direct_3_clients_5s` (name is exact — grep-addressable
    from VALIDATION.md) must assert:
    - `markdown.contains("Clients: 3")`  (summary renders the CLI arg)
    - `markdown.contains("| list ")`     (list op row is present)
    - `parse_total_ops(&markdown) >= 3`  (at least 3 clients × one cycle each)
    - If the "Errors by class" section is present, it MUST NOT contain
      `| Other` (transport/deser bugs).
    - Test MUST NOT assert `audit_rows > 0`. Read-only workload writes 0
      audit rows (RESEARCH.md §"Summary" + Risk 4). The test is free to open
      no audit DB at all; no NamedTempFile needed.

    Test setup invariants:
    - MockServer binds to `127.0.0.1:0`; default
      `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:*` covers it (don't touch the
      env var).
    - Wiremock stubs MUST NOT use `.expect(N)` — call counts are
      non-deterministic across a 5s duration run (RESEARCH.md Risk 2).
    - `get_issue` stub MUST match `query_param("body-format",
      "atlas_doc_format")` (RESEARCH.md Pitfall 2).
    - Page fixtures MUST include `createdAt`, `version.number`,
      `version.createdAt` (RESEARCH.md Pitfall 3).

    Dev-dep invariants:
    - `reposix-confluence = { path = "../reposix-confluence" }` under
      `[dev-dependencies]`.
    - `wiremock = "0.6"` under `[dev-dependencies]` (same version as in
      `reposix-confluence/Cargo.toml`).

    TDD flow: write the test first (RED — it fails to compile without the
    dev-deps), add the dev-deps (still RED — no space fixture means empty
    list), verify the test goes GREEN with the full stub set below.
  </behavior>
  <action>
  1. Edit `crates/reposix-swarm/Cargo.toml`. Under `[dev-dependencies]`, add:

     ```toml
     reposix-confluence = { path = "../reposix-confluence" }
     wiremock = "0.6"
     serde_json = { workspace = true }
     ```

     (`serde_json` is already a `[dependencies]` entry in this crate, but the
     test needs it too; tests don't inherit from `[dependencies]` for macro
     resolution unless the crate is listed in dev-deps explicitly. If the
     test compiles without adding `serde_json` to dev-deps, leave it out.)

  2. Append a new `#[tokio::test]` to
     `crates/reposix-swarm/tests/mini_e2e.rs` (do NOT touch the existing sim
     test). Name: `confluence_direct_3_clients_5s`. Skeleton:

     ```rust
     use reposix_confluence::ConfluenceCreds;
     use reposix_swarm::confluence_direct::ConfluenceDirectWorkload;
     use serde_json::json;
     use wiremock::matchers::{method, path, path_regex, query_param};
     use wiremock::{Mock, MockServer, ResponseTemplate};

     fn sample_page(id: &str, title: &str) -> serde_json::Value {
         json!({
             "id": id,
             "status": "current",
             "title": title,
             "createdAt": "2026-01-01T00:00:00Z",
             "version": {"number": 1, "createdAt": "2026-01-01T00:00:00Z"},
             "body": {
                 "atlas_doc_format": {
                     "value": {"type":"doc","version":1,"content":[]}
                 }
             }
         })
     }

     #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
     async fn confluence_direct_3_clients_5s() {
         let server = MockServer::start().await;

         // Space resolver — called repeatedly (RESEARCH.md Risk 4); no
         // .expect(N).
         Mock::given(method("GET"))
             .and(path("/wiki/api/v2/spaces"))
             .and(query_param("keys", "TESTSPACE"))
             .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                 "results": [{"id": "9001", "key": "TESTSPACE"}]
             })))
             .mount(&server)
             .await;

         // Page list — single page (empty _links) so `list_issues` exits
         // the pagination loop cleanly (RESEARCH.md Risk 3).
         Mock::given(method("GET"))
             .and(path("/wiki/api/v2/spaces/9001/pages"))
             .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                 "results": [
                     sample_page("10001", "Page 1"),
                     sample_page("10002", "Page 2"),
                     sample_page("10003", "Page 3"),
                 ],
                 "_links": {}
             })))
             .mount(&server)
             .await;

         // Page get — path_regex so any of the 3 ids match.
         Mock::given(method("GET"))
             .and(path_regex(r"^/wiki/api/v2/pages/\d+$"))
             .and(query_param("body-format", "atlas_doc_format"))
             .respond_with(
                 ResponseTemplate::new(200)
                     .set_body_json(sample_page("10001", "Page 1")),
             )
             .mount(&server)
             .await;

         let base = server.uri();
         let creds = ConfluenceCreds {
             email: "swarm@test".to_string(),
             api_token: "tok".to_string(),
         };
         let space = "TESTSPACE".to_string();

         let cfg = SwarmConfig {
             clients: 3,
             duration: Duration::from_secs(5),
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
         assert!(
             markdown.contains("| list "),
             "summary missing list row:\n{markdown}"
         );
         let total_ops = parse_total_ops(&markdown);
         assert!(
             total_ops >= 3,
             "expected >=3 total ops, got {total_ops}:\n{markdown}"
         );
         if let Some(err_section) = markdown.split("### Errors by class").nth(1) {
             assert!(
                 !err_section.contains("| Other"),
                 "confluence-direct produced Other-class errors under wiremock, \
                  which indicates transport/deser breakage:\n{markdown}"
             );
         }
         // NOTE: deliberately NO audit-row assertion. Read-only workload
         // writes 0 audit rows; see RESEARCH.md §"Summary" audit caveat.
     }
     ```

     Re-use the file's existing `parse_total_ops` helper — DO NOT duplicate.

  3. Run `cargo test -p reposix-swarm --test mini_e2e` — both the existing
     sim test and the new confluence test must pass. If the allowlist
     rejects the wiremock server URL, set
     `std::env::set_var("REPOSIX_ALLOWED_ORIGINS", server.uri())` at the top
     of the test (but try WITHOUT first — the default should cover
     `127.0.0.1`, per RESEARCH.md Assumption A1).

  4. Run `cargo clippy -p reposix-swarm --all-targets -- -D warnings` —
     clean.
  </action>
  <verify>
    <automated>cargo test -p reposix-swarm --test mini_e2e confluence_direct_3_clients_5s &amp;&amp; cargo clippy -p reposix-swarm --all-targets -- -D warnings</automated>
  </verify>
  <done>
    Wiremock test green. Dev-deps added exactly as specified. Pre-existing sim
    test still green. Clippy clean. No audit-row assertion in the new test.
  </done>
</task>

<task type="auto" tdd="false">
  <name>Task 17-B-02: Add real-tenant smoke test under #[ignore] + CHANGELOG + SUMMARY + STATE</name>
  <files>crates/reposix-swarm/tests/confluence_real_tenant.rs, CHANGELOG.md, .planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md, .planning/STATE.md</files>
  <action>
  1. Create `crates/reposix-swarm/tests/confluence_real_tenant.rs` as a new
     integration-test file. Must include crate-level lint attributes and an
     `#[ignore]`-gated test that (a) skips if any of the three env vars is
     absent, (b) otherwise runs 3 clients × 10s against the real tenant.

     ```rust
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
         // Conflict/RateLimited/NotFound are tolerated (rate limits
         // expected).
         if let Some(err_section) = markdown.split("### Errors by class").nth(1) {
             assert!(
                 !err_section.contains("| Other"),
                 "real-tenant run produced Other-class errors:\n{markdown}"
             );
         }
     }
     ```

  2. Add an entry to `CHANGELOG.md` under the v0.6.0 Unreleased section (or
     whatever the current "Unreleased" / in-progress section is). One short
     paragraph + bullet list:

     ```markdown
     ### Added — Phase 17: Swarm confluence-direct mode
     - `reposix-swarm --mode confluence-direct` spawns N read-only clients
       against `ConfluenceBackend` (list + 3×get per cycle). Closes the
       session-4 open gap; SWARM-01 + SWARM-02.
     - New wiremock CI test `confluence_direct_3_clients_5s` in
       `reposix-swarm/tests/mini_e2e.rs`.
     - New real-tenant smoke `live_confluence_direct_smoke` under
       `#[ignore]` + env-var gate (`ATLASSIAN_EMAIL`,
       `ATLASSIAN_API_KEY`, `REPOSIX_CONFLUENCE_TENANT`).
     ```

     If `CHANGELOG.md` does not have an unreleased v0.6.0 section yet, add
     one above the most recent phase's entry. Do NOT rewrite or reorder prior
     entries.

  3. Create
     `.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md`
     using the standard GSD summary template. Must include:

     - Phase goal (from CONTEXT.md)
     - Artifacts created (confluence_direct.rs, main.rs Mode variant,
       dev-deps, two test fns, CHANGELOG entry)
     - Requirements closed (SWARM-01, SWARM-02)
     - Deferred work (write ops → Phase 21 / OP-7; 50-client × 30s load
       runs → also Phase 21 HARD-01)
     - Test count delta (expected: +1 integration test in mini_e2e.rs and
       1 ignored test in confluence_real_tenant.rs; total workspace test
       count must still be ≥ 318 when counting the new wiremock test)
     - Clippy-clean status

  4. Update `.planning/STATE.md` — mark Phase 17 as complete / current
     cursor advanced. Follow the pattern of how prior phases were closed
     (grep `git log --oneline -- .planning/STATE.md` if unsure; typically a
     "Last shipped: Phase 17" line and/or phase checklist flip).

  5. Run the full verification sweep:
     - `cargo test --workspace` — must be green and test count MUST NOT
       drop below 318 (317 Phase-16 baseline + the new
       `confluence_direct_3_clients_5s`).
     - `cargo clippy --workspace --all-targets -- -D warnings` — clean.
     - `cargo test -p reposix-swarm -- --ignored live_confluence_direct_smoke`
       runs but short-circuits with "skip: ATLASSIAN_EMAIL not set" when env
       vars are absent (developer verifies by eyeball; not enforced in the
       automated verify command).
  </action>
  <verify>
    <automated>cargo test --workspace &amp;&amp; cargo clippy --workspace --all-targets -- -D warnings &amp;&amp; test -f .planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md &amp;&amp; grep -q confluence-direct CHANGELOG.md &amp;&amp; grep -q 'live_confluence_direct_smoke' crates/reposix-swarm/tests/confluence_real_tenant.rs</automated>
  </verify>
  <done>
    Real-tenant test file exists + skips cleanly when env absent.
    CHANGELOG.md has the Phase 17 entry. 17-SUMMARY.md exists and documents
    what shipped. STATE.md records Phase 17 closure. Full-workspace tests
    green; test count ≥ 318. Clippy clean workspace-wide.
  </done>
</task>

</tasks>

## Verification

Phase-level sampling for Wave 2 + full Phase 17:

- `cargo test --workspace` — full suite green, test count ≥ 318 (baseline
  317 + new `confluence_direct_3_clients_5s`).
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo test -p reposix-swarm --test mini_e2e` — existing sim test + new
  confluence test both green.
- `cargo test -p reposix-swarm -- --ignored live_confluence_direct_smoke`
  skips silently when env is absent; runs against real tenant when env is
  set (out of scope for automated CI).
- Docs smoke: `grep -q confluence-direct CHANGELOG.md` and the `17-SUMMARY.md`
  file exists.

## Threat Model

Trust boundaries touched by Wave 2:

| Boundary | Description |
|----------|-------------|
| test harness → wiremock | Local in-process HTTP mock; trust boundary collapses — still honors SG-01 allowlist because wiremock binds on 127.0.0.1. |
| test harness → real Confluence tenant | Only crossed when `--ignored` is explicitly selected AND all three env vars are present; otherwise the test returns early. |
| env vars → process memory | Real-tenant test reads `ATLASSIAN_API_KEY` via `std::env::var`; value is placed into `ConfluenceCreds` (redacted Debug) and never printed. |

STRIDE register (scoped to Wave 2 test + docs changes):

| Threat ID | Category | Component | Disposition | Mitigation |
|-----------|----------|-----------|-------------|-----------|
| T-17-05 | Information Disclosure | confluence_real_tenant.rs logging | mitigate | The skip branches use `eprintln!("skip: VAR not set")` with var NAMES only, never values. The `ConfluenceCreds` manual Debug redacts `api_token`. Test MUST NOT `dbg!(creds)` or `println!("{creds:?}")`. |
| T-17-06 | Tampering | wiremock fixtures | accept | Fixtures are attacker-shaped by the test author (that's the point of TDD). Accepted because they stay inside the test process. |
| T-17-07 | Repudiation | real-tenant silent-skip | accept | A silent skip hides misconfigured CI. Acceptance rationale: intentional — the test is an opt-in smoke, not a required gate. Developer verifies by running `--ignored` locally when setup changes. |
| T-17-08 | Denial of Service | 3 clients × 10s against real tenant | mitigate | Locked decision (not 50 × 30s). `ConfluenceBackend::rate_limit_gate` absorbs 429s per-client. No custom retry. |
| T-17-09 | Elevation of Privilege | REPOSIX_ALLOWED_ORIGINS in real-tenant test | mitigate | Test deliberately does NOT set the allowlist env var — caller must set it explicitly before running. Fail-closed default per CLAUDE.md OP #1. |

## Success Criteria

Wave 2 is done when:

- [ ] `crates/reposix-swarm/Cargo.toml` `[dev-dependencies]` adds
      `reposix-confluence = { path = "../reposix-confluence" }` and
      `wiremock = "0.6"`.
- [ ] `crates/reposix-swarm/tests/mini_e2e.rs` contains
      `confluence_direct_3_clients_5s` test; the test asserts `Clients: 3`,
      list-row presence, `total_ops >= 3`, and no `| Other` errors. It has
      NO audit-row assertion.
- [ ] `crates/reposix-swarm/tests/confluence_real_tenant.rs` exists with
      `#[ignore]` `live_confluence_direct_smoke` test that silently skips
      when any of the three env vars is missing.
- [ ] `CHANGELOG.md` has a Phase 17 entry under the current Unreleased
      section mentioning `confluence-direct`.
- [ ] `.planning/phases/17-.../17-SUMMARY.md` exists and documents
      artifacts, requirements closed (SWARM-01, SWARM-02), and deferred
      work (Phase 21 writes).
- [ ] `.planning/STATE.md` updated to reflect Phase 17 closure.
- [ ] `cargo test --workspace` green; test count ≥ 318.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean.

<output>
After completion, the phase SUMMARY itself is one of the artifacts this task
produces:
`.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md`
</output>
