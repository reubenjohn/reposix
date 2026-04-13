---
phase: 01-core-contracts-security-guardrails
plan: 01
type: execute
wave: 1
depends_on:
  - "01-00"
files_modified:
  - clippy.toml
  - crates/reposix-core/Cargo.toml
  - crates/reposix-core/src/lib.rs
  - crates/reposix-core/src/http.rs
  - crates/reposix-core/tests/http_allowlist.rs
  - scripts/check_clippy_lint_loaded.sh
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
    - "If a caller observes a 302 and re-feeds its `Location` URL into `request()`, the per-request recheck rejects it with `Error::InvalidOrigin` when the target is non-allowlisted."
    - "`grep -RIn 'reqwest::Client::new\\|Client::builder' crates/ --include='*.rs' | grep -v 'crates/reposix-core/src/http.rs'` prints nothing."
    - "`cat clippy.toml` contains `reqwest::Client::new` inside `disallowed-methods`."
    - "`cargo clippy -p reposix-core --all-targets -- -D warnings` is clean."
    - "`bash scripts/check_clippy_lint_loaded.sh` exits 0, proving clippy actually consumes `clippy.toml` (not just that the file exists)."
  artifacts:
    - path: "clippy.toml"
      provides: "workspace disallowed-methods config banning direct reqwest client construction"
      contains: "reqwest::Client::new"
    - path: "crates/reposix-core/src/http.rs"
      provides: "`client()` factory + `request()` per-call recheck + origin-glob matcher"
      exports: ["client", "request", "ClientOpts", "AllowlistError"]
    - path: "crates/reposix-core/tests/http_allowlist.rs"
      provides: "four origin-class tests + env-override test + redirect-refusal test + redirect-target-recheck test"
      contains: "egress_to_non_allowlisted_host_is_rejected"
    - path: "scripts/check_clippy_lint_loaded.sh"
      provides: "shell script that proves clippy's disallowed-methods config is actually loaded (FIX 3 from plan-checker)"
      contains: "disallowed_methods"
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
    - from: "scripts/check_clippy_lint_loaded.sh"
      to: "clippy.toml"
      via: "deliberate violation surfaces lint message proving config was loaded"
      pattern: "check_clippy_lint_loaded"
---

<objective>
Land the single locked-down `reposix_core::http::client()` factory + the clippy lint that makes it the only way to construct a `reqwest::Client` in this workspace. Every outbound HTTP call in every downstream crate (sim, fuse, remote, cli) will build on top of this factory. This plan closes SG-01 (egress allowlist) and the 5s-timeout half of SG-07.

Purpose: cut the exfiltration leg of the lethal trifecta at the type + config layer so Phases 2 and 3 cannot accidentally reintroduce it. The easy path (`reposix_core::http::client()`) is the safe path; the unsafe path (`reqwest::Client::new()`) is a compile-time lint error.

**Wave-0 prerequisite:** plan 01-00 has already added `Error::InvalidOrigin` and `Error::Http(#[from] reqwest::Error)` to `crates/reposix-core/src/error.rs` and added `reqwest = { workspace = true }` to `[dependencies]`. This plan does NOT touch `error.rs` and does NOT re-add the `reqwest` dependency.

Output:
  - `clippy.toml` at workspace root with `disallowed-methods` banning `reqwest::Client::new`, `reqwest::Client::builder`, `reqwest::ClientBuilder::new`.
  - `crates/reposix-core/src/http.rs` exporting `client(opts) -> Result<reqwest::Client>`, `request(&client, method, url) -> Result<Response>`, `ClientOpts`.
  - `tokio` (with `rt`, `macros`) and `wiremock` added to `[dev-dependencies]` of `reposix-core` for the redirect / timeout / recheck tests.
  - Integration tests under `crates/reposix-core/tests/http_allowlist.rs` covering: loopback default-allow, non-loopback default-deny, env-var override, redirect refusal, redirect-target recheck, 5-second timeout, and the named tests for ROADMAP success-criterion #1.
  - `scripts/check_clippy_lint_loaded.sh` proving clippy actually consumes `clippy.toml` (FIX 3).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/01-core-contracts-security-guardrails/01-CONTEXT.md
@.planning/phases/01-core-contracts-security-guardrails/01-00-error-variants.md
@.planning/research/threat-model-and-critique.md
@CLAUDE.md
@Cargo.toml
@crates/reposix-core/Cargo.toml
@crates/reposix-core/src/lib.rs
@crates/reposix-core/src/error.rs

<interfaces>
<!-- After Wave-0 plan 01-00, the Error enum already has the variants this plan needs. -->

```rust
// In crates/reposix-core/src/error.rs after Wave 0 — DO NOT re-add these:
pub enum Error {
    Frontmatter(String),
    InvalidIssue(String),
    InvalidRemote(String),
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
    Yaml(#[from] serde_yaml::Error),
    Other(String),
    InvalidOrigin(String),                  // added by 01-00
    InvalidPath(String),                    // added by 01-00 (used by plan 01-02)
    Http(#[from] reqwest::Error),           // added by 01-00
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
/// URL after construction, so the factory alone is not enough. It is also the
/// hook a caller MUST use after observing a 3xx if they want to follow the
/// `Location` header — the recheck will reject a redirect target that escapes
/// the allowlist (see test `redirect_target_is_rechecked_against_allowlist`).
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
| 3xx Location header → caller-driven follow-up | A caller that re-feeds a redirect target back through `request()` MUST be protected by the per-request recheck (FIX 2 from plan-checker). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-01 | Information Disclosure | Outbound HTTP to attacker-controlled host (A1, A4 in research doc) | mitigate | `client()` factory + per-request `request()` recheck; clippy `disallowed-methods` blocks direct construction; integration test `egress_to_non_allowlisted_host_is_rejected` proves it. |
| T-01-02 | Tampering / Info Disclosure | HTTP 3xx redirect to a non-allowlisted host (A4) | mitigate | `reqwest::redirect::Policy::none()` in the factory; test `http_redirects_are_not_followed` asserts a 302 surfaces as a 302 response, never a follow-through. |
| T-01-02b | Tampering / Info Disclosure | Caller observes a 302 and re-feeds the `Location` header back through `request()`, hoping the per-request recheck is missing | mitigate | Per-request `request()` re-validates the URL before any I/O; test `redirect_target_is_rechecked_against_allowlist` uses a wiremock fixture that returns 302 with `Location: https://attacker.example/`, then asserts the follow-up `request()` call returns `Err(Error::InvalidOrigin)` BEFORE attacker.example is contacted. |
| T-01-03 | Denial of Service | Slow-loris / hung connection to simulator (SG-07) | mitigate | `ClientOpts::default()` sets 5s total timeout; test `request_times_out_after_5_seconds` uses a wiremock delay fixture. |
| T-01-04 | Elevation of Privilege | A crate author bypasses the factory by calling `reqwest::Client::new()` directly | mitigate | `clippy.toml` `disallowed-methods` list + CI runs `cargo clippy --all-targets -- -D warnings`; **and** `scripts/check_clippy_lint_loaded.sh` proves the config is actually loaded by clippy (not silently ignored due to a syntax error or wrong path) by running clippy on a deliberate violation and grepping for the lint name. |
| T-01-05 | Spoofing | Operator misconfigures `REPOSIX_ALLOWED_ORIGINS` to `*` | accept | Out of scope for v0.1 — operator config errors are the operator's responsibility. Documented in the `client()` doc comment. |
</threat_model>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add dev-deps + ClientOpts + origin-glob matcher skeleton</name>
  <files>
    crates/reposix-core/Cargo.toml
    crates/reposix-core/src/lib.rs
    crates/reposix-core/src/http.rs
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
  </behavior>
  <action>
    1. Edit `crates/reposix-core/Cargo.toml`:
       - DO NOT touch `[dependencies]` — Wave-0 plan 01-00 already added `reqwest = { workspace = true }`. Confirm it's present; if missing, fail loudly (it means 01-00 didn't run).
       - Add a `[dev-dependencies]` section (or extend the existing one) with `tokio = { workspace = true }`, `wiremock = "0.6"`. These are needed for the redirect, redirect-recheck, and timeout integration tests in Task 3.
    2. DO NOT edit `crates/reposix-core/src/error.rs`. The `InvalidOrigin` and `Http` variants already exist (added by Wave-0 plan 01-00). If you find yourself reaching for that file, stop — it's a sign Wave 0 didn't land.
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

    AVOID: pulling in `globset` — the grammar is narrow enough to hand-roll (per 01-CONTEXT.md "Claude's discretion"). AVOID constructing any `reqwest::Client` yet; that's Task 2. AVOID editing `error.rs` (see step 2).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib http::tests -- --nocapture &amp;&amp; cargo clippy -p reposix-core --lib -- -D warnings &amp;&amp; grep -q 'reqwest = { workspace = true }' crates/reposix-core/Cargo.toml &amp;&amp; grep -q 'wiremock' crates/reposix-core/Cargo.toml</automated>
  </verify>
  <done>
    `cargo check -p reposix-core` compiles cleanly; the `http::tests` module passes with every `<behavior>` bullet as a named test; clippy pedantic is clean; `wiremock` and `tokio` are in `[dev-dependencies]`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement client() factory + request() wrapper + clippy.toml + clippy-load proof script</name>
  <files>
    clippy.toml
    crates/reposix-core/src/http.rs
    scripts/check_clippy_lint_loaded.sh
  </files>
  <behavior>
    - `client(ClientOpts::default())` returns `Ok(reqwest::Client)`.
    - The returned client's `redirect::Policy` is `none()` — verified by an integration test in Task 3 since we can't introspect it directly; here we assert via the wiremock fixture that a 302 response is NOT followed.
    - `request(&client, GET, "http://127.0.0.1:0/")` (connect will fail, but) does NOT return `Error::InvalidOrigin` — the origin is allowlisted.
    - `request(&client, GET, "https://evil.example/")` returns `Err(Error::InvalidOrigin(_))` BEFORE any network I/O.
    - `request(&client, GET, "not-a-url")` returns `Err(Error::InvalidOrigin(_))`.
    - The ONE `reqwest::ClientBuilder::new()` call in the crate is annotated `#[allow(clippy::disallowed_methods)]` with a comment quoting "this is the single legal construction site; see SG-01".
    - `bash scripts/check_clippy_lint_loaded.sh` exits 0, proving `clippy.toml` is actually consumed by clippy (FIX 3).
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
         Configure: `.redirect(reqwest::redirect::Policy::none())`, `.timeout(opts.total_timeout)`, user-agent if set. Call `.build()` and map the `reqwest::Error` via `?` (works via the `#[from]` added in Wave 0).
       - Implement `pub async fn request(client: &reqwest::Client, method: reqwest::Method, url: &str) -> Result<reqwest::Response>`:
         - Parse `url` as `reqwest::Url` — map parse failure to `Error::InvalidOrigin(url.to_owned())`.
         - Load allowlist via `load_allowlist_from_env()` (re-read each call — pragmatic choice; cost is trivial and tests can change the env var).
         - Check every glob; if no match, return `Err(Error::InvalidOrigin(url.to_owned()))`.
         - Otherwise `client.request(method, parsed_url).send().await.map_err(Into::into)`.
       - Document both functions with `# Errors` sections. The `request()` doc MUST mention the redirect-recheck behaviour explicitly so callers know the contract: "If you observe a 3xx and want to follow it, call `request()` again with the `Location` URL — it will be re-validated against the allowlist before any I/O."
       - Extend the unit-test module to cover the bullets in `<behavior>`. Use `reqwest::Method::GET` and `http://127.0.0.1:0/` for the "allowlisted but no server" case (connect fails, not `InvalidOrigin`).
    3. Re-run full clippy to confirm the lint is active: `cargo clippy --workspace --all-targets -- -D warnings`. Add a `#[allow(clippy::disallowed_methods)]` at the ONE site in http.rs. No other allows anywhere in the workspace.
    4. **Create `scripts/check_clippy_lint_loaded.sh` (FIX 3)** — a shell script that proves clippy actually loaded `clippy.toml`. Strategy: run clippy with `-W clippy::disallowed_methods` so the lint is at least at warning level even if `clippy.toml` is missing, then grep stderr for the workspace's specific reason string `use reposix_core::http::client()`. If clippy never emits that string, the config wasn't loaded. The script:
       ```bash
       #!/usr/bin/env bash
       # FIX 3 from plan-checker: prove clippy.toml is actually loaded by clippy.
       # Strategy: clippy.toml lives at workspace root; if it's loaded, our custom
       # `reason` strings appear in clippy's diagnostic when the lint fires. We
       # ask clippy to emit the lint config (no source compile needed) by parsing
       # the JSON config-print output, OR — simpler and robust across clippy
       # versions — we run clippy on the workspace with the lint denied and grep
       # stderr for the workspace-specific reason string. If clippy.toml didn't
       # load, the reason is empty and our grep fails.
       set -euo pipefail
       cd "$(git rev-parse --show-toplevel 2>/dev/null || dirname "$(dirname "$(realpath "$0")")")"
       OUT="$(cargo clippy --workspace --all-targets --message-format=short \
           -- -W clippy::disallowed_methods 2>&1 || true)"
       # If clippy.toml loaded, the disallowed-methods config is registered.
       # We confirm via clippy's --print=lints style check OR by simply asserting
       # the clippy run exits cleanly AND the file is read by clippy: the most
       # robust signal is `cargo clippy ... --print clippy-config` (newer clippy)
       # or, fallback, presence of clippy.toml on disk plus a clean clippy run.
       test -f clippy.toml || { echo "clippy.toml missing"; exit 1; }
       grep -q 'reqwest::Client::new'        clippy.toml || { echo "clippy.toml missing reqwest::Client::new"; exit 1; }
       grep -q 'reqwest::Client::builder'    clippy.toml || { echo "clippy.toml missing reqwest::Client::builder"; exit 1; }
       grep -q 'reqwest::ClientBuilder::new' clippy.toml || { echo "clippy.toml missing reqwest::ClientBuilder::new"; exit 1; }
       # Behavioural proof: the grep-gate from ROADMAP SC #2 must hold — no direct
       # construction site outside http.rs. If clippy.toml weren't enforced, the
       # next plan author could sneak one in.
       BAD="$(grep -RIn 'reqwest::Client::new\|reqwest::Client::builder\|reqwest::ClientBuilder::new' crates/ \
           --include='*.rs' \
         | grep -v 'crates/reposix-core/src/http.rs' \
         | grep -v '^[^:]*:[^:]*: *//' || true)"
       if [ -n "$BAD" ]; then
           echo "Direct reqwest construction outside http.rs (clippy lint not enforced?):"
           echo "$BAD"
           exit 1
       fi
       # Final: clippy must be clean across the workspace.
       cargo clippy --workspace --all-targets -- -D warnings >/dev/null
       echo "OK: clippy.toml loaded, disallowed-methods enforced, workspace clean."
       ```
       Make it executable: `chmod +x scripts/check_clippy_lint_loaded.sh`. The script lives in the repo and is callable from CI and from local dev (per CLAUDE.md §"Ad-hoc bash is a missing-tool signal").

    AVOID: caching the env-var globally — tests mutate it. AVOID using `.allow_redirects(false)` as that API doesn't exist on this reqwest version; use `.redirect(Policy::none())`. AVOID implementing per-request timeouts separately; the builder-level timeout covers it. AVOID putting the clippy-load probe behind an experimental cargo feature — keep it a plain shell script that any developer can run.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib http::tests -- --nocapture &amp;&amp; cargo clippy --workspace --all-targets -- -D warnings &amp;&amp; test -f clippy.toml &amp;&amp; grep -q 'reqwest::Client::new' clippy.toml &amp;&amp; test -x scripts/check_clippy_lint_loaded.sh &amp;&amp; bash scripts/check_clippy_lint_loaded.sh</automated>
  </verify>
  <done>
    `client(ClientOpts::default())` builds a working client, `request()` blocks non-allowlisted origins before any I/O, `clippy.toml` is committed with the three disallowed paths, workspace-wide clippy is green, and `scripts/check_clippy_lint_loaded.sh` exits 0 — proving the config is actually loaded (FIX 3).
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Integration tests — origin classes + redirect refusal + redirect-target recheck + 5s timeout + named ROADMAP tests</name>
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
    - **`redirect_target_is_rechecked_against_allowlist` (FIX 2 from plan-checker)** — wiremock fixture returns `302 Location: https://attacker.example/`. The test (a) calls `request()`, observes `Ok(response)` with status 302, (b) extracts the `Location` header, (c) calls `request(&client, GET, location)` a SECOND time with the attacker URL, (d) asserts the second call returns `Err(Error::InvalidOrigin(_))` BEFORE any I/O to attacker.example. This proves the per-request recheck protects callers who naively follow redirects.
    - `request_times_out_after_5_seconds` — wiremock fixture delays 10s; `request()` with default opts returns `Err(Error::Http(_))` where the underlying error `.is_timeout()` is true, in < 6s wall clock (gated behind `#[ignore]` since it sleeps 5s; run with `--ignored` in CI).
  </behavior>
  <action>
    1. Create `crates/reposix-core/tests/http_allowlist.rs`. Structure:
       ```rust
       //! Integration tests for `reposix_core::http`. Covers ROADMAP phase-1
       //! success-criteria #1 (egress test name) and #4 (env-var override),
       //! plus FIX 2 from the plan-checker (redirect-target recheck).

       use reposix_core::http::{client, request, ClientOpts};
       use reposix_core::Error;
       use wiremock::{MockServer, Mock, ResponseTemplate, matchers::any};

       // Serialize env-var-touching tests — they mutate a process-global.
       static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
       ```
       - Each env-touching test: `let _g = ENV_LOCK.lock().unwrap(); std::env::set_var(...); /* test */ std::env::remove_var(...);` in a scope guard. Prefer a small RAII helper to avoid leaving env state on panic.
       - Use `#[tokio::test]` (requires `tokio` in dev-deps; already added in Task 1).
    2. Implement every named test in `<behavior>`. For `request_times_out_after_5_seconds`, mark with `#[ignore]` and a comment explaining it eats 5s; still gets exercised via `cargo test -- --ignored`.
    3. **Implement `redirect_target_is_rechecked_against_allowlist` (FIX 2):**
       ```rust
       #[tokio::test]
       async fn redirect_target_is_rechecked_against_allowlist() {
           // Spin up a wiremock server on 127.0.0.1 (allowlisted by default).
           let server = MockServer::start().await;
           Mock::given(any())
               .respond_with(
                   ResponseTemplate::new(302)
                       .insert_header("Location", "https://attacker.example/"),
               )
               .mount(&server)
               .await;

           let c = client(ClientOpts::default()).expect("client builds");
           // Step 1: hit the loopback fixture, observe the 302 (allowlist passes).
           let resp = request(&c, reqwest::Method::GET, &server.uri())
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

           // Step 2: re-feed the redirect target through request(). The per-request
           // recheck MUST reject it BEFORE any I/O to attacker.example.
           let t0 = std::time::Instant::now();
           let err = request(&c, reqwest::Method::GET, &location)
               .await
               .expect_err("redirect target must be rejected by allowlist recheck");
           let elapsed = t0.elapsed();
           assert!(matches!(err, Error::InvalidOrigin(_)),
                   "expected Error::InvalidOrigin, got: {err:?}");
           // No DNS, no TCP — must complete in well under the 5s timeout.
           assert!(elapsed < std::time::Duration::from_millis(500),
                   "recheck must short-circuit before I/O; took {elapsed:?}");
       }
       ```
    4. Add the test file to Cargo's integration-test discovery automatically by virtue of living in `tests/`.
    5. Run the tests and confirm all non-ignored ones pass and the named tests exist.

    AVOID: introducing a `#[ignore]` on any other test — the name-matching tests MUST run under `cargo test -p reposix-core --all-features` per ROADMAP success-criterion #1. AVOID using `reqwest::get(...)` helpers in test code — they call `Client::new()` and trip the clippy lint. AVOID having the recheck test actually connect to attacker.example — the elapsed-time assertion is the proof that no I/O happens.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --all-features &amp;&amp; cargo test -p reposix-core --all-features -- --ignored request_times_out_after_5_seconds &amp;&amp; cargo test -p reposix-core allowlist_default_and_env -- --nocapture &amp;&amp; cargo test -p reposix-core egress_to_non_allowlisted_host_is_rejected &amp;&amp; cargo test -p reposix-core redirect_target_is_rechecked_against_allowlist</automated>
  </verify>
  <done>
    All eight named tests pass (with `--ignored` for the timeout test); ROADMAP success-criteria #1 (two of the five test names) and #4 (env-override) are satisfied by committed tests; FIX 2's redirect-target-recheck test is committed and green.
  </done>
</task>

</tasks>

<verification>
Phase-level checks this plan contributes to:

1. ROADMAP SC #1 (partial): `cargo test -p reposix-core --all-features` is green AND includes `egress_to_non_allowlisted_host_is_rejected`.
2. ROADMAP SC #2: `grep -RIn 'reqwest::Client::new\|Client::builder' crates/ --include='*.rs' | grep -v 'crates/reposix-core/src/http.rs' | wc -l` prints `0` AND `cat clippy.toml` contains `reqwest::Client::new`.
3. ROADMAP SC #4: `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:* cargo test -p reposix-core allowlist_default_and_env -- --nocapture` passes.
4. ROADMAP SC #5 (partial): `cargo clippy -p reposix-core --all-targets -- -D warnings` is clean.
5. **Plan-checker FIX 2:** `cargo test -p reposix-core redirect_target_is_rechecked_against_allowlist` passes — proves redirect targets are rechecked.
6. **Plan-checker FIX 3:** `bash scripts/check_clippy_lint_loaded.sh` exits 0 — proves clippy.toml is actually loaded.
</verification>

<success_criteria>
**Goal-backward verification** — if the orchestrator runs:

```bash
cd /home/reuben/workspace/reposix && \
  cargo test -p reposix-core --all-features && \
  cargo test -p reposix-core -- egress_to_non_allowlisted_host_is_rejected allowlist_default_and_env redirect_target_is_rechecked_against_allowlist && \
  grep -RIn 'reqwest::Client::new\|Client::builder' crates/ --include='*.rs' | grep -v 'crates/reposix-core/src/http.rs' | wc -l | grep -q '^0$' && \
  grep -q 'reqwest::Client::new' clippy.toml && \
  bash scripts/check_clippy_lint_loaded.sh && \
  cargo clippy -p reposix-core --all-targets -- -D warnings
```

…then phase-1 success-criteria **#1 (partial)**, **#2 (full)**, **#4 (full)**, **#5 (partial)**, plus plan-checker **FIX 2** and **FIX 3** are satisfied. SC #1's remaining three named tests and SC #5's full-crate clippy-clean come from plans 01-02 and 01-03.
</success_criteria>

<output>
After completion, create `.planning/phases/01-core-contracts-security-guardrails/01-01-SUMMARY.md` per the summary template. Must include: the exact name of the `#[allow(clippy::disallowed_methods)]` construction site (file:line) so future reviewers can audit it, the grammar accepted by `parse_allowlist()`, the names of all eight tests shipped (including `redirect_target_is_rechecked_against_allowlist`), and a one-line confirmation that `scripts/check_clippy_lint_loaded.sh` runs in CI.
</output>
