---
phase: 59-docs-repro-dimension-agent-ux-thin-home-perf-relocate
plan: 04
wave: D
status: shipped
duration_min: 5
commit: 4686892
shipped_at: "2026-04-27T22:30:00Z"
---

# Wave D — SIMPLIFY-07 + POLISH-AGENT-UX

## Outcome

GREEN. SIMPLIFY-07 closed via shim (not delete) due to 14 caller references in docs/examples/CLAUDE.md. POLISH-AGENT-UX confirmed no regression vs v0.9.0 baseline (BEFORE state predecessor exit 0; AFTER state migration target exit 0).

## BEFORE-state (Task 1)

```
$ bash scripts/dark-factory-test.sh sim
DARK-FACTORY DEMO COMPLETE — sim backend: agent UX is pure git.
exit code: 0
```

Caller audit (15 hits):
- `.github/workflows/ci.yml:68`
- `scripts/green-gauntlet.sh:90`
- `docs/reference/cli.md:291`
- `docs/development/contributing.md:48`
- `docs/reference/crates.md:52`
- `docs/reference/simulator.md:83`
- `docs/decisions/001-github-state-mapping.md:28,109`
- `examples/03-claude-code-skill/RUN.md:17,36`
- `examples/04-conflict-resolve/expected-output.md:36`
- `examples/05-blob-limit-recovery/{RUN.md:9,19, expected-output.md:42}`
- `README.md:108`
- `CLAUDE.md:120`

Decision: SHIM (not DELETE). Predecessor's caller surface is too wide; shim preserves OP-5 reversibility. P63 SIMPLIFY-12 may delete.

## AFTER-state

- `quality/gates/agent-ux/dark-factory.sh` — 156 lines (under 160 cap); SIMPLIFY-07 lineage header (15 lines); artifact-write block (writes `quality/reports/verifications/agent-ux/dark-factory-sim.json`); v0.9.0 invariant body unchanged. Smoke run exit 0.
- `scripts/dark-factory-test.sh` — 7-line shim that exec's the canonical path. Smoke run via shim exit 0 (transparent delegation).
- `.github/workflows/ci.yml:68` — 1-line edit: `bash scripts/dark-factory-test.sh sim` → `bash quality/gates/agent-ux/dark-factory.sh sim` (canonical path per OP-1).
- `quality/catalogs/agent-ux.json` row `agent-ux/dark-factory-sim` — flipped NOT-VERIFIED → PASS via runner. `last_verified: 2026-04-27T22:27:49Z`.

## Runner state at close

```
$ python3 quality/runners/run.py --cadence pre-pr
[PASS] agent-ux/dark-factory-sim (P1, 0.24s)
summary: 1 PASS, 0 FAIL, 0 PARTIAL, 2 WAIVED, 0 NOT-VERIFIED -> exit=0

$ python3 quality/runners/run.py --cadence pre-push
summary: 11 PASS, 0 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED -> exit=0
```

## Commit

`4686892` — `feat(p59): migrate dark-factory.sh to quality/gates/agent-ux/ (SIMPLIFY-07 + POLISH-AGENT-UX)`. Pre-push hook GREEN; pushed to origin/main.

## Regression check

None found. The v0.9.0 dark-factory invariant (helper stderr-teaching strings emit on conflict + blob-limit paths) is preserved verbatim. POLISH-AGENT-UX broaden-and-deepen complete.
