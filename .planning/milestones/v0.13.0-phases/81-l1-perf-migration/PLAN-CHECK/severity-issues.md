# Severity-classified issues

← [back to index](./index.md)

## Severity-classified issues

### HIGH (blocker — must fix before execution)

#### H1. `Cache::read_blob` is async + does a backend GET; the precheck sketch will not compile AND defeats the L1 perf win

**Plan location:** `81-01-PLAN.md` lines 1014–1018, 1071–1073 (precheck sketch); 254–262 (canonical_refs treats `read_blob` as a synchronous local lookup).

**Reality:**
- `crates/reposix-cache/src/builder.rs:442` — `pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>>`. It is `async`.
- The implementation (lines 458–469) calls `self.backend.get_record(...).await` when the blob isn't materialized — i.e., it makes a backend REST call.

**Why this is a blocker:** the precheck sketch (T02 § 2b) calls `cache.read_blob(prior_oid)` synchronously, which won't compile (it's async). Wrapping in `state.rt.block_on(cache.read_blob(...))` would compile, but it makes a backend GET per record in `changed_set ∩ push_set` — for a Confluence push that intersects the changed set with N records, that's N hidden REST calls. Add the explicit `state.rt.block_on(state.backend.get_record(...))` already in the sketch (line 1028) and you have 2N hidden REST calls per push. This defeats the success criterion "Net REST cost on success path collapses to one call (`list_changed_since`) plus actual REST writes."

**The architectural conflict:** the cache's `read_blob` is the lazy materializer — its job is to fetch the prior body from the backend on demand. For the precheck, the planner needs a different primitive: "read the blob from the cache's bare repo without materializing if absent" (i.e., a `gix::Object::find` against `cache.repo`). RESEARCH.md § Pattern 1 cites `Cache::sync`'s pattern (which IS the local-only path); that pattern reads the object from gix directly, NOT via `read_blob`.

**Remediation:**
1. Add a NEW `pub fn read_blob_cached(&self, oid: gix::ObjectId) -> Result<Option<Tainted<Vec<u8>>>>` in `cache.rs` that does a direct gix-only lookup (returning `Ok(None)` when absent rather than fetching from the backend). This is a LOCAL gix `find_object` without async + without backend egress.
2. The precheck calls `read_blob_cached` for prior-version extraction; on `Ok(None)` it falls through to the no-conflict path (record is in cache OID-map but blob hasn't been materialized — treat as no conflict, `plan()` will refetch on demand via the existing `read_blob` async path during execute).
3. Add `read_blob_cached` to the must_haves and to T02's verify block.

Alternative: have the precheck use `cache.list_record_ids()` + `cache.find_oid_for_record()` to identify cache prior membership (already on disk; no async, no backend GET) and SKIP the per-record version comparison in cache-prior parse — instead, ALWAYS re-fetch via `state.rt.block_on(state.backend.get_record(...))` for the version field. That's ONE GET per record in `changed_set ∩ push_set` — bounded by the typical 1–5 records — which is consistent with Pitfall 5's "lazy parse" recommendation but reads the version from backend instead of cache. Document the reason inline.

Either way: the plan must NOT call `read_blob` in the precheck.

#### H2. `Tainted::peek()` does not exist — wrong accessor name in 4 plan locations + RESEARCH

**Plan location:** `81-01-PLAN.md` lines 1019, 1074; `81-PLAN-OVERVIEW.md` line 437; canonical_refs line 312; threat-model T-81-02 mitigation lines 339; RESEARCH.md § Pitfall 7 line 214.

**Reality:** `crates/reposix-core/src/taint.rs:32–58` shows `Tainted<T>` has `pub fn new(value: T) -> Self`, `pub fn into_inner(self) -> T`, `pub fn inner_ref(&self) -> &T`. No `peek()`.

**Remediation:** replace all `.peek()` references in the plan with `.inner_ref()` (returns `&T`). For `Tainted<Vec<u8>>` the call becomes `prior_bytes.inner_ref().as_slice()` or `String::from_utf8_lossy(prior_bytes.inner_ref())`. RESEARCH.md should be patched in the same PR but is a P81-internal correction.

#### H3. `precheck.rs` cannot import `State` from `crate::main::State` — main.rs is the binary root, not a sub-module

**Plan location:** `81-01-PLAN.md` line 921 (`use crate::main::{issue_id_from_path, State};`).

**Reality:** `reposix-remote` is a binary crate. `main.rs` is the binary root (`fn main()` is at `crates/reposix-remote/src/main.rs:79`). `crate::main` is not a valid path. Sibling modules (`mod precheck;` declared in `main.rs`) are children of the binary root, not children of a `main` module. `State` must be defined in either (a) `lib.rs` of a renamed crate, or (b) `main.rs` and accessed via `crate::State` from `precheck.rs`.

Furthermore, `State` is currently `struct State` (private) at line 42 of `main.rs`. For `precheck.rs` to use it, `State` must be `pub(crate)` and most of its fields (`rt`, `backend`, `project`, `cache`) must also be `pub(crate)`.

**Remediation:**
1. T02 must include a step "widen `State` visibility": `pub(crate) struct State` + `pub(crate)` on `rt`, `backend`, `project`, `cache` fields.
2. `precheck.rs`'s import becomes `use crate::{State, issue_id_from_path};` (no `::main::` segment).
3. Add this to the `<must_haves>` and to T02's verify block.

#### H4. Error variant names asserted in precheck sketch don't exist — `crates/reposix-remote/src/error.rs` doesn't exist; remote crate uses `anyhow::Result`

**Plan location:** `81-01-PLAN.md` lines 920 (`use crate::error::{Error, Result};`), 974 (`Error::BackendUnreachable`), 985 (same), 1009 (`Error::Cache`), 1018 (same), 1029 (same), 1067 (same), 1073 (same).

**Reality:** `ls crates/reposix-remote/src/` returns no `error.rs`. `crates/reposix-remote/src/main.rs:18` shows `use anyhow::{Context, Result};` — the remote crate uses anyhow throughout. The "no new error variants" must_have (line 250 of plan) is technically satisfied by NOT introducing typed variants, but the precheck sketch then introduces them anyway (`Error::BackendUnreachable`, `Error::Cache`).

**Remediation:**
1. Rewrite the precheck sketch to use `anyhow::Result` (or `Result<PrecheckOutcome, anyhow::Error>`).
2. Replace `Error::BackendUnreachable(format!("..."))` with `anyhow::anyhow!("backend-unreachable: ...")` or use `.context("backend-unreachable: list_changed_since")?`.
3. The reject-path string `"backend-unreachable"` is preserved at the call site of `fail_push` in `handle_export` (which already takes `&str` for the reason); the precheck returns an anyhow::Error and the caller in `handle_export` matches/maps it.
4. The "no new error variants" line in must_haves is preserved (anyhow stays).

**Note:** the plan's mention of `crate::error::Result` (line 920) is fictional — there is no `error.rs` and no `Result` type alias. T02-Q2 marks this as a deferred verification, but the entire error-flow design is missing from the plan, not just the variant names.

### MEDIUM (warning — should fix; execution may proceed but with rework)

#### M1. SC3 ("both push paths use same L1 mechanism") is structural-only; bus handler doesn't exist yet

The plan places `precheck_export_against_changed_set` in `precheck.rs` `pub(crate)` so P82's bus handler can call it. That's the right shape. But the function signature `(state: &mut State, parsed: &ParsedExport)` couples to the single-backend `State`. P82's bus handler will have a `BusState { sot: ..., mirror: ... }` with different fields. The `&mut State` parameter will need refactoring at P82 time, OR the function should accept narrower parameters (e.g., `cache: &Cache, backend: &dyn BackendConnector, project: &str, rt: &Runtime, parsed: &ParsedExport`) that both single-backend and bus paths can construct.

**Remediation:** revise `precheck_export_against_changed_set` to accept its dependencies explicitly rather than via `&mut State`. This is a cheap planner-time change that costs ~10 lines of plumbing in `handle_export` but unlocks P82's reuse. Alternative: explicitly call out in plan that "P82 will refactor this signature" and accept the rework — but that defeats the spirit of SC3.

#### M2. `drive_export_verb_single_record_edit` impl is `todo!()` with no concrete sizing

**Plan location:** `81-01-PLAN.md` lines 1715–1733.

The helper builds a `State`, synthesizes a `ParsedExport`, and calls `handle_export` directly. The plan punts this to "see P80's mirror_refs.rs test pattern" without verifying the test is reusable. A 30–40 line State-construction helper involves:
- Tokio Runtime (`Runtime::new()?`)
- `Arc<dyn BackendConnector>` from `sim_backend(&server)`
- `Cache::open(...)` against `cache_root`
- A `Protocol<R, W>` synthesized from in-memory pipes
- A `parse_export_stream`-compatible byte buffer for "single record edit" — this is non-trivial: the `ParsedExport` struct's exact field shape (`commit_message`, `blobs: HashMap<u32, Vec<u8>>`, `tree: BTreeMap<String, u32>`) must match real fast-export-stream output.

Hand-rolling this for the test risks 60–80 lines, not 30–40. M2 risk: T04's perf test ships as a stub or balloons in scope.

**Remediation:**
1. T04's `<read_first>` should explicitly require reading `crates/reposix-remote/tests/mirror_refs.rs` (P80) AND grep for any existing `pub` test-helper.
2. If P80's mirror_refs.rs test inlines its setup (no reusable helper), T04 should size the inline at 60–80 lines and budget executor context accordingly.
3. Alternative: drive via subprocess + `CARGO_BIN_EXE_git_remote_reposix` env var (similar to the smoke-test fallback at T03 lines 1525–1532). Subprocess is 10–15 lines but adds wall-clock latency.

#### M3. T04's `mod common;` reuse assumption is unverified

**Plan location:** `81-01-PLAN.md` lines 1647 (test imports `mod common; use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard}`), 1505–1512.

The `common` module shape (with these specific helpers) lives in `crates/reposix-cache/tests/common.rs`. The plan assumes `crates/reposix-remote/tests/common.rs` and `crates/reposix-cli/tests/common.rs` either ship the same helpers or can be inlined. Cargo's test harness creates a separate test binary per file — `mod common;` is per-crate-tests-directory.

**Remediation:** T04's `<read_first>` already mentions this; promote it to a hard-block step: "If `crates/reposix-remote/tests/common.rs` does not contain `sample_issues`, `seed_mock`, `sim_backend`, `CacheDirGuard`, copy them from `crates/reposix-cache/tests/common.rs` BEFORE writing perf_l1.rs." Same for the T03 sync test in `crates/reposix-cli/tests/`.

#### M4. SC4's "≥1 list_changed_since call" — wiremock matcher must be loose on the `since` value

**Plan location:** `81-01-PLAN.md` line 1689 (`.and(query_param("since", wiremock::matchers::any()))`).

`wiremock::matchers::any()` does not exist as a query-param-value matcher in wiremock 0.6.x. The matcher API is `query_param(K, V) where V: Into<String>` — exact match only. To match "any value for the `since` param," the test needs either (a) `query_param_contains` (does not exist either), (b) a custom `Match` impl checking `req.url.query_pairs().any(|(k, _)| k == "since")`, or (c) two separate `Mock::given` clauses.

**Remediation:** rewrite the matcher inline as a custom `Match` impl (e.g., `HasSinceQueryParam`) or use `Mock::given(...).and(path(...))` without a since-param matcher and rely on the `NoSinceQueryParam` exclusion + path-only inclusion to disambiguate. This is the same shape as `NoSinceQueryParam` (which IS correctly designed in the plan); the planner just needs the symmetric matcher.

### LOW (informational — improvement opportunities)

#### L1. Plan body duplicates D-01 prose verbatim in `precheck.rs` module-doc

The 65-line `precheck.rs` module-doc-comment in T02 § 2b restates D-01 from the overview. That's the right call (CLAUDE.md "two surfaces" pattern), but the prose is ~50 lines of duplicated text. Trim the precheck.rs module-doc to ~15 lines pointing at architecture-sketch + v0.14.0 doc + one-sentence summary; the full ratification lives in CLAUDE.md.

#### L2. Verifier shells `set -euo pipefail` + `tail -20` swallow exit codes

`cargo test ... 2>&1 | tail -20` will pipe the cargo output and the verifier exits with the `tail` exit code (always 0), not the cargo test exit code. This means the verifier passes even when the test fails.

**Remediation:** use `set -o pipefail` (already in `set -euo pipefail`) AND inspect `${PIPESTATUS[0]}` after the pipe, OR use `cargo test --quiet ... > /tmp/log 2>&1 || { tail -20 /tmp/log; exit 1; }`. P80's verifier shell precedent should be checked against this pitfall too.

#### L3. CLAUDE.md update doesn't include the L1 reject-path stderr hint

The reject path in `handle_export` (lines 384–427) emits stderr hints citing `refs/mirrors/<sot>-synced-at`. Per ROADMAP SC2 ("helper stderr hints" mention `reposix sync --reconcile`), the conflict-reject diagnostic should ALSO mention `reposix sync --reconcile` as a recovery path when applicable. The plan doesn't add this.

**Remediation:** T02 § 2c could include a 2-line addition to the existing `diag(...)` block in the conflict-reject branch citing `reposix sync --reconcile` for the cache-desync case. Optional polish.
