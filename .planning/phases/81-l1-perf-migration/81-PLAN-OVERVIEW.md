---
phase: 81
title: "L1 perf migration — `list_changed_since`-based conflict detection"
milestone: v0.13.0
requirements: [DVCS-PERF-L1-01, DVCS-PERF-L1-02, DVCS-PERF-L1-03]
depends_on: [80]
plans:
  - 81-01-PLAN.md  # DVCS-PERF-L1-01..03 (catalog → cache cursor + precheck wiring → sync subcommand → perf regression test + close)
waves:
  1: [81-01]
---

# Phase 81 — L1 perf migration (overview)

This is the THIRD DVCS-substantive phase of milestone v0.13.0 — the
"performance" leg that lands BEFORE the bus remote ships in P82–P83 so
the bus inherits the cheap conflict-detection path, NOT the expensive
one. Per `decisions.md` Q3.1 (RATIFIED inline L1) and the
architecture-sketch's strong recommendation: ship L1 as part of v0.13.0
or the bus remote's `git push` round-trip will do 100+ REST calls on
every push and dismiss reposix as a toy. **Single plan, four sequential
tasks** per RESEARCH.md § "Plan Splitting":

- **T01 — Catalog-first.** Three rows mint BEFORE any Rust edits: one in
  `quality/catalogs/perf-targets.json`
  (`perf/handle-export-list-call-count`), one in
  `quality/catalogs/agent-ux.json` (`agent-ux/sync-reconcile-subcommand`),
  one in `quality/catalogs/doc-alignment.json`
  (`docs-alignment/perf-subtlety-prose-bound`, minted via the
  `reposix-quality doc-alignment bind` verb — Principle A applies in
  the docs-alignment dim). Two new TINY shell verifiers under
  `quality/gates/perf/` and `quality/gates/agent-ux/`. Initial status
  `FAIL`. Hand-edit per documented gap (NOT Principle A) for the perf +
  agent-ux rows — same shape as P80's `agent-ux/mirror-refs-*` rows,
  tracked by GOOD-TO-HAVES-01.
- **T02 — Cache cursor wrappers + helper precheck rewrite.** Two new
  thin public methods on `Cache` (`read_last_fetched_at`,
  `write_last_fetched_at`) that wrap the existing
  `meta::get_meta`/`set_meta` calls keyed by `"last_fetched_at"` (NOT a
  new table). PLUS one new sync gix-only primitive
  `Cache::read_blob_cached(oid) -> Result<Option<Tainted<Vec<u8>>>>`
  (H1 fix from PLAN-CHECK 2026-05-01) — local-only inspector that
  returns `Ok(None)` on cache miss instead of fetching from backend.
  The precheck path uses this NOT the async `read_blob` (which would
  add a hidden backend GET per cache prior, defeating L1). Helper `handle_export` (lines 334–348 + 384–427 + 489–528
  in `crates/reposix-remote/src/main.rs` post-P80) gets the L1
  precheck-rewrite: read cursor → `list_changed_since(since)` → for each
  changed record AND in our push, parse cache prior + re-GET backend
  current → compare versions → reject on mismatch; otherwise `plan()`
  against cache-derived prior; on success update cursor. New free
  function `precheck_export_against_changed_set` lives in a new module
  `crates/reposix-remote/src/precheck.rs` so both `handle_export` (P81)
  and the future bus handler (P82–P83) consume the same code path —
  the SoT-precheck and the single-backend precheck are the same
  algorithm. L1-strict delete trade-off ratified in plan body (D-04
  below) and surfaced as an inline comment in `precheck.rs` citing
  `architecture-sketch.md § Performance subtlety` and the v0.14.0
  L2/L3 deferral target. First-push fallback: when `last_fetched_at`
  is `None`, fall through to the existing `list_records` walk for THIS
  push only, then write the cursor; subsequent pushes hit the L1 fast
  path.
- **T03 — `reposix sync --reconcile` CLI.** New `Sync { reconcile }`
  subcommand in `crates/reposix-cli/src/main.rs` + new module
  `crates/reposix-cli/src/sync.rs` (~30 lines: a thin wrapper over
  `Cache::build_from`). Without `--reconcile` the subcommand prints a
  one-line hint pointing at `--reconcile` (NOT a no-op error — the
  command exists in v0.13.0 specifically as the L1 escape hatch).
  Smoke test in `crates/reposix-cli/tests/sync.rs` (one
  `#[tokio::test]`) drives `reposix sync --reconcile` against the sim,
  asserts cache was rebuilt (`last_fetched_at` advanced).
- **T04 — Perf regression test + CLAUDE.md update + verifier flip +
  close.** New integration test
  `crates/reposix-remote/tests/perf_l1.rs` with N=200 records seeded
  in a wiremock mock server, asserts the precheck makes ≥1
  `list_changed_since` REST call AND **zero** `list_records` REST
  calls. Includes a positive-control test that flips `expect(0)` to
  `expect(1)` and confirms the matcher fails RED if reverted —
  closes RESEARCH.md MEDIUM risk "wiremock semantics need confirmation".
  Flip the three catalog rows FAIL → PASS via the runner before the
  per-phase push. CLAUDE.md update lands in the same commit (one
  paragraph in § Commands documenting `reposix sync --reconcile`; one
  paragraph in § Architecture naming the L1 cost-vs-correctness trade
  + pointing at `architecture-sketch.md`). `git push origin main`
  with pre-push GREEN. The orchestrator then dispatches the verifier
  subagent.

Sequential — never parallel. Even though T02 (cache crate + remote
crate) and T03 (cli crate) touch different crates, sequencing per
CLAUDE.md "Build memory budget" rule (one cargo invocation at a time,
never two in parallel) makes this strictly sequential.

## Wave plan

Strictly sequential — one plan, four tasks. T01 → T02 → T03 → T04
within the same plan body. The plan is its own wave.

| Wave | Plans  | Cargo? | File overlap        | Notes                                                                                    |
|------|--------|--------|---------------------|------------------------------------------------------------------------------------------|
| 1    | 81-01  | YES    | none with prior phase | catalog + cache wrappers + helper precheck rewrite + new CLI subcommand + perf test + close — all in one plan body |

`files_modified` audit (single-plan phase, no cross-plan overlap to
audit; line numbers cited at planning time and require re-confirmation
during T02 read_first):

| Plan  | Files                                                                                                                                                                                                                                                                          |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 81-01 | `crates/reposix-cache/src/cache.rs` (new methods on `impl Cache`), `crates/reposix-remote/src/main.rs` (lines 334–348 rewritten + 489–528 cursor write), `crates/reposix-remote/src/precheck.rs` (new), `crates/reposix-remote/src/lib.rs` or `mod precheck;` declaration in `main.rs`, `crates/reposix-remote/tests/perf_l1.rs` (new), `crates/reposix-cli/src/main.rs` (new `Sync` Cmd variant), `crates/reposix-cli/src/lib.rs` (new `pub mod sync;`), `crates/reposix-cli/src/sync.rs` (new), `crates/reposix-cli/tests/sync.rs` (new), `quality/catalogs/perf-targets.json` (1 new row), `quality/catalogs/agent-ux.json` (1 new row), `quality/catalogs/doc-alignment.json` (1 row bound via `reposix-quality doc-alignment bind`), `quality/gates/perf/list-call-count.sh` (new), `quality/gates/agent-ux/sync-reconcile-subcommand.sh` (new), `CLAUDE.md` |

Per CLAUDE.md "Build memory budget" the executor holds the cargo lock
sequentially across T02 → T03 → T04. No parallel cargo invocations.
Doc-only tasks (T01: catalog rows + 2 verifier shells; T04 epilogue:
CLAUDE.md edit) do NOT compile and may interleave freely with other
doc-only work outside this phase if the orchestrator schedules them —
but within this plan they remain sequential for executor simplicity.

## Plan summary table

| Plan  | Goal                                                                                                          | Tasks | Cargo? | Catalog rows minted | Tests added                                                                                                           | Files modified (count) |
|-------|---------------------------------------------------------------------------------------------------------------|-------|--------|----------------------|-----------------------------------------------------------------------------------------------------------------------|------------------------|
| 81-01 | L1 conflict detection (cache cursor + helper precheck rewrite + sync --reconcile CLI + perf regression test) | 4     | YES    | 3 (status FAIL → PASS at T04) | 2 unit (cursor read/write round-trip + cursor None-when-absent) + 1 cli smoke + 2 integration (perf regression positive + positive-control RED) = 5 total | ~14 (1 new precheck module + 1 new sync module + 1 new cli test + 1 new perf test + 2 new verifier shells + 3 catalog edits + cache.rs + main.rs + lib.rs + CLAUDE.md) |

Total: 4 tasks across 1 plan. Wave plan: sequential.

Test count: 2 unit cursor wrappers + 2 unit read_blob_cached (in `cache.rs` `#[cfg(test)] mod tests`) + 1
CLI smoke (`crates/reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor`) + 2
integration (in `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`
+ `positive_control_list_records_call_fails_red`) = 5 total.

## Decisions ratified at plan time

The five open questions surfaced by RESEARCH.md § "Open Questions for the
Planner" are RATIFIED here so the executing subagent and the verifier
subagent both grade against the same contract. Each decision references
the source artifact and the rationale.

### D-01 — L1-strict delete trade-off (RATIFIED)

**Decision:** L1-strict — the cache is trusted as the prior set; backend-side
deletes are NOT detected by the precheck. The agent's `PATCH` against a
backend-deleted record fails at REST time with a 404, surfaced to the user
as a normal write error. The user recovery path is `reposix sync --reconcile`
(T03) which does a full `list_records` walk and rebuilds the cache.

**Why this trade-off is acceptable:** the "agent edits a record someone
else deleted on backend" race is rare (the Confluence/JIRA/GitHub UIs all
discourage delete-without-archive); the failure mode is user-visible AND
recoverable (REST 404 with a clear error citing the record id); L2/L3
hardening (v0.14.0) addresses the residual gap via a background reconcile
job (L2) or transactional cache writes (L3).

**Surface in three places per RESEARCH.md § Documentation Deferrals:**

1. Inline comment in `crates/reposix-remote/src/precheck.rs` near the
   precheck function citing `.planning/research/v0.13.0-dvcs/architecture-sketch.md
   § Performance subtlety` AND
   `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md
   § L2/L3 cache-desync hardening`. Future agents reading the helper
   code shouldn't have to rediscover the cost-vs-correctness tradeoff
   from scratch.
2. Plan body (this overview + 81-01-PLAN.md `<must_haves>` block) names
   the trade-off verbatim so the verifier subagent grades against the
   same contract.
3. CLAUDE.md update in T04 — one paragraph in § Architecture (or §
   Threat model) names the L1 trade and points at the architecture-sketch.

**Source:** `.planning/research/v0.13.0-dvcs/decisions.md` Q3.1 (RATIFIED
inline L1); `.planning/research/v0.13.0-dvcs/architecture-sketch.md`
§ "Performance subtlety: today's `list_records` walk on every push";
RESEARCH.md § Pitfall 3.

### D-02 — `Sync { reconcile }` subcommand (chosen over `Refresh --reconcile`)

**Decision:** New `reposix sync --reconcile` subcommand (RESEARCH.md path a).

**Why not extend `refresh`:** `reposix refresh` writes `.md` files into a
working tree (different concern from cache rebuild). Conflating the two
would muddy CLI semantics — a user reading `reposix refresh --reconcile`
might reasonably expect a working-tree refresh, not a cache rebuild. The
ROADMAP and architecture-sketch already canonically use `reposix sync
--reconcile`; users will be told to type that exact string in error
messages and docs.

**`reposix sync` (no flags) behavior:** prints a one-line hint pointing
at `--reconcile`, exits 0. NOT an error. Rationale: the architecture-sketch
positions `reposix sync` as a v0.13.0+ surface (the bus-remote handler may
later call it on certain reject paths). Reserving the bare `reposix sync`
form for future flag combinations (e.g., `--push-only` in v0.14.0) is
cheaper than reclaiming a name that errored out.

**Source:** RESEARCH.md § Open Questions #2; `.planning/REQUIREMENTS.md`
DVCS-PERF-L1-02 names `reposix sync --reconcile` verbatim.

### D-03 — `plan()` signature unchanged (helper materializes prior from cache)

**Decision:** keep `plan(prior: &[Record], parsed: &ParsedExport)` (RESEARCH.md
path a). The helper materializes a `Vec<Record>` from
`cache.list_record_ids()` + per-id `find_oid_for_record` + `read_blob` +
`frontmatter::parse` BEFORE calling `plan()`. `diff.rs` is untouched.

**Why not widen `plan()`'s signature:** widening to `Vec<RecordId>` + a
closure for lazy fetch would require updating every existing test in
`crates/reposix-remote/src/diff.rs::tests` (34 tests as of P80) plus any
internal callers — wider blast radius than the local helper rewrite. The
parse-cost overhead (5–10 records typical per push) is negligible. The
hot path (record not in `changed_set`) skips the parse entirely — the
materialization loop only walks the cache prior for IDs ALSO in our push.

**Implementation note:** the helper-side prior-materialization helper
lives in the new `precheck.rs` module as a free function so both the
single-backend `handle_export` AND the future bus handler share it.

**Source:** RESEARCH.md § Open Questions #3; `crates/reposix-remote/src/diff.rs:99`.

### D-04 — `perf-targets.json` is the catalog home (NOT a new file)

**Decision:** add the new perf row to the existing
`quality/catalogs/perf-targets.json` (one row joins the existing 3 rows;
none of the existing rows conflict). NOT a new `dvcs-perf.json` file.

**Why:** dimension catalogs are routed to `quality/gates/<dim>/` runner
discovery — `perf` is the existing dimension. Splitting the perf dimension
into two catalog files would force the runner to discover both via tag,
adding indirection for no benefit. The 3 existing perf rows are all
WAIVED (P63 deferral); the new L1 row is the first non-WAIVED perf row
since v0.12.0, which is a positive signal in its own right.

**Source:** RESEARCH.md § Open Questions #4;
`quality/catalogs/perf-targets.json` (existing file with 3 WAIVED rows).

### D-05 — CLAUDE.md update scope (two paragraphs, two sections)

**Decision:** T04 lands two paragraphs in CLAUDE.md, in the same PR as
the implementation, per QG-07:

1. **§ Commands → "Local dev loop" block** — one bullet documents
   `reposix sync --reconcile` with a one-line example (post the existing
   `reposix init sim::demo` line):
   ```
   reposix sync --reconcile                                  # full list_records walk + cache rebuild (L1 escape hatch)
   ```
2. **§ Architecture (after the cache reconciliation paragraph) OR a new
   `## L1 conflict detection` sub-section under § Architecture** — one
   paragraph (3–5 sentences):
   ```
   L1 conflict detection (P81+). On every push, the helper reads its
   cache cursor (`meta.last_fetched_at`), calls `backend.list_changed_since(since)`,
   and only conflict-checks records that overlap the push set with the
   changed-set. The cache is trusted as the prior; the agent's PATCH
   against a backend-deleted record fails at REST time with a 404 —
   recoverable via `reposix sync --reconcile`. L2/L3 hardening
   (background reconcile / transactional cache writes) defers to v0.14.0
   per `.planning/research/v0.13.0-dvcs/architecture-sketch.md
   § Performance subtlety`.
   ```

**Why both placements:** § Commands gets the user-facing mention; §
Architecture gets the cost-vs-correctness rationale. A single
combined paragraph in one section would either (a) leak architectural
detail into the Commands block, or (b) bury the user-facing escape
hatch in the Architecture block. Two separate paragraphs match the
existing CLAUDE.md style (e.g., the `reposix attach` documentation
already lives in both § Architecture and § Commands).

**Source:** RESEARCH.md § Open Questions #5; CLAUDE.md § Commands +
§ Architecture existing structure.

## Subtle architectural points (read before T02)

The two below are flagged because they are the most likely sources of
T02 review friction. Executor must internalize them before writing
the wiring code.

### S1 — `last_fetched_at` is the INBOUND cursor; `refs/mirrors/<sot>-synced-at` is the OUTBOUND cursor

These two timestamps move together on a successful push but are
conceptually distinct. P80 introduced `refs/mirrors/<sot>-synced-at`
as the OUTBOUND cursor (when did the GH mirror last receive a push from
us). P81 wires the existing INBOUND cursor (`meta.last_fetched_at` —
when did the cache last fetch from SoT) into the helper's precheck
path.

**Why this matters for T02.** A reviewer skimming the wiring may expect
the helper to read `cache.read_mirror_synced_at(&backend_name)` for the
"since" parameter. That would be wrong. `mirror_synced_at` measures
OUTBOUND staleness (last successful mirror push); `list_changed_since`
needs INBOUND staleness (last successful SoT fetch). Read
`cache.read_last_fetched_at()` (NEW in T02) — the wrapper around
`meta::get_meta(conn, "last_fetched_at")` already populated by
`Cache::build_from` and `Cache::sync` (`crates/reposix-cache/src/builder.rs:119`,
`:329` — the existing canonical writers).

P80's `mirror_refs.rs` doc-comment explicitly distinguishes the two; P81
adds a parallel comment in `precheck.rs` so future agents don't conflate
them.

### S2 — First-push fallback (no cursor yet)

The cache's `meta.last_fetched_at` row is populated by the FIRST call to
`Cache::build_from` (during `reposix init` or `reposix attach`). On a
fresh install where the agent runs `reposix init` and then immediately
`git push`, the cursor IS populated — `init` calls `build_from` which
writes `last_fetched_at = Utc::now()` per
`crates/reposix-cache/src/builder.rs:119`. Push 0 already has a cursor.

**However**, there is a real scenario where the cursor is `None`: when
`state.cache` is `Some` but the cache was opened lazily by the helper
(i.e., the user `git clone`'d the cache's bare repo manually OR a
malformed install) and no `build_from` has run. In that case
`read_last_fetched_at()` returns `Ok(None)`.

**Decision (per RESEARCH.md § Pitfall 1):** treat `None` as "fall through
to the existing `list_records` walk for THIS push only; subsequent pushes
hit the L1 fast path." The cost is unchanged for the rare first-push
case; every subsequent push is fast.

**Alternative considered + rejected:** `None` → `epoch (1970-01-01)`.
Rejected because for any non-tiny backend, `list_changed_since(epoch)`
returns the entire dataset and the call is paginated identically to
`list_records` — no win. Confluence CQL `lastModified > "1970-01-01
00:00:00"` against a TokenWorld-sized space would still take 100 calls.

T02's wiring uses an explicit `match` on the cursor and routes to the
existing `list_records` code path (verbatim from current `handle_export`
lines 334–348) when the cursor is absent, then writes the cursor on
success.

## Hard constraints (carried into the plan body)

Per the user's directive (orchestrator instructions for P81) and
CLAUDE.md operating principles:

1. **Catalog-first (QG-06).** T01 mints THREE rows + TWO verifier shells
   BEFORE T02–T04 implementation. Initial status `FAIL`. The
   `perf-targets.json` and `agent-ux.json` rows are hand-edited per
   documented gap (NOT Principle A) — annotated in commit message
   referencing GOOD-TO-HAVES-01. The `doc-alignment.json` row is
   minted via `reposix-quality doc-alignment bind` (Principle A applies
   to docs-alignment dim).
2. **Per-crate cargo only (CLAUDE.md "Build memory budget").** Never
   `cargo --workspace`. Use `cargo check -p reposix-cache`,
   `cargo check -p reposix-remote`, `cargo check -p reposix-cli`,
   `cargo nextest run -p <crate>`. Pre-push hook runs the workspace-wide
   gate; phase tasks never duplicate.
3. **Sequential execution.** Tasks T01 → T02 → T03 → T04 — never parallel,
   even though T02 (cache + remote) and T03 (cli) touch different crates.
   CLAUDE.md "Build memory budget" rule is "one cargo invocation at a
   time" — sequencing the tasks naturally honors this.
4. **L1-strict delete trade-off RATIFIED (D-01).** The plan body, the
   inline comment in `precheck.rs`, and CLAUDE.md all carry the same
   verbatim trade-off statement.
5. **Both push paths use the same L1 mechanism (DVCS-PERF-L1-03).** No
   path-specific copies. `precheck_export_against_changed_set` lives in
   `crates/reposix-remote/src/precheck.rs` so both `handle_export` (P81)
   and the future bus handler (P82–P83) consume the same code path.
6. **`last_fetched_at` cursor is meta-table, not new table (S1).** Two
   thin Cache wrappers (`read_last_fetched_at`, `write_last_fetched_at`)
   over the existing `meta::get_meta`/`set_meta` API keyed by
   `"last_fetched_at"`. NO new table; NO new SQL DDL.
7. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
   codified 2026-04-30).** T04 ends with `git push origin main`; pre-push
   gate must pass; verifier subagent grades the three catalog rows
   AFTER push lands. Verifier dispatch is an orchestrator-level action
   AFTER this plan completes — NOT a plan task.
8. **CLAUDE.md update in same PR (QG-07; D-05).** T04 documents
   `reposix sync --reconcile` (§ Commands) + the L1 cost-vs-correctness
   trade-off (§ Architecture, citing `architecture-sketch.md`).
9. **First-push fallback (S2; D-02).** When `last_fetched_at` is `None`,
   the helper falls through to the existing `list_records` walk for
   THIS push only, then writes the cursor. NOT `epoch`-fallback. Surfaced
   via `tracing::info!` (single log line, NOT a hot path at scale).
10. **Performance regression test with positive control.** N=200 records
    via wiremock harness; counter-based assertion (`expect(0)` for
    `list_records`, `expect(1+)` for `list_changed_since`); positive-control
    test included as a sibling that flips `expect(0)` to `expect(1)` and
    confirms wiremock fails RED if the matcher is reverted (closes the
    MEDIUM risk in RESEARCH.md § Pitfalls and Risks).
11. **No new error variants.** Per the existing `Cache::log_*` family
    pattern, cursor-write failure WARN-logs and does NOT poison the push
    ack. NO new `RemoteError` variant nor new `cache::Error` variant.

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces NO new
trifecta surface. The L1 migration changes WHICH REST endpoint the
helper hits but does not introduce a new HTTP construction site:

| Existing surface              | What P81 changes                                                                                                                                                                                                                                                                |
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP          | UNCHANGED — `list_changed_since` is already implemented in all 4 connectors (`reposix-sim`, `reposix-confluence`, `reposix-github`, `reposix-jira`); no new HTTP call site is introduced. The same `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist applies. |
| Cache prior parse (Tainted bytes) | NEW: the precheck reads `cache.read_blob_cached(prior_oid)` (NEW sync gix-only primitive — H1 fix; returns `Ok(Some(Tainted<Vec<u8>>))` or `Ok(None)` on cache miss; does NOT touch the backend). Parsing tainted bytes is fine (no I/O side effects), but care is needed not to leak the tainted body into a log line. STRIDE category: Information Disclosure — mitigated by reusing the existing `log_helper_push_rejected_conflict` shape (records `id + versions` only; never echoes blob body). |
| Cursor write (`last_fetched_at`) | NEW: writes a single-row SQL upsert into `meta`. SQLite autocommit makes this atomic. Best-effort semantics match P80's `write_mirror_synced_at`. STRIDE category: Tampering — mitigated by the existing `meta::set_meta` API (parameterized SQL; no string concatenation). |
| Push reject diagnostics       | UNCHANGED — same `log_helper_push_rejected_conflict` shape with id + versions; no new bytes leak.                                                                                                                                                                              |

No `<threat_model>` STRIDE register addendum required beyond the three
threats the plan body's `<threat_model>` section enumerates per CLAUDE.md
template requirements:

- **T-81-01 (Tampering — cursor):** `meta.set_meta` parameterized SQL.
- **T-81-02 (Information Disclosure — Tainted prior bytes):** existing
  log_helper_push_rejected_conflict shape preserved (no body bytes leak).
- **T-81-03 (Denial of Service — false-positive on own-write race after
  cursor tick):** documented as a known L1 quirk (RESEARCH.md § Pitfall 2);
  self-healing on next push via `find_oid_for_record` returning the
  just-synced version. No new mitigation needed.

## Phase-close protocol

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

## Risks + mitigations

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

## +2 reservation: out-of-scope candidates

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

## Subagent delegation

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

## Verification approach (developer-facing)

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
