# 82-01 Plan Summary — Bus remote: URL parser, prechecks, fetch dispatch

Single-plan phase, 6 sequential tasks, 8 atomic commits, GREEN verdict at `quality/reports/verdicts/p82/VERDICT.md`.

## Tasks shipped (6/6)

**T01 — Catalog-first.** Six catalog rows minted in `quality/catalogs/agent-ux.json` with `status: FAIL` BEFORE any Rust:
- `agent-ux/bus-url-parses-query-param-form`
- `agent-ux/bus-url-rejects-plus-delimited`
- `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`
- `agent-ux/bus-precheck-b-sot-drift-emits-fetch-first`
- `agent-ux/bus-fetch-not-advertised`
- `agent-ux/bus-no-remote-configured-error`

Each with a TINY shell verifier under `quality/gates/agent-ux/`. Promoted ad-hoc bash JSON validation to `scripts/p82-validate-catalog-rows.py` (per CLAUDE.md §4 "Ad-hoc bash is a missing-tool signal").

**T02 — `bus_url.rs` parser.** Sibling module that calls `parse_remote_url` after stripping `?<query>`. New `pub(crate) enum Route { Single(ParsedRemote), Bus { sot, mirror_url } }` branches at `argv[2]` parse-time. Rejects `+`-delimited form with verbatim Q3.3 hint; rejects unknown query keys (D-03). 4 inline unit tests.

**T03 — `precheck_sot_drift_any` wrapper.** New 10-line coarser wrapper in `precheck.rs`: `precheck_sot_drift_any(cache, backend, project, rt) -> SotDriftOutcome { Drifted | Stable }`. P81's `precheck_export_against_changed_set` preserved verbatim. 1 unit test.

**T04 — `bus_handler.rs` + main.rs Route dispatch.** New `handle_bus_export` function: PRECHECK A (`git ls-remote -- <mirror> refs/heads/main` with `-`-prefix reject) → PRECHECK B (`precheck_sot_drift_any`) → emit clean `error refs/heads/main bus-write-not-yet-shipped` + stderr "bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83" (D-02). Capabilities branching at `main.rs:150-172`: `if matches!(route, Route::Single(_)) { proto.send_line("stateless-connect")?; }` gates DVCS-BUS-FETCH-01. State extension: ONE `Option<String> mirror_url` field. `diag` (line 80→90) + `ensure_cache` (line 219→256) widened from `fn`-private to `pub(crate)` per M2; `fail_push` stays private (`bus_handler` defines local `bus_fail_push`). `State.backend_name` widened to `pub(crate)` for diagnostic composition.

**T05 — Integration tests + tests/common.rs copy.** 5a-prime hard-block: `cp crates/reposix-cache/tests/common/mod.rs crates/reposix-remote/tests/common.rs` (P81 M3 gap closed). Then 11 integration tests across 4 files:
- `tests/bus_url.rs` (3 tests): query-param parse round-trip with positive `stdout.contains("import") && stdout.contains("export")` cap-advertise assertion (H1 fix); `+`-delimited reject; unknown-key reject.
- `tests/bus_capabilities.rs` (2 tests): single-backend advertises `stateless-connect`; bus URL OMITS it.
- `tests/bus_precheck_a.rs` (4 tests): drifting mirror → `error fetch first` + `git fetch <mirror>` hint; synced mirror → passes through; `-`-prefix reject; missing-remote error.
- `tests/bus_precheck_b.rs` (2 tests): wiremock-seeded SoT drift → `error fetch first` + `git pull --rebase` hint; stable SoT → reaches deferred-shipped error (proves PRECHECK B passed).

**T06 — Catalog flip + CLAUDE.md + close.** Runner-driven catalog flip FAIL→PASS for all 6 rows. CLAUDE.md updated: § Architecture "Bus URL form (P82+)" paragraph; § Commands "Local dev loop" bus push bullet. `git push origin main`.

## Commits

| SHA | Subject |
|---|---|
| `9818f16` | quality(agent-ux): mint 6 bus-remote catalog rows + 6 TINY verifiers (catalog-first) |
| `c754eac` | feat(remote): bus URL parser — `bus_url::parse` + `Route::Single\|Bus` enum |
| `b88dad0` | feat(remote): coarser SoT-drift wrapper `precheck_sot_drift_any` |
| `a721b56` | feat(remote): `bus_handler` + main.rs Route dispatch + capabilities branching + State extension |
| `455cd4a` | test(remote): copy `tests/common.rs` from reposix-cache (P81 M3 gap, 5a-prime) |
| `74f915b` | test(remote): 4 integration tests — bus_url + bus_capabilities + bus_precheck_a + bus_precheck_b |
| `d38e6e1` | quality(agent-ux): flip 6 P82 rows FAIL→PASS + CLAUDE.md update + verifier-shell test-name fix |
| `682a8dc` | fix(remote): clippy match-wildcard-for-single-variants + doc-markdown SoT/Route in test panics (pre-push fix) |

## D-01..D-06 ratified at plan time

- **D-01** — by-URL-match for the no-mirror-configured check (Q-A); first-alphabetical + WARN on multi-match. Zero-match emits verbatim Q3.5 hint without auto-mutating git config.
- **D-02** — P82 emits clean `error refs/heads/main bus-write-not-yet-shipped` after both prechecks pass (Q-B); does NOT read stdin; does NOT fall through to `handle_export`.
- **D-03** — reject unknown query keys; only `mirror=` recognized (Q-C). Forward-compat-via-explicit-opt-in.
- **D-04** — `agent-ux.json` is the catalog home (NOT a new `bus-remote.json`).
- **D-05** — State extension is ONE `Option<String> mirror_url` field (no `BusState` type-state explosion).
- **D-06** — `git ls-remote` shell-out (NOT gix-native) per `doctor.rs` idiom; `--` separator + reject `-`-prefixed mirror_url (T-82-01 mitigation).

## In-phase deviations (eager-resolution per OP-8)

All 7 deviations met OP-8 carve-out criteria (< 1 hour, no new dependency, no new file outside planned set):

1. **Verifier-shell `cargo test` test-name argument shape.** cargo test only accepts ONE positional TESTNAME; additional names must come after `--` as substring filters. Fixed both verifier shells. Also discovered `route_single_for_bare_reposix_url` only exists at the unit-test layer (inline in `src/bus_url.rs`); narrowed the integration-test verifier to `parses_query_param_form_round_trip`.
2. **Working-tree fixture missing object DB.** `git update-ref refs/remotes/mirror/main <sha>` refuses to point at a SHA the local object DB doesn't have. Fix: insert `git fetch mirror` between `remote add mirror` and `update-ref`. Applied to both `bus_precheck_a.rs` and `bus_precheck_b.rs` synced fixture.
3. **Test-target requires explicit branch checkout.** Added explicit `git checkout -b main` in scratch repo init so the seed commit lands on `main` regardless of `init.defaultBranch`.
4. **Pre-push gate caught 6 stricter clippy violations under `--all-targets`.** Plan's `cargo clippy -p reposix-remote -- -D warnings` skipped tests; pre-push runs `--workspace --all-targets`. Fix-forward: replaced wildcard match arms with explicit variant arms; backticked `Route::Bus` / `Route::Single` / `SoT` doc strings.
5. **Promoted ad-hoc bash JSON-validation to `scripts/p82-validate-catalog-rows.py`** per CLAUDE.md §4. Two modes: `present` (T01) / `pass` (T06).
6. **Temporary `#[allow(dead_code)]` scope-hygiene** on T02/T03 symbols across the T02→T04 commit window. Removed at T04 atomically with the wiring commit.
7. **Visibility widening per `<must_haves>`.** `diag` + `ensure_cache` + `State.backend_name` widened to `pub(crate)` so sibling `bus_handler` module can call them.

## Acceptance

- All 4 DVCS-BUS-* requirements shipped, observable test coverage, GREEN verdict by unbiased subagent.
- Catalog rows at PASS; pre-push runner shows 26 PASS / 0 FAIL at phase close.
- CLAUDE.md updated in-phase (QG-07).
- No SURPRISES-INTAKE entries — every deviation was eager-resolved.
- 11 integration tests + 5 unit tests added; full reposix-remote sweep: 0 regressions.
