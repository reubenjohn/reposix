← [back to index](./index.md)

# Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria
across every v0.13.0 phase":

1. **All commits pushed.** Plan terminates with `git push origin main`
   in T04 (per CLAUDE.md "Push cadence — per-phase", codified
   2026-04-30, closes backlog 999.4). Pre-push gate-passing is part of
   the plan's close criterion.
2. **Pre-push gate GREEN.** If pre-push BLOCKS: treat as plan-internal
   failure (fix, NEW commit, re-push). NO `--no-verify` per CLAUDE.md
   git safety protocol.
3. **Verifier subagent dispatched.** AFTER 81-01 pushes (i.e., after
   T04 completes), the orchestrator dispatches an unbiased verifier
   subagent per `quality/PROTOCOL.md` § "Verifier subagent prompt
   template" (verbatim copy). The subagent grades the three P81
   catalog rows from artifacts with zero session context.
4. **Verdict at `quality/reports/verdicts/p81/VERDICT.md`.** Format per
   `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P80 SHIPPED ... next P81" → "P81 SHIPPED 2026-MM-DD"
   (commit SHA cited).
6. **CLAUDE.md updated in T04.** T04's CLAUDE.md edit lands in the
   terminal commit (two paragraphs per D-05).
7. **REQUIREMENTS.md DVCS-PERF-L1-01..03 checkboxes flipped.**
   Orchestrator (top-level) flips `[ ]` → `[x]` after verifier GREEN.
   NOT a plan task.

# Risks + mitigations

| Risk                                                                                                  | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                |
|-------------------------------------------------------------------------------------------------------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`wiremock::Mock::expect(0)` doesn't actually fail RED if the matcher is reverted** (RESEARCH.md MEDIUM) | MEDIUM     | T04 includes a positive-control sibling test (`positive_control_list_records_call_fails_red`) that flips `expect(0)` to `expect(1)` and confirms wiremock panics on Drop when the assertion fails. If the positive-control test SKIPs or PASSes when it should FAIL, the assertion contract is broken and the executor surfaces this as a SURPRISES-INTAKE candidate per OP-8. |
| **Backend-side delete race (L1-strict trade-off; D-01)**                                              | LOW        | The trade-off is RATIFIED (D-01); T02 inline comment, plan body, and CLAUDE.md all surface it; user recovery is `reposix sync --reconcile` (T03). NOT a SURPRISES candidate — the trade is intentional and documented.                                                                                  |
| **Clock skew false-positives on own-write race (T-81-03; RESEARCH.md § Pitfall 2)**                   | LOW        | Self-healing on next push (the just-written record now matches in cache; precheck passes). T02 inline comment names this as a known L1 quirk; no mitigation code needed. If real-world incidence is high, file as v0.14.0 OTel work (already in scope).                                              |
| **`plan()`'s `prior` slice shape requires materializing Records from cache (D-03)**                   | LOW        | The materialization helper lives in the new `precheck.rs` module (free function) so the new code path is grep-discoverable. Hot-path optimization (RESEARCH.md § Pitfall 5): only parse if id is in `changed_set` AND in our push. Bound: 5–10 records typical per push.                                |
| **Helper's `state.rt.block_on` over `list_changed_since` adds latency** (RESEARCH.md § Pitfall 6)     | LOW        | Existing pattern — `handle_export` already calls `state.rt.block_on(state.backend.list_records(...))` at line 335. Same idiom for `list_changed_since`. No latency regression introduced.                                                                                                              |
| **Cache prior-blob parse leaks Tainted bytes into log lines (T-81-02)**                               | LOW-MED    | Existing `log_helper_push_rejected_conflict` shape preserves the contract: id + versions only. T02 wiring uses `Tainted::inner_ref()` solely for `version` extraction; never echoes body bytes. Unit-tested via the existing conflict-detection coverage in `crates/reposix-remote/src/`.                |
| **First-push fallback path skipped in tests (S2)**                                                   | LOW        | T04's perf regression test ALSO seeds the cursor before driving the export verb (the wiremock setup writes `last_fetched_at` via the same `Cache::build_from` path that `reposix init` uses). The fallback path is exercised separately by an existing unit test in `crates/reposix-cache/`'s sync coverage.                |
| **`reposix sync` (no flags) printing a hint vs erroring** (D-02)                                     | LOW        | Decision RATIFIED (D-02). The smoke test in `crates/reposix-cli/tests/sync.rs` covers `--reconcile`; the bare-form behavior is a single `println!` line in the handler.                                                                                                                                  |
| **Cargo memory pressure** (load-bearing CLAUDE.md rule)                                              | LOW        | Strict serial cargo across all four tasks. Per-crate fallback (`cargo check -p reposix-cache` then `cargo check -p reposix-remote` then `cargo check -p reposix-cli`) is documented in each task. T01 + T04 epilogue are doc-only; T02 + T03 + T04 test-run are the cargo-bearing tasks (sequential).                                  |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P81**                                    | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`.                                                                                                                                          |

# +2 reservation: out-of-scope candidates

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` exist already (created during P79). P81 surfaces
candidates only when they materialize during execution — none pre-filed
at planning time.

Anticipated candidates the plan flags (per OP-8):

- **LOW** — `wiremock::Mock` matcher API name differs from RESEARCH.md
  (e.g., `query_param_exists` vs the actual function name).
  Eager-resolve in T04 by reading the wiremock 0.6.x docs; if a custom
  `Match` impl is needed, bound ≤ 30 lines and document in T04's commit
  message. NOT a SURPRISES candidate unless the impl exceeds 30 lines.
- **LOW** — `BackendConnector::list_changed_since` async signature
  doesn't compose cleanly with `state.rt.block_on(...)` (e.g., a borrow
  issue with `&state.project`). Eager-resolve in T02 by mirroring the
  existing `block_on(state.backend.list_records(...))` pattern at line 335.
  NOT a SURPRISES candidate.
- **LOW** — `Cache::list_record_ids` query returns rows from previous
  `(backend, project)` pairs in the same database (cross-pair contamination).
  Verified at planning time via `crates/reposix-cache/src/cache.rs:345-368`
  — the SQL has `WHERE backend = ?1 AND project = ?2`. NOT a candidate.
- **LOW** — gix `read_blob_cached` returns `Option<Tainted<Vec<u8>>>` and
  the call site that parses frontmatter doesn't handle Tainted explicitly.
  Eager-resolve in T02 by using `Tainted::inner_ref()` (documented existing
  accessor) on the `Some` arm. If the gix 0.83 "object not found" discriminant
  doesn't cleanly match (the new `read_blob_cached` discriminates via
  `to_string().contains("not found" | "NotFound")` as a fallback per
  the implementation sketch), file as SURPRISES-INTAKE.

Items NOT in scope for P81 (deferred per the v0.13.0 ROADMAP):

- Bus remote URL parser / prechecks / writes (P82+). The L1 precheck
  function lives in `precheck.rs` so the bus handler (P82) consumes it
  directly; bus integration is P82's territory.
- Webhook-driven sync (P84). Out of scope. P81 has no webhook surface.
- DVCS docs (P85). Out of scope; T04 only updates CLAUDE.md. The
  `docs/concepts/dvcs-topology.md` user-facing explanation of L1 +
  `--reconcile` defers to P85.
- L2 cache-desync hardening (background reconcile job). Deferred to
  v0.14.0 per `architecture-sketch.md § Performance subtlety`.
- L3 transactional cache writes (cache invariants enforced at write
  time). Deferred to v0.14.0.
- Multi-SoT attach (Q1.2). Out of scope per v0.13.0 vision.

# Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                                                      | Delegation                                                                                                                                                                                                                  |
|------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 81-01 T01 (3 catalog rows + 2 verifier shells)                  | `gsd-executor` — catalog-first commit; **hand-edits perf-targets.json + agent-ux.json per documented gap (NOT Principle A); doc-alignment row minted via `reposix-quality doc-alignment bind` (Principle A applies)**.       |
| 81-01 T02 (cache wrappers + helper precheck rewrite)            | Same 81-01 executor. Cargo lock held for `reposix-cache` then `reposix-remote`. Per-crate cargo only.                                                                                                                       |
| 81-01 T03 (`reposix sync --reconcile` CLI + smoke test)         | Same 81-01 executor. Cargo lock held for `reposix-cli`. Per-crate cargo only.                                                                                                                                                |
| 81-01 T04 (perf regression test + verifier flip + CLAUDE.md + push) | Same 81-01 executor (terminal task). Cargo lock for `reposix-remote` integration test run. Per-crate cargo only.                                                                                                            |
| Phase verifier (P81 close)                                       | Unbiased subagent dispatched by orchestrator AFTER 81-01 T04 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades the three catalog rows from artifacts.        |

Phase verifier subagent's verdict criteria (extracted for P81):

- **DVCS-PERF-L1-01:** `crates/reposix-remote/src/main.rs::handle_export`
  no longer calls `state.backend.list_records(&state.project)` on the
  hot path (cursor-present case); the precheck function lives in
  `crates/reposix-remote/src/precheck.rs`; the precheck rejects on
  version-mismatch with detailed error citing record id + cache version
  + backend version; success path updates `cache.write_last_fetched_at(now)`;
  perf regression test passes (`cargo test -p reposix-remote --test perf_l1`).
- **DVCS-PERF-L1-02:** `reposix sync --reconcile` subcommand exists in
  `crates/reposix-cli/src/main.rs` (clap-derive); handler in
  `crates/reposix-cli/src/sync.rs` calls `Cache::build_from`; smoke
  test passes (`cargo test -p reposix-cli --test sync`); helper-stderr
  hint cites `reposix sync --reconcile` on cache-desync error paths.
- **DVCS-PERF-L1-03:** the precheck function in `precheck.rs` is the
  single conflict-detection mechanism — `handle_export` calls it AND
  the future bus handler (P82+) will call it; no path-specific copies.
  L2/L3 deferral comment present in `precheck.rs` with verbatim cite to
  `architecture-sketch.md § Performance subtlety` and the v0.14.0 doc.
- New catalog rows in `quality/catalogs/perf-targets.json` (1) +
  `quality/catalogs/agent-ux.json` (1) + `quality/catalogs/doc-alignment.json`
  (1 BOUND); each verifier exits 0; status PASS after T04.
- Recurring (per phase): catalog-first ordering preserved (T01 commits
  catalog rows BEFORE T02–T04 implementation); per-phase push completed;
  verdict file at `quality/reports/verdicts/p81/VERDICT.md`; CLAUDE.md
  updated in T04 (two paragraphs per D-05).

# Verification approach (developer-facing)

After T04 pushes and the orchestrator dispatches the verifier subagent:

```bash
# Verifier-equivalent invocations (informational; the verifier subagent runs from artifacts):
bash quality/gates/perf/list-call-count.sh
bash quality/gates/agent-ux/sync-reconcile-subcommand.sh
python3 quality/runners/run.py --cadence pre-pr  # re-grade catalog rows
cargo nextest run -p reposix-cache               # cursor wrapper unit tests
cargo nextest run -p reposix-cli --test sync     # CLI smoke
cargo nextest run -p reposix-remote --test perf_l1 # perf regression + positive control
node ./node_modules/@gsd-build/sdk/dist/cli.js query verify.docs-alignment-walk \
    --catalog quality/catalogs/doc-alignment.json   # walker: doc-alignment row remains BOUND
```

The fixture for the perf regression test is **wiremock** per RESEARCH.md
§ "Test Fixture Strategy" — same approach as P73 connector contract tests.
The N=200 figure makes the difference observable while keeping the test
sub-second; the sim's actual page size MUST be confirmed in T04 read_first
(if the sim doesn't paginate at 50, scale N up so the assertion `expect(0)`
is meaningful).

This is a **subtle point worth flagging**: success criterion 4 (perf
regression test) is satisfied by COUNTING REST calls via wiremock matchers,
NOT by measuring wall-clock latency. The catalog row's verifier shell
delegates to the cargo test (TINY shape, ~30 lines: cargo build → cargo
test → grep "test result: ok" → exit 0).
