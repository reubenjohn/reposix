---
phase: 21
plan: C
type: execute
wave: 3
depends_on: [21-A]
files_modified:
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/main.rs
autonomous: true
requirements:
  - HARD-02
  - HARD-05
user_setup: []
tags: [hardening, confluence, truncation, tenant-redaction, SG-05]

must_haves:
  truths:
    - "User calling ConfluenceBackend::list_issues on a space with > 500 pages gets Ok(first 500) AND a tracing::warn! with the pages count"
    - "User calling ConfluenceBackend::list_issues_strict on a space with > 500 pages gets Err(::Other(...)) and NO partial Ok result"
    - "User running `reposix list --backend confluence --no-truncate` receives an error (non-zero exit) when the backend would have truncated"
    - "The error message returned by list_issues on HTTP failure contains only `url.path_and_query()` — never the full URL with tenant hostname"
  artifacts:
    - path: "crates/reposix-confluence/src/lib.rs"
      provides: "list_issues_strict concrete method + redact_url helper + URL-stripped error messages"
      contains: "pub async fn list_issues_strict"
    - path: "crates/reposix-cli/src/list.rs"
      provides: "ListBackend::Confluence branch honours no_truncate flag"
      contains: "no_truncate"
    - path: "crates/reposix-cli/src/main.rs"
      provides: "#[arg(long)] no_truncate: bool on the List subcommand"
      contains: "no_truncate"
  key_links:
    - from: "crates/reposix-cli/src/main.rs"
      to: "crates/reposix-cli/src/list.rs"
      via: "List.no_truncate arg threaded into run() invocation"
      pattern: "no_truncate"
    - from: "crates/reposix-cli/src/list.rs"
      to: "crates/reposix-confluence/src/lib.rs"
      via: "if no_truncate { backend.list_issues_strict(&project).await } else { backend.list_issues(&project).await }"
      pattern: "list_issues_strict"
---

<objective>
Close HARD-02 (SG-05 silent-truncation taint escape) and HARD-05 (tenant-URL leak in tracing logs) in a single plan. They share the same file (`crates/reposix-confluence/src/lib.rs`) and both involve sanitising what `ConfluenceBackend` exposes to the caller.

Purpose: Per threat-model SG-05, "a silent truncation is a taint escape — the agent thinks it has the whole space when it doesn't". Per OP-7, "Tenant-name leakage" in 429 logs violates defence-in-depth when tracing is shipped to third-party observability. This plan fixes both.

Output: `ConfluenceBackend::list_issues_strict` method (Err on cap); `--no-truncate` CLI flag wiring; URL-path-only error messages in list_issues; unit tests for all three.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/CONTEXT.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md
@crates/reposix-confluence/src/lib.rs
@crates/reposix-cli/src/main.rs
@crates/reposix-cli/src/list.rs

<interfaces>
<!-- Contracts the executor modifies -->

From crates/reposix-confluence/src/lib.rs (current):
```rust
const MAX_ISSUES_PER_LIST: usize = 500;   // line 92

#[async_trait]
impl IssueBackend for ConfluenceBackend {
    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> {
        // lines 747–820: paginated loop; at cap emits tracing::warn! and returns Ok(capped)
    }
}
```

From crates/reposix-cli/src/main.rs (List subcommand):
```rust
List {
    project: String,
    #[arg(long, default_value = "http://127.0.0.1:7878")]
    origin: String,
    #[arg(long, value_enum, default_value_t = list::ListBackend::Sim)]
    backend: list::ListBackend,
    #[arg(long, value_enum, default_value_t = list::ListFormat::Json)]
    format: list::ListFormat,
    // ADD: #[arg(long)] no_truncate: bool  (Confluence-only)
}
```

From crates/reposix-cli/src/list.rs (dispatcher):
```rust
pub async fn run(
    project: &str,
    origin: &str,
    backend: ListBackend,
    format: ListFormat,
    // ADD: no_truncate: bool,
) -> anyhow::Result<()> {
    // ... dispatches over `ListBackend::Confluence` → ConfluenceBackend::list_issues
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task C1: ConfluenceBackend::list_issues_strict + tenant-URL redaction in error paths</name>
  <files>
    crates/reposix-confluence/src/lib.rs
  </files>
  <read_first>
    - crates/reposix-confluence/src/lib.rs (focus: lines 92 [MAX_ISSUES_PER_LIST], 547–578 [ingest_rate_limit], 747–820 [list_issues pagination loop + error construction])
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md (section "Pattern 2" and "Pattern 3")
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md (section "crates/reposix-confluence/src/lib.rs")
  </read_first>
  <behavior>
    - NEW pub async fn list_issues_strict(&self, project: &str) -> Result<Vec<Issue>>
      - Identical pagination loop to list_issues
      - At the `pages > cap` branch: return Err(Error::Other(format!("Confluence space '{}' exceeds {MAX_ISSUES_PER_LIST}-page cap; refusing to truncate (strict mode)", project)))
      - At the `out.len() >= MAX_ISSUES_PER_LIST` branch: same Err behaviour
    - MODIFY list_issues: unchanged behaviour (still returns Ok(capped) with warn), except:
    - MODIFY every `format!("... {url}: ...")` site in list_issues error construction to use a new `redact_url` helper that returns `url::Url::parse(&url).map(|u| u.path_and_query().map(ToString::to_string).unwrap_or_else(|| u.path().to_string())).unwrap_or_else(|_| "<url parse error>".into())`
    - Refactor: extract the shared pagination loop into `async fn list_issues_impl(&self, project: &str, strict: bool) -> Result<Vec<Issue>>` — both public methods delegate to it
    - Three new #[cfg(test)] unit tests:
      1. `truncation_warn_on_default_list` — mock 600 pages, assert list_issues returns Ok(len == 500), assert tracing captured contains "reached MAX_ISSUES_PER_LIST cap" (use `tracing-subscriber` test layer; mirror any existing tracing-test pattern in the crate)
      2. `truncation_errors_in_strict_mode` — same 600-page mock, assert list_issues_strict returns Err whose Display contains the string `strict mode` and `500-page cap`
      3. `list_error_message_omits_tenant` — mock returns 500 Internal on GET; assert the resulting Err's Display does NOT contain the full origin host (e.g. `reuben-john.atlassian.net`), only the `/wiki/api/v2/...` path
  </behavior>
  <action>
    Open `crates/reposix-confluence/src/lib.rs`.

    **Step 1 — redact helper.** Add near the top of the `impl ConfluenceBackend` block or as a free-standing private fn near the other helpers:
    ```rust
    /// Extract only the path+query from an HTTP URL so tenant hostnames never
    /// appear in error messages or tracing spans (see OP-7 HARD-05).
    fn redact_url(raw: &str) -> String {
        url::Url::parse(raw)
            .map(|u| {
                let path = u.path();
                match u.query() {
                    Some(q) => format!("{path}?{q}"),
                    None => path.to_string(),
                }
            })
            .unwrap_or_else(|_| "<url parse error>".to_string())
    }
    ```
    Confirm `url` crate is already a dep (it is — `reqwest::Url` re-exports it; check `Cargo.toml` and add `url = "2"` only if necessary).

    **Step 2 — refactor list_issues.** Extract the pagination loop to a private helper:
    ```rust
    async fn list_issues_impl(&self, project: &str, strict: bool) -> Result<Vec<Issue>> {
        // ... the existing loop, with both truncation sites gated on `strict`:
        //   if pages > cap {
        //       if strict {
        //           return Err(Error::Other(format!(
        //               "Confluence space '{project}' exceeds {MAX_ISSUES_PER_LIST}-page cap; \
        //                refusing to truncate (strict mode)"
        //           )));
        //       }
        //       tracing::warn!(pages, "reached MAX_ISSUES_PER_LIST cap; stopping pagination");
        //       break;
        //   }
        //   if out.len() >= MAX_ISSUES_PER_LIST {
        //       if strict {
        //           return Err(Error::Other(format!(
        //               "Confluence space '{project}' exceeds {MAX_ISSUES_PER_LIST}-page cap; \
        //                refusing to truncate (strict mode)"
        //           )));
        //       }
        //       return Ok(out);
        //   }
        // ... also change `for GET {url}` format args to `for GET {}`, redact_url(&url)
    }
    ```

    **Step 3 — expose both public methods.** In `impl ConfluenceBackend`:
    ```rust
    impl ConfluenceBackend {
        /// Strict variant that errors rather than silently truncating. Confluence-only.
        /// # Errors
        /// Returns `Error::Other` if pagination would exceed `MAX_ISSUES_PER_LIST`
        /// plus any error the default `list_issues` would raise.
        pub async fn list_issues_strict(&self, project: &str) -> Result<Vec<Issue>> {
            self.list_issues_impl(project, true).await
        }
    }

    #[async_trait]
    impl IssueBackend for ConfluenceBackend {
        async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> {
            self.list_issues_impl(project, false).await
        }
        // ... other methods unchanged
    }
    ```

    **Step 4 — replace every `{url}` in error construction** in list_issues_impl. Audit sites (per RESEARCH.md line 782–784 and PATTERNS.md §lib.rs). Each site that today reads `format!("confluence returned {status} for GET {url}: ...")` becomes `format!("confluence returned {status} for GET {}: ...", redact_url(&url))`.

    **Step 5 — add unit tests.** Look at the existing test module in `lib.rs` (there's a substantial one starting around line 1100+ based on the grep output). Follow its wiremock + `ConfluenceBackend::with_http_client` pattern. Lower the cap for testing by introducing a `#[cfg(test)]` override of `MAX_ISSUES_PER_LIST` *only if the existing tests don't already have a pattern*; otherwise just mock 501+ pages using paginated wiremock responses.

    Preferred: extract `MAX_ISSUES_PER_LIST` behind a helper that takes a cap parameter, keep the const at 500 for prod, and have tests call `list_issues_impl_with_cap(project, strict, 5)` to keep mock responses small (6 pages). If extracting a test-only helper is ugly, accept 501 pages in the mock — wiremock can enumerate fast.

    Add a `tracing_test` dev-dep only if needed; otherwise rely on a recorded subscriber. Simplest test for the warn: use `tracing_subscriber::fmt::TestWriter` captured into a `Vec<u8>` under `Arc<Mutex<...>>` and assert on the captured bytes.

    For test 3 (tenant redaction): set up wiremock on 127.0.0.1:<ephemeral>, point the backend at `http://<mock-host>:<port>/wiki`, return 500 for the first page GET, and assert the Err's Display does NOT contain the host:port segment. It's a 127.0.0.1 host in tests, so checking "does NOT contain `:<port>`" is the practical assertion. Also assert the Display DOES contain `/wiki/api/v2/` (the path portion survived).

    **Step 6** — run:
    ```
    cargo test -p reposix-confluence --locked truncation_ 
    cargo test -p reposix-confluence --locked list_error_message_omits_tenant
    cargo test -p reposix-confluence --locked   # full crate — no regressions
    cargo clippy -p reposix-confluence --all-targets -- -D warnings
    ```
    All must pass.

    Commit: `git add crates/reposix-confluence/src/lib.rs && git commit -m "feat(21-C): list_issues_strict + tenant-URL redaction (HARD-02 HARD-05)"`
  </action>
  <verify>
    <automated>cargo test -p reposix-confluence --locked truncation_ && cargo test -p reposix-confluence --locked list_error_message_omits_tenant && cargo clippy -p reposix-confluence --all-targets -- -D warnings</automated>
  </verify>
  <acceptance_criteria>
    - `grep -qE "pub async fn list_issues_strict" crates/reposix-confluence/src/lib.rs`
    - `grep -qE "fn list_issues_impl" crates/reposix-confluence/src/lib.rs` (the shared helper)
    - `grep -qE "fn redact_url" crates/reposix-confluence/src/lib.rs`
    - `grep -q "strict mode" crates/reposix-confluence/src/lib.rs` (the error string)
    - `grep -cE "for GET \{url\}" crates/reposix-confluence/src/lib.rs` returns 0 (no un-redacted `{url}` in error strings in this file)
    - `grep -cE "^\s*(#\[test\]|#\[tokio::test\]).*\n.*fn truncation_" crates/reposix-confluence/src/lib.rs` — at least 2 matches (two truncation tests)
    - `grep -q "fn list_error_message_omits_tenant" crates/reposix-confluence/src/lib.rs`
    - `cargo test -p reposix-confluence --locked` exits 0
    - `cargo clippy -p reposix-confluence --all-targets -- -D warnings` exits 0
  </acceptance_criteria>
  <done>
    Strict mode landed; warn-mode unchanged; error messages URL-redacted; three new unit tests green.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task C2: Wire --no-truncate flag through CLI → ConfluenceBackend::list_issues_strict</name>
  <files>
    crates/reposix-cli/src/main.rs,
    crates/reposix-cli/src/list.rs
  </files>
  <read_first>
    - crates/reposix-cli/src/main.rs (focus: lines 98–135, the List subcommand)
    - crates/reposix-cli/src/list.rs (focus: the `ListBackend::Confluence` branch around lines 77+)
    - crates/reposix-confluence/src/lib.rs (confirm list_issues_strict signature from C1)
  </read_first>
  <behavior>
    - `reposix list --backend confluence --no-truncate --project <space>` invokes `ConfluenceBackend::list_issues_strict`
    - `reposix list --backend confluence` (without --no-truncate) still invokes `ConfluenceBackend::list_issues` (unchanged behaviour)
    - `reposix list --backend sim --no-truncate` is accepted silently (the flag is a no-op for non-Confluence backends — documented in help text)
    - A new integration test in `crates/reposix-cli/tests/` (or #[cfg(test)] unit test in list.rs — match whichever pattern already exists) asserts the flag is plumbed through
  </behavior>
  <action>
    **Step 1 — main.rs: add --no-truncate to the List subcommand variant.** Following the existing `#[arg(long, ...)]` style (per PATTERNS.md §crates/reposix-cli/src/main.rs), add after the `format` field:
    ```rust
    List {
        project: String,
        #[arg(long, default_value = "http://127.0.0.1:7878")]
        origin: String,
        #[arg(long, value_enum, default_value_t = list::ListBackend::Sim)]
        backend: list::ListBackend,
        #[arg(long, value_enum, default_value_t = list::ListFormat::Json)]
        format: list::ListFormat,
        /// Error instead of silently capping at 500 pages (Confluence only).
        /// No-op for --backend sim and --backend github.
        #[arg(long)]
        no_truncate: bool,
    },
    ```

    **Step 2 — thread the flag to list::run.** Find the `Cmd::List { ... } => list::run(...).await?` dispatch site in main.rs and append `no_truncate` as a new argument to `list::run(...)`.

    **Step 3 — list.rs: update the `run` signature and Confluence branch.**
    ```rust
    pub async fn run(
        project: &str,
        origin: &str,
        backend: ListBackend,
        format: ListFormat,
        no_truncate: bool,
    ) -> anyhow::Result<()> {
        let issues = match backend {
            ListBackend::Sim => { /* unchanged — no_truncate ignored */ }
            ListBackend::Github => { /* unchanged — no_truncate ignored */ }
            ListBackend::Confluence => {
                let creds = /* existing creds load */;
                let backend = reposix_confluence::ConfluenceBackend::new(creds)?;
                if no_truncate {
                    backend.list_issues_strict(project).await?
                } else {
                    backend.list_issues(project).await?
                }
            }
        };
        // ... existing formatting/output
    }
    ```

    **Step 4 — add a unit or integration test.** Simplest path: in list.rs, add a `#[cfg(test)]` module that imports `reposix_confluence` behind a wiremock server and asserts that `run(project, origin, ListBackend::Confluence, ListFormat::Json, true)` returns an error when the mock serves > cap pages, and returns Ok when `no_truncate == false`.

    If the CLI crate doesn't already have wiremock in dev-dependencies, prefer testing this indirectly by adding an integration test in `crates/reposix-cli/tests/no_truncate.rs` that spawns a real wiremock server and invokes the CLI binary via `std::process::Command::new(env!("CARGO_BIN_EXE_reposix"))`. Assert non-zero exit + stderr contains "strict mode".

    Test name must contain `no_truncate` so `cargo test --locked no_truncate` targets it.

    **Step 5** — run:
    ```
    cargo test --workspace --locked no_truncate
    cargo test --workspace --locked            # full sweep — no regressions
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check
    ```
    All must pass. Also confirm `cargo run -p reposix-cli -- list --help 2>&1 | grep -q "no-truncate"`.

    Commit: `git add crates/reposix-cli/src/main.rs crates/reposix-cli/src/list.rs crates/reposix-cli/tests/no_truncate.rs && git commit -m "feat(21-C): --no-truncate flag on reposix list (HARD-02 CLI surface)"`
  </action>
  <verify>
    <automated>cargo test --workspace --locked no_truncate && cargo clippy --workspace --all-targets -- -D warnings && cargo run -p reposix-cli -- list --help 2>&1 | grep -q "no-truncate"</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q "no_truncate" crates/reposix-cli/src/main.rs`
    - `grep -q "no_truncate" crates/reposix-cli/src/list.rs`
    - `grep -q "list_issues_strict" crates/reposix-cli/src/list.rs`
    - `cargo run -p reposix-cli -- list --help 2>&1 | grep -q "no-truncate"` succeeds (flag is in help text)
    - At least one test file or test function contains `no_truncate` and exercises the strict path
    - `cargo test --workspace --locked` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
  </acceptance_criteria>
  <done>
    `--no-truncate` plumbed end to end; CLI help advertises the flag; integration test proves the plumbing.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Confluence tenant HTTP response → ConfluenceBackend | tainted text, attacker-influenced |
| ConfluenceBackend::list_issues → agent consumer | "is this the whole space?" boundary — SG-05 |
| Error messages / tracing → third-party observability | tenant name must not cross this boundary — HARD-05 |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-C-01 | Information Disclosure | list_issues error messages | mitigate | `redact_url(&url)` strips host/tenant; test `list_error_message_omits_tenant` asserts the assertion fails loud |
| T-21-C-02 | Elevation of Privilege (SG-05 taint escape) | list_issues at 500-page cap | mitigate | Strict mode returns Err; agents that pass `--no-truncate` cannot be misled into believing they saw the whole space |
| T-21-C-03 | Information Disclosure | tracing::warn! in rate-limit path | accept | RESEARCH.md VERIFIED: ingest_rate_limit warn at lines 563–569 does NOT include URL. No change needed. Documented as verified, not modified. |
| T-21-C-04 | Tampering | --no-truncate flag ignored by non-Confluence backends | accept | Documented as no-op in help text; future backends that gain pagination caps MUST error instead of silently truncating (add to per-backend tests at that time) |
</threat_model>

<verification>
- `cargo test --workspace --locked` green
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- `cargo run -p reposix-cli -- list --help` shows `--no-truncate`
- Grep `crates/reposix-confluence/src/lib.rs` for any `\{url\}` format specifier in error construction → 0 matches
</verification>

<success_criteria>
HARD-02 closes: silent truncation is now an opt-in feature, not a default. HARD-05 closes: tenant hostnames cannot leak through list_issues error messages. Both tested.
</success_criteria>

<output>
After completion, create `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-C-SUMMARY.md` with: list_issues_strict signature, --no-truncate help snippet, test count delta, before/after error message example (redacted).
</output>
