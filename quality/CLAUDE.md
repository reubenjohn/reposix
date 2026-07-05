# quality/CLAUDE.md — quality-gates rules (auto-loaded under quality/)

Extends root `CLAUDE.md`. **Read `quality/PROTOCOL.md` first** for the runtime contract.

## Catalog-first rule

Every phase's FIRST commit writes the catalog rows defining that phase's GREEN contract;
later commits cite the row id. The unbiased verifier subagent reads catalog rows that
existed BEFORE the implementation landed.

## Verifier-subagent dispatch

Phase close MUST dispatch an unbiased subagent that grades catalog rows from committed
artifacts with zero session context — the executing agent does not grade itself. Verbatim
prompt: `quality/PROTOCOL.md` § "Verifier subagent prompt template". The milestone-close
9th probe (`run.py --cadence pre-release-real-backend`, exit 0) is non-skippable and
never carries a waiver.

## Dimension routing

9 dimensions (code, docs-alignment, docs-build, docs-repro, release, structure,
agent-ux, perf, security); 8 cadences; 6 kinds. Adding a gate = one catalog row + one
verifier in `quality/gates/<dim>/`; the runner discovers by tag. Full taxonomy: root
`CLAUDE.md` § "Quality Gates" + `quality/catalogs/README.md`. Pivot journal:
`quality/SURPRISES.md` (append-only, ≤200 lines). When an owner catches a quality miss:
fix it, update CLAUDE.md/ORCHESTRATION.md, AND tag the dimension.
