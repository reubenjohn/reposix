# Per-question findings (1–12)

← [back to index](./index.md)

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
