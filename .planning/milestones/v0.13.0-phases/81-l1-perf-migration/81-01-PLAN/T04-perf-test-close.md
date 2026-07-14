← [back to index](./index.md) · phase 81 plan 01

## Task 81-01-T04 — Perf regression test + positive-control + catalog flip + CLAUDE.md update + per-phase push

<read_first>
- **HARD-BLOCK (M2):** `crates/reposix-remote/tests/mirror_refs.rs`
  (entire file, ~250 lines) — the perf test's subprocess invocation
  pattern is copied verbatim from this file's
  `drive_helper_export` (line 110), `one_file_export` (line 76),
  `render_with_overrides` (line 49), and `sample_issue` (line 29)
  helpers. Read FIRST so the inline helpers in perf_l1.rs match the
  byte-exact shapes — do not invent a new pattern.
- **HARD-BLOCK (M3):** `crates/reposix-remote/tests/common.rs` —
  CHECK whether this file exists AND contains `sample_issues`, `seed_mock`,
  `sim_backend`, `CacheDirGuard`. As of 2026-05-01 these helpers live
  ONLY in `crates/reposix-cache/tests/common/mod.rs` (verified during
  plan-check). Cargo's test harness does NOT share `mod common;`
  across crates — each crate's `tests/` directory has its own.
  If `crates/reposix-remote/tests/common.rs` does NOT have these
  helpers, COPY them from `crates/reposix-cache/tests/common/mod.rs`
  BEFORE writing perf_l1.rs. Same applies to T03's `crates/reposix-cli/tests/common.rs`
  (T03's `tests/sync.rs` may need the same copy step).
- `crates/reposix-cache/tests/sync_tags.rs` (entire file — wiremock
  setup pattern; complementary to mirror_refs.rs above).
- `crates/reposix-cache/tests/common/mod.rs` (entire file — the
  authoritative copy of the helpers to be replicated per M3).
- `crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact`
  (the wiremock byte-exact matcher precedent from P73).
- `crates/reposix-core/src/backend/sim.rs:281-303` — `list_changed_since`
  emits `?since=<RFC3339>` query param. The wiremock matcher checks
  for the presence of this param.
- `crates/reposix-remote/src/main.rs::handle_export` (post-T02 state
  — confirm precheck wiring is in place).
- `crates/reposix-remote/Cargo.toml` `[dev-dependencies]` — confirm
  `wiremock` is present (it is — used by mirror_refs.rs; same workspace
  pin applies). `assert_cmd` and `tempfile` likewise already present.
- CLAUDE.md § Commands → "Local dev loop" block (find the bullet list
  to extend).
- CLAUDE.md § Architecture (find the right section for the L1 paragraph
  — likely after the cache reconciliation paragraph).
- `quality/runners/run.py` (find the `--cadence pre-pr` invocation
  shape that re-grades the catalog rows).
- `.planning/STATE.md` (find the "Current Position" cursor to advance
  AFTER the verifier subagent dispatches — note that the cursor advance
  is an orchestrator-level action, NOT a plan task).
</read_first>

<action>
Five concerns: perf regression test → positive-control → catalog flip
→ CLAUDE.md update → per-phase push. The push is the terminal action.

### 4a. Perf regression test — `crates/reposix-remote/tests/perf_l1.rs`

Author the new test file. Two `#[tokio::test]` functions — the primary
regression test and the positive-control sibling. Estimated 200-250
lines including the wiremock setup.

```rust
//! Perf regression test for L1 conflict-detection migration
//! (DVCS-PERF-L1-01..03).
//!
//! Asserts the helper's precheck path makes >=1 list_changed_since
//! REST calls AND ZERO list_records REST calls when the cache cursor
//! is populated (the hot path). Includes a positive-control sibling
//! that flips expect(0) to expect(1) and confirms wiremock fails RED
//! when the matcher is unmet — closing RESEARCH.md MEDIUM risk.

#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use reposix_cache::Cache;
use reposix_core::BackendConnector;
use wiremock::matchers::{method, path};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};

/// Custom matcher: matches GET /projects/<project>/issues with NO
/// `since` query param (i.e., the unconditional list_records call,
/// not the L1 list_changed_since delta). wiremock 0.6 supports custom
/// `Match` impls.
struct NoSinceQueryParam;
impl Match for NoSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        // Parse the URL's query and assert NO key named "since".
        req.url
            .query_pairs()
            .all(|(k, _)| k != "since")
    }
}

/// Custom matcher (M4 fix): symmetric to NoSinceQueryParam — matches
/// requests that DO have a `since` query param, regardless of value.
/// wiremock 0.6's `query_param(K, V)` is byte-exact (returns
/// `HeaderExactMatcher`-shape); there is no `query_param_exists` or
/// wildcard-value form. A custom Match impl is the canonical idiom.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn l1_precheck_uses_list_changed_since_not_list_records() {
    // Seed N=200 records; large enough to make pagination observable.
    // The sim's actual page size is configured at sim startup; we use
    // wiremock so the page boundary is irrelevant — the test counts
    // calls, not pages.
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 200);
    seed_mock(&server, project, &issues).await;

    // Mock: list_records (NO `since` query param) — assert ZERO calls.
    // If precheck regresses to the unconditional walk, this mock will
    // intercept the call and wiremock's Drop-on-test-end will panic.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(NoSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .expect(0)
        .mount(&server)
        .await;

    // Mock: list_changed_since (HAS `since` query param) — assert
    // >=1 call. Empty-result is the cheap success path.
    // M4 fix: wiremock 0.6 has no `query_param`-with-wildcard-value
    // matcher; use a custom Match impl symmetric to NoSinceQueryParam.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(&serde_json::json!([])))
        .expect(1..)
        .mount(&server)
        .await;

    // Set up cache with cursor populated (seed sync writes the cursor).
    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync");

    // Drive the export verb directly via the helper's stdin. The
    // pattern mirrors crates/reposix-remote/tests/mirror_refs.rs::write_on_success_updates_both_refs
    // (P80 precedent — single-record edit fast-import stream on stdin,
    // run handle_export, assert results).
    drive_export_verb_single_record_edit(&server.uri(), project, cache_root.path()).await;

    // wiremock asserts via Drop: panics if expectations unmet.
    // The helper exited cleanly (drove the export verb to completion);
    // the mock expectations are checked at MockServer Drop.
}

/// Helper: drives the export verb against a wiremock-backed sim via
/// the `git-remote-reposix` CARGO_BIN_EXE subprocess. This is the
/// SAME pattern P80's `mirror_refs.rs::write_on_success_updates_both_refs`
/// uses (see lines 110-124 + 142-159 of that file): build a fast-export
/// stream byte buffer, write it to the helper subprocess's stdin via
/// `assert_cmd::Command::cargo_bin("git-remote-reposix")` with
/// `REPOSIX_CACHE_DIR=cache_root` env, and assert exit success.
///
/// Subprocess (option b in M2 fix) preferred over in-process State
/// construction because:
/// - mirror_refs.rs uses subprocess; the precedent is established.
/// - Avoids fabricating `State::new_for_test` (which doesn't exist).
/// - 10-15 lines of subprocess wiring vs 60-80 lines of State + Protocol
///   + ParsedExport synthesis (rejected: too much scope creep).
/// - Wall-clock is ~50ms per spawn — acceptable for a single test.
///
/// Estimated 30-40 lines including the fast-export stream synthesis
/// (which is reusable from mirror_refs.rs::one_file_export +
/// render_with_overrides).
async fn drive_export_verb_single_record_edit(
    server_uri: &str,
    project: &str,
    cache_root: &std::path::Path,
) {
    // Sketch of the subprocess invocation (full impl ~30-40 lines):
    //
    //   let blob = render_with_overrides(/* id */ 1, "issue 1", "edited body\n", /* version */ 2, /* id_override */ 1);
    //   let stream = one_file_export("0001.md", &blob, "edit issue 1\n");
    //   let mut stdin_data = Vec::new();
    //   writeln!(&mut stdin_data, "export").unwrap();
    //   stdin_data.extend_from_slice(&stream);
    //   let url = format!("reposix::{}/projects/{project}", server_uri);
    //   let cache_path = cache_root.to_path_buf();
    //   let assert = tokio::task::spawn_blocking(move || {
    //       Command::cargo_bin("git-remote-reposix")
    //           .expect("binary built")
    //           .args(["origin", &url])
    //           .env("REPOSIX_CACHE_DIR", &cache_path)
    //           .write_stdin(stdin_data)
    //           .timeout(std::time::Duration::from_secs(15))
    //           .assert()
    //   })
    //   .await
    //   .unwrap();
    //   let out = assert.get_output();
    //   assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    //
    // The render_with_overrides + one_file_export helpers MUST be
    // copied into perf_l1.rs OR into crates/reposix-remote/tests/common.rs
    // FIRST (M3 hard-block — see <read_first>).
    todo!("M2 fix: subprocess invocation (option b). 30-40 lines per the sketch above. The render_with_overrides + one_file_export helpers are copied from mirror_refs.rs (M3 hard-block).")
}

/// Positive control: flips `expect(0)` to `expect(1)` on the
/// list_records mock and asserts wiremock panics on Drop. Closes
/// RESEARCH.md MEDIUM risk "wiremock semantics need confirmation
/// during Task 4". If this test SKIPs or PASSes when it should FAIL,
/// the assertion contract is broken.
#[tokio::test(flavor = "multi_thread")]
#[should_panic(expected = "Verifications failed")]
async fn positive_control_list_records_call_fails_red() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 200);
    seed_mock(&server, project, &issues).await;

    // FLIPPED: expect 1 list_records call. Since L1 precheck does NOT
    // call list_records on the cursor-present hot path, wiremock will
    // see ZERO calls — and the expectation's Drop will panic.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(NoSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .expect(1)            // <-- DELIBERATELY MISMATCHED
        .mount(&server)
        .await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync");

    drive_export_verb_single_record_edit(&server.uri(), project, cache_root.path()).await;

    // The MockServer's Drop will panic with "Verifications failed: ..."
    // because the expect(1) was unmet. The #[should_panic(expected = ...)]
    // attribute confirms the panic message contains the wiremock
    // assertion-fail string.
}
```

**Critical: `drive_export_verb_single_record_edit` impl.** The function
above is `todo!()` because the State + handle_export driver pattern is
multi-step (build State, synthesize ParsedExport, run handle_export).
The actual implementation in T04 is **inlined ~30-40 lines** following
the P80 pattern in `crates/reposix-remote/tests/mirror_refs.rs::write_on_success_updates_both_refs`.
Confirm during T04 read_first whether that test exposes a reusable
helper or inlines its setup.

**Wiremock matcher API (M4 fix).** wiremock 0.6 has only `query_param(K, V)`
(byte-exact match on both key AND value). There is no
`query_param_exists` and no wildcard-value form (`wiremock::matchers::any()`
is fictional in this context — `any()` is a request matcher, not a value
matcher). The plan therefore uses TWO custom `Match` impls — `NoSinceQueryParam`
(asserts `since` is absent) and `HasSinceQueryParam` (asserts `since` is
present, value irrelevant). This is the same `impl Match for X` shape
P73's tests use; cross-check `crates/reposix-confluence/tests/auth_header.rs`
for the `Match` trait import path (`use wiremock::{Match, Request};`).

**wiremock dev-dep.** If `crates/reposix-remote/Cargo.toml` does not
list `wiremock` as a `[dev-dependencies]` entry, add it
(workspace-pinned version):

```toml
[dev-dependencies]
wiremock.workspace = true
```

(or whatever the workspace version pin is — match the other crates'
existing entries.)

### 4b. Catalog flip — flip 3 rows FAIL → PASS

Run the catalog runner to re-grade after T02 + T03 + T04 land:

```bash
python3 quality/runners/run.py --cadence pre-pr 2>&1 | tee /tmp/p81-runner.log
```

The runner reads each row's `verifier.script`, runs it, and updates
`status` based on exit code. After T02–T04 ship, the perf row's
verifier passes (delegates to the new `cargo test --test perf_l1`),
the agent-ux row's verifier passes (delegates to the new `cargo test
--test sync`), and the doc-alignment row remains BOUND.

Confirm via:

```bash
python3 -c '
import json
for f, expected_id, expected_status in [
    ("quality/catalogs/perf-targets.json", "perf/handle-export-list-call-count", "PASS"),
    ("quality/catalogs/agent-ux.json", "agent-ux/sync-reconcile-subcommand", "PASS"),
]:
    rows = json.load(open(f))["rows"]
    target = next(r for r in rows if r["id"] == expected_id)
    assert target["status"] == expected_status, f"{expected_id} not {expected_status}: {target['status']}"
print("perf + agent-ux flipped to PASS")
'
```

### 4c. CLAUDE.md update — two paragraphs (D-05)

Edit `CLAUDE.md`. Two insertions:

1. **§ Commands → "Local dev loop" block.** Add a bullet for
   `reposix sync --reconcile` post the existing
   `reposix init sim::demo /tmp/repo` line:

   ```
   reposix sync --reconcile                                  # full list_records walk + cache rebuild (L1 escape hatch)
   ```

2. **§ Architecture.** Add the L1 paragraph (3-5 sentences). Place
   AFTER the existing "Cache reconciliation table" paragraph (or
   wherever the architecture flow names cache state). New paragraph
   verbatim:

   ```
   **L1 conflict detection (P81+).** On every push, the helper reads
   its cache cursor (`meta.last_fetched_at`), calls
   `backend.list_changed_since(since)`, and only conflict-checks records
   that overlap the push set with the changed-set. The cache is
   trusted as the prior; the agent's PATCH against a backend-deleted
   record fails at REST time with a 404 — recoverable via
   `reposix sync --reconcile`. L2/L3 hardening (background reconcile
   / transactional cache writes) defers to v0.14.0 per
   `.planning/research/v0.13.0-dvcs/architecture-sketch.md
   § Performance subtlety`.
   ```

### 4d. Per-phase push (terminal action)

The push is the LAST commit of the plan. T04's flip + CLAUDE.md edit
land in a single commit, then the push runs.

```bash
git add quality/catalogs/perf-targets.json \
        quality/catalogs/agent-ux.json \
        crates/reposix-remote/tests/perf_l1.rs \
        CLAUDE.md
git commit -m "$(cat <<'EOF'
test(remote): N=200 wiremock perf regression + positive-control + flip catalogs FAIL→PASS + CLAUDE.md update (DVCS-PERF-L1-01..03 close)

- crates/reposix-remote/tests/perf_l1.rs (new) — l1_precheck_uses_list_changed_since_not_list_records (positive: zero list_records, >=1 list_changed_since with N=200) + positive_control_list_records_call_fails_red (sibling that flips expect(0)→expect(1) and asserts wiremock panics on Drop; closes RESEARCH.md MEDIUM risk)
- quality/catalogs/perf-targets.json — perf/handle-export-list-call-count flipped FAIL → PASS by runner
- quality/catalogs/agent-ux.json — agent-ux/sync-reconcile-subcommand flipped FAIL → PASS by runner
- quality/catalogs/doc-alignment.json — docs-alignment/perf-subtlety-prose-bound remains BOUND
- CLAUDE.md — § Commands gains `reposix sync --reconcile` bullet; § Architecture gains the L1 conflict-detection paragraph naming the L1-strict delete trade-off and the v0.14.0 L2/L3 deferral

Phase 81 / Plan 01 / Task 04 / DVCS-PERF-L1-01..03 (close).
EOF
)"
git push origin main
```

If pre-push BLOCKS: treat as plan-internal failure. Diagnose, fix, NEW
commit (NEVER amend). Do NOT bypass with `--no-verify`. Re-run
`git push origin main` until it succeeds.

After the push lands, the orchestrator dispatches the verifier subagent
per `quality/PROTOCOL.md § "Verifier subagent prompt template"`. The
subagent grades the three catalog rows from artifacts with zero session
context. The dispatch is an orchestrator-level action AFTER this plan
completes — NOT a plan task.
</action>

<verify>
  <automated>cargo nextest run -p reposix-remote --test perf_l1 && python3 -c 'import json; perf = next(r for r in json.load(open("quality/catalogs/perf-targets.json"))["rows"] if r["id"]=="perf/handle-export-list-call-count"); ux = next(r for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"] if r["id"]=="agent-ux/sync-reconcile-subcommand"); assert perf["status"] == "PASS" and ux["status"] == "PASS", f"rows not PASS: perf={perf[\"status\"]}, ux={ux[\"status\"]}"' && grep -q "reposix sync --reconcile" CLAUDE.md && grep -q "L1 conflict detection" CLAUDE.md</automated>
</verify>

<done>
- `crates/reposix-remote/tests/perf_l1.rs` exists with both
  `l1_precheck_uses_list_changed_since_not_list_records` and
  `positive_control_list_records_call_fails_red`.
- `cargo nextest run -p reposix-remote --test perf_l1` exits 0.
- The positive-control test invokes `#[should_panic]` and PASSes —
  wiremock's Drop-on-test-end actually fails RED when its expectation
  is unmet (closes RESEARCH.md MEDIUM risk).
- `quality/catalogs/perf-targets.json` row
  `perf/handle-export-list-call-count` has `status: PASS`.
- `quality/catalogs/agent-ux.json` row
  `agent-ux/sync-reconcile-subcommand` has `status: PASS`.
- `quality/catalogs/doc-alignment.json` row
  `docs-alignment/perf-subtlety-prose-bound` has `status: BOUND`.
- CLAUDE.md § Commands → "Local dev loop" block includes
  `reposix sync --reconcile`.
- CLAUDE.md § Architecture includes the L1 conflict-detection
  paragraph naming the L1-strict trade-off and pointing at the
  architecture-sketch.
- `git push origin main` succeeded with pre-push GREEN. The phase's
  terminal commit cites all three requirements (DVCS-PERF-L1-01..03).
- Cargo serialized: T04 cargo invocations run only after T03's commit
  has landed; per-crate fallback used.
- Verifier subagent dispatch (orchestrator-level action) follows after
  the push.
</done>

---

