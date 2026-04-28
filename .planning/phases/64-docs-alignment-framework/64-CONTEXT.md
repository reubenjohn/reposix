# Phase 64: Docs-alignment dimension — framework, CLI, skill, hook wiring (Context)

**Gathered:** 2026-04-28
**Status:** Ready for execution
**Mode:** `--auto` (decisions inherited from the design bundle; brief is normative)

<domain>
## Phase Boundary

Build the **docs-alignment** quality dimension end-to-end so that P65's backfill audit can run on top of it. This includes:

- A new workspace crate `crates/reposix-quality/` with a `clap` subcommand surface for the docs-alignment dimension verbs (`bind`, `propose-retire`, `confirm-retire` env-guarded, `mark-missing-test`, `plan-refresh`, `plan-backfill`, `merge-shards`, `walk`, `status`, `verify`) plus generic `run --gate/--cadence`.
- A `syn`-based hash binary at `quality/gates/docs-alignment/hash_test_fn` that hashes Rust function bodies as `to_token_stream()` sha256 — comments + whitespace normalized away.
- The empty-state catalog `quality/catalogs/doc-alignment.json` with summary block + zero rows + `floor=0.50`.
- Three structure-dimension freshness rows in `quality/catalogs/freshness-invariants.json` asserting catalog presence/shape/floor-monotonicity.
- The skill `.claude/skills/reposix-quality-doc-alignment/` mirroring the P61 `reposix-quality-review` shape — **already populated by orchestrator preflight** (SKILL.md, refresh.md, backfill.md, prompts/extractor.md, prompts/grader.md committed in the catalog-first commit).
- Two thin slash-command skills `.claude/skills/reposix-quality-{refresh,backfill}/` that delegate to the umbrella — also already populated by orchestrator preflight.
- Pre-push hook integration via `reposix-quality run --cadence pre-push` invoking the deterministic hash walker.
- Two project-wide principles added to `quality/PROTOCOL.md` ("subagents propose with citations; tools validate and mint" / "tools fail loud, structured, agent-resolvable") with cross-tool examples.
- CLAUDE.md updates (new `docs-alignment` row in the dimension matrix, P64 H3 subsection ≤40 lines).

**Explicitly NOT in scope (deferred to P65 / v0.12.1):**
- Extracting any claim from any doc (P65).
- Migrating existing `quality/gates/` scripts under the umbrella (future phase).
- Touching the Confluence symlink regression (v0.12.1 gap-closure phase).

</domain>

<decisions>
## Implementation Decisions

All locked by the design bundle at `.planning/research/v0.12.0-docs-alignment-design/`.
The brief at `05-p64-infra-brief.md` is **normative** — re-read before deviating.

### Crate shape

- **D-01:** Single workspace crate `crates/reposix-quality/`. Self-contained — no `reposix-runtime` / `reposix-core` imports — keeping a future standalone-spinoff a `cargo init` away.
- **D-02:** Binary entry point produces `target/release/reposix-quality`. Library exposes `clap`-derived command tree under `src/commands/`.
- **D-03:** Crate-level lints: `#![forbid(unsafe_code)]` + `#![warn(clippy::pedantic)]`. Allow-list specific pedantic lints with rationale; never blanket-allow.

### Subcommand surface (locked verbs from `02-architecture.md`)

```
reposix-quality doc-alignment bind             --row-id X --claim "..." --source f:l-l --test f::sym --grade GREEN --rationale "..."
reposix-quality doc-alignment propose-retire   --row-id X --claim "..." --source f:l-l --rationale "..."
reposix-quality doc-alignment confirm-retire   --row-id X            # human-only; refuses if $CLAUDE_AGENT_CONTEXT is set
reposix-quality doc-alignment mark-missing-test --row-id X --claim "..." --source f:l-l [--rationale ...]
reposix-quality doc-alignment plan-refresh     <doc-file>            # stale-row manifest as JSON on stdout
reposix-quality doc-alignment plan-backfill                          # writes MANIFEST.json under quality/reports/doc-alignment/backfill-<ts>/
reposix-quality doc-alignment merge-shards     <run-dir>             # deterministic dedup; writes catalog OR fails with CONFLICTS.md
reposix-quality doc-alignment walk                                   # hash drift walker; updates last_verdict only
reposix-quality doc-alignment status                                 # prints summary block
reposix-quality verify  --row-id X                                   # read-only inspection
reposix-quality run     --gate <name> | --cadence <name>             # cadence-driven invocation
```

### Catalog row schema (locked from `02-architecture.md` § "Catalog row schema")

`quality/catalogs/doc-alignment.json` ships with shape:

```jsonc
{
  "schema_version": "1.0",
  "summary": {
    "claims_total": 0,
    "claims_bound": 0,
    "claims_missing_test": 0,
    "claims_retire_proposed": 0,
    "claims_retired": 0,
    "alignment_ratio": 1.0,
    "floor": 0.50,
    "trend_30d": "+0.00",
    "last_walked": null
  },
  "rows": []
}
```

`alignment_ratio = claims_bound / max(1, claims_total - claims_retired)`. When `claims_total == 0`, ratio is 1.0.

### Hash semantics (locked from `02-architecture.md` § "Hash semantics")

- `source_hash`: sha256 of the exact line range from the cited markdown file. Computed by the binary at `bind` time.
- `test_body_hash`: `syn::ItemFn::to_token_stream().to_string()` then sha256. Implemented by the standalone Rust binary `quality/gates/docs-alignment/hash_test_fn` (also a workspace crate or a `[[bin]]` target inside `reposix-quality`).
- **Hashes update only when `bind` is called with a fresh GREEN grade.** The walker NEVER refreshes hashes; it only sets `last_verdict`.

### Project-wide principles (P64 ships into `quality/PROTOCOL.md`)

- **Principle A — Subagents propose with citations; tools validate and mint.** LLM agents emit proposals with file:line citations only; deterministic tools validate and persist canonical state. Cross-tool examples enumerated.
- **Principle B — Tools fail loud, structured, agent-resolvable.** Tools assert preconditions and emit machine-readable failure when preconditions don't hold. They never silently default. Cross-tool examples enumerated.

### Catalog-first commit

P64's first commit ships:
- `quality/catalogs/doc-alignment.json` (empty-state schema)
- 3 new rows in `quality/catalogs/freshness-invariants.json`:
  - `structure/doc-alignment-catalog-present`
  - `structure/doc-alignment-summary-block-valid`
  - `structure/doc-alignment-floor-not-decreased`
- `quality/catalogs/README.md` extended with the doc-alignment schema spec.

Pre-push must pass for this commit. The structure-dimension verifiers
ship in the same commit as Python stubs (extending
`quality/gates/structure/freshness-invariants.py`).

### Skill is preflight (committed before catalog-first)

The 5 skill files (`.claude/skills/reposix-quality-doc-alignment/{SKILL.md, refresh.md, backfill.md, prompts/extractor.md, prompts/grader.md}` + the 2 thin slash-command skills) were written by the orchestrator before P64 plan execution to surface permission prompts. They are committed as the FIRST commit of the phase ("docs(p64): scaffold reposix-quality-doc-alignment skill files (preflight)") and are normative inputs to the executor — the binary surface in those files defines the CLI contract the executor implements.

### Verifier dispatch

Path B in-session per project precedent (P56–P63). `gsd-executor` lacks `Task`, so the verifier subagent runs as in-session Claude grading with explicit `dispatched_via: P64-Path-B-in-session` disclosure in the verdict. Verdict written to `quality/reports/verdicts/p64/VERDICT.md`.

### Cargo memory budget

- One cargo invocation at a time.
- Prefer `cargo check -p reposix-quality` and `cargo test -p reposix-quality` over workspace-wide.
- Workspace-wide `cargo check --workspace` runs ONCE at end of P64 before verifier dispatch.

### Claude's Discretion

- File-internal Rust module organization within `crates/reposix-quality/src/` — split by subcommand groups (`commands/doc_alignment/{bind,retire,walk,merge,...}.rs`) is recommended but not load-bearing.
- Test placement (inline `#[cfg(test)] mod tests` vs `tests/` integration) — follow project convention; integration tests for golden-test cases (`merge-shards` dedup, `bind` rejection, `confirm-retire` env-guard).
- Choice of sha256 crate (`sha2` is the project default — use it).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design bundle (load-bearing)
- `.planning/research/v0.12.0-docs-alignment-design/README.md` — entry point + read order.
- `.planning/research/v0.12.0-docs-alignment-design/01-rationale.md` — why this dimension exists.
- `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` — catalog schema, hash semantics, binary surface, skill layout. **Most important file.**
- `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` — top-level vs executor; CLAUDE.md note source.
- `.planning/research/v0.12.0-docs-alignment-design/04-overnight-protocol.md` — deadline 08:00, suspicion-of-haste rule, cargo-memory-budget reminders.
- `.planning/research/v0.12.0-docs-alignment-design/05-p64-infra-brief.md` — full implementation spec for P64. **Normative.**
- `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md` — P65 protocol; informs P64 only by defining what `plan-backfill` and `merge-shards` must produce.

### Project-level
- `CLAUDE.md` — operating principles, cargo memory budget, subagent delegation, freshness invariants, threat model.
- `.planning/REQUIREMENTS.md` — DOC-ALIGN-01..07 are P64; DOC-ALIGN-08..10 are P65.
- `.planning/ROADMAP.md` — Phase 64 entry (line 223+); Phase 65 entry (line 247+).
- `.planning/STATE.md` — current cursor; status flipped to `in-progress` when P64 + P65 added.

### Catalog + dimension precedent
- `quality/PROTOCOL.md` — runtime contract; gains the two principles in P64.
- `quality/catalogs/README.md` — unified catalog schema spec; gains doc-alignment row schema in P64.
- `quality/catalogs/freshness-invariants.json` — structure-dimension catalog; gains 3 P64 rows.
- `quality/catalogs/subjective-rubrics.json` — P61 precedent for a catalog with verifier-dispatched grading.
- `quality/gates/structure/README.md` — dimension-home README precedent.
- `quality/gates/structure/freshness-invariants.py` — structure verifier precedent (extended in P64).

### Skill precedent (mirror)
- `.claude/skills/reposix-quality-review/SKILL.md` — P61 skill; copy this shape.
- `.claude/skills/reposix-quality-review/dispatch.sh` — bash dispatch precedent.

### Hook precedent
- `.githooks/pre-push` (or `scripts/hooks/pre-push`) — extended in P64 to include `reposix-quality run --cadence pre-push`.
- `quality/runners/run.py` — Python runner orchestration; the umbrella binary likely shells out to / from this rather than reimplementing it.

### Workspace
- `Cargo.toml` (workspace root) — gains `crates/reposix-quality` member.
- `crates/reposix-cli/Cargo.toml` — Cargo.toml conventions for project crates.
- `crates/reposix-cli/src/main.rs` — clap usage precedent.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`quality/runners/run.py`** — cadence-driven Python runner. The umbrella binary's `run --cadence X` subcommand likely shells to this for orchestration; the binary's role is dimension-specific verbs, not reimplementing the runner.
- **`quality/runners/_freshness.py`** — TTL parser; `is_stale` predicate. Reusable by the catalog walker for `last_walked` semantics.
- **`quality/gates/structure/freshness-invariants.py`** — structure-dimension verifier precedent. Pattern: stdlib-only Python, `--json` mode for machine-readable summary, exits non-zero with stderr message naming the slash command on failure.
- **`.claude/skills/reposix-quality-review/dispatch.sh`** — bash dispatch precedent for skill entry points.
- **`crates/reposix-cli/src/main.rs`** — clap derive precedent for nested subcommand trees.
- **`sha2` crate** — already in workspace deps (used by reposix-cache for blob hashing). Re-use, don't add a different sha implementation.
- **`syn` crate** — needed for hash_test_fn; not yet in workspace. Add at workspace level.
- **`serde` + `serde_json`** — already in workspace; used for catalog (de)serialization.

### Established Patterns

- **Catalog-first commit per phase** — P56–P63 precedent; P64 follows.
- **Stdlib-only Python verifiers** for the runner (no pip deps in pre-push path).
- **Path B in-session verifier dispatch** — P56–P63 precedent for executor-mode phases (Task tool unavailable).
- **`#![forbid(unsafe_code)]` + `#![warn(clippy::pedantic)]`** in every crate.
- **Tests live next to code** (`#[cfg(test)] mod tests`); integration tests in `tests/`.
- **`thiserror` for typed crate errors, `anyhow` only at binary boundaries.**
- **Catalog row IDs are kebab-case `<dimension>/<verb>-<noun>`** — followed across all 4 existing catalogs.

### Integration Points

- **Workspace `Cargo.toml`** — register `crates/reposix-quality` member.
- **Workspace deps** — add `syn = "2"`, `quote` (for token-stream display), confirm `sha2` and `clap` versions.
- **`quality/runners/run.py`** — the runner's `pre-push` cadence invokes the new umbrella's walker. Likely just a row in the runner's gate registry; the binary itself is invoked by the runner via subprocess. (Decision: don't reimplement runner orchestration in Rust; the Python runner stays the orchestrator, and Rust binary is the dimension verifier.)
- **`.githooks/pre-push`** — already exec'd by the chained hook. The runner invocation already includes structure dimension; we add the doc-alignment dimension via a new gate row in the runner config.
- **Skill files committed in P64 first commit** — preflight done; the executor doesn't write skill files, only commits them.

</code_context>

<specifics>
## Specific Ideas

- The hash binary `hash_test_fn` should be a `[[bin]]` target inside `reposix-quality`, not a separate crate. Reasoning: same syn/quote/sha2 deps; one cargo build artifact; same release pipeline. The path `quality/gates/docs-alignment/hash_test_fn` is a wrapper script (1 line: `exec target/release/hash_test_fn "$@"`) so the dimension home is self-contained from a discovery POV.
- `confirm-retire` env-guard: refuse if `$CLAUDE_AGENT_CONTEXT` is set OR if not a TTY (fallback for harness contexts that don't set the env). The agent-context env was added by GSD around P56; the TTY check is belt-and-suspenders.
- The catalog walker's `last_walked` timestamp is the only mutation it's allowed to make to the summary block other than recomputing `claims_*` counters from row states. Hash refresh is forbidden.
- `merge-shards` dedup key is `(claim_text_normalized, test)` where `claim_text_normalized = lowercase(trim(claim))`. Multi-source rows: same key, different sources → one row, multi-citation `source` array.

</specifics>

<deferred>
## Deferred Ideas

### Reviewed Todos (not folded)
- **Confluence symlink regression fix** — surfaces in P65 as `MISSING_TEST`; closed in v0.12.1 gap-closure phase.
- **Migrating `quality/gates/structure/*.py` under `reposix-quality structure <verb>`** — explicit out-of-scope per `01-rationale.md` § "What this milestone does not aim to fix".
- **Reverse coverage (test → claim binding)** — out of scope; possibly a future narrow rule for `dark_factory_real_*` tests only.
- **Helper-module extraction for 402-LOC `freshness-invariants.py`** — P62 carry-forward; reassess in v0.12.1 if Wave 6 flagged it.

</deferred>

---

*Phase: 64-docs-alignment-framework*
*Context gathered: 2026-04-28*
*Source-of-truth bundle: .planning/research/v0.12.0-docs-alignment-design/*
