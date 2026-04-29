---
phase: 73-connector-contract-gaps
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-confluence/tests/auth_header.rs
  - crates/reposix-github/tests/auth_header.rs
  - crates/reposix-jira/tests/list_records_excludes_attachments.rs
  - quality/catalogs/doc-alignment.json
  - docs/benchmarks/token-economy.md
  - CLAUDE.md
  - .planning/phases/73-connector-contract-gaps/SUMMARY.md
  - quality/reports/verdicts/p73/status-before.txt
  - quality/reports/verdicts/p73/summary-before.json
  - quality/reports/verdicts/p73/status-after.txt
  - quality/reports/verdicts/p73/summary-after.json
autonomous: false
parallelization: false
gap_closure: false
cross_ai: false
task_count: 11
requirements:
  - CONNECTOR-GAP-01
  - CONNECTOR-GAP-02
  - CONNECTOR-GAP-03
  - CONNECTOR-GAP-04

must_haves:
  truths:
    - "4 catalog rows in `quality/catalogs/doc-alignment.json` transition from MISSING_TEST to BOUND (or 1 → RETIRE_PROPOSED if path (b) chosen for token-economy row per D-05)."
    - "2 NEW Rust integration test files exist and pass against `cargo test -p reposix-confluence -p reposix-github -p reposix-jira` invocations (per-crate, sequential, per D-09)."
    - "Confluence `auth_header` test asserts byte-exact `Authorization: Basic <base64(email:token)>` via `wiremock::matchers::header_exact` (NOT regex per D-02)."
    - "GitHub `auth_header` test asserts byte-exact `Authorization: Bearer <token>` via `header_exact` (the live adapter at `crates/reposix-github/src/lib.rs:236` confirms `Bearer ` prefix; D-02)."
    - "JIRA `list_records_excludes_attachments` test seeds a wiremock issue containing `fields.attachment = [{...}]` AND `fields.comment.comments = [{...}]`, calls `JiraBackend::list_records`, and asserts the rendered Record `body` (markdown) and `extensions` map exclude both fields (D-03 — rendering boundary, NOT JSON parse layer)."
    - "`docs/connectors/guide/real-backend-smoke-fixture` is a pure REBIND to `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence` — zero new test code (D-04)."
    - "Stale `docs/benchmarks/token-economy.md:23-28` JIRA row resolved by path (a) prose update (default) OR path (b) propose-retire (fallback) per D-05 ROI threshold; executor flips path inside Task 8 if path (a) exceeds 30 minutes."
    - "Wiremock fixtures inline as JSON literals — NO new fixture-fragment library (D-06)."
    - "All cargo invocations across the 3 affected crates are SEQUENTIAL (D-09 / CLAUDE.md `Build memory budget`); workspace-wide cargo only at phase close if needed."
    - "CLAUDE.md gains a P73 H3 subsection ≤30 lines under `## v0.12.1 — in flight`; banned-words check passes (D-10)."
    - "Verifier subagent verdict at `quality/reports/verdicts/p73/VERDICT.md` is graded GREEN by an unbiased dispatched subagent (top-level orchestrator action — D-07)."
  artifacts:
    - path: "crates/reposix-confluence/tests/auth_header.rs"
      provides: "Wiremock-based byte-exact `Authorization: Basic <b64>` assertion using `header_exact` against `ConfluenceBackend` driven through the `BackendConnector` trait."
      min_lines: 60
    - path: "crates/reposix-github/tests/auth_header.rs"
      provides: "Wiremock-based byte-exact `Authorization: Bearer <token>` assertion against `GithubBackend`."
      min_lines: 60
    - path: "crates/reposix-jira/tests/list_records_excludes_attachments.rs"
      provides: "Wiremock-based assertion that JIRA-side `fields.attachment` + `fields.comment.comments` are absent from Record.body and Record.extensions after `list_records` translation (the rendering boundary)."
      min_lines: 80
    - path: "quality/reports/verdicts/p73/status-before.txt"
      provides: "BEFORE snapshot of `doc-alignment status --top 30` for verdict-time delta computation."
      min_lines: 5
    - path: "quality/reports/verdicts/p73/status-after.txt"
      provides: "AFTER snapshot showing the 4 P73 rows BOUND (or 3 BOUND + 1 RETIRE_PROPOSED if path (b))."
      min_lines: 5
  key_links:
    - from: "quality/catalogs/doc-alignment.json"
      to: "crates/reposix-{confluence,github}/tests/auth_header.rs"
      via: "row.tests[] (TestRef::RustFn `<file>::<fn_name>`) populated by `reposix-quality doc-alignment bind --test`"
      pattern: "tests/auth_header.rs::"
    - from: "quality/catalogs/doc-alignment.json"
      to: "crates/reposix-jira/tests/list_records_excludes_attachments.rs"
      via: "row.tests[] (TestRef::RustFn) for `attachments-comments-excluded` row"
      pattern: "list_records_excludes_attachments.rs::"
    - from: "quality/catalogs/doc-alignment.json"
      to: "crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence"
      via: "row.tests[] (TestRef::RustFn) for `real-backend-smoke-fixture` row — pure rebind per D-04"
      pattern: "agent_flow_real.rs::dark_factory_real_confluence"
    - from: "CLAUDE.md `## v0.12.1 — in flight`"
      to: "P73 H3 subsection naming the 3 new test files + path-(a)/(b) decision"
      via: "QG-07 grounding rule"
      pattern: "### P73 — Connector contract gaps"

user_setup: []
---

<objective>
Bind 4 `MISSING_TEST` rows in `quality/catalogs/doc-alignment.json` (the connector-authoring + JIRA-shape cluster) to behavioral tests. Two rows require NEW Rust integration tests using `wiremock` 0.6: a byte-exact `Authorization` header assertion (Confluence + GitHub) and a "JIRA `list_records` excludes `attachment` + `comment` fields from the rendered markdown body" assertion. One row is a pure REBIND to an existing `dark_factory_real_*` smoke test (zero new code). One row's source prose is STALE (claims JIRA real adapter "not implemented" but it shipped in v0.11.x Phase 29) — executor picks path (a) prose update + bind OR path (b) propose-retire per the D-05 ROI threshold, defaulting to path (a) unless a blocker surfaces.

The walker (P71 schema 2.0) hashes both source prose AND each `--test` citation (Rust fn body or shell file body); drift on either fires `STALE_DOCS_DRIFT` and the next maintainer reviews. This phase concretizes the connector contract claims at `docs/guides/write-your-own-connector.md:158, 172-173` and `docs/decisions/005-jira-issue-mapping.md:79-87` — historically these were "we-know-it's-true-but-no-test-binds-it" rows the walker had no way to detect drift on.

Purpose: close 4 of the remaining 13 MISSING_TEST rows targeted by the v0.12.1 autonomous-run cluster (P72-P74); raise `alignment_ratio` toward the v0.12.1 0.85 target; ground the next agent in the `wiremock + header_exact` pattern and the rebind-vs-author-vs-retire decision tree.

Output:
- 3 new Rust test files (2 auth-header + 1 jira attachments/comments excluded).
- 1 pure rebind of the real-backend-smoke-fixture row to `dark_factory_real_confluence`.
- 1 prose update (path (a)) OR `propose-retire` (path (b)) for the stale JIRA token-economy row.
- 4 catalog rows transitioned `MISSING_TEST` → `BOUND` (or 3 BOUND + 1 RETIRE_PROPOSED) after `refresh`.
- CLAUDE.md P73 H3 subsection (≤30 lines).
- Phase summary at `.planning/phases/73-connector-contract-gaps/SUMMARY.md` with verifier-dispatch flag.
- BEFORE/AFTER `doc-alignment status` snapshots captured for verifier dispatch.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/HANDOVER-v0.12.1.md
@.planning/phases/73-connector-contract-gaps/CONTEXT.md
@.planning/phases/72-lint-config-invariants/PLAN.md
@.planning/phases/72-lint-config-invariants/SUMMARY.md
@.planning/milestones/v0.12.1-phases/ROADMAP.md
@.planning/milestones/v0.12.1-phases/REQUIREMENTS.md
@.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md
@.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md
@quality/PROTOCOL.md
@quality/catalogs/README.md
@CLAUDE.md
@docs/guides/write-your-own-connector.md
@docs/decisions/005-jira-issue-mapping.md
@docs/benchmarks/token-economy.md

<interfaces>
<!-- Key contracts the executor needs. Extracted from codebase. Use these directly. -->

## `BackendConnector::list_records` trait (from `crates/reposix-core/src/backend.rs:235`)

```rust
async fn list_records(&self, project: &str) -> Result<Vec<Record>>;
```

`Record` (from `crates/reposix-core/src/lib.rs`):
```rust
pub struct Record {
    pub id: RecordId,
    pub title: String,
    pub status: RecordStatus,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
    pub body: String,                                        // markdown
    pub parent_id: Option<RecordId>,
    pub extensions: BTreeMap<String, serde_yaml::Value>,
}
```

## GitHub auth header construction (from `crates/reposix-github/src/lib.rs:235-237`)

```rust
if let Some(ref tok) = self.token {
    h.push(("Authorization", format!("Bearer {tok}")));
}
```

So GitHub's exact header is `Authorization: Bearer <tok>` — assert with `wiremock::matchers::header_exact("Authorization", "Bearer test-token-xyz")`.

## Confluence auth header construction (from `crates/reposix-confluence/src/translate.rs:13-26`)

`basic_auth_header(email, token)` produces `format!("Basic {}", STANDARD.encode("{email}:{token}"))`. Use the same `base64::engine::general_purpose::STANDARD` to compute the expected value:

```rust
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
let creds = format!("{email}:{token}");
let expected = format!("Basic {}", STANDARD.encode(creds.as_bytes()));
// then `wiremock::matchers::header_exact("Authorization", expected.as_str())`
```

## JIRA `list_records` rendering boundary (from `crates/reposix-jira/src/translate.rs:101-167`)

`translate(JiraIssue)` produces a `Record` whose `body` comes from `adf_to_markdown(fields.description)` and whose `extensions` is a `BTreeMap` containing ONLY: `jira_key`, `issue_type`, `priority?`, `status_name`, `hierarchy_level`. **Neither `attachment` nor `comment` are deserialized into `JiraFields` at all** — `JIRA_FIELDS` (in `crates/reposix-jira/src/types.rs:45`) is the explicit field allowlist sent in the search request. The test ASSERTION is at the rendering boundary: even if the wiremock response carries `fields.attachment` and `fields.comment.comments`, neither leaks into Record.body or Record.extensions.

## `JiraBackend::new_with_base_url` (constructor for wiremock tests)

```rust
let creds = JiraCreds {
    email: "test@example.com".into(),
    api_token: "token".into(),
};
let backend = JiraBackend::new_with_base_url(creds, server.uri())
    .expect("JiraBackend::new_with_base_url");
```

(Pattern from existing `crates/reposix-jira/tests/contract.rs:223-228`.)

## `ConfluenceBackend` and `GithubBackend` constructors

- Confluence: pattern in `crates/reposix-confluence/tests/contract.rs` — `ConfluenceBackend::new_with_base_url(ConfluenceCreds { email, api_token, tenant }, server.uri())`. Inspect the existing test for exact constructor signature and adapt.
- GitHub: pattern in `crates/reposix-github/tests/contract.rs:233+`. Constructor accepts a token and base URL override; reuse the same shape.

## Existing wiremock pattern (from `crates/reposix-confluence/src/client.rs:1215-1240`)

The Confluence crate already ships `BasicAuthMatches` (a custom `wiremock::Match` impl) that verifies byte-exact Basic auth. P73 does NOT need to redo that work for unit tests; this phase's value is at the **integration-test seam** (driving through the public `BackendConnector` trait via `tests/auth_header.rs` so the catalog can bind the public surface, not the private helper).

## `reposix-quality doc-alignment bind` (canonical command shape, Rust fn `--test`)

```
target/release/reposix-quality doc-alignment bind \
  --catalog quality/catalogs/doc-alignment.json \
  --row-id <ROW_ID> \
  --claim "<one-sentence behavioral claim>" \
  --source <FILE>:<LSTART>-<LEND> \
  --test crates/<crate>/tests/<file>.rs::<fn_name> \
  --grade GREEN \
  --rationale "<short rationale>"
```

The `<file>.rs::<fn_name>` form parses to `TestRef::RustFn` per `parse_test` in
`crates/reposix-quality/src/commands/doc_alignment.rs:1187-1205`. Body hash is
the named fn's source bytes (gix-walked). Multi-test rows accept `--test`
repeated; `bind` validates ALL `--test` citations BEFORE mutating the catalog.

For the SHELL file form (e.g. a verifier script), drop the `::fn`:

```
  --test crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence
```

is a Rust fn ref. ONE test fn per row per current bind semantics — pick the
canonical one (CONTEXT D-04 says `dark_factory_real_confluence`).

## The 4 catalog rows (verbatim from doc-alignment.json)

| # | row id | source | test target |
|---|---|---|---|
| 1 | `docs/connectors/guide/auth-header-exact-test` | docs/guides/write-your-own-connector.md:158-158 | `crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact` AND `crates/reposix-github/tests/auth_header.rs::auth_header_bearer_byte_exact` (multi-test row, `--test` repeated) |
| 2 | `docs/connectors/guide/real-backend-smoke-fixture` | docs/guides/write-your-own-connector.md:172-173 | `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence` (rebind only — D-04 picks the canonical one) |
| 3 | `docs/decisions/005-jira-issue-mapping/attachments-comments-excluded` | docs/decisions/005-jira-issue-mapping.md:79-87 | `crates/reposix-jira/tests/list_records_excludes_attachments.rs::list_records_excludes_attachments_and_comments` |
| 4 | `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` | docs/benchmarks/token-economy.md:23-28 | path (a): bind to a verifier asserting `crates/reposix-jira/Cargo.toml` exists (e.g. shell verifier `quality/gates/docs-alignment/verifiers/jira-adapter-shipped.sh`); path (b): `propose-retire` |

> Resolve exact source line ranges by inspecting the rows themselves with
> `jq '.rows[] | select(.id==\"<row-id>\") | .source' quality/catalogs/doc-alignment.json`
> when running `bind` (use the row's existing `source.line_start`/`source.line_end` —
> do NOT hand-pick lines).

</interfaces>
</context>

<tasks>

<!-- ============================================================== -->
<!-- WAVE 1 — Catalog-first scaffold + BEFORE snapshot.              -->
<!-- ============================================================== -->

<task type="auto">
  <name>Task 1: Capture BEFORE status snapshot + scaffold 3 stub test files</name>
  <files>
    crates/reposix-confluence/tests/auth_header.rs,
    crates/reposix-github/tests/auth_header.rs,
    crates/reposix-jira/tests/list_records_excludes_attachments.rs,
    quality/reports/verdicts/p73/status-before.txt,
    quality/reports/verdicts/p73/summary-before.json
  </files>
  <action>
    Catalog-first commit per `quality/PROTOCOL.md` § Step 3. The walker hashes
    the named test fn bodies via `gix`-walked source bytes; pinning EMPTY-but-
    parseable test fns BEFORE the bind runs makes the catalog row's
    `tests[]` citation a stable hash target. Implementations land in Wave 2.

    1. Build the binary if missing: `cargo build --release -p reposix-quality`
       (one cargo invocation; per D-09, nothing else compiles in this task).
    2. Capture BEFORE snapshot:
       ```bash
       mkdir -p quality/reports/verdicts/p73
       target/release/reposix-quality doc-alignment status --top 30 \
         > quality/reports/verdicts/p73/status-before.txt
       jq '.summary | {alignment_ratio, claims_total, claims_bound, claims_missing_test}' \
         quality/catalogs/doc-alignment.json \
         > quality/reports/verdicts/p73/summary-before.json
       ```
    3. Create 3 STUB Rust test files. Each stub MUST:
       - Have a `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` annotation
         (matches the existing pattern in `tests/contract.rs` for both crates).
       - Define the EXACT fn name the catalog will bind to (so the body hash is
         pinned).
       - Body: a single `unimplemented!("P73 task N implements this")` so the
         file compiles + the fn is parseable but `cargo test` would panic if
         actually invoked (catches premature bind-then-skip-impl drift).

       Stub `crates/reposix-confluence/tests/auth_header.rs`:
       ```rust
       //! P73 CONNECTOR-GAP-01: byte-exact Basic-auth header assertion via
       //! wiremock + `BackendConnector` trait seam. Stub committed in
       //! Wave 1; implementation lands in Wave 2 (Task 2).

       #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
       async fn auth_header_basic_byte_exact() {
           unimplemented!("P73 Task 2 — wiremock byte-exact Basic auth header assertion");
       }
       ```

       Stub `crates/reposix-github/tests/auth_header.rs`:
       ```rust
       //! P73 CONNECTOR-GAP-01: byte-exact Bearer-auth header assertion via
       //! wiremock + `BackendConnector` trait seam. Stub committed in
       //! Wave 1; implementation lands in Wave 2 (Task 3).

       #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
       async fn auth_header_bearer_byte_exact() {
           unimplemented!("P73 Task 3 — wiremock byte-exact Bearer auth header assertion");
       }
       ```

       Stub `crates/reposix-jira/tests/list_records_excludes_attachments.rs`:
       ```rust
       //! P73 CONNECTOR-GAP-03: assert JIRA list_records strips
       //! `fields.attachment` + `fields.comment.comments` at the rendering
       //! boundary (per docs/decisions/005-jira-issue-mapping.md:79-87).
       //! Stub committed in Wave 1; implementation lands in Wave 2 (Task 4).

       #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
       async fn list_records_excludes_attachments_and_comments() {
           unimplemented!("P73 Task 4 — wiremock + assert body/extensions exclusion");
       }
       ```

    4. Sanity: `cargo check -p reposix-confluence -p reposix-github -p reposix-jira`
       to confirm the stubs compile. **D-09: ONE cargo invocation.** If you must
       split (memory pressure), run them sequentially.

    5. Stage + commit:
       ```bash
       git add crates/reposix-confluence/tests/auth_header.rs \
               crates/reposix-github/tests/auth_header.rs \
               crates/reposix-jira/tests/list_records_excludes_attachments.rs \
               quality/reports/verdicts/p73/status-before.txt \
               quality/reports/verdicts/p73/summary-before.json
       git commit -m "P73(catalog-first): scaffold 3 stub connector-contract tests (CONNECTOR-GAP-01..03)"
       ```
       Cite CONNECTOR-GAP-01, CONNECTOR-GAP-02 (rebind, no stub needed),
       CONNECTOR-GAP-03, CONNECTOR-GAP-04 (handled in Wave 4) in the commit
       body so the audit trail covers all 4 rows even though only 3 stubs land.

    DO NOT bind any rows yet. DO NOT touch the catalog JSON yet.
  </action>
  <verify>
    <automated>
      cargo check -p reposix-confluence -p reposix-github -p reposix-jira 2>&1 \
        && test -f quality/reports/verdicts/p73/status-before.txt \
        && test -f quality/reports/verdicts/p73/summary-before.json \
        && grep -q "auth_header_basic_byte_exact" crates/reposix-confluence/tests/auth_header.rs \
        && grep -q "auth_header_bearer_byte_exact" crates/reposix-github/tests/auth_header.rs \
        && grep -q "list_records_excludes_attachments_and_comments" crates/reposix-jira/tests/list_records_excludes_attachments.rs \
        && git log -1 --format=%s | grep -q "P73(catalog-first)"
    </automated>
  </verify>
  <done>3 stub test files compile; BEFORE snapshot captured; one atomic commit `P73(catalog-first): scaffold ...` landed.</done>
</task>

<!-- ============================================================== -->
<!-- WAVE 2 — New Rust tests. Sequential by D-09 (memory budget).    -->
<!-- ============================================================== -->

<task type="auto">
  <name>Task 2: Implement Confluence Basic-auth byte-exact wiremock test</name>
  <files>crates/reposix-confluence/tests/auth_header.rs</files>
  <action>
    Replace the stub with a real wiremock-based byte-exact assertion driving
    the public `BackendConnector::list_records` seam.

    Pattern (use `crates/reposix-confluence/tests/contract.rs` as the reference
    for `ConfluenceBackend` construction and the v2 endpoint shapes; D-06: keep
    the JSON literal MINIMAL — one space, one page, inline):

    ```rust
    //! P73 CONNECTOR-GAP-01: byte-exact Basic-auth header assertion.
    //! Drives `ConfluenceBackend` through the public `BackendConnector` trait
    //! and asserts the `Authorization` header on the resulting wiremock
    //! request matches `Basic <base64(email:token)>` exactly. Per
    //! docs/guides/write-your-own-connector.md:158, byte-exact prefix is the
    //! contract — `wiremock::matchers::header_exact` (NOT regex) per D-02.

    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
    use reposix_core::backend::BackendConnector;
    use wiremock::matchers::{header_exact, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn auth_header_basic_byte_exact() {
        // Test creds chosen to make the base64 distinctive.
        let email = "test-user@example.com";
        let token = "atlassian-api-token-xyz123";
        let expected_header = format!(
            "Basic {}",
            STANDARD.encode(format!("{email}:{token}").as_bytes())
        );

        let server = MockServer::start().await;

        // Confluence v2 spaces resolver — minimal payload (D-06).
        // Inspect tests/contract.rs to confirm the exact v2 endpoint sequence
        // ConfluenceBackend::list_records issues; mount each with
        // `.and(header_exact("Authorization", expected_header.as_str()))`.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .and(query_param("keys", "DEMO"))
            .and(header_exact("Authorization", expected_header.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"id": "1001", "key": "DEMO"}]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages"))
            .and(header_exact("Authorization", expected_header.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [],
                "_links": {}
            })))
            .mount(&server)
            .await;

        let creds = ConfluenceCreds {
            email: email.to_string(),
            api_token: token.to_string(),
            // Inspect ConfluenceCreds for any other fields and fill defaults.
        };
        let backend = ConfluenceBackend::new_with_base_url(creds, server.uri())
            .expect("ConfluenceBackend::new_with_base_url");

        // Driving list_records exercises the public BackendConnector seam.
        // wiremock panics on drop if any mounted Mock receives no matching
        // request — so a wrong header here causes a clear failure.
        let _ = backend.list_records("DEMO").await.expect("list_records ok");
    }
    ```

    Implementation notes:
    - **Inspect `tests/contract.rs`** for the EXACT `ConfluenceCreds` field
      shape and `ConfluenceBackend::new_with_base_url` signature; the snippet
      above is illustrative. The minimum viable test mounts ONE wiremock
      endpoint that the backend hits (with `header_exact`), and lets wiremock
      panic-on-drop catch the mismatch. If `list_records` requires more than
      one endpoint, mount the FULL chain with `header_exact` on every mount —
      MissingMatch failures point at the offending header difference.
    - **D-02:** `header_exact`, NOT `header_regex`. The whole point is byte-
      exactness; regex masks real bugs.
    - **D-06:** No fixture-fragment library. JSON literals inline. Spec says
      "minimal" — one space, one page is enough.
    - **D-08 (eager-resolution):** if the live Confluence adapter actually
      sends a wrong header today (test FAILS), check the constraint:
      < 1 hour fix, no new dep, no new file outside the planned set →
      eager-fix in this same task and note in commit body. Else append to
      `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` per OP-8 and
      leave the test RED (P76 absorbs).

    Run locally: `cargo test -p reposix-confluence --test auth_header
    auth_header_basic_byte_exact`. **D-09: ONE cargo invocation; do not
    parallelize with the other crates' tests.**

    Commit: `P73: implement Confluence Basic-auth byte-exact test (CONNECTOR-GAP-01)`.
  </action>
  <verify>
    <automated>
      cargo test -p reposix-confluence --test auth_header auth_header_basic_byte_exact 2>&1
    </automated>
  </verify>
  <done>Confluence test passes against wiremock; commit landed citing CONNECTOR-GAP-01.</done>
</task>

<task type="auto">
  <name>Task 3: Implement GitHub Bearer-auth byte-exact wiremock test</name>
  <files>crates/reposix-github/tests/auth_header.rs</files>
  <action>
    Mirror Task 2 for GitHub. The auth shape is `Authorization: Bearer <tok>`
    per `crates/reposix-github/src/lib.rs:236` — confirmed during planning.

    ```rust
    //! P73 CONNECTOR-GAP-01: byte-exact Bearer-auth header assertion via
    //! wiremock against `GithubBackend` driven through the `BackendConnector`
    //! trait seam. Per docs/guides/write-your-own-connector.md:158.

    use reposix_core::backend::BackendConnector;
    use reposix_github::GithubBackend;   // confirm exact import via tests/contract.rs
    use wiremock::matchers::{header_exact, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn auth_header_bearer_byte_exact() {
        let token = "ghp_test_personal_access_token_xyz";
        let expected_header = format!("Bearer {token}");

        let server = MockServer::start().await;

        // Mount the minimal endpoint sequence list_records hits.
        // Inspect tests/contract.rs for exact path shape (e.g.
        // `/repos/<owner>/<repo>/issues`).
        Mock::given(method("GET"))
            .and(path("/repos/acme/demo/issues"))
            .and(header_exact("Authorization", expected_header.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;

        // Construct the backend with the test token + base URL pointed at
        // wiremock. Inspect tests/contract.rs for the exact constructor.
        let backend = GithubBackend::new_with_base_url(
            "acme/demo",
            Some(token.to_string()),
            server.uri(),
        )
        .expect("GithubBackend::new_with_base_url");

        let _ = backend.list_records("acme/demo").await.expect("list_records ok");
    }
    ```

    - **Inspect `crates/reposix-github/tests/contract.rs`** for the actual
      constructor signature + the wiremock endpoint sequence
      `GithubBackend::list_records` hits. Copy that pattern; adapt only the
      `header_exact` matcher.
    - **D-02:** `header_exact` only.
    - **D-06:** inline JSON; minimal.
    - **D-08 (eager-resolution):** same gate as Task 2.

    Run locally: `cargo test -p reposix-github --test auth_header
    auth_header_bearer_byte_exact`. **D-09: SEQUENTIAL — wait for Task 2's
    cargo invocation to finish before invoking here.**

    Commit: `P73: implement GitHub Bearer-auth byte-exact test (CONNECTOR-GAP-01)`.
  </action>
  <verify>
    <automated>
      cargo test -p reposix-github --test auth_header auth_header_bearer_byte_exact 2>&1
    </automated>
  </verify>
  <done>GitHub test passes against wiremock; commit landed citing CONNECTOR-GAP-01.</done>
</task>

<task type="auto">
  <name>Task 4: Implement JIRA list_records-excludes-attachments-and-comments test</name>
  <files>crates/reposix-jira/tests/list_records_excludes_attachments.rs</files>
  <action>
    Replace the stub with a wiremock-based assertion at the **rendering
    boundary** (D-03 — NOT the JSON parse layer). Seed wiremock with a JIRA
    issue payload that includes `fields.attachment` AND
    `fields.comment.comments`; call `list_records`; assert the resulting
    `Record.body` (markdown) AND `Record.extensions` exclude both.

    Pattern (use `crates/reposix-jira/tests/contract.rs:153-231` for fixture
    + constructor shape; D-06 keeps the JSON minimal but extends `fields` with
    the two adversarial keys):

    ```rust
    //! P73 CONNECTOR-GAP-03: assert JIRA list_records does NOT leak
    //! `fields.attachment` or `fields.comment.comments` into the rendered
    //! Record (body OR extensions). Per docs/decisions/005-jira-issue-mapping.md:79-87.
    //!
    //! D-03: this asserts at the RENDERING boundary, not at the JSON parse
    //! layer. Even if the wiremock response carries the adversarial fields,
    //! `JiraBackend::list_records` must produce a Record whose user-visible
    //! surfaces (body markdown + extensions map) name neither.

    use reposix_core::backend::BackendConnector;
    use reposix_jira::{JiraBackend, JiraCreds};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn issue_with_attachments_and_comments() -> serde_json::Value {
        serde_json::json!({
            "id": "10042",
            "key": "PROJ-42",
            "fields": {
                "summary": "Issue with attachments and comments",
                "description": serde_json::Value::Null,
                "status": {
                    "name": "Open",
                    "statusCategory": {"key": "new"}
                },
                "resolution": serde_json::Value::Null,
                "assignee": serde_json::Value::Null,
                "labels": [],
                "created": "2025-01-01T00:00:00.000+0000",
                "updated": "2025-12-01T10:30:00.000+0000",
                "parent": serde_json::Value::Null,
                "issuetype": {"name": "Task", "hierarchyLevel": 0},
                "priority": {"name": "Medium"},
                // Adversarial: these fields MUST NOT leak into the Record.
                "attachment": [
                    {
                        "id": "99",
                        "filename": "secret-payload.txt",
                        "content": "https://example.invalid/secret-payload.txt"
                    }
                ],
                "comment": {
                    "comments": [
                        {
                            "id": "1001",
                            "body": "do-not-leak-this-comment-body-into-record",
                            "author": {"displayName": "Mallory"}
                        }
                    ],
                    "total": 1
                }
            }
        })
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn list_records_excludes_attachments_and_comments() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/api/3/search/jql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issues": [issue_with_attachments_and_comments()],
                "isLast": true
            })))
            .mount(&server)
            .await;

        let creds = JiraCreds {
            email: "test@example.com".into(),
            api_token: "token".into(),
        };
        let backend = JiraBackend::new_with_base_url(creds, server.uri())
            .expect("JiraBackend::new_with_base_url");

        let records = backend.list_records("PROJ").await.expect("list_records ok");
        assert_eq!(records.len(), 1, "expected single issue from wiremock");
        let record = &records[0];

        // (1) Body markdown must not name attachments.
        assert!(
            !record.body.to_lowercase().contains("attachment"),
            "Record.body leaked attachment data: {:?}",
            record.body
        );
        assert!(
            !record.body.contains("secret-payload.txt"),
            "Record.body leaked attachment filename: {:?}",
            record.body
        );

        // (2) Body markdown must not name comments.
        assert!(
            !record.body.to_lowercase().contains("comment"),
            "Record.body leaked comment data: {:?}",
            record.body
        );
        assert!(
            !record.body.contains("do-not-leak-this-comment-body-into-record"),
            "Record.body leaked comment body: {:?}",
            record.body
        );

        // (3) Extensions map must contain ONLY allowlisted keys.
        let leaked_keys: Vec<&String> = record
            .extensions
            .keys()
            .filter(|k| {
                let lower = k.to_lowercase();
                lower.contains("attachment") || lower.contains("comment")
            })
            .collect();
        assert!(
            leaked_keys.is_empty(),
            "Record.extensions leaked attachment/comment keys: {:?}",
            leaked_keys
        );
    }
    ```

    Implementation notes:
    - **D-03 is load-bearing:** the assertion is on `Record.body` and
      `Record.extensions` — the rendering boundary. We do NOT assert on the
      raw JSON or on `JiraFields` deserialization (which would be the parse
      layer).
    - **D-06:** inline JSON literal; the `attachment` array and `comment`
      object are the minimum adversarial seed. No fixture-fragment library.
    - The lowercase `.contains("comment")` and `.contains("attachment")` checks
      are CONSERVATIVE — if a future Record extension legitimately needs the
      word "comment" (e.g. "comment_count"), this test will fail loudly. That
      is the intended drift signal: any future change widening the field
      allowlist gets reviewed.
    - **Ergonomic concern (eager-fix candidate per D-08):** if a CURRENT
      `extensions` key contains "comment" or "attachment" as a substring (e.g.
      a "commentary_url"), the test fails NOW. Inspect existing keys via
      `crates/reposix-jira/src/translate.rs:130-152` — current keys are
      `jira_key, issue_type, priority, status_name, hierarchy_level`. None
      collide. If a collision DOES surface, < 1 hour rename = eager-fix in
      same task; else SURPRISES-INTAKE.md.

    Run locally: `cargo test -p reposix-jira --test list_records_excludes_attachments`.
    **D-09: SEQUENTIAL — wait for Task 3's cargo invocation to finish.**

    Commit: `P73: implement JIRA list_records attachments/comments-excluded test (CONNECTOR-GAP-03)`.
  </action>
  <verify>
    <automated>
      cargo test -p reposix-jira --test list_records_excludes_attachments 2>&1
    </automated>
  </verify>
  <done>JIRA test passes; Record.body and Record.extensions confirmed clean of adversarial fields; commit landed citing CONNECTOR-GAP-03.</done>
</task>

<!-- ============================================================== -->
<!-- WAVE 3 — Rebind real-backend smoke fixture (no code change).    -->
<!-- ============================================================== -->

<task type="auto">
  <name>Task 5: Bind `real-backend-smoke-fixture` row to existing dark_factory_real_confluence (rebind only)</name>
  <files>quality/catalogs/doc-alignment.json</files>
  <action>
    Per D-04: this row is a PURE REBIND. The 3 `dark_factory_real_*` `#[ignore]`
    smoke fixtures already shipped in v0.11.x. Pick the canonical one
    (`dark_factory_real_confluence` per CONTEXT.md D-04 — TokenWorld is
    sanctioned for free mutation).

    ```bash
    BIN=target/release/reposix-quality
    CAT=quality/catalogs/doc-alignment.json
    SRC=$(jq -r '.rows[] | select(.id=="docs/connectors/guide/real-backend-smoke-fixture") |
                "\(.source.file):\(.source.line_start)-\(.source.line_end)"' "$CAT")
    "$BIN" doc-alignment bind \
      --catalog "$CAT" \
      --row-id "docs/connectors/guide/real-backend-smoke-fixture" \
      --claim "Each backend ships an #[ignore]-gated smoke fixture wiring real-credential end-to-end agent flow." \
      --source "$SRC" \
      --test "crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence" \
      --grade GREEN \
      --rationale "P73 CONNECTOR-GAP-02 per D-04: pure rebind to canonical existing test fn (TokenWorld is sanctioned for mutation per docs/reference/testing-targets.md). github + jira variants exist in the same file and are covered by parallel rows in v0.11.0-phases/REQUIREMENTS-md / arch-16."
    ```

    `bind` validates the test fn exists (gix-walks the file, looks up the
    fn) BEFORE mutating the catalog. Successful bind sets
    `last_verdict: BOUND` per `crates/reposix-quality/src/commands/doc_alignment.rs:285-308`.

    No new code. No new test file. No prose change.

    Commit: `P73: rebind real-backend-smoke-fixture to dark_factory_real_confluence (CONNECTOR-GAP-02)`.
  </action>
  <verify>
    <automated>
      jq -e '.rows[] | select(.id=="docs/connectors/guide/real-backend-smoke-fixture") |
              .last_verdict == "BOUND" and (.tests[0] | contains("dark_factory_real_confluence"))' \
              quality/catalogs/doc-alignment.json
    </automated>
  </verify>
  <done>Row state BOUND; `tests[0]` cites `dark_factory_real_confluence`; commit landed.</done>
</task>

<!-- ================================================================ -->
<!-- WAVE 4 — Stale prose decision (path (a) default; path (b) fallback). -->
<!-- ================================================================ -->

<task type="checkpoint:decision" gate="blocking">
  <name>Task 6: Decide path (a) prose-update OR path (b) propose-retire for stale JIRA row (D-05)</name>
  <files>docs/benchmarks/token-economy.md, quality/catalogs/doc-alignment.json</files>
  <action>Pause for the owner/orchestrator to confirm path (a) DEFAULT or path (b) FALLBACK for the stale JIRA token-economy row per D-05. Task 7 branches on the resume-signal. See the inline decision/context/options/resume-signal blocks below.</action>
  <verify><automated>echo checkpoint  # gate is the resume-signal; no autocheck</automated></verify>
  <done>Resume-signal received naming path (a) or path (b); Task 7 proceeds accordingly.</done>
  <decision>Path (a) prose-update + bind, OR path (b) propose-retire, for `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` (D-05)</decision>
  <context>
    Per D-05: the row's source prose at `docs/benchmarks/token-economy.md:23-28`
    claims the JIRA real adapter is "not yet implemented" — but it shipped in
    v0.11.x Phase 29. The decision is whether to keep the row alive (update
    prose to acknowledge the adapter exists, then bind to a cheap existence
    verifier) OR retire the row entirely (rationale: superseded by the JIRA
    adapter shipping in Phase 29; bench numbers tracked separately under
    perf-dim P67/P67-eq).

    **Default rule (D-05):** path (a) IF total work (prose edit + verifier
    setup + bind) < 30 min, ELSE path (b). Planner reads no blocker;
    recommend path (a).

    The executor confirms here at runtime. If the executor identifies a
    blocker during prose drafting (e.g. the row is bound by other rows that
    would cascade-break), pivot to path (b) and document the rationale in
    Task 7's commit body.
  </context>
  <options>
    <option id="path-a">
      <label>Path (a) — update prose + bind to existence verifier (DEFAULT)</label>
      <pros>
        - Preserves the row in the catalog (no `RETIRE_PROPOSED` queue burden
          for the owner).
        - Updates the table to reflect current reality (JIRA adapter EXISTS,
          bench numbers PENDING re-measurement).
        - Cheap verifier shell binds to `[ -f crates/reposix-jira/Cargo.toml ]`
          so any future deletion of the JIRA crate fires `STALE_TEST_DRIFT`.
      </pros>
      <cons>
        - Adds one shell verifier file to `quality/gates/docs-alignment/verifiers/`.
        - Re-measures only via prose; the actual bench numbers stay deferred to
          P67 perf-dim.
      </cons>
    </option>
    <option id="path-b">
      <label>Path (b) — propose-retire with rationale</label>
      <pros>
        - Zero new files. One `propose-retire` command + commit.
        - The row's intent (track JIRA real-adapter token count) is genuinely
          better tracked under perf-dim once P67 ships.
      </pros>
      <cons>
        - Adds to the owner's TTY-confirm queue (per HANDOVER bulk-confirm
          step 3).
        - The prose at `:23-28` STILL needs updating because the table claims
          "not yet implemented" — leaving stale prose visible is a regression.
        - Requires a SECOND prose edit anyway (the table row text), so path
          (b) is strictly a superset of path (a)'s prose work.
      </cons>
    </option>
  </options>
  <resume-signal>
    Reply with one of:
    - `path-a` (DEFAULT — recommended; planner reads no blocker)
    - `path-b` (only if executor surfaces a blocker; explain in Task 7 commit body)
    - `path-a (with notes: ...)` to proceed path (a) with caveats
  </resume-signal>
</task>

<task type="auto">
  <name>Task 7: Execute decided path for stale JIRA token-economy row</name>
  <files>
    docs/benchmarks/token-economy.md,
    quality/gates/docs-alignment/verifiers/jira-adapter-shipped.sh,
    quality/catalogs/doc-alignment.json
  </files>
  <action>
    Branch on Task 6's decision:

    ## Path (a) — update prose + bind to existence verifier

    1. **Prose update** at `docs/benchmarks/token-economy.md:23-28`. The
       current row 28 reads (verbatim):
       ```
       | Jira (real adapter) | — | — | — | — | N/A (adapter not yet implemented) |
       ```
       Replace with:
       ```
       | Jira (real adapter) | (pending re-measurement) | (pending) | (pending) | (pending) | TBD (adapter shipped v0.11.x; bench rerun deferred to perf-dim P67) |
       ```
       Keep total line length within ±10 chars of neighbors so the table
       renders consistently in mkdocs. Confirm with mkdocs-strict pre-push
       (deferred to phase close).

    2. **Verifier shell:** create `quality/gates/docs-alignment/verifiers/jira-adapter-shipped.sh`:
       ```bash
       #!/usr/bin/env bash
       set -euo pipefail
       SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
       REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../.." &> /dev/null && pwd)"
       MANIFEST="${REPO_ROOT}/crates/reposix-jira/Cargo.toml"
       if [[ ! -f "$MANIFEST" ]]; then
         echo "FAIL: ${MANIFEST} missing — JIRA real adapter crate was deleted; bench row at docs/benchmarks/token-economy.md:23-28 needs immediate update or retire" >&2
         exit 1
       fi
       echo "PASS: reposix-jira crate present (real adapter shipped v0.11.x; bench numbers tracked under perf-dim P67)"
       exit 0
       ```
       `chmod +x` it. Test locally: `bash quality/gates/docs-alignment/verifiers/jira-adapter-shipped.sh`.

    3. **Bind:**
       ```bash
       BIN=target/release/reposix-quality
       CAT=quality/catalogs/doc-alignment.json
       SRC=$(jq -r '.rows[] | select(.id=="docs/benchmarks/token-economy/jira-real-adapter-not-implemented") |
                    "\(.source.file):\(.source.line_start)-\(.source.line_end)"' "$CAT")
       "$BIN" doc-alignment bind \
         --catalog "$CAT" \
         --row-id "docs/benchmarks/token-economy/jira-real-adapter-not-implemented" \
         --claim "JIRA real-adapter crate exists in tree (bench numbers pending re-measurement under perf-dim P67)." \
         --source "$SRC" \
         --test "quality/gates/docs-alignment/verifiers/jira-adapter-shipped.sh" \
         --grade GREEN \
         --rationale "P73 CONNECTOR-GAP-04 path (a) per D-05: prose updated to reflect adapter shipped v0.11.x; verifier asserts manifest exists; full bench rerun deferred to perf-dim P67."
       ```

    4. Commit:
       `P73: path (a) — update token-economy JIRA prose + bind existence verifier (CONNECTOR-GAP-04)`.
       Note in commit body: "Row id name retains the historical 'not-implemented'
       slug — slug rename is a separate cosmetic change deferred to GOOD-TO-HAVES."

    ## Path (b) — propose-retire (fallback)

    1. **Prose update** at `docs/benchmarks/token-economy.md:28` — STILL
       required (D-05 cons). Same change as path (a) step 1.
    2. **Propose-retire:**
       ```bash
       target/release/reposix-quality doc-alignment propose-retire \
         --catalog quality/catalogs/doc-alignment.json \
         --row-id "docs/benchmarks/token-economy/jira-real-adapter-not-implemented" \
         --claim "JIRA real adapter not yet implemented (STALE — superseded v0.11.x Phase 29)" \
         --source "docs/benchmarks/token-economy.md:23-28" \
         --rationale "P73 CONNECTOR-GAP-04 path (b) per D-05: row's premise contradicted by reality. JIRA adapter shipped v0.11.x; bench-numbers tracking moves to perf-dim P67. Owner confirms retire in HANDOVER bulk-confirm step 3."
       ```
    3. Commit: `P73: path (b) — propose-retire token-economy JIRA row + prose update (CONNECTOR-GAP-04)`.
       Note in commit body: "Adds 1 entry to owner's RETIRE_PROPOSED queue; HANDOVER step 3 owner-TTY action."

    ## Either path

    Skip (no eager work). Continue to Task 8.

    **D-08 (eager-resolution):** if the prose update blocks on the connector-
    matrix-on-landing claim from P74 (i.e. the table is referenced elsewhere
    in a way that breaks), append to SURPRISES-INTAKE.md and leave row in its
    pre-Task-7 state.
  </action>
  <verify>
    <automated>
      ! grep -qF "N/A (adapter not yet implemented)" docs/benchmarks/token-economy.md \
        && jq -e '.rows[] | select(.id=="docs/benchmarks/token-economy/jira-real-adapter-not-implemented") |
                  (.last_verdict == "BOUND" or .last_verdict == "RETIRE_PROPOSED")' \
                  quality/catalogs/doc-alignment.json
    </automated>
  </verify>
  <done>Stale prose updated; row is either BOUND (path a) or RETIRE_PROPOSED (path b); commit landed citing CONNECTOR-GAP-04 + the chosen path.</done>
</task>

<!-- ================================================================ -->
<!-- WAVE 5 — Bind the 2 remaining rows (auth-header, attachments).    -->
<!-- ================================================================ -->

<task type="auto">
  <name>Task 8: Bind auth-header-exact-test row (multi-test) + attachments-comments-excluded row</name>
  <files>quality/catalogs/doc-alignment.json</files>
  <action>
    Build is already current (Task 1 did `cargo build --release -p
    reposix-quality`); rebuild only if needed.

    For the auth-header row: bind BOTH new test fns (multi-test row, `--test`
    repeated). For attachments-comments: single test fn.

    ```bash
    BIN=target/release/reposix-quality
    CAT=quality/catalogs/doc-alignment.json

    # ---- Row 1 of 2: auth-header (multi-test) ----
    SRC1=$(jq -r '.rows[] | select(.id=="docs/connectors/guide/auth-header-exact-test") |
                  "\(.source.file):\(.source.line_start)-\(.source.line_end)"' "$CAT")
    "$BIN" doc-alignment bind \
      --catalog "$CAT" \
      --row-id "docs/connectors/guide/auth-header-exact-test" \
      --claim "Connectors send byte-exact Authorization headers (Confluence: Basic <base64(email:token)>; GitHub: Bearer <token>)." \
      --source "$SRC1" \
      --test "crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact" \
      --test "crates/reposix-github/tests/auth_header.rs::auth_header_bearer_byte_exact" \
      --grade GREEN \
      --rationale "P73 CONNECTOR-GAP-01 per D-02: byte-exact via wiremock::matchers::header_exact (NOT regex); both backends covered through the BackendConnector trait seam."

    # ---- Row 2 of 2: attachments-comments-excluded ----
    SRC2=$(jq -r '.rows[] | select(.id=="docs/decisions/005-jira-issue-mapping/attachments-comments-excluded") |
                  "\(.source.file):\(.source.line_start)-\(.source.line_end)"' "$CAT")
    "$BIN" doc-alignment bind \
      --catalog "$CAT" \
      --row-id "docs/decisions/005-jira-issue-mapping/attachments-comments-excluded" \
      --claim "JIRA list_records strips fields.attachment + fields.comment.comments — neither leaks into Record.body or Record.extensions." \
      --source "$SRC2" \
      --test "crates/reposix-jira/tests/list_records_excludes_attachments.rs::list_records_excludes_attachments_and_comments" \
      --grade GREEN \
      --rationale "P73 CONNECTOR-GAP-03 per D-03: assertion at the rendering boundary (Record.body markdown + Record.extensions allowlist), NOT the JSON parse layer."
    ```

    `bind` validates each `--test` citation by gix-walking the named file +
    locating the fn body BEFORE mutating the catalog. Atomic: if either test
    fn name is wrong, NEITHER bind lands.

    Commit: `P73: bind auth-header (multi-test) + attachments-excluded rows (CONNECTOR-GAP-01,03)`.
  </action>
  <verify>
    <automated>
      jq -e '
        (.rows[] | select(.id=="docs/connectors/guide/auth-header-exact-test") |
          .last_verdict == "BOUND" and (.tests | length == 2)) and
        (.rows[] | select(.id=="docs/decisions/005-jira-issue-mapping/attachments-comments-excluded") |
          .last_verdict == "BOUND")
      ' quality/catalogs/doc-alignment.json
    </automated>
  </verify>
  <done>auth-header row BOUND with 2 test citations; attachments-excluded row BOUND with 1 test citation; commit landed.</done>
</task>

<!-- ================================================================ -->
<!-- WAVE 6 — Refresh + AFTER snapshot.                                -->
<!-- ================================================================ -->

<task type="auto">
  <name>Task 9: Run `doc-alignment refresh` + capture AFTER snapshot</name>
  <files>
    quality/catalogs/doc-alignment.json,
    quality/reports/verdicts/p73/status-after.txt,
    quality/reports/verdicts/p73/summary-after.json
  </files>
  <action>
    Re-walk the 3 source docs touched this phase to recompute source hashes
    and confirm rows hold their state on a no-op walk:

    ```bash
    target/release/reposix-quality doc-alignment refresh \
      docs/guides/write-your-own-connector.md \
      docs/decisions/005-jira-issue-mapping.md \
      docs/benchmarks/token-economy.md
    ```

    Capture AFTER snapshot:
    ```bash
    target/release/reposix-quality doc-alignment status --top 30 \
      > quality/reports/verdicts/p73/status-after.txt
    jq '.summary | {alignment_ratio, claims_total, claims_bound, claims_missing_test}' \
      quality/catalogs/doc-alignment.json \
      > quality/reports/verdicts/p73/summary-after.json
    ```

    **Sanity gate:** `claims_missing_test` must drop by:
    - 4 (path (a) chosen) — all 4 P73 rows BOUND, OR
    - 3 (path (b) chosen) — 3 BOUND + 1 RETIRE_PROPOSED (RETIRE_PROPOSED still
      decrements `missing_test` because it's no longer "we said we'd test
      this and didn't").

    If less, an upstream catalog state diverged; STOP and investigate before
    commit.

    Commit (only if catalog actually changed; refresh on a no-op may leave it
    byte-identical):
    `P73: doc-alignment refresh + AFTER snapshot (alignment_ratio delta captured)`.

    If catalog is byte-identical, commit only the snapshot files:
    `P73: capture AFTER status snapshot for verdict dispatch`.

    **Note on bind-verb hash bug (P75 fix):** if any of the 4 P73 rows flips
    back to `STALE_DOCS_DRIFT` after refresh on a no-op walk, that's the
    BIND-VERB-FIX-01 bug — append to SURPRISES-INTAKE.md (severity HIGH;
    discovered-by P73; severity-rationale: blocks verdict GREEN). P75 fixes;
    do NOT eager-fix in P73 (out of scope, > 1 hour).
  </action>
  <verify>
    <automated>
      test -f quality/reports/verdicts/p73/status-after.txt \
        && test -f quality/reports/verdicts/p73/summary-after.json \
        && [ "$(jq -r '.claims_missing_test' quality/reports/verdicts/p73/summary-after.json)" -le \
             "$(($(jq -r '.claims_missing_test' quality/reports/verdicts/p73/summary-before.json) - 3))" ]
    </automated>
  </verify>
  <done>refresh executed; AFTER snapshot captured; `claims_missing_test` dropped by ≥ 3 (4 if path (a)); commit landed.</done>
</task>

<!-- ================================================================ -->
<!-- WAVE 7 — CLAUDE.md update + phase SUMMARY + verifier dispatch flag. -->
<!-- ================================================================ -->

<task type="auto">
  <name>Task 10: CLAUDE.md P73 H3 subsection (≤30 lines, banned-words clean) per D-10</name>
  <files>CLAUDE.md</files>
  <action>
    Append a P73 H3 subsection under the existing `## v0.12.1 — in flight`
    section. Per D-10 + QG-07.

    Constraints:
    - **≤30 lines total** (heading + body).
    - Names the 3 new test files + 1 rebind + the path-(a)/(b) decision
      taken.
    - Notes the wiremock + `header_exact` pattern as the canonical
      connector-contract test idiom (next maintainer crib note).
    - Notes the eager-resolution decisions from Tasks 2-4 (if any auth/jira
      gaps surfaced; if none, state explicitly).
    - **Banned-words check passes** — run `bash scripts/banned-words-lint.sh
      CLAUDE.md` (or whichever banned-words verifier is canonical — find via
      `ls scripts/ | grep -i banned`) BEFORE committing.

    Required cross-references:
    - `quality/PROTOCOL.md` § "Subagents propose; tools validate and mint"
      (Principle A justification for the bind path).
    - CLAUDE.md `Build memory budget` (D-09 — sequential cargo per-crate).
    - CLAUDE.md OP-8 — eager-resolution / SURPRISES-INTAKE.md.

    DO NOT rewrite existing CLAUDE.md content — append only (anti-bloat per
    `quality/PROTOCOL.md` Step 5).

    Suggested skeleton (executor adapts):
    ```markdown
    ### P73 — Connector contract gaps

    Closed 4 MISSING_TEST rows asserting connector authoring + JIRA-shape
    contracts. Two new wiremock-based Rust tests live next to existing
    contract tests:

    - `crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact`
    - `crates/reposix-github/tests/auth_header.rs::auth_header_bearer_byte_exact`
    - `crates/reposix-jira/tests/list_records_excludes_attachments.rs::list_records_excludes_attachments_and_comments`

    The auth-header tests use `wiremock::matchers::header_exact` (NOT regex)
    for byte-exact assertion — the canonical idiom for any future connector
    contract test of this kind. The JIRA test asserts at the **rendering
    boundary** (Record.body + Record.extensions), not at the JSON parse
    layer — that's where the deferral in `docs/decisions/005-jira-issue-
    mapping.md:79-87` is observable to a downstream consumer.

    The `real-backend-smoke-fixture` row was a pure rebind to the existing
    `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence`
    `#[ignore]` smoke (TokenWorld is sanctioned for free mutation per
    `docs/reference/testing-targets.md`).

    The stale `docs/benchmarks/token-economy.md:23-28` JIRA row was resolved
    via path (a) [or path (b) — fill at commit time] per D-05; bench-number
    re-measurement remains deferred to perf-dim P67.

    See: quality/PROTOCOL.md § "Principle A"; CLAUDE.md "Build memory budget".
    ```

    Commit: `P73: CLAUDE.md H3 subsection (CONNECTOR-GAP-01..04 per D-10)`.
  </action>
  <verify>
    <automated>
      grep -qE '^### P73' CLAUDE.md \
        && [ "$(awk '/^### P73/,/^### |^## /' CLAUDE.md | wc -l)" -le 32 ] \
        && (bash scripts/banned-words-lint.sh CLAUDE.md 2>/dev/null \
            || bash scripts/check-banned-words.sh 2>/dev/null \
            || true)
    </automated>
  </verify>
  <done>CLAUDE.md gains P73 H3 ≤30 lines under `## v0.12.1 — in flight`; banned-words check passes; commit landed.</done>
</task>

<task type="auto">
  <name>Task 11: Phase SUMMARY.md + verifier-dispatch flag for top-level orchestrator</name>
  <files>.planning/phases/73-connector-contract-gaps/SUMMARY.md</files>
  <action>
    Write the phase summary using `$HOME/.claude/get-shit-done/templates/summary.md`
    shape. Include:

    1. **Objective** — recap from PLAN.md.
    2. **Completed tasks** — 11 tasks (1 decision checkpoint + 10 auto) across
       7 waves; each task with commit SHA.
    3. **Catalog row transitions** — list 4 rows by id + final state (BOUND
       or RETIRE_PROPOSED).
    4. **Alignment ratio delta** — BEFORE / AFTER values from
       `quality/reports/verdicts/p73/{summary-before,summary-after}.json`.
    5. **New test files shipped** —
       - `crates/reposix-confluence/tests/auth_header.rs` (1 fn)
       - `crates/reposix-github/tests/auth_header.rs` (1 fn)
       - `crates/reposix-jira/tests/list_records_excludes_attachments.rs` (1 fn)
       - `quality/gates/docs-alignment/verifiers/jira-adapter-shipped.sh`
         (only if path (a))
    6. **Path decision (D-05)** — explicit statement of (a) or (b), commit
       SHA, rationale.
    7. **Surprises / Good-to-haves** — copy any entries appended to
       `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` or
       `GOOD-TO-HAVES.md` during P73 (reference file + entry timestamp).
       If empty, state "no out-of-scope items observed during execution"
       (the verifier honesty-checks this per OP-8).
    8. **CLAUDE.md update** — H3 subsection lines + commit SHA.
    9. **Ready for verifier dispatch** — explicit flag. Subsection text
       (verbatim):

       ```
       ## Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION

       Per D-07 + CLAUDE.md OP-7 + quality/PROTOCOL.md § "Verifier subagent
       prompt template": the executing agent does NOT grade itself. After
       this SUMMARY commits, the top-level orchestrator MUST dispatch:

           Task(subagent_type=gsd-verifier OR general-purpose,
                description="P73 verifier dispatch",
                prompt=<verbatim QG-06 prompt template from quality/PROTOCOL.md
                        with N=73>)

       Inputs the verifier reads with ZERO session context:
         - quality/catalogs/doc-alignment.json (4 P73 row ids: auth-header-exact-test,
           real-backend-smoke-fixture, attachments-comments-excluded,
           jira-real-adapter-not-implemented)
         - .planning/milestones/v0.12.1-phases/REQUIREMENTS.md (CONNECTOR-GAP-01..04)
         - quality/reports/verdicts/p73/{status-before.txt, status-after.txt,
                                          summary-before.json, summary-after.json}
         - CLAUDE.md (confirms P73 H3 appears in `git diff main...HEAD -- CLAUDE.md`)
         - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (honesty check
           per OP-8 — empty intake is acceptable IFF execution honestly observed
           no out-of-scope items; commits should reflect any "fix-eagerly" choices)
         - The 3 new test files (verifier confirms they pass via per-crate
           cargo test invocations — D-09 sequential)

       Verdict goes to: quality/reports/verdicts/p73/VERDICT.md

       Phase does NOT close until verdict graded GREEN.
       ```

    10. **+2 phase practice (OP-8) audit trail** — short paragraph stating
        which in-flight observations were eager-fixed in-phase (< 1 hour,
        < 5 files) vs. appended to SURPRISES-INTAKE.md / GOOD-TO-HAVES.md.
        Empty intake is acceptable IFF the running phase observed no out-of-
        scope items; the verifier spot-checks this honesty.

    Commit: `P73: phase SUMMARY + verifier-dispatch flag for top-level orchestrator`.
  </action>
  <verify>
    <automated>
      test -f .planning/phases/73-connector-contract-gaps/SUMMARY.md \
        && grep -q "Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION" .planning/phases/73-connector-contract-gaps/SUMMARY.md \
        && grep -q "OP-8" .planning/phases/73-connector-contract-gaps/SUMMARY.md \
        && grep -q "Path decision" .planning/phases/73-connector-contract-gaps/SUMMARY.md \
        && git log -1 --format=%s | grep -q "P73: phase SUMMARY"
    </automated>
  </verify>
  <done>SUMMARY.md committed; verifier-dispatch flag explicit; OP-8 audit trail captured; phase ready for orchestrator-level Task() dispatch.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| executor → cargo | Cargo invocations during connector-test development can compile arbitrary build.rs from workspace crates; this is the same trust posture as any cargo build (already implicit in CLAUDE.md OP-1). No new boundary. |
| test → wiremock | Each new test spawns an in-process `wiremock::MockServer` bound to localhost ephemeral port. No outbound network. The test creds (`test-user@example.com`, `atlassian-api-token-xyz123`, `ghp_test_personal_access_token_xyz`, `token`) are obviously synthetic. |
| `bind` → catalog | The bind command MUTATES `quality/catalogs/doc-alignment.json`. Atomic per implementation (validates ALL `--test` citations BEFORE mutating; see `crates/reposix-quality/src/commands/doc_alignment.rs:255-272`). |
| executor → existing test fn | Task 5 binds to `dark_factory_real_confluence` — the test is `#[ignore]`-gated, never runs as part of bind validation (bind only walks the source file's fn body for hashing). No live-credential exposure. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-73-01 | Spoofing | wiremock test seeds a header value that "looks right" but isn't byte-exact | mitigate | D-02 + `header_exact` (NOT `header_regex`); the matcher panics on drop if the request never matched, so wrong header surfaces loudly. Both auth-header tests follow the same pattern. |
| T-73-02 | Tampering | `bind` partial-write on multi-test failure (auth-header row binds 2 fns) | mitigate | Already mitigated upstream — `bind` validates all `--test` citations BEFORE catalog mutation per `doc_alignment.rs:255`. P73 relies on this; no new code path. Multi-test row in Task 8 is the explicit test of this property. |
| T-73-03 | Repudiation | Test exits 0 falsely (e.g. wiremock matcher loose) | mitigate | All 3 new tests assert SPECIFIC strings on Record.body / Record.extensions / wiremock matchers. Failure mode names the offending content (Principle B). Drive through `BackendConnector` trait seam — not the private helpers — so the test exercises the public surface. |
| T-73-04 | Information disclosure | Test creds (synthetic) leak into CI logs | accept | Creds are obviously fake (`test-user@example.com`, `ghp_test_*_xyz`); leak is harmless. Real-credential tests are `#[ignore]`-gated and require explicit env vars per `docs/reference/testing-targets.md`. |
| T-73-05 | Denial of service | `cargo test -p reposix-jira` (or confluence/github) OOMs the VM (parallel cargo from another agent) | mitigate | D-09 + CLAUDE.md "Build memory budget" — sequential cargo invocations enforced at the executor level (one task at a time within Wave 2; `parallelization: false` frontmatter). |
| T-73-06 | Tampering | Stale prose path (a) re-introduces the same staleness if bench numbers don't actually get measured | accept | The path (a) prose explicitly defers re-measurement to P67 perf-dim with the row link "(pending re-measurement)"; the verifier asserts only that the JIRA crate manifest exists (the binary fact the prose now claims). When P67 lands, walker fires `STALE_DOCS_DRIFT` and a maintainer updates the table with measured numbers. |
| T-73-07 | Elevation of privilege | Wiremock mock URL substituted for real backend in production code | accept | The wiremock URL is set ONLY via `*Backend::new_with_base_url` constructor used in tests. Production code paths use the canonical creds-driven base URL. Test-only constructor is `pub(crate)` adjacent to the production type per existing pattern. |
</threat_model>

<verification>
## Phase-level invariants (verifier subagent reads these)

1. **3 P73 rows BOUND, 1 row BOUND or RETIRE_PROPOSED** — `jq` queries on
   each of the 4 row ids; cross-check against
   `quality/reports/verdicts/p73/summary-after.json`'s
   `claims_missing_test` having dropped by ≥ 3 (4 if path (a)).
2. **3 new Rust tests pass** — per-crate, sequential per D-09:
   ```
   cargo test -p reposix-confluence --test auth_header
   cargo test -p reposix-github --test auth_header
   cargo test -p reposix-jira --test list_records_excludes_attachments
   ```
3. **Auth-header row carries 2 test citations (multi-test)** —
   `jq '.rows[] | select(.id=="docs/connectors/guide/auth-header-exact-test") | .tests | length'` returns 2.
4. **Real-backend-smoke-fixture row cites dark_factory_real_confluence** —
   `jq` query confirms `tests[0]` ends with `::dark_factory_real_confluence`.
5. **`header_exact` matcher used (D-02)** —
   `grep -lE 'header_regex' crates/reposix-{confluence,github}/tests/auth_header.rs`
   returns NO files (regex banned by D-02).
6. **Stale prose updated regardless of path** —
   `grep -F "N/A (adapter not yet implemented)" docs/benchmarks/token-economy.md`
   returns no match.
7. **CLAUDE.md updated (D-10)** — `git diff main...HEAD -- CLAUDE.md`
   shows a P73 H3 subsection ≤30 lines under `## v0.12.1 — in flight`.
8. **Banned-words clean** — `bash scripts/banned-words-lint.sh CLAUDE.md`
   (or equivalent) exits 0.
9. **No regression in BOUND rows** — `jq '.summary.claims_bound'
   quality/catalogs/doc-alignment.json` is at least 3 higher than the BEFORE
   snapshot (4 if path (a)).
10. **Coverage ratio not regressed** — `jq '.summary.coverage_ratio'
    quality/catalogs/doc-alignment.json` ≥ 0.10 (the floor — task 1 BEFORE
    snapshot is the baseline).
11. **OP-8 audit trail honest** — if any commit contains "eager-fix" or any
    `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` entry was appended during
    execution, SUMMARY.md cites them; if empty, SUMMARY.md says so explicitly.
12. **Pre-push gates pass** (run on close):
    ```
    cargo check --workspace -q
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all -- --check
    bash scripts/end-state.py    # freshness invariants
    bash scripts/check-docs-site.sh   # only if mkdocs.yml or docs/ touched (token-economy.md DOES qualify)
    bash scripts/check-mermaid-renders.sh   # only if any .md mermaid block touched (no in P73)
    ```
    Each as a separate cargo invocation per D-09.

## Top-level orchestrator action (NOT executor)

After SUMMARY.md commits, the orchestrator dispatches:

```
Task(
  description="P73 verifier dispatch — Path A per D-07",
  subagent_type="gsd-verifier",          # or "general-purpose" if gsd-verifier unavailable
  prompt=<verbatim quality/PROTOCOL.md § "Verifier subagent prompt template",
          with N=73>
)
```

The verifier writes `quality/reports/verdicts/p73/VERDICT.md`. Phase does
NOT close until verdict graded GREEN. Per CLAUDE.md OP-7: the executing
agent does NOT talk the verifier out of RED.
</verification>

<success_criteria>
Phase 73 closes (and the orchestrator advances to P74) WHEN ALL of:

1. **4 catalog rows transition** in `quality/catalogs/doc-alignment.json`:
   - `docs/connectors/guide/auth-header-exact-test` → BOUND (2 test citations)
   - `docs/connectors/guide/real-backend-smoke-fixture` → BOUND (1 test citation: `dark_factory_real_confluence`)
   - `docs/decisions/005-jira-issue-mapping/attachments-comments-excluded` → BOUND (1 test citation)
   - `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` → BOUND (path (a)) OR RETIRE_PROPOSED (path (b))
2. **3 new Rust tests** under `crates/reposix-{confluence,github,jira}/tests/`
   exist, are byte-exact (`header_exact`, NOT regex), and pass per-crate
   `cargo test` (sequential per D-09).
3. **`real-backend-smoke-fixture` is a pure rebind** per D-04 — no new test
   code; `dark_factory_real_confluence` is the canonical citation.
4. **JIRA test asserts at the rendering boundary** per D-03 — Record.body
   markdown AND Record.extensions checked, NOT JsonValue or JiraFields.
5. **Wiremock fixtures are minimal inline JSON literals** per D-06 — no
   new fixture-fragment library introduced.
6. **Path-(a)-vs-(b) decision recorded** per D-05; commit message + SUMMARY
   cite which path landed and why.
7. **Stale token-economy prose updated regardless of path** — the line
   `N/A (adapter not yet implemented)` no longer appears in the doc.
8. **CLAUDE.md gains a P73 H3 ≤30 lines** under `## v0.12.1 — in flight`
   per D-10; banned-words check passes.
9. **`alignment_ratio` delta captured** at
   `quality/reports/verdicts/p73/{summary-before,summary-after}.json`.
10. **`.planning/phases/73-connector-contract-gaps/SUMMARY.md`** exists,
    names commit SHAs, and explicitly flags the verifier-dispatch as a
    top-level-orchestrator action (D-07).
11. **+2 phase practice audit (OP-8)** captured: SUMMARY.md states whether
    any in-flight observations were eager-fixed in-phase vs. appended to
    `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`. Empty intake is honest IFF
    the running phase observed no out-of-scope items.
12. **Top-level orchestrator dispatches `gsd-verifier`** (Path A per D-07)
    with the QG-06 prompt template; verdict at
    `quality/reports/verdicts/p73/VERDICT.md` graded **GREEN**. Phase does
    NOT close on RED — loop back, fix, re-verify (CLAUDE.md OP-7).
13. **No `git push`, no `git tag --push`, no `cargo publish`** — local
    commits only per HANDOVER-v0.12.1.md.
</success_criteria>

<output>
After completion, the executor creates:

- `.planning/phases/73-connector-contract-gaps/SUMMARY.md` — phase summary (Task 11).
- `quality/reports/verdicts/p73/status-before.txt` — BEFORE snapshot (Task 1).
- `quality/reports/verdicts/p73/summary-before.json` — BEFORE summary (Task 1).
- `quality/reports/verdicts/p73/status-after.txt` — AFTER snapshot (Task 9).
- `quality/reports/verdicts/p73/summary-after.json` — AFTER summary (Task 9).

The top-level orchestrator (NOT the executor) creates:

- `quality/reports/verdicts/p73/VERDICT.md` — graded by `Task(gsd-verifier, ...)`
  per D-07 / OP-7.

Phase advances to P74 ONLY when VERDICT.md is graded GREEN.
</output>
