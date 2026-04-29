# quality/runners/ — Gate orchestration

Two entry points that compose the entire runtime:

## `run.py` — execute gates by cadence

```bash
python3 quality/runners/run.py --cadence pre-push
python3 quality/runners/run.py --cadence weekly
python3 quality/runners/run.py --cadence pre-release
```

Reads every `quality/catalogs/*.json`, filters rows by the requested cadence,
executes each row's `verifier.script` within `verifier.timeout_s`, and writes
per-row artifacts to `quality/reports/verifications/<dim>/<slug>.json`.

**Exit codes:** `0` = every P0+P1 row PASS or WAIVED. `1` = any P0+P1 row RED.

Freshness-TTL enforcement: rows with `kind: subagent-graded` and expired
`freshness_ttl` flip to NOT-VERIFIED (treated as RED for P0+P1).

## `verdict.py` — collate artifacts into verdicts

```bash
python3 quality/runners/verdict.py --cadence pre-push   # single cadence
python3 quality/runners/verdict.py --phase 57           # phase-close verdict
python3 quality/runners/verdict.py session-end          # cross-cadence roll-up
python3 quality/runners/verdict.py                      # all cadences
```

Reads artifacts from `quality/reports/verifications/`, writes a markdown
verdict to `quality/reports/verdicts/<scope>/<timestamp>.md` and updates
`quality/reports/badge.json`.

**Exit codes:** `0` = GREEN. `1` = RED.

## Supporting files

| File | Purpose |
|---|---|
| `_freshness.py` | Freshness-TTL computation (shared by run.py + verdict.py) |
| `check_p60_red_rows.py` | One-off P60 diagnostic (historical) |
| `test_freshness.py` | Unit tests for freshness logic |
| `test_freshness_synth.py` | Synthetic/property tests for freshness |

## Integration points

- **Pre-push hook:** calls `run.py --cadence pre-push` (blocking, <60s).
- **CI workflow:** calls `run.py --cadence pre-pr` (blocking, <10min).
- **Phase close:** agent runs `verdict.py --phase <N>` then dispatches verifier subagent.
