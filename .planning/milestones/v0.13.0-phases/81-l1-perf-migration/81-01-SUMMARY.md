# 81-01 Plan Summary — L1 perf migration (`list_changed_since`-based conflict detection)

Single-plan phase, 4 sequential tasks, 6 atomic commits (4 task commits + 1 docs-alignment refresh + 1 CI fix-forward), GREEN verdict at `quality/reports/verdicts/p81/VERDICT.md`.

## Tasks shipped (4/4)

**T01 — Catalog-first.** Three catalog rows minted with `status: FAIL` BEFORE any Rust landed:

- `perf-targets/handle-export-list-call-count` → `quality/gates/perf/list-call-count.sh`
- `agent-ux/sync-reconcile-subcommand` → `quality/gates/agent-ux/sync-reconcile-subcommand.sh`
- `docs-alignment/perf-subtlety-prose-bound` → minted via `reposix-quality doc-alignment bind` (deferred from T01 → T04 per OP-8 eager-resolution; the bind verb requires the cited test file on disk, so the bind ran alongside `perf_l1.rs` creation in T04).

**T02 — Cache + remote.** The cache crate (`crates/reposix-cache/`) gained:

- `Cache::read_blob_cached` — sync gix-local primitive (NOT async; NOT backend-egress) for the precheck path. Returns `Ok(None)` when the blob isn't materialized.
- `Cache::read_last_fetched_at` / `Cache::write_last_fetched_at` — thin wrappers over the `meta` cursor table, parallel to P80's `read_mirror_synced_at`.

The remote crate (`crates/reposix-remote/`) gained:

- New module `precheck.rs` with `precheck_export_against_changed_set(cache, backend, project, rt, sot_host, parsed) -> anyhow::Result<PrecheckOutcome>` (narrow-deps signature per M1 — P82's bus handler can call it without coupling to the single-backend `State`).
- `pub(crate) struct State` (visibility widening per H3 fix; `rt`, `backend`, `project`, `cache` fields all `pub(crate)` so `precheck.rs` can read them).
- `handle_export` rewritten to call `precheck.rs` instead of the unconditional `list_records` walk (lines ~334–348 pre-P81; line numbers shifted by P80's wiring).
- Inline reject-path module-doc comment names the L1-strict delete trade-off and cites `architecture-sketch.md § Performance subtlety` + v0.14.0 deferral target.

**T03 — `reposix sync --reconcile` subcommand.** `crates/reposix-cli/src/sync.rs` (new) + clap `Sync { reconcile }` variant; 3-test smoke suite at `crates/reposix-cli/tests/sync.rs`.

**T04 — Integration + close.** `crates/reposix-remote/tests/perf_l1.rs` ships:

- `l1_precheck_uses_list_changed_since_not_list_records` — N=200 records, wiremock with `NoSinceQueryParam.expect(0)` + `HasSinceQueryParam` matchers. Asserts the L1 hot-path makes 0 `list_records` calls and ≥1 `list_changed_since` call.
- `positive_control_list_records_call_fails_red` — flips `expect(0)` → `expect(1)` and uses `#[should_panic(expected = "Verifications failed")]` to confirm wiremock's matcher actually fails RED when violated. Closes RESEARCH MEDIUM risk on wiremock semantics.

CLAUDE.md updated per D-05: § Commands → `reposix sync --reconcile` bullet; § Architecture → 3-5 sentence "L1 conflict detection" paragraph naming the cost-vs-correctness trade-off + L2/L3 v0.14.0 deferral.

## Commits

| SHA | Subject |
|---|---|
| `3a44a9e` | quality(perf,agent-ux): mint L1-perf catalog rows + 2 TINY verifiers (P81-01 T01 catalog-first) |
| `1bd50b5` | feat(cache,remote): L1 precheck — `read_last_fetched_at` + `precheck.rs` + `handle_export` rewrite (DVCS-PERF-L1-01, DVCS-PERF-L1-03) |
| `9321499` | feat(cli): `reposix sync --reconcile` subcommand (DVCS-PERF-L1-02) |
| `d21160c` | test(remote): N=200 perf regression + positive control + flip catalogs FAIL→PASS + CLAUDE.md update (DVCS-PERF-L1-01..03 close) |
| `4869545` | quality(docs-alignment): refresh `cli-subcommand-surface` row `source_hashes` for P81 Sync variant addition |
| `c0c8e54` | fix(remote): serialize `REPOSIX_CACHE_DIR` env-var mutation in `perf_l1` tests (CI fix-forward — process-wide env-var race between two `tokio::test`s) |

## D-01..D-05 ratified at plan time

- **D-01** — L1-strict delete trade-off RATIFIED. Cache trusted as prior; backend-side deletes surface as REST 404 on PATCH at write time; user recovery via `reposix sync --reconcile`. L2/L3 hardening defers v0.14.0. Surfaced in plan body, inline `precheck.rs` module-doc, CLAUDE.md § Architecture.
- **D-02** — `Sync { reconcile }` clap variant chosen over `Refresh --reconcile`. `refresh` writes `.md` files into a working tree (different concern); architecture-sketch + ROADMAP canonically use `reposix sync --reconcile`.
- **D-03** — `plan()` keeps its `&[Record]` signature. Helper materializes `Vec<Record>` from cache before calling `plan()`. Avoids widening blast radius of the 34 existing `diff.rs` tests.
- **D-04** — `quality/catalogs/perf-targets.json` is the catalog home (NOT a new `dvcs-perf.json`). The new L1 row is the first non-WAIVED perf row since v0.12.0.
- **D-05** — CLAUDE.md update spans § Commands ("Local dev loop" bullet) + § Architecture (L1 paragraph) per QG-07.

## Q-A/Q-B/Q-C executor-time path resolutions

- **Q-A** (gix 0.83 "object not found" discriminant): `gix::Repository::try_find_object(oid)` returns `Result<Option<Object<'_>>, ...>` cleanly — no string-fallback needed.
- **Q-B** (`state.backend.as_ref()` vs `&*state.backend`): `as_ref()` worked on `Arc<dyn BackendConnector>`.
- **Q-C** (`Cache::list_record_ids` vs `find_oid_for_record` types): both use `RecordId` consistently.

## In-phase deviations (eager-resolution per OP-8)

1. **`refresh_for_mirror_head` no-op skip.** P80's unconditional `refresh_for_mirror_head` on every successful `handle_export` makes a `list_records` REST call — would defeat the perf regression test's "ZERO list_records calls" assertion on the no-op-push hot path. Eager-resolution: skip the call when `files_touched == 0`. SURPRISES-INTAKE filed (RESOLVED).

2. **Bind-verb schedule shift T01 → T04.** The `reposix-quality doc-alignment bind` verb validates that the cited test file exists on disk. Since `perf_l1.rs` is created in T04, the bind in T01 fails. Schedule shift: docs-alignment bind moved to T04 alongside test creation. SURPRISES-INTAKE filed (OPEN; suggests `--test-pending` flag for tooling polish).

3. **Per-issue GET mocks added to two existing tests** (`mirror_refs.rs::reject_hint_cites_synced_at_with_age` + `push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest`) so the L1 precheck's `backend.get_record(id)` returns the matching version. Test fixture extension only — no production code change.

4. **Process-wide env-var race in perf_l1 tests** (CI fix-forward, commit `c0c8e54`). Local cargo test passed; CI failed because `tokio::test(flavor = "multi_thread")` runs the two tests on different threads sharing the parent process's env vars. Fix: `static ENV_LOCK: Mutex<()>` serializes the `set_var → Cache::open → remove_var` critical section in both tests. Subprocess invocations via `.env(...)` on Command are child-local and unaffected.

## Acceptance

- All 3 DVCS-PERF-L1-* requirements shipped, observable test coverage, GREEN verdict by unbiased subagent.
- Catalog rows at PASS; `python3 quality/runners/run.py --cadence pre-push` shows 26 PASS / 0 FAIL / 0 WAIVED at phase close.
- CLAUDE.md updated in-phase (QG-07).
- 2 SURPRISES-INTAKE entries filed for the two non-trivial deviations.
- CI fix-forward (`c0c8e54`) closes the perf_l1 env-var race; both tests pass under parallel execution.
