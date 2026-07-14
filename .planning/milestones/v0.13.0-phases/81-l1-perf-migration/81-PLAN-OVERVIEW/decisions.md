← [back to index](./index.md)

# Decisions ratified at plan time

The five open questions surfaced by RESEARCH.md § "Open Questions for the
Planner" are RATIFIED here so the executing subagent and the verifier
subagent both grade against the same contract. Each decision references
the source artifact and the rationale.

## D-01 — L1-strict delete trade-off (RATIFIED)

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

## D-02 — `Sync { reconcile }` subcommand (chosen over `Refresh --reconcile`)

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

## D-03 — `plan()` signature unchanged (helper materializes prior from cache)

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

## D-04 — `perf-targets.json` is the catalog home (NOT a new file)

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

## D-05 — CLAUDE.md update scope (two paragraphs, two sections)

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
