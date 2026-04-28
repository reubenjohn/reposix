# 05 — P64 implementation brief: docs-alignment infrastructure

## Phase identity

- **Phase number:** P64
- **Milestone:** v0.12.0
- **Title:** Docs-alignment dimension — framework, CLI, skill, hook wiring
- **Execution mode:** `executor` (delegates to `gsd-executor`)
- **Goal:** Ship the docs-alignment dimension end-to-end so that P65 can run a backfill on top of it.
- **Requirements:** DOC-ALIGN-01 through DOC-ALIGN-07 (added to `.planning/REQUIREMENTS.md` v0.12.0 active block by this brief).

## Success criteria (verifier-graded)

1. `crates/reposix-quality/` workspace crate exists, compiles clean, has `#![forbid(unsafe_code)]` and `#![warn(clippy::pedantic)]` per project convention.
2. `reposix-quality` binary exposes the full subcommand surface from `02-architecture.md` § "Binary surface" — every subcommand listed, every flag documented in `--help`.
3. `quality/catalogs/doc-alignment.json` exists with a valid empty-state shape: summary block populated (`claims_total: 0`, `alignment_ratio: 1.0`, `floor: 0.50`), zero rows. Schema documented in `quality/catalogs/README.md` (extend the existing file).
4. `quality/gates/docs-alignment/` directory exists with:
   - `README.md` — dimension home, P64-relevant rows table.
   - `hash_test_fn.rs` (compiled standalone Rust binary or workspace member) — given `--file` and `--fn`, prints sha256 of `syn` token-stream of the named function. Tests cover comment-edit invariance and rename detection.
   - The hash walker is a Rust subcommand of the umbrella, NOT a separate script.
5. `.claude/skills/reposix-quality-doc-alignment/` directory exists with `SKILL.md`, `refresh.md`, `backfill.md`, `prompts/extractor.md`, `prompts/grader.md`. Both playbooks reference Path A as primary, with the depth-2 + subscription rationale explicit.
6. Slash commands `/reposix-quality-refresh` and `/reposix-quality-backfill` are wired (the project's existing slash-command convention; see how `.claude/skills/reposix-quality-review/` is invoked for precedent).
7. `scripts/hooks/pre-push` is updated to chain through `reposix-quality run --cadence pre-push` for the docs-alignment hash walker. The walker exits non-zero with a stderr message naming the slash command on `STALE_DOCS_DRIFT` / `MISSING_TEST` / `STALE_TEST_GONE` / `TEST_MISALIGNED` / `RETIRE_PROPOSED`.
8. `quality/PROTOCOL.md` gains the two project-wide principles from `02-architecture.md` § "Two project-wide principles" with the cross-tool examples enumerated.
9. CLAUDE.md gains:
   - A new row in the dimensions matrix under "Quality Gates — dimension/cadence/kind taxonomy": `docs-alignment` | "doc claims have tests; hash drift detection" | `quality/gates/docs-alignment/`.
   - The "Orchestration-shaped phases" note from `03-execution-modes.md` (CLAUDE.md addition section).
   - A P64 H3 subsection under "Quality Gates" (≤40 lines, banned-words clean) describing what this phase shipped.
10. Tests:
    - `merge-shards` golden test for the same-claim-different-source dedup case AND for the same-claim-different-test conflict case (former auto-resolves with multi-citation row, latter exits non-zero with CONFLICTS.md).
    - `bind` rejects citations to nonexistent files / line ranges / fn symbols.
    - `confirm-retire` exits non-zero when `$CLAUDE_AGENT_CONTEXT` is set.
    - Hash binary: token-stream invariance test (rename a comment, hash unchanged; rename a fn, hash differs).
11. `cargo clippy --workspace --all-targets -- -D warnings` clean.
12. `cargo fmt --all -- --check` clean.
13. `cargo test -p reposix-quality` passes.
14. Phase-close verifier subagent verdict at `quality/reports/verdicts/p64/VERDICT.md` is GREEN. (Path B in-session disclosure per project precedent — see CLAUDE.md note on Path A unavailability inside executor.)

## Catalog rows P64 ships (catalog-first commit)

P64's first commit MUST land these rows in `quality/catalogs/doc-alignment.json` (all empty-state until backfill, but the schema must exist) AND these structure-dimension rows in `quality/catalogs/freshness-invariants.json`:

- `structure/doc-alignment-catalog-present` — verifier asserts `quality/catalogs/doc-alignment.json` exists and parses; pre-push.
- `structure/doc-alignment-summary-block-valid` — verifier asserts the summary block has expected keys + alignment_ratio is computable from row counts; pre-push.
- `structure/doc-alignment-floor-not-decreased` — verifier asserts `floor` field never decreases between commits (parses git history); weekly.

Plus a `code/cargo-test-pass` row inside the `reposix-quality` crate (extending the existing pattern), and a `code/clippy-pedantic-clean` row if the umbrella crate's clippy budget needs explicit tracking.

## Implementation order (planner guidance)

The planner produces `PLAN.md` covering these waves. Adjust as `gsd-pattern-mapper` finds existing analogs.

1. **Wave 1 — catalog-first commit.** Write `quality/catalogs/doc-alignment.json` (empty schema with summary block), update `quality/catalogs/README.md` schema spec, write the 3 structure-dimension rows in `freshness-invariants.json`. Commit. Pre-push must pass (the new structure rows have verifier stubs that validate the JSON shape).
2. **Wave 2 — crate skeleton.** `cargo new --lib crates/reposix-quality`, register in workspace `Cargo.toml`, set up `clap` skeleton for the subcommand tree (`doc-alignment` group + `run`/`verify`/`walk` siblings), wire `#![forbid(unsafe_code)]` + `#![warn(clippy::pedantic)]`. Add a smoke test that `reposix-quality --help` emits expected subcommands. Commit.
3. **Wave 3 — hash binary.** `quality/gates/docs-alignment/hash_test_fn.rs`. `syn` parses, `to_token_stream()` hashes, `--file` + `--fn` flags. Tests for comment-edit invariance + rename detection. Commit.
4. **Wave 4 — `bind` and the validators.** `bind`, `mark-missing-test`, `propose-retire`, `confirm-retire` (env-guarded), `verify`. Each subcommand validates its inputs against the live filesystem; refuses on invalid citations. Tests. Commit.
5. **Wave 5 — `walk` (the hash walker).** Reads catalog, computes current hashes from filesystem, sets `last_verdict` to one of the BOUND / STALE_* states. Never refreshes the stored hashes. Exits non-zero on any blocking state with a stderr message that names the relevant slash command. Tests with synthetic catalogs. Commit.
6. **Wave 6 — `plan-refresh`, `plan-backfill`, `merge-shards`.** The chunker for backfill (deterministic, directory-affinity, ≤3 files per shard). The merger (deterministic dedup, conflict detection writing CONFLICTS.md). Golden tests for both. Commit.
7. **Wave 7 — runner integration.** `reposix-quality run --cadence pre-push` invokes `walk` plus existing freshness invariants. Stitches into the existing `quality/runners/run.py` orchestration (probably via shelling out from Python; do not rewrite the runner in Rust). Hook wiring update at `scripts/hooks/pre-push`. Tests via `test-pre-push.sh`. Commit.
8. **Wave 8 — skill + slash commands.** `.claude/skills/reposix-quality-doc-alignment/` files, slash command registration. Reference Path A as primary; document the depth-2 / subscription rationale in `SKILL.md`. Commit.
9. **Wave 9 — `quality/PROTOCOL.md` updates.** Two project-wide principles section with cross-tool examples. Commit.
10. **Wave 10 — CLAUDE.md update.** Dimension matrix row, orchestration-shaped phases note, P64 H3 subsection. Banned-words check via the existing `reposix-banned-words` skill. Commit.
11. **Wave 11 — verifier subagent dispatch.** Path B in-session per precedent. Verdict written to `quality/reports/verdicts/p64/VERDICT.md`. Phase close.

## Catalog-first details

The doc-alignment JSON schema lives at `quality/catalogs/doc-alignment.json`:

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

When `claims_total == 0`, `alignment_ratio` is 1.0 by definition. Floor of 0.50 is the initial soft target; the walker BLOCKs if `claims_total > 0 && alignment_ratio < floor`.

## Pivot policies

If during P64 implementation a design assumption breaks (e.g., `syn` cannot hash the way we want; clap cannot express the subcommand tree cleanly; the hook wiring conflicts with existing P60 cutover), do NOT silently redesign. Append to `quality/SURPRISES.md` with the obstacle, the proposed pivot, and an explicit owner check via the standard pivot rule. If the pivot is small (rename a flag, swap a hashing algorithm), proceed and document. If the pivot is large (e.g., split the binary into two crates, restructure the catalog schema), pause and write a checkpoint to `.planning/STATE.md` for the human.

## What P64 explicitly does not do

- Does NOT extract claims from any docs file. That is P65.
- Does NOT migrate any existing `quality/gates/` script under the umbrella. That's a future phase.
- Does NOT alter or remove existing dimension verifiers. The umbrella runs alongside them.
- Does NOT touch the Confluence symlink regression. P65 surfaces it; v0.12.1 fixes it.
