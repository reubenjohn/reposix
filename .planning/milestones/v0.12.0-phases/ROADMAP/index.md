## v0.12.0 Quality Gates (PLANNING)

> **Status:** scoping complete; Phases 56–63 scaffolded 2026-04-27. v0.11.x bolted on a §0.8 SESSION-END-STATE framework that caught the regression class IT was designed for — but missed the curl-installer URL going dark for two releases (release-plz cut over to per-crate `reposix-cli-v*` tags, but `release.yml` only matches the workspace-wide `v*` glob, so the workflow stopped firing and `assets:[]` never repopulated). v0.12.0 generalizes §0.8 into a dimension-tagged **Quality Gates** system. Source-of-truth handover bundle (NOT YET WRITTEN — owner authoring 2026-04-27): `.planning/research/v0.12.0/vision-and-mental-model.md`, `.planning/research/v0.12.0/naming-and-architecture.md`, `.planning/research/v0.12.0/roadmap-and-rationale.md`, `.planning/research/v0.12.0/autonomous-execution-protocol.md`, `.planning/research/v0.12.0/install-regression-diagnosis.md`, `.planning/research/v0.12.0/decisions-log.md`, `.planning/research/v0.12.0/open-questions-and-deferrals.md`. DRAFT seed: `.planning/docs_reproducible_catalog.json`.

**Thesis.** Catalogs are the data; verifiers are the code; reports are the artifacts; runners compose by tag. Every gate answers three orthogonal questions — **dimension** (code / docs-build / docs-repro / release / structure / perf / security / agent-ux), **cadence** (pre-push / pre-pr / weekly / pre-release / post-release / on-demand), **kind** (mechanical / container / asset-exists / subagent-graded / manual). Adding a future gate is one catalog row + one verifier in the right dimension dir — never another bespoke `scripts/check-*.sh`. The framework REPLACES the ad-hoc surfaces; after v0.12.0, `scripts/` holds only `hooks/` and `install-hooks.sh`. Every phase ends with an unbiased verifier subagent grading the catalog rows GREEN — no phase ships on the executing agent's word.

**Recurring success criteria for EVERY phase (P56–P63)** — these are non-negotiable per the v0.12.0 autonomous-execution protocol (QG-06, QG-07, OP-4, OP-2, OP-6):

1. **Catalog-first.** The phase's FIRST commit writes the catalog rows (the end-state contract) under `quality/catalogs/<file>.json` BEFORE any implementation commit. The verifier subagent grades against catalog rows that already exist.
2. **CLAUDE.md updated in the same PR.** Every phase that introduces a new file, convention, gate, or operational rule MUST update the relevant CLAUDE.md section in the same PR — not deferred to P63. (QG-07.)
3. **Phase close = unbiased verifier subagent dispatch.** The orchestrator dispatches an isolated subagent with zero session context that grades all catalog rows for this phase against artifacts under `quality/reports/verifications/`; verdict written to `quality/reports/verdicts/<phase>/<ts>.md`; phase does not close on RED. (QG-06.)
4. **SIMPLIFY absorption (where applicable).** Phases hosting SIMPLIFY-* items end with every named source surface either folded into `quality/gates/<dim>/`, reduced to a one-line shim with a header-comment reason, or carrying an explicit `quality/catalogs/orphan-scripts.json` waiver row with a reason. No script in scope for a dimension is left untouched.
5. **Fix every RED row the dimension's gates flag (broaden-and-deepen).** When a phase ships a new gate, the gate's first run almost always finds NOT-VERIFIED or FAIL rows. Those rows MUST be either (a) FIXED in the same phase (cite commit), (b) WAIVED with explicit `until` + `reason` + `dimension_owner` per the waiver protocol (capped at 90 days), or (c) filed as a v0.12.1 carry-forward via MIGRATE-03. The milestone does NOT close on NOT-VERIFIED P0+P1 rows. Phases hosting POLISH-* items in `.planning/REQUIREMENTS.md` carry the same closure burden. Goal: after v0.12.0 closes, every dimension's catalog is all-GREEN-or-WAIVED. Owner directive: "I'm really hoping that after this milestone the codebase is pristine and high quality across all the dimensions."

---

## Chapters

- **[Phase 56](./phase-56.md)** — Restore release artifacts — fix the broken installer URLs.
- **[Phase 57](./phase-57.md)** — Quality Gates skeleton + structure dimension migration.
- **[Phase 58](./phase-58.md)** — Release dimension gates + code-dimension absorption.
- **[Phase 59](./phase-59.md)** — Docs-repro dimension + tutorial replay + agent-ux thin-home.
- **[Phase 60](./phase-60.md)** — Docs-build migration + composite runner cutover.
- **[Phase 61](./phase-61.md)** — Subjective gates skill + freshness TTL enforcement.
- **[Phase 62](./phase-62.md)** — Repo-org-gaps cleanup — close the v0.11.1 audit.
- **[Phase 63](./phase-63.md)** — Retire migrated sources + final CLAUDE.md cohesion + v0.12.1 carry-forward.
- **[Phase 64](./phase-64.md)** — Docs-alignment dimension — framework, CLI, skill, hook wiring.
- **[Phase 65](./phase-65.md)** — Docs-alignment backfill — surface the punch list.
