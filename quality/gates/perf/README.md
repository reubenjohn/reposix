# quality/gates/perf/

Verifiers backing `quality/catalogs/perf-targets.json` (3 rows, perf dimension).

**File-relocate only at v0.12.0.** Full gate logic deferred to v0.12.1 stub per `MIGRATE-03`.

| Verifier | Catalog rows backed | Cadence | Status |
|---|---|---|---|
| `latency-bench.sh` | `perf/latency-bench` | weekly | WAIVED until 2026-07-26 |
| `bench_token_economy.py` | `perf/token-economy-bench` | weekly | WAIVED until 2026-07-26 |
| `headline-numbers-cross-check.py` | `perf/headline-numbers-cross-check` | weekly | WAIVED until 2026-07-26 |

## Source-script lineage (Wave E, SIMPLIFY-11)

- `scripts/latency-bench.sh` -> `quality/gates/perf/latency-bench.sh`
- `scripts/bench_token_economy.py` -> `quality/gates/perf/bench_token_economy.py`
- `scripts/test_bench_token_economy.py` -> `quality/gates/perf/test_bench_token_economy.py`
- `benchmarks/fixtures/*` stays in place (test inputs, not gates).
- `headline-numbers-cross-check.py` is NEW in v0.12.1 (no predecessor; locks the cross-check between docs/index.md + README.md headlines and the bench fixture files).

## Waiver explanation

All 3 catalog rows wear a 90-day waiver (`until: 2026-07-26`, `tracked_in: MIGRATE-03 v0.12.1`). The waiver makes the rows `WAIVED` at runner time, so they do not block GREEN at v0.12.0. v0.12.1 ships:

1. The actual file moves (Wave E v0.12.0 stops here -- file-relocate only).
2. Cross-check verifier code (compares bench output against doc headlines).
3. Removal of the waivers + flip to active enforcement.

### P63 -- v0.12.1 carry-forward stubs (added 2026-04-28)

The 3 perf-targets rows (`perf/latency-bench`,
`perf/token-economy-bench`, `perf/headline-numbers-cross-check`)
remain stubs at P63 close: file-relocate landed in P59 SIMPLIFY-11,
but the cross-check verifier code (bench output diff vs doc headline
numbers) is deferred to v0.12.1 per `MIGRATE-03`. P63 leaves the
existing waivers in place and confirms the catalog rows anchor the
v0.12.1 implementer's contract.

Cross-reference: `.planning/REQUIREMENTS.md` MIGRATE-03 (v0.12.1).

## Cross-references

- `quality/catalogs/perf-targets.json` -- 3-row catalog (Wave A)
- `.planning/REQUIREMENTS.md` MIGRATE-03 -- v0.12.1 carry-forward
- `quality/PROTOCOL.md` -- waiver protocol
