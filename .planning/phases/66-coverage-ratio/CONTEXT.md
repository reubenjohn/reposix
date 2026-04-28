# Phase 66: coverage_ratio metric — docs-alignment second axis (Context)

**Gathered:** 2026-04-28
**Status:** Ready for execution
**Mode:** `--auto` (sequential gsd-executor on main; depth-1 — no further subagent dispatch)
**Milestone:** v0.12.1

<domain>
## Phase Boundary

Add the `coverage_ratio` metric to the docs-alignment dimension as the **second axis** alongside the existing `alignment_ratio`. Two-axis grading captures two distinct quality regressions:

- `alignment_ratio = claims_bound / max(1, claims_total - claims_retired)` — answers **"of the claims we extracted, how many bind to passing tests?"** (EXISTS, shipped P64).
- `coverage_ratio = lines_covered_by_any_row / total_eligible_lines` — answers **"of the prose we said we'd mine, what fraction of lines did we actually cover with at least one row?"** (NEW; this phase).

The two together yield the agent's mental model:

| | high alignment | low alignment |
|---|---|---|
| **high coverage** | ideal — most prose mined, most claims tested | extracted everything, most claims unbound |
| **low coverage** | tested what we found, missed most prose | haven't started |

Without coverage, an agent can ship a high `alignment_ratio` by extracting only easy claims and binding them. The coverage axis closes that loophole and surfaces the per-file gap-target view: an agent looking to widen coverage runs `reposix-quality doc-alignment status --top 10` and reads which files have the worst coverage to mine next.

**Explicitly NOT in scope:**
- Auto-tuning `coverage_floor` (it's human-tuned via deliberate commit; default 0.10 ships in this phase).
- Lifting any existing waiver (the owner removed the `floor_waiver` and the `docs-alignment/walk` row waiver intentionally — pre-push BLOCKing on docs-alignment is the INTENDED state of v0.12.1 until cluster-closing phases land).
- Migrating any other dimension (perf, security stay v0.12.1 P67/P68).
- Closing any of the 388 existing `MISSING_TEST` / `RETIRE_PROPOSED` rows (those are P72+).

</domain>

<decisions>
## Implementation Decisions

### D-01: Four new Summary fields, all `#[serde(default)]`
`coverage_ratio` (f64), `lines_covered` (u64), `total_eligible_lines` (u64), `coverage_floor` (f64; default 0.10 via `default_coverage_floor` fn). Back-compat with the populated 388-row catalog is non-negotiable.

### D-02: Coverage computed per-file, summed globally
Per-file: filter rows whose `source.file == path`, extract their inclusive 1-based line ranges, merge overlapping/adjacent, sum `(end - start + 1)`. Global: sum across eligible files. Empty input (0 eligible lines) → ratio 0.0 (NOT 1.0; differs from alignment's empty-state semantics).

### D-03: Eligible file set mirrors plan-backfill
`docs/**/*.md` + `README.md` + `.planning/milestones/v0.{6..11}.0-phases/REQUIREMENTS.md`. Files not on disk: warn stderr + skip. Same set the chunker mines so the metric grades against the same prose universe.

### D-04: Multi-source rows attribute to each cited file independently
A row whose `source` is `Source::Multi([{file: A, ...}, {file: B, ...}])` contributes its A-range to A's per-file count AND its B-range to B's. No inflation: each file's coverage is independent.

### D-05: Out-of-eligible rows warn + skip
A row whose `source.file` is outside the eligible set (file moved/renamed/deleted): warn to stderr, do NOT count. This is a forensic signal — the row points at prose that no longer lives where the catalog says.

### D-06: Walker BLOCKs on `coverage_ratio < coverage_floor`
Adds to the existing `alignment_ratio < floor` BLOCK; both can fire on the same walk. Stderr message names `/reposix-quality-backfill` as the recovery move (extends extraction) OR ratchet `coverage_floor` down via deliberate human commit. Walker NEVER auto-tunes `coverage_floor`.

### D-07: `coverage_floor` starts at 0.10
Rationale: the actual measured coverage on the 388-row corpus is unknown until first walk. 0.10 is low enough that the BLOCK probably doesn't bite immediately (even sparse mining typically hits 15-25%) but armors against future regression. Future phases ratchet up.

### D-08: Status verb shows global + per-file (worst-first)
Per-file table sorted ascending by `ratio`. Default `--top 20`; `--all` for everything; `--json` for machine-readable. The "ZERO ROWS" annotation flags `row_count == 0 && total_lines > 50` (small files might be redirects; don't pester).

### D-09: Verifier dispatch — Path B in-session
gsd-executor at depth-1 lacks `Task` tool. Verdict at `quality/reports/verdicts/p66/VERDICT.md` with explicit `dispatched_via: P66-Path-B-in-session` disclosure (P56-P64 precedent).

### D-10: NO waivers re-added
The owner removed `floor_waiver` from `quality/catalogs/doc-alignment.json` and the `docs-alignment/walk` waiver from `quality/catalogs/freshness-invariants.json` intentionally. Pre-push BLOCKing on docs-alignment is the INTENDED state of v0.12.1 — the gate is hard until cluster-closing phases close enough rows AND/OR widen coverage. This phase MUST NOT re-add either waiver.

### Cargo memory budget
- One cargo invocation at a time.
- `cargo check -p reposix-quality` / `cargo test -p reposix-quality` / `cargo clippy -p reposix-quality --all-targets -- -D warnings -W clippy::pedantic` per crate.
- Workspace-wide cargo runs ONCE at the END of the phase (Commit D).

</decisions>

<canonical_refs>
## Canonical References

- `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` — catalog row schema + summary block (extended by this phase).
- `.planning/research/v0.12.0-docs-alignment-design/01-rationale.md` — why the dimension exists.
- `crates/reposix-quality/src/catalog.rs` — Summary struct (extended), Row struct, Source enum.
- `crates/reposix-quality/src/commands/doc_alignment.rs` — walk + status verbs (extended).
- `quality/catalogs/doc-alignment.json` — 388 populated rows (the metric grades against this).
- `quality/catalogs/README.md` — schema spec (extended with 2x2 section).
- `quality/catalogs/freshness-invariants.json` — `docs-alignment/walk` row (no waiver — intentional).
- `crates/reposix-quality/tests/walk.rs` — integration test pattern.
- CLAUDE.md `Build memory budget` § — load-bearing for parallel cargo prohibition.

</canonical_refs>

<specifics>
## Specific Ideas

- `merge_ranges` is the single load-bearing helper. Test it standalone with ≥4 cases (overlap, adjacent, disjoint, single-element).
- Use `BufRead::lines()` for `line_count`; UTF-8 safe; don't read whole file into a String for huge corpora (the v0.6/v0.7 archived REQUIREMENTS may not exist in repo, but the docs tree does).
- Status verb's per-file output is the AGENT'S gap-target view — emphasis on legibility (ratio printed to 3 decimals; row count for context; ZERO ROWS hint for the most actionable misses).
- Capture actual measured `coverage_ratio` from the live 388-row catalog in the verdict — that number is the v0.12.1 ratchet baseline.

</specifics>

<deferred>
## Deferred Ideas

- Auto-tuning `coverage_floor` (always human-tuned via commit; principle preserved).
- Per-section coverage (heading-aware) — out of scope; line-based coverage is the unit-of-measure for this phase.
- Coverage trend (30-day delta tracking) — defer to a later phase that adds `coverage_trend_30d` once we have enough history.

</deferred>

---

*Phase: 66-coverage-ratio*
*Context gathered: 2026-04-28*
*Source: P66 prompt + v0.12.0 docs-alignment design bundle.*
