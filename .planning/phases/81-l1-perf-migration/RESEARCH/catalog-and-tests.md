# Catalog Row Design and Test Fixture Strategy

← [back to index](./index.md)

## Catalog Row Design (catalog-first per QG-06)

Three rows mint BEFORE the helper edit lands:

### Row 1 — `perf-targets/handle-export-list-call-count`
**Dimension:** `perf` (existing `quality/catalogs/perf-targets.json`)
**Cadence:** `pre-pr` (NOT weekly — this is a regression test, not a benchmark)
**Kind:** `mechanical`
**Sources:** `crates/reposix-remote/src/main.rs::handle_export`, `crates/reposix-remote/tests/perf_l1.rs`
**Verifier:** new shell script `quality/gates/perf/list-call-count.sh` that runs `cargo test -p reposix-remote --test perf_l1 -- --include-ignored` and asserts exit 0.
**Asserts:** "with N=200 records seeded in the sim and a one-record edit pushed, the precheck makes ≤1 `list_changed_since` REST call AND zero `list_records` REST calls (modulo the one-time first-push fallback in test setup)."

### Row 2 — `agent-ux/sync-reconcile-subcommand`
**Dimension:** `agent-ux` (existing `quality/catalogs/agent-ux.json`)
**Cadence:** `pre-pr`
**Kind:** `mechanical`
**Sources:** `crates/reposix-cli/src/main.rs`, `crates/reposix-cli/src/sync.rs`, `crates/reposix-cli/tests/sync.rs`
**Verifier:** `cargo run -p reposix-cli -- sync --reconcile --help` exits 0 AND a smoke test in `tests/sync.rs` runs `reposix sync --reconcile` against the sim and asserts the cache was rebuilt (e.g., `last_fetched_at` advanced).

### Row 3 — `docs-alignment/perf-subtlety-prose-bound`
**Dimension:** `docs-alignment` (existing catalog)
**Sources:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance subtlety: today's `list_records` walk on every push" — the prose paragraph asserting "L1 trades one safety property: today's `list_records` would catch a record that exists on backend but is missing from cache" — bound to `crates/reposix-remote/tests/perf_l1.rs::l1_does_not_call_list_records`.
**Use the existing `bind` verb** in `reposix-quality doc-alignment bind`. No new verifier script (the test IS the verifier).

## Test Fixture Strategy

The `crates/reposix-sim` simulator does not currently expose REST-call counters. **Use wiremock instead** — same approach as the P73 connector contract tests. Pattern:

```rust
// crates/reposix-remote/tests/perf_l1.rs (NEW)
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate, Request};

#[tokio::test]
async fn l1_precheck_uses_list_changed_since_not_list_records() {
    let server = MockServer::start().await;
    let project = "demo";

    // Seed 200 records via the sim's existing JSON shape.
    let records: Vec<serde_json::Value> = (1..=200).map(seeded_record).collect();

    // Mock list_records — assert NOT called.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(no_since_query())  // helper: query_param_exists is wiremock 0.6+; we want
                                // "no `since` query param" → list_records, not the delta.
        .respond_with(ResponseTemplate::new(200).set_body_json(&records))
        .expect(0)              // CRITICAL: zero list_records calls on success path.
        .mount(&server).await;

    // Mock list_changed_since — assert called exactly once.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(query_param_exists("since"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&serde_json::json!([])))
        .expect(1)              // Empty result → no per-record GET, no conflict.
        .mount(&server).await;

    // Drive the helper: env var override origin → server.uri(); export verb;
    // single-record edit fast-import stream on stdin.
    drive_export_verb(&server.uri(), project, single_record_edit()).await;

    // wiremock asserts via Drop: panics if expectations unmet.
}
```

The "no_since" matcher is a closure: `Mock::given(...).and(|r: &Request| !r.url.query_pairs().any(|(k, _)| k == "since"))`. wiremock 0.6 supports custom matchers via `Match` trait.

The N=200 figure comes from architecture-sketch (5,000-record / page-50 = 100 calls; a 200-record / page-50 simulation = 4 paginated calls — enough to make the difference observable while keeping the test sub-second). Confirm sim's page size in P81 Task 1; if sim doesn't paginate at 50, scale N up so the assertion `expect(0)` is meaningful.
