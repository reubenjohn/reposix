# quality/runners/ — Gate orchestration

Two entry points that compose the entire runtime:

## `run.py` — execute gates by cadence

```bash
python3 quality/runners/run.py --cadence pre-push             # GATE  (validate-only)
python3 quality/runners/run.py --cadence pre-push --persist   # MINT  (writes catalog status)
```

Reads every `quality/catalogs/*.json`, filters rows whose `cadences: list[str]`
contains the requested cadence, executes each row's `verifier.script` within
`verifier.timeout_s`, and writes per-row artifacts to
`quality/reports/verifications/<dim>/<slug>.json`. A single row may declare
multiple cadences (e.g., `["pre-commit", "pre-push", "pre-pr"]`) so the same
gate fires at every relevant trigger.

**GRADE / PERSIST split (D-P96-01).** A bare cadence run is **validate-only**:
it grades in memory, writes per-row artifacts, and blocks RED via the exit code,
but does **not** write graded `status` back to `quality/catalogs/`. Only
`--persist` mints (writes the committed catalog) — the phase-close /
verifier-subagent grading invocation. Hooks + CI run WITHOUT `--persist`, so a
gate run never self-mutates the catalog (regression-locked by
`structure/catalog-immutable-on-read`). A validate-only run prints a
`note: ... status flips NOT persisted (...)` line naming what a `--persist` mint
would flip — effectively the dry-run preview.

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
| `test_run.py` | Persist-gate tests (D-P96-01): validate-only never mutates the catalog; `--persist` mints; validate-only still blocks RED |

## Integration points

- **Pre-commit hook:** calls `run.py --cadence pre-commit` (blocking, <2s; cheap mechanical checks only; validate-only — no `--persist`).
- **Pre-push hook:** calls `run.py --cadence pre-push` (blocking, <60s; validate-only — no `--persist`, so it never writes the catalog).
- **CI workflow:** calls `run.py --cadence pre-pr` (blocking, <10min; validate-only).
- **Phase close:** agent runs `run.py --cadence <c> --persist` to MINT graded status, then `verdict.py --phase <N>`, then dispatches verifier subagent.
