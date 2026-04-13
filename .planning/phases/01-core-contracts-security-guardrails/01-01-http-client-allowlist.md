---
phase: 01-core-contracts-security-guardrails
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - clippy.toml
  - crates/reposix-core/Cargo.toml
  - crates/reposix-core/src/lib.rs
  - crates/reposix-core/src/http.rs
  - crates/reposix-core/src/error.rs
  - crates/reposix-core/tests/http_allowlist.rs
autonomous: true
requirements:
  - SG-01
  - SG-07
user_setup: []

must_haves:
  truths:
    - "`reposix_core::http::client(ClientOpts::default())` returns a client that can reach 127.0.0.1 and localhost but not evil.example."
    - "`REPOSIX_ALLOWED_ORIGINS=http://evil.example` overrides the default allowlist (loopback is then rejected)."
    - "Redirects are never followed: a 3xx response surfaces as `reqwest::Response` with the 3xx status, never a follow-through."
    - "`grep -RIn 'reqwest::Client::new\\|Client::builder' crates/ --include='*.rs' | grep -v 'crates/reposix-core/src/http.rs'` prints nothing."
    - "`cat clippy.toml` contains `reqwest::Client::new` inside `disallowed-methods`."
    - "`cargo clippy -p reposix-core --all-targets -- -D warnings` is clean."
  artifacts:
    - path: "clippy.toml"
      provides: "workspace disallowed-methods config banning direct reqwest client construction"
      contains: "reqwest::Client::new"
    - path: "crates/reposix-core/src/http.rs"
      provides: "`client()` factory + `request()` per-call recheck + origin-glob matcher"
      exports: ["client", "request", "ClientOpts", "AllowlistError"]
    - path: "crates/reposix-core/tests/http_allowlist.rs"
      provides: "four origin-class tests + env-override test + redirect-refusal test"
      contains: "egress_to_non_allowlisted_host_is_rejected"
  key_links:
    - from: "crates/reposix-core/src/http.rs"
      to: "REPOSIX_ALLOWED_ORIGINS env var"
      via: "std::env::var read once inside client(); parsed into a Vec<OriginGlob>"
      pattern: "REPOSIX_ALLOWED_ORIGINS"
    - from: "crates/reposix-core/src/http.rs"
      to: "reqwest::ClientBuilder"
      via: "the one #[allow(clippy::disallowed_methods)] construction site with justifying comment"
      pattern: "clippy::disallowed_methods"
    - from: "clippy.toml"
      to: "cargo clippy"
      via: "disallowed-methods lint picks up every call site outside http.rs"
      pattern: "disallowed-methods"
---

<objective>
Land the single locked-down `reposix_core::http::client()` factory + the clippy lint that makes it the only way to construct a `reqwest::Client` in this workspace. Every outbound HTTP call in every downstream crate (sim, fuse, remote, cli) will build on top of this factory. This plan closes SG-01 (egress allowlist) and the 5s-timeout half of SG-07.

Purpose: cut the exfiltration leg of the lethal trifecta at the type + config layer so Phases 2 and 3 cannot accidentally reintroduce it. The easy path (`reposix_core::http::client()`) is the safe path; the unsafe path (`reqwest::Client::new()`) is a compile-time lint error.

Output:
  - `clippy.toml` at workspace root with `disallowed-methods` banning `reqwest::Client::new`, `reqwest::Client::builder`, `reqwest::ClientBuilder::new`.
  - `crates/reposix-core/src/http.rs` exporting `client(opts) -> Result<reqwest::Client>`, `request(&client, method, url) -> Result<Response>`, `ClientOpts`, and an `AllowlistError` variant added to `Error`.
  - `reqwest` added to `reposix-core`'s `[dependencies]` with the workspace features already set; `tokio` (with `rt`, `macros`) and `wiremock` (or an `axum`-based fixture) added to `[dev-dependencies]` for the redirect test.
  - Integration tests under `crates/reposix-core/tests/http_allowlist.rs` covering: loopback default-allow, non-loopback default-deny, env-var override, redirect refusal, 5-second timeout, and a name-matching test for ROADMAP success-criterion #1.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/01-core-contracts-security-guardrails/01-CONTEXT.md
@.planning/research/threat-model-and-critique.md
@CLAUDE.md
@Cargo.toml
@crates/reposix-core/Cargo.toml
@crates/reposix-core/src/lib.rs
@crates/reposix-core/src/error.rs

<interfaces>
<!-- Extracted from crates/reposix-core/src/error.rs — the Error enum this plan extends. -->

```rust
// Already in the crate — this plan ADDS one variant, does not rewrite:
pub enum Error {
    Frontmatter(String),
    InvalidIssue(String),
    InvalidRemote(String),
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
    Yaml(#[from] serde_yaml::Error),
    Other(String),
    // ADD in this plan:
    // InvalidOrigin(String),   // e.g. "https://evil.example/ is not in REPOSIX_ALLOWED_ORIGINS"
    // Http(#[from] reqwest::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
```

Public surface this plan MUST expose from `reposix_core::http`:

```rust
/// Options for constructing an HTTP client. `Default` produces the 5s-timeout
/// client the other 95% of callers want.
pub struct ClientOpts {
    pub total_timeout: Duration,   // default Duration::from_secs(5)
    pub user_agent: Option<String>, // default Some("reposix/0.1.0")
}
impl Default for ClientOpts { /* ... */ }

/// Build the one-and-only legal HTTP client for this workspace.
///
/// # Errors
/// Returns `Error::Other` if `REPOSIX_ALLOWED_ORIGINS` is set but un-parseable,
/// or `Error::Http` if `reqwest` itself refuses to build the client.
pub fn client(opts: ClientOpts) -> Result<reqwest::Client>;

/// Send a request through `client`, re-checking the URL against the allowlist
/// first. This is belt-and-braces: `reqwest::Client` lets callers override the
/// URL after construction, so the factory alone is not enough.
///
/// # Errors
/// Returns `Error::InvalidOrigin` if `url`'s scheme+host+port do not match any
/// allowlist entry; `Error::Http` for transport failures.
pub async fn request(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
) -> Result<reqwest::Response>;
```
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| reposix process → network | Every outbound HTTP call crosses here; attacker controls DNS/CDN/redirects on the far side. |
| process env → `client()` factory | Operator-controlled (via `REPOSIX_ALLOWED_ORIGINS`) but a compromised parent process can widen the allowlist. |
| user code → `reqwest::Client` constructor | The footgun boundary — anyone who calls `reqwest::Client::new()` has bypassed our allowlist. Clippy lint enforces. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-01 | Information Disclosure | Outbound HTTP to attacker-controlled host (A1, A4 in research doc) | mitigate | `client()` factory + per-request `request()` recheck; clippy `disallowed-methods` blocks direct construction; integration test `egress_to_non_allowlisted_host_is_rejected` proves it. |
| T-01-02 | Tampering / Info Disclosure | HTTP 3xx redirect to a non-allowlisted host (A4) | mitigate | `reqwest::redirect::Policy::none()` in the factory; test `http_redirects_are_not_followed` asserts a 302 surfaces as a 302 response, never a follow-through. |
| T-01-03 | Denial of Service | Slow-loris / hung connection to simulator (SG-07) | mitigate | `ClientOpts::default()` sets 5s total timeout; test `request_times_out_after_5_seconds` uses a wiremock delay fixture. |
| T-01-04 | Elevation of Privilege | A crate author bypasses the factory by calling `reqwest::Client::new()` directly | mitigate | `clippy.toml` `disallowed-methods` list + CI runs `cargo clippy --all-targets -- -D warnings`; grep-gate in ROADMAP success-criterion #2 provides a second layer. |
| T-01-05 | Spoofing | Operator misconfigures `REPOSIX_ALLOWED_ORIGINS` to `*` | accept | Out of scope for v0.1 — operator config errors are the operator's responsibility. Documented in the `client()` doc comment. |
</threat_model>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add reqwest dep + ClientOpts + origin-glob matcher skeleton</name>
  <files>
    crates/reposix-core/Cargo.toml
    crates/reposix-core/src/lib.rs
    crates/reposix-core/src/http.rs
    crates/reposix-core/src/error.rs
  </files>
  <behavior>
    - `ClientOpts::default().total_timeout == Duration::from_secs(5)` (per SG-07).
    - `parse_allowlist("http://127.0.0.1:*,http://localhost:*")` returns 2 entries.
    - `parse_allowlist("")` returns the default allowlist (empty string = unset semantics).
    - `parse_allowlist("not a url")` returns `Err(Error::Other(_))` with a helpful message.
    - `OriginGlob::matches("http://127.0.0.1:7878")` is true for `http://127.0.0.1:*`.
    - `OriginGlob::matches("https://127.0.0.1:7878")` is false for `http://127.0.0.1:*` (scheme matters).
    - `OriginGlob::matches("http://evil.example:80")` is false for `http://127.0.0.1:*`.
    - `OriginGlob::matches("http://127.0.0.1:80")` is true for `http://127.0.0.1:80` (exact port).
    - `Error::InvalidOrigin` and `Error::Http` variants exist; `Error::Http` is `#[from] reqwest::Error`.
  </behavior>
  <action>
    1. Edit `crates/reposix-core/Cargo.toml`:
       - Add `reqwest = { workspace = true }` to `[dependencies]`.
       - Add a `[dev-dependencies]` section with `tokio = { workspace = true }`, `wiremock = "0.6"`.
    2. Edit `crates/reposix-core/src/error.rs`: add two variants to `Error`:
       ```rust
       /// URL rejected by the egress allowlist.
       #[error("blocked origin: {0}")]
       InvalidOrigin(String),
       /// Underlying HTTP/transport error.
       #[error(transparent)]
       Http(#[from] reqwest::Error),
       ```
       Keep the enum's `#[derive]`s; do not alphabetize existing variants (diff-noise).
    3. Create `crates/reposix-core/src/http.rs` with:
       - `#![allow(clippy::module_name_repetitions)]` at top (the single allowed blanket inside this crate).
       - `pub struct ClientOpts { pub total_timeout: Duration, pub user_agent: Option<String> }` with `Default` impl.
       - `struct OriginGlob { scheme: String, host: String, port: Option<u16> }` (where `port: None` means "any port, i.e. `*`").
       - `impl OriginGlob { fn matches(&self, url: &Url) -> bool { ... } }` — scheme MUST match exactly; host MUST match exactly; port matches if `self.port.is_none()` or `self.port == url.port_or_known_default()`.
       - `fn parse_allowlist(raw: &str) -> Result<Vec<OriginGlob>>` — splits on `,`, trims whitespace, parses each as a URL-shaped pattern. An empty / all-whitespace input returns `DEFAULT_ALLOWLIST.clone()` (i.e. loopback-only). Use `url` parsing only loosely — the pattern `http://127.0.0.1:*` is not a legal URL per RFC, so hand-roll the parser: require `scheme://host[:port]`, where `port` is either digits or `*`.
       - `const DEFAULT_ALLOWLIST_RAW: &str = "http://127.0.0.1:*,http://localhost:*";`
       - `fn load_allowlist_from_env() -> Result<Vec<OriginGlob>>` — reads `REPOSIX_ALLOWED_ORIGINS`; if unset, returns parsed default.
       - Unit tests in a `#[cfg(test)] mod tests` covering every bullet in `<behavior>` above.
    4. Add `pub mod http;` to `crates/reposix-core/src/lib.rs`. Do NOT re-export — callers write `reposix_core::http::client(...)`.
    5. Keep every public item documented; every `Result`-returning fn has a `# Errors` section per CLAUDE.md.

    AVOID: pulling in `globset` — the grammar is narrow enough to hand-roll (per 01-CONTEXT.md "Claude's discretion"). AVOID constructing any `reqwest::Client` yet; that's Task 2.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib http::tests -- --nocapture &amp;&amp; cargo clippy -p reposix-core --lib -- -D warnings</automated>
  </verify>
  <done>
    `cargo check -p reposix-core` compiles cleanly; the `http::tests` module passes with every `<behavior>` bullet as a named test; clippy pedantic is clean.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement client() factory + request() wrapper + clippy.toml</name>
  <files>
    clippy.toml
    crates/reposix-core/src/http.rs
  </files>
  <behavior>
    - `client(ClientOpts::default())` returns `Ok(reqwest::Client)`.
    - The returned client's `redirect::Policy` is `none()` — verified by an integration test in Task 3 since we can't introspect it directly; here we assert via the wiremock fixture that a 302 response is NOT followed.
    - `request(&client, GET, "http://127.0.0.1:0/")` (connect will fail, but) does NOT return `Error::InvalidOrigin` — the origin is allowlisted.
    - `request(&client, GET, "https://evil.example/")` returns `Err(Error::InvalidOrigin(_))` BEFORE any network I/O.
    - `request(&client, GET, "not-a-url")` returns `Err(Error::InvalidOrigin(_))`.
    - The ONE `reqwest::ClientBuilder::new()` call in the crate is annotated `#[allow(clippy::disallowed_methods)]` with a comment quoting "this is the single legal construction site; see SG-01".
  </behavior>
  <action>
    1. Create `clippy.toml` at the workspace root (`/home/reuben/workspace/reposix/clippy.toml`) with:
       ```toml
       disallowed-methods = [
         { path = "reqwest::Client::new",        reason = "use reposix_core::http::client()" },
         { path = "reqwest::Client::builder",    reason = "use reposix_core::http::client()" },
         { path = "reqwest::ClientBuilder::new", reason = "use reposix_core::http::client()" },
       ]
       ```
    2. Extend `crates/reposix-core/src/http.rs`:
       - Implement `pub fn client(opts: ClientOpts) -> Result<reqwest::Client>`. Inside this function ONLY:
         ```rust
         #[allow(clippy::disallowed_methods)] // SG-01: this is the single legal construction site.
         let builder = reqwest::ClientBuilder::new();
         ```
         Configure: `.redirect(reqwest::redirect::Policy::none())`, `.timeout(opts.total_timeout)`, user-agent if set. Call `.build()` and map the `reqwest::Error` via `?` (works via the `#[from]` added in Task 1).
       - Implement `pub async fn request(client: &reqwest::Client, method: reqwest::Method, url: &str) -> Result<reqwest::Response>`:
         - Parse `url` as `reqwest::Url` — map parse failure to `Error::InvalidOrigin(url.to_owned())`.
         - Load allowlist via `load_allowlist_from_env()` (cached in a `once_cell::sync::Lazy` keyed by env-var value, OR re-read each call — pragmatic choice: re-read, the cost is trivial and tests can change the env var).
         - Check every glob; if no match, return `Err(Error::InvalidOrigin(url.to_owned()))`.
         - Otherwise `client.request(method, parsed_url).send().await.map_err(Into::into)`.
       - Document both functions with `# Errors` sections.
       - Extend the unit-test module to cover the bullets in `<behavior>`. Use `reqwest::Method::GET` and `http://127.0.0.1:0/` for the "allowlisted but no server" case (connect fails, not `InvalidOrigin`).
    3. Re-run full clippy to confirm the lint is active: `cargo clippy --workspace --all-targets -- -D warnings`. Add a `#[allow(clippy::disallowed_methods)]` at the ONE site in http.rs. No other allows anywhere in the workspace.

    AVOID: caching the env-var globally — tests mutate it. AVOID using `.allow_redirects(false)` as that API doesn't exist on this reqwest version; use `.redirect(Policy::none())`. AVOID implementing per-request timeouts separately; the builder-level timeout covers it.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib http::tests -- --nocapture &amp;&amp; cargo clippy --workspace --all-targets -- -D warnings &amp;&amp; test -f clippy.toml &amp;&amp; grep -q 'reqwest::Client::new' clippy.toml</automated>
  </verify>
  <done>
    `client(ClientOpts::default())` builds a working client, `request()` blocks non-allowlisted origins before any I/O, `clippy.toml` is committed with the three disallowed paths, and workspace-wide clippy is green.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Integration tests — four origin classes + redirect + 5s timeout + ROADMAP success-criterion names</name>
  <files>
    crates/reposix-core/tests/http_allowlist.rs
  </files>
  <behavior>
    - `egress_to_non_allowlisted_host_is_rejected` — the exact test name ROADMAP success-criterion #1 mandates. Asserts `request()` to `https://evil.example/` returns `Err(Error::InvalidOrigin(_))` with no network I/O (use a very short timeout and assert the call returns faster than that timeout would permit if it actually connected).
    - `allowlist_default_and_env` — exact name ROADMAP success-criterion #4 mandates. Runs under default allowlist (loopback allowed), then under `REPOSIX_ALLOWED_ORIGINS=http://evil.example` (loopback rejected, evil.example allowed in principle — though the test doesn't actually connect there).
    - `loopback_is_allowed_by_default` — connects to a wiremock server on 127.0.0.1:random_port; `request()` returns `Ok(response)` with status 200.
    - `non_loopback_is_denied_by_default` — `request()` to `http://93.184.216.34/` (example.com's IP — deliberately an IP so DNS doesn't fire) returns `Err(Error::InvalidOrigin(_))`.
    - `env_override_redefines_allowlist` — with `REPOSIX_ALLOWED_ORIGINS=http://other.example:8080`, a call to `http://127.0.0.1:7878` returns `Err(Error::InvalidOrigin(_))`.
    - `http_redirects_are_not_followed` — wiremock fixture returns `302 Location: https://attacker.example/`; `request()` returns `Ok(response)` with status 302, NOT a connect error to attacker.example.
    - `request_times_out_after_5_seconds` — wiremock fixture delays 10s; `request()` with default opts returns `Err(Error::Http(_))` where the underlying error `.is_timeout()` is true, in < 6s wall clock (gated behind `#[ignore]` since it sleeps 5s; run with `--ignored` in CI).
  </behavior>
  <action>
    1. Create `crates/reposix-core/tests/http_allowlist.rs`. Structure:
       ```rust
       //! Integration tests for `reposix_core::http`. Covers ROADMAP phase-1
       //! success-criteria #1 (egress test name) and #4 (env-var override).

       use reposix_core::http::{client, request, ClientOpts};
       use reposix_core::Error;
       use wiremock::{MockServer, Mock, ResponseTemplate, matchers::any};

       // Serialize env-var-touching tests — they mutate a process-global.
       static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
       ```
       - Each env-touching test: `let _g = ENV_LOCK.lock().unwrap(); std::env::set_var(...); /* test */ std::env::remove_var(...);` in a scope guard. Prefer a small RAII helper to avoid leaving env state on panic.
       - Use `#[tokio::test]` (requires `tokio` in dev-deps; already added in Task 1).
    2. Implement every named test in `<behavior>`. For `request_times_out_after_5_seconds`, mark with `#[ignore]` and a comment explaining it eats 5s; still gets exercised via `cargo test -- --ignored`.
    3. Add the test file to Cargo's integration-test discovery automatically by virtue of living in `tests/`.
    4. Run the tests and confirm all non-ignored ones pass and the named tests exist.

    AVOID: introducing a `#[ignore]` on any other test — the name-matching tests MUST run under `cargo test -p reposix-core --all-features` per ROADMAP success-criterion #1. AVOID using `reqwest::get(...)` helpers in test code — they call `Client::new()` and trip the clippy lint.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --all-features &amp;&amp; cargo test -p reposix-core --all-features -- --ignored request_times_out_after_5_seconds &amp;&amp; cargo test -p reposix-core allowlist_default_and_env -- --nocapture &amp;&amp; cargo test -p reposix-core egress_to_non_allowlisted_host_is_rejected</automated>
  </verify>
  <done>
    All seven named tests pass (with `--ignored` for the timeout test); ROADMAP success-criteria #1 (two of the five test names) and #4 (env-override) are satisfied by committed tests.
  </done>
</task>

</tasks>

<verification>
Phase-level checks this plan contributes to:

1. ROADMAP SC #1 (partial): `cargo test -p reposix-core --all-features` is green AND includes `egress_to_non_allowlisted_host_is_rejected`.
2. ROADMAP SC #2: `grep -RIn 'reqwest::Client::new\|Client::builder' crates/ --include='*.rs' | grep -v 'crates/reposix-core/src/http.rs' | wc -l` prints `0` AND `cat clippy.toml` contains `reqwest::Client::new`.
3. ROADMAP SC #4: `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:* cargo test -p reposix-core allowlist_default_and_env -- --nocapture` passes.
4. ROADMAP SC #5 (partial): `cargo clippy -p reposix-core --all-targets -- -D warnings` is clean.
</verification>

<success_criteria>
**Goal-backward verification** — if the orchestrator runs:

```bash
cd /home/reuben/workspace/reposix && \
  cargo test -p reposix-core --all-features && \
  cargo test -p reposix-core -- egress_to_non_allowlisted_host_is_rejected allowlist_default_and_env && \
  grep -RIn 'reqwest::Client::new\|Client::builder' crates/ --include='*.rs' | grep -v 'crates/reposix-core/src/http.rs' | wc -l | grep -q '^0$' && \
  grep -q 'reqwest::Client::new' clippy.toml && \
  cargo clippy -p reposix-core --all-targets -- -D warnings
```

…then phase-1 success-criteria **#1 (partial)**, **#2 (full)**, **#4 (full)**, and **#5 (partial)** pass. SC #1's remaining three named tests and SC #5's full-crate clippy-clean come from plans 01-02 and 01-03.
</success_criteria>

<output>
After completion, create `.planning/phases/01-core-contracts-security-guardrails/01-01-SUMMARY.md` per the summary template. Must include: the exact name of the `#[allow(clippy::disallowed_methods)]` construction site (file:line) so future reviewers can audit it, the grammar accepted by `parse_allowlist()`, and the names of all seven tests shipped.
</output>
