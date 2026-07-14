[← index](./index.md)

# Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria
across every v0.13.0 phase":

1. **All commits pushed.** Plan terminates with `git push origin main`
   in T06 (per CLAUDE.md "Push cadence — per-phase", codified
   2026-04-30, closes backlog 999.4). Pre-push gate-passing is part of
   the plan's close criterion.
2. **Pre-push gate GREEN.** If pre-push BLOCKS: treat as plan-internal
   failure (fix, NEW commit, re-push). NO `--no-verify` per CLAUDE.md
   git safety protocol.
3. **Verifier subagent dispatched.** AFTER 82-01 pushes (i.e., after
   T06 completes), the orchestrator dispatches an unbiased verifier
   subagent per `quality/PROTOCOL.md` § "Verifier subagent prompt
   template" (verbatim copy). The subagent grades the six P82
   catalog rows from artifacts with zero session context.
4. **Verdict at `quality/reports/verdicts/p82/VERDICT.md`.** Format per
   `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P81 SHIPPED ... next P82" → "P82 SHIPPED 2026-MM-DD"
   (commit SHA cited).
6. **CLAUDE.md updated in T06.** T06's CLAUDE.md edit lands in the
   terminal commit (one § Architecture paragraph + one § Commands
   bullet per QG-07).
7. **REQUIREMENTS.md DVCS-BUS-URL-01 / DVCS-BUS-PRECHECK-01 / DVCS-BUS-PRECHECK-02 /
   DVCS-BUS-FETCH-01 checkboxes flipped.** Orchestrator (top-level)
   flips `[ ]` → `[x]` after verifier GREEN. NOT a plan task.

# Risks + mitigations

| Risk                                                                                                  | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                |
|-------------------------------------------------------------------------------------------------------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`url` crate's `form_urlencoded::parse` doesn't preserve raw `mirror=` value with embedded `:` and `@`** (RESEARCH.md Assumption A3) | MEDIUM     | T02's unit tests include a parser case for `?mirror=git@github.com:org/repo.git` (verbatim, NOT percent-encoded). If `form_urlencoded::parse` mangles the value, fall back to manual `split_once('=')` after `split_once('?')` per RESEARCH.md fallback path. T02's done criteria includes the round-trip assertion. |
| **`git ls-remote -- <mirror_url>` against `file://` fixture hangs on macOS** (RESEARCH.md Pitfall 3 sibling) | LOW        | T05's `bus_precheck_a.rs` uses `tempfile::tempdir()` + `git init --bare` + `file://` URL. Linux + macOS both handle file:// without blocking. If a CI environment has unusual git config (e.g., `protocol.file.allow=user`), the test sets `GIT_CONFIG_NOSYSTEM=1` + `GIT_CONFIG_GLOBAL=/dev/null` per CLAUDE.md test isolation precedent. |
| **`git config --get-regexp` returns multiple matches** (RESEARCH.md Pitfall 4)                       | MEDIUM     | D-01 ratified: pick first alphabetically + emit stderr WARNING naming the chosen remote. T05's `bus_precheck_a.rs` includes a multi-match fixture asserting the WARNING appears + the chosen remote is the alphabetically-first. |
| **Mirror SHA from `git ls-remote` is empty** (empty mirror; first push) | LOW        | P82 treats empty `git ls-remote` output as `MirrorDriftOutcome::Stable` (no drift possible). P84 (webhook sync) handles the first-push-to-empty-mirror case via a separate code path. T05's `bus_precheck_a.rs` includes an empty-mirror fixture asserting `Stable`. |
| **PRECHECK B firing on the same-record self-edit case** (RESEARCH.md Pitfall 5) | LOW        | P81's RATIFIED `>`-strict semantics on `list_changed_since(since)` mean a same-second self-write is filtered cleanly. T05's `bus_precheck_b.rs` includes a self-edit case asserting `Stable`. |
| **`reposix::<sot>?mirror=<mirror>` URL with query in the mirror value** (RESEARCH.md Pitfall 7) | LOW-MED    | Document the percent-encoding requirement in `bus_url.rs` module-doc + CLAUDE.md § Architecture (D-03). T02 has a test case asserting the percent-encoded form parses correctly. The non-encoded form errors with a clear message per Q-C (extra `?` introduces an unknown key). |
| **Capability branching breaks existing single-backend tests** (S1; D-05)                              | LOW        | The capability arm change is a SINGLE-LINE addition wrapping the existing `proto.send_line("stateless-connect")?;`. Single-backend invocations have `state.mirror_url.is_none() == true`, so the `stateless-connect` line still fires. T05's `bus_capabilities.rs` covers the bus case; the existing `crates/reposix-remote/tests/stateless_connect.rs` covers single-backend. |
| **`State` extension breaks compilation of `handle_export` or `handle_stateless_connect`**             | LOW        | Adding ONE `Option<String>` field with a default `None` initializer in `real_main` is purely additive. `handle_export` doesn't read `state.mirror_url`; `handle_stateless_connect` doesn't read it. T04 confirms via `cargo check -p reposix-remote` after the State edit lands. |
| **Cargo memory pressure** (load-bearing CLAUDE.md rule)                                              | LOW        | Strict serial cargo across all six tasks. Per-crate (`cargo check -p reposix-remote`, `cargo nextest run -p reposix-remote`) only. T01 + T02 + T04 + T06 doc-or-source-edit cargo-checks; T03 + T05 are the cargo-test-bearing tasks (sequential). |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P82**                                    | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`. |
| **No-remote-configured detection: false-negative when remote URL has trailing slash variant** (RESEARCH.md A5) | LOW        | T04 normalizes `mirror_url` (strip trailing `/`) BEFORE the byte-equal compare against config values. T05's `bus_no_remote_configured` test includes a fixture where one remote URL has trailing `/` and the bus URL doesn't — assertion: still matches. |

# +2 reservation: out-of-scope candidates

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` exist already (created during P79). P82 surfaces
candidates only when they materialize during execution — none pre-filed
at planning time.

Anticipated candidates the plan flags (per OP-8):

- **LOW** — `url::form_urlencoded::parse` requires the input be a URL
  (full scheme + host); the bus URL form starts with `reposix::` which
  is NOT a real URL scheme. Eager-resolve in T02 by parsing manually
  via `split_once('?')` + `split('&')` + `split_once('=')` (the
  resulting parser is ~20 lines and avoids the URL crate's strictness).
  RESEARCH.md flagged this as Assumption A3 (MEDIUM risk).
- **LOW** — `git config --get-regexp` outputs values that contain
  whitespace (rare — URLs don't usually have whitespace). Eager-resolve
  in T04 by using `splitn(2, char::is_whitespace)` for the
  `(key, value)` split, NOT the simpler `split_whitespace`.
- **LOW** — A new `bus_handler.rs` test file's wiremock fixture races
  with the `bus_precheck_b.rs`'s wiremock fixture if both run in the
  same nextest process (port conflicts). Eager-resolve via wiremock's
  per-test `MockServer::start()` returning a unique port — same idiom
  as `crates/reposix-remote/tests/perf_l1.rs` (P81 precedent). NOT a
  candidate unless ports clash in CI.
- **LOW-MED** — `state.cache.as_ref()` returns `None` during
  PRECHECK B because `ensure_cache` hasn't fired yet (the bus_handler
  runs BEFORE `handle_export`'s `ensure_cache` line). Eager-resolve
  in T04 by calling `ensure_cache(state)?` at the top of
  `handle_bus_export` (best-effort like in `handle_export`); if cache
  unavailable, PRECHECK B's `precheck_sot_drift_any(None, ...)` returns
  `Stable` (matching the no-cursor first-push policy). NOT a candidate.

Items NOT in scope for P82 (deferred per the v0.13.0 ROADMAP):

- Bus write fan-out (P83). The `bus_handler.rs` body's deferred-error
  stub is a single `return Ok(())` site that P83 replaces with the
  SoT-write + mirror-write logic.
- 30s TTL cache for the `git ls-remote` precheck (Q3.2 DEFERRED).
  Measure first; add only if push latency is hot. Filed as v0.13.0
  GOOD-TO-HAVE candidate.
- Webhook-driven mirror sync (P84). Out of scope.
- DVCS docs (P85). Out of scope; T06 only updates CLAUDE.md.
- Real-backend tests (TokenWorld + reubenjohn/reposix issues). Out of
  scope per OP-1 — milestone-close gates them, not phase-close.
- Multi-SoT bus URL form (`reposix::sot1+sot2?mirror=...`). Out of
  scope per Q3.3 (1+1 bus only in v0.13.0).

# Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                                                      | Delegation                                                                                                                                                                                                                  |
|------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 82-01 T01 (6 catalog rows + 6 verifier shells)                  | `gsd-executor` — catalog-first commit; **hand-edits agent-ux.json per documented gap (NOT Principle A)**.                                                                                                                  |
| 82-01 T02 (`bus_url.rs` parser + 4 unit tests)                  | Same 82-01 executor. Cargo lock held for `reposix-remote`. Per-crate cargo only.                                                                                                                                            |
| 82-01 T03 (`precheck_sot_drift_any` wrapper + 1 unit test)      | Same 82-01 executor. Cargo lock held for `reposix-remote`. Per-crate cargo only.                                                                                                                                            |
| 82-01 T04 (`bus_handler.rs` + main.rs dispatch + capabilities)  | Same 82-01 executor. Cargo lock held for `reposix-remote`. Per-crate cargo only.                                                                                                                                            |
| 82-01 T05 (4 integration tests: bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b) | Same 82-01 executor (cargo-heavy task). Cargo lock for `reposix-remote` integration test run. Per-crate cargo only.                                                                                                            |
| 82-01 T06 (catalog flip + CLAUDE.md + push)                      | Same 82-01 executor (terminal task).                                                                                                                                                                                        |
| Phase verifier (P82 close)                                       | Unbiased subagent dispatched by orchestrator AFTER 82-01 T06 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades the six catalog rows from artifacts.        |

Phase verifier subagent's verdict criteria (extracted for P82):

- **DVCS-BUS-URL-01:** `crates/reposix-remote/src/bus_url.rs` exists;
  `pub(crate) enum Route { Single(ParsedRemote), Bus { sot: ParsedRemote,
  mirror_url: String } }`; `pub(crate) fn parse(url) -> Result<Route>`
  parses `reposix::sim::demo?mirror=file:///tmp/m.git` to `Route::Bus`;
  rejects `+`-delimited form with verbatim "use `?mirror=` instead"
  hint; rejects unknown query keys per Q-C; unit tests
  (`cargo test -p reposix-remote --test bus_url`) pass.
- **DVCS-BUS-PRECHECK-01:** `bus_handler::handle_bus_export` runs
  PRECHECK A via `git ls-remote -- <mirror_url> refs/heads/main`;
  on drift emits `error refs/heads/main fetch first` to stdout +
  hint to stderr (*"your GH mirror has new commits..."*); bails
  BEFORE PRECHECK B; integration test
  (`cargo test -p reposix-remote --test bus_precheck_a`) passes;
  `--` separator + `-`-prefix reject ARE in code (grep-able).
- **DVCS-BUS-PRECHECK-02:** `bus_handler::handle_bus_export` runs
  PRECHECK B via `precheck::precheck_sot_drift_any(...)`; on `Drifted`
  emits `error refs/heads/main fetch first` + hint citing
  `refs/mirrors/<sot>-synced-at` (when populated, via
  `read_mirror_synced_at`); bails BEFORE stdin read; integration
  test (`cargo test -p reposix-remote --test bus_precheck_b`) passes.
- **DVCS-BUS-FETCH-01:** `crates/reposix-remote/src/main.rs:150-172`
  capabilities arm gates `proto.send_line("stateless-connect")?;`
  on `state.mirror_url.is_none()`; integration test
  (`cargo test -p reposix-remote --test bus_capabilities`) asserts
  bus URL omits `stateless-connect` from capability list.
- **No-remote-configured (SC5 + 6th catalog row):** `bus_handler` STEP
  0's URL-match lookup fires BEFORE PRECHECK A; zero matches → emit
  verbatim Q3.5 hint; integration test asserts the hint string
  appears in stderr verbatim.
- New catalog rows in `quality/catalogs/agent-ux.json` (6); each
  verifier exits 0; status PASS after T06.
- Recurring (per phase): catalog-first ordering preserved (T01 commits
  catalog rows BEFORE T02–T06 implementation); per-phase push completed;
  verdict file at `quality/reports/verdicts/p82/VERDICT.md`; CLAUDE.md
  updated in T06.

# Verification approach (developer-facing)

After T06 pushes and the orchestrator dispatches the verifier subagent:

```bash
# Verifier-equivalent invocations (informational; the verifier subagent runs from artifacts):
bash quality/gates/agent-ux/bus-url-parses-query-param-form.sh
bash quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh
bash quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh
bash quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh
bash quality/gates/agent-ux/bus-fetch-not-advertised.sh
bash quality/gates/agent-ux/bus-no-remote-configured-error.sh
python3 quality/runners/run.py --cadence pre-pr  # re-grade catalog rows
cargo nextest run -p reposix-remote --test bus_url           # parser unit tests
cargo nextest run -p reposix-remote --test bus_capabilities  # capability list omits stateless-connect for bus URL
cargo nextest run -p reposix-remote --test bus_precheck_a    # mirror drift fixture
cargo nextest run -p reposix-remote --test bus_precheck_b    # SoT drift wiremock fixture
cargo nextest run -p reposix-remote                          # full crate test sweep
```

The fixtures for PRECHECK A use **two local bare repos**
(`tempfile::tempdir()` + `git init --bare` + `file://` URL) per
RESEARCH.md § "Test Fixture Strategy". Same approach as
`scripts/dark-factory-test.sh`. The PRECHECK B fixture uses
**wiremock** mirroring P81's `tests/perf_l1.rs` setup pattern.

This is a **subtle point worth flagging**: success criteria 2-3 (the
prechecks) are satisfied by two contracts simultaneously: (a) the
helper exits non-zero AND emits the expected stdout/stderr lines, AND
(b) the helper makes ZERO REST writes to wiremock AND ZERO stdin
reads. The integration tests assert BOTH.
