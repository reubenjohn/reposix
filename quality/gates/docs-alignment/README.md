# quality/gates/docs-alignment/ -- docs-alignment dimension

Verifiers backing the catalog rows in `quality/catalogs/doc-alignment.json`. The dimension answers the question: **"every behavioral claim in user-facing prose is bound to a passing test, and the binding has not silently drifted."**

## Status (P64 Wave 1)

Empty-state seed only. Catalog ships with `summary.claims_total = 0` and `rows = []`. Subsequent waves of P64 ship the binary surface (`reposix-quality doc-alignment <verb>`) and hash walker; P65 runs the first backfill and populates rows.

## Quick start

```bash
# Inspect the empty-state catalog
python3 -c "import json; print(json.dumps(json.load(open('quality/catalogs/doc-alignment.json'))['summary'], indent=2))"

# Run the structure-dimension freshness rows that guard this catalog (P64 Wave 1)
python3 quality/gates/structure/freshness-invariants.py --row-id structure/doc-alignment-catalog-present
python3 quality/gates/structure/freshness-invariants.py --row-id structure/doc-alignment-summary-block-valid
python3 quality/gates/structure/freshness-invariants.py --row-id structure/doc-alignment-floor-not-decreased

# After Wave 5 lands the walker:
# reposix-quality walk                                  # hash drift walker; updates last_verdict only
# reposix-quality doc-alignment status                  # prints summary block
# reposix-quality run --cadence pre-push                # everything that runs at this cadence
```

## Catalog rows (P64 Wave 1 -- empty)

| Row id | Cadence | Blocks pre-push? | Status |
|---|---|---|---|
| (none yet) | -- | -- | -- |

The schema is documented in `quality/catalogs/README.md` § "docs-alignment dimension". The row state machine -- `BOUND` / `MISSING_TEST` / `STALE_DOCS_DRIFT` / `STALE_TEST_DRIFT` / `STALE_TEST_GONE` / `TEST_MISALIGNED` / `RETIRE_PROPOSED` / `RETIRE_CONFIRMED` -- is documented in `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` § "Row state machine".

## Structure-dimension guards on this catalog

Three rows in `quality/catalogs/freshness-invariants.json` guard the catalog file itself:

| Row id | Cadence | What it asserts |
|---|---|---|
| `structure/doc-alignment-catalog-present` | pre-push | `quality/catalogs/doc-alignment.json` exists + parses + has `schema_version`/`summary`/`rows` keys + schema_version is `'1.0'` |
| `structure/doc-alignment-summary-block-valid` | pre-push | summary has all 9 required keys + `alignment_ratio == claims_bound / max(1, claims_total - claims_retired)` within 0.001 epsilon + floor in `[0.0, 1.0]` |
| `structure/doc-alignment-floor-not-decreased` | weekly | `git log --reverse -- quality/catalogs/doc-alignment.json` shows the floor field is monotone non-decreasing across commits (audit signal -- floor only ratchets up via deliberate human commit) |

## Conventions

- The dimension's binary `reposix-quality` lives at `crates/reposix-quality/` (Wave 2). It mints catalog state on behalf of subagents per the "subagents propose with citations; tools validate and mint" principle (`quality/PROTOCOL.md`).
- Subagents NEVER write `quality/catalogs/doc-alignment.json` directly. All state mutation flows through `reposix-quality doc-alignment <subcmd>` -- the binary validates citations, computes hashes, and refuses on invalid input.
- Hash walker is read-only on the stored hashes. `bind` is the only verb that refreshes them, and only after a grader returns GREEN.

## Cross-references

- `quality/catalogs/doc-alignment.json` -- the catalog this dimension owns.
- `quality/catalogs/README.md` -- unified catalog schema spec; docs-alignment subsection details row schema + state machine.
- `quality/PROTOCOL.md` -- runtime contract; the two project-wide principles (proposals-with-citations / fail-loud-structured) shipping into PROTOCOL.md in P64 Wave 9.
- `.claude/skills/reposix-quality-doc-alignment/` -- orchestrator skill (refresh + backfill playbooks).
- `.claude/skills/reposix-quality-refresh/` and `.claude/skills/reposix-quality-backfill/` -- thin slash-command entry points.
- `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` -- catalog schema, hash semantics, binary surface.
- `.planning/research/v0.12.0-docs-alignment-design/05-p64-infra-brief.md` -- P64 implementation spec.
- `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md` -- P65 backfill protocol.
