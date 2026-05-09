# Phase 98: Skeleton + shared-compute lift + edge model + walker + catalog + tracker schemas - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning
**Mode:** `--auto` discuss (autonomous workstream B; phase is heavily SPEC-locked via milestone ROADMAP + 28 ADRs)

<domain>
## Phase Boundary

Stand up the cross-link-fidelity dimension as a **sub-command of `reposix-quality`** (Q2 ratified). Three things, in this order:

1. **LIFT** `coverage.rs::{walk_md, eligible_files}` into a new shared module `crates/reposix-quality/src/md_walker.rs` (ADR-27, intra-crate option (b)). Both `docs-alignment` and `cross-link-fidelity` consume it.
2. **EXTEND** `hash.rs` with `pub fn heading_subtree_hash(file: &Path, slug: &str) -> Result<String>` (ADR-28). pulldown-cmark 0.13 AST walk; sha256 from matched heading until next same-or-higher-level heading.
3. **LAND** the cross-link skeleton: edge data model (path-derived edge identity per ADR-19), markdown walker emitting `(source, target, anchors)` tuples on top of the lifted module, **catalog schema** at `quality/catalogs/cross-link-fidelity.json` (~4 runner-readable rows per ADR-25), and **tracker schema** at `quality/state/cross-link-fidelity-tracker.json` (per-edge state per ADR-25).

EXPLICITLY OUT of P98: any L0/L1/L2/L3 verifier wiring (those are P100/P101/P102), bulk-move recovery (`cross-link rebind --auto` lands in P105), config TOML schema (P99), pre-commit hook integration (P100).

</domain>

<decisions>
## Implementation Decisions

### md_walker.rs lift (ADR-27 chosen option (b))
- **D-01:** Lift to `crates/reposix-quality/src/md_walker.rs` (intra-crate). Standalone-crate extraction option (c) is the v0.13.3 target, NOT v1.
- **D-02:** `coverage.rs` re-exports the lifted symbols for back-compat: `pub use crate::md_walker::{walk_md, eligible_files};`. All existing `coverage.rs` callers continue to compile unchanged.
- **D-03:** New module is `pub mod md_walker;` from `lib.rs`. Symbols are `pub fn walk_md(...)` and `pub fn eligible_files() -> Vec<PathBuf>` (currently `pub fn eligible_files`, `fn walk_md` — `walk_md` becomes `pub` as part of the lift since cross-link's walker calls it directly).
- **D-04:** Doc-alignment full test suite must pass against the lifted module BEFORE P98 closes. Verifier subagent grades RED if `cargo test -p reposix-quality` regresses.

### heading_subtree_hash (ADR-28)
- **D-05:** Function lands in `hash.rs` between `source_hash` (line 29) and `test_body_hash` (line 92). Same module — three hash flavors, one error-handling path.
- **D-06:** Implementation: pulldown-cmark 0.13 (already a workspace dep) parses the file; locate first heading whose GFM slug matches `slug`; sha256 the byte range from the matched heading START to the byte just before the NEXT same-or-higher-level heading (or EOF).
- **D-07:** Slug normalization follows GFM rules (lowercase, spaces→hyphens, strip non-`[a-z0-9-]`). This matches mkdocs-material slug behavior — required so cross-link's L1 anchor-existence checks (P100) align with the docs-alignment hashing semantics.
- **D-08:** Unit tests (≥3 per success-criterion 2): (a) matched heading; (b) unknown slug → `Err`; (c) multi-level nesting (H2 with nested H3 — subtree includes the H3, stops at next H2). Ride in `crates/reposix-quality/src/hash.rs#[cfg(test)] mod tests`.

### Cross-link sub-command shape (Q2 ratification + success criterion 3)
- **D-09:** Sub-command tree: `reposix-quality cross-link {walk, status, ...}` — registered as a clap subcommand sibling to existing `doc-alignment`. Shape mirrors the `doc-alignment` verb-tree exactly (consistency = lower agent-onboarding cost).
- **D-10:** First verb wired in P98 = `cross-link walk`. Output = JSON array of edge objects on stdout. Schema decided at D-13.
- **D-11:** `cross-link status` is STUBBED in P98 (returns "not yet implemented; see P100+") — placeholder so the CLI shape settles now. Real status output ships in P101.

### Edge data model (ADR-19 path-derived identity)
- **D-12:** Edge identity = `sha256(source_path || "::" || target_path || "::" || anchor_or_empty)`. Bulk-move recovery via `cross-link rebind --auto` is OUT of P98 (lands in P105 alongside `suggest-scopes`).
- **D-13:** Edge JSON shape (P98 walker output + tracker rows):
  ```json
  {
    "edge_id": "<sha256-hex>",
    "source": "docs/index.md",
    "source_line": 42,
    "target": "docs/concepts/foo.md",
    "anchor": "five-primitives",
    "anchor_line_in_target": 156
  }
  ```
  `anchor_line_in_target` is `null` when anchor is `null`. `source_line` is the 1-indexed line where the markdown link occurs in the source file.

### Catalog schema (ADR-25 + success criterion 5)
- **D-14:** `quality/catalogs/cross-link-fidelity.json` ships with exactly 4 runner-readable rows in P98 (no L0/L1/L2/L3 rows yet — those land in their respective phases):
  1. `cross-link-fidelity/skeleton-builds` — `kind: mechanical`, cadence `pre-push`, asserts `cargo build -p reposix-quality` succeeds.
  2. `cross-link-fidelity/walker-emits-edges` — `kind: mechanical`, cadence `pre-push`, asserts `reposix-quality cross-link walk | jq 'length'` returns a number in `[350, 450]` (success criterion 7).
  3. `cross-link-fidelity/tracker-schema-validates` — `kind: mechanical`, cadence `pre-push`, asserts the example tracker rows under `tests/` validate against the documented schema.
  4. `cross-link-fidelity/catalog-schema-validates` — `kind: mechanical`, cadence `pre-push`, asserts `quality/catalogs/cross-link-fidelity.json` itself conforms to `quality/catalogs/README.md`'s unified row schema.
- **D-15:** All 4 rows conform to the unified row schema in `quality/catalogs/README.md`. Discovered cleanly by `quality/runners/run.py:62-69` (verified by running the runner once during the phase).
- **D-16:** **Catalog-first commit rule:** these 4 rows land in the FIRST commit of P98, BEFORE any source code. Subsequent commits cite the row id in their message footer.

### Tracker schema (ADR-25 + ADR-1 strict-semver)
- **D-17:** `quality/state/cross-link-fidelity-tracker.json` ships with schema version `1.0.0` per ADR-1 strict-semver. Top-level shape:
  ```json
  {
    "schema_version": "1.0.0",
    "edges": [
      {
        "edge_id": "...",
        "source": "...", "source_line": 42,
        "target": "...", "anchor": "...",
        "scope": "default",
        "state": "UNGRADED",
        "last_graded_target_hash": null,
        "last_graded_at": null,
        "last_verdict": null,
        "last_verdict_id": null,
        "last_cost_cents": null
      }
    ]
  }
  ```
  States allowed (per ADR-11; full classifier ships in P101): `UNGRADED | GRADED | STALE | BROKEN`. P98 only writes `UNGRADED` (no graders wired).
- **D-18:** Tracker example fixtures live at `crates/reposix-quality/tests/fixtures/cross_link_tracker/{minimal,multi_scope,broken_edge}.json` — three rows minimum per success criterion 6. Schema validation test in `crates/reposix-quality/tests/cross_link_tracker_schema.rs`.
- **D-19:** Runner does NOT touch the tracker (per success criterion 6). Tracker is gate-internal state owned exclusively by `cross-link grade`/`cross-link walk --update-tracker` (latter ships in P101).

### CLAUDE.md + PROTOCOL.md updates (success criterion 8)
- **D-20:** CLAUDE.md § "Quality Gates" 9-dimension table grows to 10. New row: `cross-link | edge-walker + 4-level scrutiny ladder (L0 file exists → L1 anchor → L2 hash → L3 LLM judge) | quality/gates/cross-link/`. Detail row content fills incrementally as L0/L1/L2/L3 land in later phases.
- **D-21:** `quality/PROTOCOL.md` updated with (a) cross-link dimension's runtime contract, (b) the catalog/tracker file-split convention (catalog = runner-readable + asserts; tracker = gate-internal per-edge state).

### Verifier-subagent dispatch (OP-7 + recurring success criterion 4)
- **D-22:** Phase close MUST dispatch an unbiased verifier subagent that grades all 4 catalog rows from `quality/reports/verifications/cross-link-fidelity/` artifacts with zero session context. Verdict written to `quality/reports/verdicts/p98/VERDICT.md`. RED → loop back; do NOT close.
- **D-23:** Pre-push gate (`bash scripts/end-state.py` + dimension gates) must pass GREEN BEFORE verifier dispatch. Per CLAUDE.md "Push cadence — per-phase".

### Claude's Discretion
- Internal helper functions in `md_walker.rs` (path normalization helpers, ignore-rule constants) stay `pub(crate)` unless cross-link's walker explicitly needs them.
- Test fixture file names under `crates/reposix-quality/tests/fixtures/cross_link_tracker/` may be expanded beyond the three named in D-18 if implementation reveals more boundary cases worth pinning.
- Exact stderr/stdout split for `cross-link walk` (e.g., progress to stderr, JSON to stdout) follows the existing `doc-alignment walk` convention without re-deciding.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone-level (P98–P107 scope)
- `.planning/milestones/v0.13.2-phases/ROADMAP.md` — full per-phase scope, success criteria, recurring success criteria for all of v0.13.2. P98 details live at "### Phase 98".
- `.planning/milestones/v0.13.2-phases/REQUIREMENTS.md` — REQ-IDs cited by this phase.
- `.planning/milestones/v0.13.2-phases/SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md` — +2 reservation intake files (initialized from P98 onward).

### Cross-link-fidelity research bundle
- `.planning/research/v0.13.2-cross-link-fidelity/index.md` — entry point, thesis, mental model.
- `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` § "Five primitives" + § "Scope model" + § "Edge state taxonomy" + § "Four-level scrutiny ladder".
- `.planning/research/v0.13.2-cross-link-fidelity/03-schemas.md` § "Catalog schema" + § "Tracker schema (gate-internal)" + § "Config schema".
- `.planning/research/v0.13.2-cross-link-fidelity/06-decisions-log.md` — ALL 28 ADRs. P98-critical: ADR-1 (strict-semver), ADR-19 (path-derived edge identity), ADR-25 (catalog/tracker split), ADR-27 (md_walker lift), ADR-28 (heading_subtree_hash).
- `.planning/research/v0.13.2-cross-link-fidelity/07-extraction-plan.md` — v0.13.3 standalone crate extraction (NOT in P98 scope, but informs lift decision).
- `.planning/research/v0.13.2-cross-link-fidelity/08-open-questions.md` § "Owner ratification" — Q2 (Rust sub-command of `reposix-quality`), Q6 (cred-hygiene only at v1), Q14 (10-phase decomposition) — all RATIFIED.
- `.planning/research/v0.13.2-cross-link-fidelity/examples/tracker-row.json` — canonical tracker row example (informs D-17 + D-18).

### Existing codebase touch points (lift / extension targets)
- `crates/reposix-quality/src/coverage.rs:46` (`eligible_files`) + `coverage.rs:73` (`walk_md`) — LIFT TARGETS for ADR-27.
- `crates/reposix-quality/src/hash.rs:29` (`source_hash`) + `hash.rs:92` (`test_body_hash`) — EXTENSION TARGETS for ADR-28; new `heading_subtree_hash` lands between them.
- `crates/reposix-quality/src/lib.rs` — module declarations; new `pub mod md_walker;` lands here.
- `crates/reposix-quality/src/main.rs` (or `bin/reposix-quality.rs`) — clap subcommand registration; new `cross-link` verb-tree lands here.
- `quality/catalogs/README.md` — unified row schema; new catalog file MUST conform.
- `quality/runners/run.py:62-69` — catalog discovery loop; new file must be cleanly discovered.

### Project-wide grounding
- `CLAUDE.md` § "Quality Gates" — 9-dimension table grows to 10 in P98.
- `CLAUDE.md` § "Build memory budget" — cargo serialization rule (one cargo invocation at a time across both worktrees).
- `quality/PROTOCOL.md` — catalog row schema, verifier subagent prompt template, latency budgets.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `coverage.rs::walk_md` (recursive markdown collector) — moves verbatim into `md_walker.rs`; visibility flips `fn → pub fn`.
- `coverage.rs::eligible_files` (returns `Vec<PathBuf>` for `docs/**/*.md` + `README.md` + archived `REQUIREMENTS.md` files) — moves verbatim; called by both gates.
- `hash.rs::source_hash` + `hash.rs::test_body_hash` — sibling pattern for `heading_subtree_hash`; share the workspace's `sha2` + `pulldown-cmark` deps.
- `quality/runners/run.py` catalog discovery loop — already reads any `quality/catalogs/*.json` shaped per the unified schema; new dimension is auto-discovered with zero runner edit.
- `doc-alignment` clap subcommand tree in `reposix-quality` — D-09 mirrors this exactly so cross-link's CLI shape is consistent.

### Established Patterns
- **Catalog-first commit rule** (CLAUDE.md): phase's FIRST commit writes catalog rows BEFORE implementation; later commits cite row id. → drives commit ordering for P98 (D-16).
- **Tainted-by-default** (OP-2): bytes from any external source carry `Tainted<T>`. P98 only walks local markdown (no external bytes), so `Tainted` is not load-bearing in this phase — but it IS load-bearing in P102 grading_context.
- **Dual audit table** (OP-3): cache audit (`reposix-cache::audit`) + backend audit (`reposix-core::audit`). P98 writes neither — first cross-link audit row lands in P102 (L3 dispatch).
- **Per-phase push** (CLAUDE.md): `git push origin main` BEFORE verifier dispatch. P98 closes with a push to `workstream/v0.13.2`, then the verifier subagent runs against the pushed tip.
- **No cargo parallelism** (CLAUDE.md "Build memory budget"): before any cargo invocation, check `ps aux | grep -E "cargo|rustc"` for active rustc on the VM (workstream A may be running cargo).

### Integration Points
- `crates/reposix-quality/src/lib.rs` — declare `pub mod md_walker;` and (likely) `pub mod cross_link;` (or split — implementor's call).
- `crates/reposix-quality/src/main.rs` (clap binary entry) — register `cross-link` subcommand.
- `quality/catalogs/cross-link-fidelity.json` — new file; runner discovers automatically.
- `quality/state/cross-link-fidelity-tracker.json` — new file; gate-internal, runner does NOT read it.
- `crates/reposix-quality/tests/cross_link_tracker_schema.rs` — new integration test validating ≥3 fixture rows.

</code_context>

<specifics>
## Specific Ideas

- **Edge count target = 350–450** (success criterion 7, from Q4 measurement). If the walker emits outside this range, investigate before declaring P98 GREEN — likely false positive (code-fence-wrapped link) or false negative (anchor not extracted). False-positive guards (code-fence + commented-out + footnote-ref exclusion) are explicitly assigned to P100, but P98's walker SHOULD already exclude these to land in the target range — partial implementation here is acceptable.
- **Tracker version 1.0.0 = first commit, semver-locked** per ADR-1. Any later schema-shape change requires a major bump + migration story; that's P107-or-later territory, not P98.
- **`reposix-quality cross-link walk` outputs JSON to stdout** so `jq` pipelines work for catalog assertions and dogfooding (`reposix-quality cross-link walk | jq 'length'` is the catalog assertion in D-14 row 2).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. Items the milestone ROADMAP explicitly defers AWAY from P98 (and where they land):
- L0/L1 verifiers + pre-commit hook → P100
- L2 hash-drift + state classifier → P101
- L3 dispatcher + grading_context + cred-hygiene → P102
- Config TOML + scope resolution + glob matcher → P99
- `bootstrap` + `plan-refresh` + cron CI → P103
- `suggest-scopes` migration assistant + `cross-link rebind --auto` → P104/P105
- Pre-push hook + enforcement modes + `max_l3_per_push` → P105
- Reposix dogfood + flip default to `block-newedge` (real Anthropic spend) → P106
- Standalone `crates/reposix-md-walker/` extraction → v0.13.3 GOOD-TO-HAVES (per ADR-27 (c) deferral)
- `${...}` reject + 2KB cap on grading_context → v0.13.3 GOOD-TO-HAVES (per Q6 ratification, P102 only)

</deferred>

---

*Phase: 98-skeleton-shared-lift-edge-model*
*Context gathered: 2026-05-08*
