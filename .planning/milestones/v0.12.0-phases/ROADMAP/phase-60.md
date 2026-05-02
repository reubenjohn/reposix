# Phase 60: Docs-build migration + composite runner cutover (v0.12.0)

**Goal:** Move the docs-build surface fully into the framework with no behaviour change. `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` move to `quality/gates/docs-build/` (DOCS-BUILD-01 + SIMPLIFY-08); the pre-push hook delegates to `quality/runners/run.py --cadence pre-push` instead of chaining shell scripts (SIMPLIFY-10). `scripts/green-gauntlet.sh` is supplanted by `quality/runners/run.py --cadence pre-pr` and either deleted or reduced to a one-line shim (SIMPLIFY-09). `scripts/install-hooks.sh` stays as-is (developer install-of-git-hooks is its own concern — not a quality gate). The only behaviour change permitted is the gate composition: previously each pre-push hook line invoked a different script; after this phase, the pre-push hook is one runner invocation that fans out by tag. Old paths get shims if other tooling imports them; otherwise deleted. Operating-principle hooks: **OP-5 reversibility** — keep old paths as shims for one merge cycle so any hidden caller surfaces; **OP-1 close the feedback loop** — playwright walks (per CLAUDE.md docs-site validation rule) keep firing post-cutover, the runner just composes them.

**Requirements:** DOCS-BUILD-01, BADGE-01, QG-09 (P60 portion: mkdocs publish + endpoint badge), SIMPLIFY-08, SIMPLIFY-09, SIMPLIFY-10, POLISH-DOCS-BUILD

**Depends on:** P59 GREEN. **Gate-state precondition:** P59's docs-repro-dimension catalog shows GREEN in `quality/reports/verdicts/p59/`, AND the runner has demonstrated parity for two pre-push cycles (so the hook cutover is safe per OP-5 parallel-migration rule).

**Success criteria:**
1. `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` are moved to `quality/gates/docs-build/` with no behaviour change. Old paths are one-line shims that call the new location, OR deleted if no caller exists. `quality/catalogs/docs-build.json` carries a row per gate.
2. `scripts/hooks/pre-push` body is simplified to a single `quality/runners/run.py --cadence pre-push` invocation. `scripts/hooks/test-pre-push.sh` is updated to test the new entry point and passes. `scripts/install-hooks.sh` is unchanged (out of scope).
3. `scripts/green-gauntlet.sh` is supplanted by `quality/runners/run.py --cadence pre-pr` and is either deleted or reduced to a one-line shim. CI workflows that invoked green-gauntlet are updated to invoke the runner directly.
4. **Parity demonstrated:** `quality/reports/verdicts/pre-push/` shows two consecutive GREEN runs across the new runner that match (or improve on) the pre-cutover script-chain output. Documented in `quality/SURPRISES.md` if any divergence.
5. **SIMPLIFY absorption (P60 dimension = docs-build + composite):** `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py`, `scripts/green-gauntlet.sh` all moved/shimmed/waived. The pre-push hook body is one line. `scripts/hooks/test-pre-push.sh` is updated.
6. **Recurring (catalog-first):** Catalog rows for DOCS-BUILD-01 + SIMPLIFY-08..10 land in `quality/catalogs/{docs-build,orphan-scripts}.json` BEFORE the file-move + hook-rewrite commits.
7. **Recurring (CLAUDE.md):** CLAUDE.md "Docs-site validation" section updated to point at `quality/gates/docs-build/` + the runner cadence; the "Subagent delegation rules" section updated if the docs-build composition changes how subagents dispatch playwright walks. In the same PR.
8. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p60/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Docs-build dimension migration" + § "Aggressive simplification" SIMPLIFY-08..10, `.planning/research/v0.12.0/naming-and-architecture.md` § "docs-build dimension" (gate inventory), existing `scripts/check-docs-site.sh` + `scripts/check-mermaid-renders.sh` + `scripts/check-doc-links.py` + `scripts/green-gauntlet.sh` + `scripts/hooks/pre-push` (surfaces being absorbed).

**Plans:** 8 plans
- [ ] 60-01-PLAN.md — Wave A catalog-first commit: docs-build.json (4 rows) + code.json (+2 rows) + freshness-invariants.json (+1 row + 1 amend) + docs-build dimension README (DOCS-BUILD-01 + BADGE-01 + SIMPLIFY-08/09/10 contract; short-lived waivers per the catalog-first pattern)
- [ ] 60-02-PLAN.md — Wave B docs-build verifier migrations: git mv 3 verifiers (mkdocs-strict.sh + mermaid-renders.sh + link-resolution.py) + path-arithmetic fixes + thin shims at old paths (SIMPLIFY-08; 3 of 4 verifiers)
- [ ] 60-03-PLAN.md — Wave C BADGE-01 verifier (badges-resolve.py) ships + both badges-resolve catalog rows unwaived (BADGE-01)
- [ ] 60-04-PLAN.md — Wave D 3 verifier wrappers (cargo-fmt-check.sh + cargo-clippy-warnings.sh + cred-hygiene.sh) + green-gauntlet shim (SIMPLIFY-09) + 3 catalog rows unwaived
- [ ] 60-05-PLAN.md — Wave E pre-push hook one-liner rewrite + test-pre-push.sh validation (SIMPLIFY-10)
- [ ] 60-06-PLAN.md — Wave F QG-09 publish: docs/badge.json + README + docs/index.md endpoint badge + WAVE_F_PENDING_URLS clear (QG-09 P60 portion)
- [ ] 60-07-PLAN.md — Wave G POLISH-DOCS-BUILD broaden-and-deepen sweep: 4 cadences GREEN; fix every RED row in-phase or carry-forward via MIGRATE-03 (POLISH-DOCS-BUILD)
- [ ] 60-08-PLAN.md — Wave H phase close: CLAUDE.md QG-07 + STATE.md cursor + REQUIREMENTS.md traceability flips + SURPRISES.md update + verifier subagent verdict GREEN (QG-06 + QG-07)
