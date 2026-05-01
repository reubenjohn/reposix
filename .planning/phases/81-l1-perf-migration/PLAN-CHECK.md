# Phase 81 — Plan Check

**Date:** 2026-05-01
**Reviewer:** plan-checker subagent (goal-backward verification)
**Plans audited:** `81-PLAN-OVERVIEW.md` (~555 lines), `81-01-PLAN.md` (~1,954 lines)
**Reference materials loaded:** RESEARCH.md, ROADMAP.md § Phase 81, REQUIREMENTS.md DVCS-PERF-L1-01..03, `architecture-sketch.md` § Performance subtlety, `decisions.md` Q3.1, source files (cache.rs, builder.rs, main.rs, backend.rs, taint.rs).

## Verdict: **RED**

The plans are well-organized and goal-aligned at the architectural level (D-01..D-05 ratify the right trade-offs; catalog-first is honored; first-push fallback is decided; threat model is present), BUT three HIGH-severity factual errors in the executor-facing code sketches will cause T02 to fail under `cargo check` without significant in-flight rewriting. These are not "executor will catch via grep" planner-time deferrals — they're load-bearing API contracts that the plan asserts and that grep already disproves.

Re-plan recommended before execution begins.

---

## Per-question findings (1–12)

### 1. Will execution actually deliver the phase goal?

**At the abstract level, yes** — the precheck-flow narrative in plan body, S1 (inbound vs outbound cursor), and the `PrecheckOutcome` shape correctly describe the L1 algorithm. The flow `read cursor → list_changed_since(since) → intersect changed-set with push-set → reject on overlap with version-mismatch → on success update last_fetched_at = now` is faithful to RESEARCH.md § Architecture Patterns and `architecture-sketch.md` § Performance subtlety.

**At the executable level, no** — see #7 + finding-H1 + finding-H2 below. The precheck algorithm sketch in T02 § 2b uses `cache.read_blob(prior_oid)` synchronously on every record in `changed_set ∩ push_set`. But `Cache::read_blob` is `pub async fn ... -> Result<Tainted<Vec<u8>>>` (`crates/reposix-cache/src/builder.rs:442`) AND its implementation does a backend `get_record` REST call (`builder.rs:458–469`) when the blob is not yet materialized. This means: (a) the sync sketch will not compile, (b) even after wrapping in `state.rt.block_on(...)`, the precheck makes a HIDDEN backend GET per cache prior — defeating the L1 perf win that the phase exists to deliver. The `.peek()` accessor cited throughout the plan does not exist on `Tainted<T>`; the actual API is `inner_ref()` / `into_inner()` (`crates/reposix-core/src/taint.rs:48`).

The plan also writes the cursor AFTER `log_helper_push_accepted` which is correct; but it computes `since` from the cache cursor whose value was set by `Cache::build_from` / `Cache::sync` — so on an attached repo where `build_from` fired during `reposix attach`, the cursor IS populated and the L1 path engages. That is correct.

### 2. All 7 ROADMAP success criteria covered?

| SC | Description | Coverage in plan | Status |
|----|-------------|------------------|--------|
| SC1 | `list_records` walk replaced in `handle_export` | T02 § 2c + `<done>` block | COVERED (modulo H1/H2/H3) |
| SC2 | `reposix sync --reconcile` exists | T03 (full task) | COVERED |
| SC3 | Both push paths use same L1 mechanism | `precheck.rs` is `pub(crate)` and named the shared module; D-01 + D-03 | COVERED at structure level — see M1 below |
| SC4 | Perf regression test (N=200, ≤1 list call) | T04 § 4a, two tests | COVERED (modulo H4 + M2) |
| SC5 | L2/L3 inline deferral comment + CLAUDE.md | D-01 names 3 surfaces; T02 § 2b; T04 § 4c | COVERED |
| SC6 | Catalog rows mint FIRST + CLAUDE.md update in same PR | T01 (catalog-first commit BEFORE T02) + T04 (CLAUDE.md in same commit as flip) | COVERED |
| SC7 | Push BEFORE verifier dispatch | T04 § 4d explicitly + Plan-internal close protocol | COVERED |

All 7 covered at the planning level. The execution-time failures are architectural (#1, #7) not coverage gaps.

### 3. Catalog-first invariant respected?

YES. T01 mints all 3 rows + 2 verifier shells with `status: FAIL` BEFORE any T02 implementation. The runner re-grades to PASS in T04. Verifier shells delegate to `cargo test` in TINY shape (~30 lines each). The hand-edit annotation citing GOOD-TO-HAVES-01 (perf + agent-ux dim are not yet bind-verb-supported per Principle A) is correct; matches P80's precedent.

### 4. L1-strict delete trade-off RATIFIED in 3 places?

YES, verbatim:

1. **Plan body** — D-01 in `81-PLAN-OVERVIEW.md` lines 131–164 + `<must_haves>` block in `81-01-PLAN.md` lines 251–253.
2. **Inline `precheck.rs` comment** — module doc-comment in T02 § 2b (`81-01-PLAN.md` lines 859–911) cites both `architecture-sketch.md § Performance subtlety` and v0.14.0 `vision-and-mental-model.md § L2/L3 cache-desync hardening` verbatim.
3. **CLAUDE.md update** — D-05 specifies the § Architecture paragraph (`81-PLAN-OVERVIEW.md` lines 240–248; `81-01-PLAN.md` lines 1849–1860).

All three locations are concrete prose, not promises.

### 5. First-push case decided?

YES. Decision is RATIFIED in plan body (S2, lines 292–320 of overview; lines 105–110 + 891–898 + 967–987 of plan). Specifically: when `last_fetched_at` is `None`, fall back to the existing `list_records` walk for THIS push only, then write the cursor. The alternative (`epoch (1970-01-01)`) is explicitly considered and rejected with rationale (Confluence CQL paginates identically — no win). The decision is consistent across overview, plan body, must_haves, and the precheck.rs sketch (matches with explicit `match` on `Option<DateTime<Utc>>`).

### 6. Cargo discipline?

YES. Per-crate everywhere (`-p reposix-cache`, `-p reposix-remote`, `-p reposix-cli`); no `--workspace`. Sequential ordering documented at multiple levels (overview lines 322–342 hard constraints; per-task `<verify>` blocks; commit ordering T01 → T02 → T03 → T04). Per-task `<done>` blocks reaffirm "cargo serialized." Pre-push hook covers the workspace-wide gate so the plan correctly never duplicates it.

### 7. Threat model integrity?

PARTIALLY YES — STRIDE register is present (T-81-01 Tampering, T-81-02 Information Disclosure, T-81-03 Denial of Service); trust boundaries enumerated; audit-table impact correctly identified as UNCHANGED. T-81-02's mitigation specifically claims "`Tainted::peek()` is used only for `frontmatter::parse(...)` to extract the `version` field" — but **`Tainted::peek()` does not exist** (HIGH issue H3). The intent is correct (don't echo body bytes to logs); the asserted accessor name is wrong throughout the plan (overview line 437; plan body lines 1019, 1074; threat-model T-81-02). RESEARCH.md § Pitfall 7 also uses "peek()" — the planner inherited the error from research without grep-checking. Actual API: `Tainted::inner_ref(&self) -> &T` (`crates/reposix-core/src/taint.rs:48`).

### 8. Plan size + executor context budget

1,954 lines is at the high end. Most content is justified (4 task bodies with code sketches, threat model, must_haves, canonical_refs), but ~150 lines could trim:

- The 65-line `precheck.rs` module-doc duplicates content in D-01 of the overview (~50 lines verbatim repetition). Reduce to 15 lines pointing at architecture-sketch + v0.14.0 doc.
- The "Important: error-variant compatibility" digression (lines 1088–1098) and the "Wiremock matcher API" digression (lines 1782–1792) are executor-time questions that belong in `<read_first>`, not the action body.
- T04's `drive_export_verb_single_record_edit` `todo!()` body with a 5-line comment block (lines 1715–1733) hides MEDIUM scope risk (M2 below) — should be sized concretely.

Trim to ~1,750 lines is achievable without losing contract-binding content.

### 9. Open questions Q1–Q7 — legitimately deferrable?

| Q | Description | Verdict |
|---|-------------|---------|
| T02-Q1 | `handle_export` line numbers re-confirm via grep | ACCEPTABLE — line numbers shift; grep at execute time is appropriate. Grep already done above: line 336 = current `list_records` call. |
| T02-Q2 | `Error` variant names | NOT ACCEPTABLE as deferrable (H4). The plan asserts `Error::BackendUnreachable` and `Error::Cache` exist; in fact `crates/reposix-remote/src/error.rs` does NOT exist — the remote crate uses `anyhow::Result`. The precheck sketch uses `Result<PrecheckOutcome>` with no concrete error import. Executor must redesign error flow at execute time with no plan guidance. |
| T02-Q3 | `Tainted::peek()` accessor name | NOT ACCEPTABLE (H3). Wrong function name in 4 plan locations + RESEARCH. Should be `inner_ref()`. Easy planner fix; surprising the executor with a grep miss is sloppy. |
| T03-Q4 | Cache-from-worktree accessor name | ACCEPTABLE — the plan offers `resolve_cache_for_worktree` as a placeholder and explicitly directs executor to read `attach.rs` / `refresh.rs`. Reasonable degree of planner-time uncertainty. |
| T04-Q5 | wiremock API + `NoSinceQueryParam` impl | ACCEPTABLE-WITH-CAVEAT — wiremock 0.6's exact `query_param` matcher API + custom `Match` trait shape is verifiable in `cargo doc`. The positive-control test (#4 closes the MEDIUM risk explicitly) correctly inoculates against wiremock semantics surprises. |
| T04-Q6 | `drive_export_verb_single_record_edit` impl marked `todo!()` | NOT ACCEPTABLE (M2). This is a 30-40 line helper that builds State + ParsedExport + drives `handle_export` directly. The plan punts this to "see P80's mirror_refs.rs test pattern" without pointing at the test's actual signature. Without it, the perf regression test is non-runnable; T04 effectively contains a 40-line implementation gap. |
| T01-Q7 | `bind` flag names | ACCEPTABLE — the plan correctly cites RESEARCH.md and instructs executor to read `bind --help` first. |

Two of seven (Q2 + Q3) are NOT legitimately deferrable; one (Q6) understates scope.

### 10. Phase-shape risks (T03 ⊃ T02; T04 ⊃ T02 + T03)?

YES, sequential chain is sound:

- T03 (sync.rs) needs `Cache::build_from` (already shipped pre-P81) — does NOT need T02's cursor wrappers per se, BUT the smoke test asserts `last_fetched_at` advances (which `build_from` already writes via `crates/reposix-cache/src/builder.rs:119`). The smoke test can run independently of the precheck rewrite.
- T04 (perf regression test) DOES need T02's precheck to be in place to assert "zero `list_records` calls" (the helper must take the L1 path). T04 also implicitly needs T03 for the `--reconcile` agent-ux verifier.

The dependency chain T01 → T02 → T03 → T04 is correct and matches CLAUDE.md "Build memory budget" sequential cargo discipline.

### 11. Cross-phase contract integrity (P82 inherits PRECHECK B)?

PARTIAL. The precheck function placement (`crates/reposix-remote/src/precheck.rs`, `pub(crate)`) is correct for the SAME-CRATE bus handler in P82. The `precheck_export_against_changed_set` signature accepts `&mut State` — but `State` is a `struct State` in `main.rs`, not `pub(crate)` (currently private to `main.rs`). For `precheck.rs` (a sibling module of `main.rs`) to import `State`, **the `State` struct's visibility must be widened to `pub(crate)` in T02**, AND its `cache: Option<Cache>`, `rt: Runtime`, `backend: Arc<dyn BackendConnector>`, `project: String` fields must also be `pub(crate)`. The plan asserts `use crate::main::{issue_id_from_path, State};` (T02 § 2b line 921) but `crate::main` is not a valid path inside a binary crate — `main.rs` is the binary root, not a sub-module. This is M3 below.

### 12. Build memory budget compliance?

YES. Sequential cargo across all four tasks; per-crate only; no `cargo --workspace` invocations slipping through; no parallel cargo invocations even when T02 and T03 touch different crates. Strict serial discipline matches CLAUDE.md "Build memory budget" rule.

---

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

---

## Summary table

| Question | Status |
|----------|--------|
| 1. Goal achievable? | NO at executable level (H1) |
| 2. SC1–SC7 covered? | YES at planning level |
| 3. Catalog-first? | YES |
| 4. D-01 ratified in 3 places? | YES |
| 5. First-push fallback? | YES |
| 6. Cargo discipline? | YES |
| 7. STRIDE register? | PARTIAL (T-81-02 cites non-existent peek()) |
| 8. Plan size budget? | YES (1,954 lines, ~150 lines trim available) |
| 9. Open questions deferrable? | NO for Q2 + Q3; partial for Q6 |
| 10. Phase shape sound? | YES |
| 11. P82 contract integrity? | PARTIAL (signature couples to State) |
| 12. Build memory budget? | YES |

## HIGH issue count: 4
## MEDIUM issue count: 4
## LOW issue count: 3

---

## Recommended path forward

The phase **goal is sound** (L1 conflict detection, single shared module, sync --reconcile escape hatch, D-01 ratified). The plans **understand the architecture correctly** at the strategic level. But the four HIGH issues are concrete API contract errors that the planner could have caught with `grep read_blob`, `grep peek`, `grep "pub enum.*Error" remote/src/`, and reading 4 lines of `crates/reposix-remote/src/main.rs:42–71`. The planner asserted contracts and the source disproves them.

**Re-plan with these specific fixes:**

1. (H1) Add `Cache::read_blob_cached(...)` to `cache.rs` (sync, gix-local, returns `Option<Tainted<Vec<u8>>>`); use it in precheck.rs. OR rewrite precheck to read version from backend GET (one per `changed_set ∩ push_set` record) instead of cache. Either way, do NOT call `read_blob` in the precheck.
2. (H2) Replace `.peek()` with `.inner_ref()` in 4 plan locations + must_haves + canonical_refs + threat_model. Patch RESEARCH.md in same PR.
3. (H3) Widen `State` to `pub(crate)` (struct + 4 fields); change `precheck.rs` import to `use crate::{State, issue_id_from_path};` (drop `::main::`).
4. (H4) Rewrite precheck sketch to use `anyhow::Result` + `anyhow::Error` (no typed variants). Add a 4-line subsection in T02 § 2b titled "Error flow" specifying anyhow throughout.
5. (M1) Refactor `precheck_export_against_changed_set` signature to accept narrow dependencies (`cache: &Cache, backend: &dyn BackendConnector, project: &str, rt: &Runtime, parsed: &ParsedExport`). 10 lines of plumbing in `handle_export` cost; unlocks P82 reuse cleanly.
6. (M2) Size `drive_export_verb_single_record_edit` at 60–80 lines OR pivot to subprocess invocation; add "read mirror_refs.rs FIRST" hard-block to T04 read_first.
7. (M3) Promote `mod common;` requirement check to T03 + T04 hard-block.
8. (M4) Rewrite the "≥1 list_changed_since call" matcher with a custom `Match` impl symmetric to `NoSinceQueryParam`.
9. (L1, L2, L3) Optional polish — tighten if revising anyway.

After these revisions, the plan should be executable end-to-end without surprise rework. Estimated revision effort: 1 hour (planner-time grep + 10-line edits + 1 new function in cache.rs sketch).
