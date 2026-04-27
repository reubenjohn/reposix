# quality/gates/agent-ux/

Verifiers backing `quality/catalogs/agent-ux.json` (1 row, agent-ux dimension).

**Intentionally sparse at v0.12.0** -- the dark-factory regression is the only gate. Perf and security stubs land in v0.12.1 per `MIGRATE-03`.

| Verifier | Catalog rows backed | Cadence |
|---|---|---|
| `dark-factory.sh` | `agent-ux/dark-factory-sim` | pre-pr |

## Source-script lineage (Wave D, SIMPLIFY-07)

`scripts/dark-factory-test.sh` is migrated to `quality/gates/agent-ux/dark-factory.sh` per SIMPLIFY-07 (Wave D ships the file move; Wave A locked the catalog row's `verifier.script` path so the contract anchors on the new home from day one).

The v0.9.0 dark-factory invariant is preserved verbatim: stderr-teaching strings emit on conflict + blob-limit paths; `reposix init sim::demo <path>` configures partial-clone; pure-git agent UX with zero in-context learning. The CI job at `.github/workflows/ci.yml` continues invoking the migrated path at pre-pr cadence.

## Conventions

- Cadence `pre-pr` matches the CLAUDE.md "Local dev loop" section's dark-factory invocation.
- `kind: mechanical` -- runs against the local cargo workspace + sim; no docker required.
- Stdlib-bash only (no Python helper for v0.12.0; v0.12.1 may add one if multi-persona dispatch lands).

## Cross-references

- `quality/PROTOCOL.md` -- runner contract
- `quality/SURPRISES.md` -- pivots journal
- CLAUDE.md "Local dev loop" -- dark-factory regression
- `.planning/REQUIREMENTS.md` MIGRATE-03 -- v0.12.1 carry-forward (perf + security stubs)
