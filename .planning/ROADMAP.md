# Roadmap: reposix

## Milestones

- ✅ **v0.1.0 MVD** — Phases 1-4, S (shipped 2026-04-13) · [archive](milestones/v0.8.0-phases/ROADMAP.md)
- ✅ **v0.2.0-alpha** — Phase 8: GitHub read-only adapter (shipped 2026-04-13)
- ✅ **v0.3.0** — Phase 11: Confluence Cloud read-only adapter (shipped 2026-04-14)
- ✅ **v0.4.0** — Phase 13: Nested mount layout pages/+tree/ (shipped 2026-04-14)
- ✅ **v0.5.0** — Phases 14-15: IssueBackend decoupling + bucket _INDEX.md (shipped 2026-04-14)
- ✅ **v0.6.0** — Phases 16-20: Write path + full sitemap (shipped 2026-04-14)
- ✅ **v0.7.0** — Phases 21-26: Hardening + Confluence expansion + docs (shipped 2026-04-16)
- ✅ **v0.8.0 JIRA Cloud Integration** — Phases 27-29 (shipped 2026-04-16)
- ✅ **v0.9.0 Architecture Pivot — Git-Native Partial Clone** — Phases 31–36 (shipped 2026-04-24) · [archive](milestones/v0.9.0-phases/ROADMAP.md)
- ✅ **v0.10.0 Docs & Narrative Shine** — Phases 40–45 (shipped 2026-04-25) · [archive](milestones/v0.10.0-phases/ROADMAP.md)
- ✅ **v0.11.x Polish & Reproducibility** — Phases 50–55 + POLISH2-* polish passes (v0.11.0 shipped 2026-04-25; v0.11.1 + v0.11.2 polish passes shipped 2026-04-26 / 2026-04-27 via release-plz; all 8 crates published to crates.io at v0.11.2)
- 🚧 **v0.12.0 Quality Gates** — Phases 56–63 (planning 2026-04-27; dimension-tagged gates, catalog-first phases, mandatory verifier-subagent grading, weekly-cadence drift detectors, restore broken curl/PowerShell installer URLs)

## Phases

## v0.12.0 Quality Gates (PLANNING)

> **Status:** scoping complete; Phases 56–63 scaffolded 2026-04-27. v0.11.x bolted on a §0.8 SESSION-END-STATE framework that caught the regression class IT was designed for — but missed the curl-installer URL going dark for two releases (release-plz cut over to per-crate `reposix-cli-v*` tags, but `release.yml` only matches the workspace-wide `v*` glob, so the workflow stopped firing and `assets:[]` never repopulated). v0.12.0 generalizes §0.8 into a dimension-tagged **Quality Gates** system. Source-of-truth handover bundle (NOT YET WRITTEN — owner authoring 2026-04-27): `.planning/research/v0.12.0-vision-and-mental-model.md`, `.planning/research/v0.12.0-naming-and-architecture.md`, `.planning/research/v0.12.0-roadmap-and-rationale.md`, `.planning/research/v0.12.0-autonomous-execution-protocol.md`, `.planning/research/v0.12.0-install-regression-diagnosis.md`, `.planning/research/v0.12.0-decisions-log.md`, `.planning/research/v0.12.0-open-questions-and-deferrals.md`. DRAFT seed: `.planning/docs_reproducible_catalog.json`.

**Thesis.** Catalogs are the data; verifiers are the code; reports are the artifacts; runners compose by tag. Every gate answers three orthogonal questions — **dimension** (code / docs-build / docs-repro / release / structure / perf / security / agent-ux), **cadence** (pre-push / pre-pr / weekly / pre-release / post-release / on-demand), **kind** (mechanical / container / asset-exists / subagent-graded / manual). Adding a future gate is one catalog row + one verifier in the right dimension dir — never another bespoke `scripts/check-*.sh`. The framework REPLACES the ad-hoc surfaces; after v0.12.0, `scripts/` holds only `hooks/` and `install-hooks.sh`. Every phase ends with an unbiased verifier subagent grading the catalog rows GREEN — no phase ships on the executing agent's word.

**Recurring success criteria for EVERY phase (P56–P63)** — these are non-negotiable per the v0.12.0 autonomous-execution protocol (QG-06, QG-07, OP-4, OP-2, OP-6):

1. **Catalog-first.** The phase's FIRST commit writes the catalog rows (the end-state contract) under `quality/catalogs/<file>.json` BEFORE any implementation commit. The verifier subagent grades against catalog rows that already exist.
2. **CLAUDE.md updated in the same PR.** Every phase that introduces a new file, convention, gate, or operational rule MUST update the relevant CLAUDE.md section in the same PR — not deferred to P63. (QG-07.)
3. **Phase close = unbiased verifier subagent dispatch.** The orchestrator dispatches an isolated subagent with zero session context that grades all catalog rows for this phase against artifacts under `quality/reports/verifications/`; verdict written to `quality/reports/verdicts/<phase>/<ts>.md`; phase does not close on RED. (QG-06.)
4. **SIMPLIFY absorption (where applicable).** Phases hosting SIMPLIFY-* items end with every named source surface either folded into `quality/gates/<dim>/`, reduced to a one-line shim with a header-comment reason, or carrying an explicit `quality/catalogs/orphan-scripts.json` waiver row with a reason. No script in scope for a dimension is left untouched.
5. **Fix every RED row the dimension's gates flag (broaden-and-deepen).** When a phase ships a new gate, the gate's first run almost always finds NOT-VERIFIED or FAIL rows. Those rows MUST be either (a) FIXED in the same phase (cite commit), (b) WAIVED with explicit `until` + `reason` + `dimension_owner` per the waiver protocol (capped at 90 days), or (c) filed as a v0.12.1 carry-forward via MIGRATE-03. The milestone does NOT close on NOT-VERIFIED P0+P1 rows. Phases hosting POLISH-* items in `.planning/REQUIREMENTS.md` carry the same closure burden. Goal: after v0.12.0 closes, every dimension's catalog is all-GREEN-or-WAIVED. Owner directive: "I'm really hoping that after this milestone the codebase is pristine and high quality across all the dimensions."

### Phase 56: Restore release artifacts — fix the broken installer URLs (v0.12.0)

**Goal:** Close the user-facing breakage that motivated this milestone. `release.yml` does not fire on release-plz's per-crate `reposix-cli-v*` tags; consequently the curl/PowerShell installer URLs return `Not Found`, the homebrew tap formula has not auto-updated, and `cargo binstall reposix-cli reposix-remote` falls back to source build because no GH binary asset exists. Pick the cleaner of two options diagnosed in `.planning/research/v0.12.0-install-regression-diagnosis.md` (extend `on.push.tags` glob to match `reposix-cli-v*` and key the dist version off the cli tag, OR add a release-plz post-publish step that mirrors a workspace `vX.Y.Z` tag). Cut a fresh `reposix-cli-v0.11.3` (or equivalent) release and verify all 5 install paths end-to-end against the freshly-published assets. This phase is the catalyst that proves the framework is needed; the framework itself lands in P57. Operating-principle hooks: **OP-1 close the feedback loop** — fetch each install URL from a fresh container or curl session, do not trust the workflow log; **OP-6 ground truth obsession** — the verifier subagent runs each install path verbatim from the docs and asserts the binary lands on PATH.

**Requirements:** RELEASE-01, RELEASE-02, RELEASE-03

**Depends on:** (nothing — entry-point phase; v0.11.x shipped state is the precondition)

**Success criteria:**
1. `release.yml` fires on the appropriate tag pattern (per the chosen option in the diagnosis doc); a fresh release tag triggers the workflow and produces non-empty `assets:[…]` on the GH Release.
2. **All 5 install paths verified end-to-end against the fresh release:** `curl … | sh` (Linux/macOS), `iwr … | iex` (PowerShell), `brew install reubenjohn/reposix/reposix-cli`, `cargo binstall reposix-cli reposix-remote` (resolves to prebuilt binary, not source fallback), `cargo install reposix-cli` (build-from-source). Each path's success is recorded as a row in `quality/reports/verifications/release/install-paths/<path>.json`.
3. The `upload-homebrew-formula` job in `release.yml` runs and bumps the tap formula to the new version.
4. Catalog rows for the install paths land in `quality/catalogs/install-paths.json` (or equivalent — the unified-schema name is finalized in P57; for P56 the rows may live in a temporary catalog that P57 migrates) BEFORE the release.yml fix commit.
5. CLAUDE.md updated to reflect this phase's contributions (release.yml tag-glob convention, install-path verification expectation, the new install-paths catalog reference) in the same PR.
6. Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p56/<ts>.md`. RELEASE-01..03 flip from `planning` → `shipped` only after the verifier verdict.

**Context anchor:** `.planning/REQUIREMENTS.md` `## v0.12.0 Requirements — Quality Gates` § "Release dimension — close the immediate breakage", `.planning/research/v0.12.0-install-regression-diagnosis.md` (root cause + two fix options + recommended choice), v0.11.x release-plz workflow at `.github/workflows/release-plz.yml`, the broken `release.yml` it stopped firing.

### Phase 57: Quality Gates skeleton + structure dimension migration (v0.12.0)

**Goal:** Stand up the framework. `quality/{gates,catalogs,reports,runners}/` directory layout lands; `quality/PROTOCOL.md` documents the autonomous-mode runtime contract (gate routing, catalog-first rule, waiver TTL, pivot triggers, skill-dispatch patterns, anti-bloat rules); `quality/SURPRISES.md` opens as the append-only pivot journal; `quality/runners/run.py --cadence X` and `quality/runners/verdict.py` ship as the single composition entry point for pre-push / pre-pr / weekly / pre-release / post-release. The structure dimension migrates first because it is the lowest-blast-radius surface (existing freshness rows in `scripts/end-state.py`) and proves the catalog → verifier → runner → verdict round-trip end-to-end. The 6 freshness rows from `scripts/end-state.py` move to `quality/gates/structure/`; `scripts/end-state.py` reduces to a ≤30-line shim that delegates to `quality/runners/verdict.py session-end` with an anti-bloat header comment warning future agents off growing it. SIMPLIFY-01 (`scripts/banned-words-lint.sh` → `quality/gates/structure/banned-words.sh`), SIMPLIFY-02 (the end-state.py shim), and SIMPLIFY-03 (`scripts/catalog.py` audit — fold or document boundary) absorb the existing structure-dimension surfaces. QG-08 enforces the "top-level REQUIREMENTS.md / ROADMAP.md hold ONLY the active milestone" convention as a structure-catalog row that fails any future drift. Operating-principle hooks: **OP-4 self-improving infrastructure** — the framework IS this principle made structural; **OP-5 reversibility** — old `scripts/end-state.py` and new `quality/runners/run.py --cadence pre-push` run side-by-side for two pre-push cycles before the shim cutover; **OP-2 aggressive subagent delegation** — the QG-06 verifier subagent pattern is documented in PROTOCOL.md and dogfooded for this phase's own close.

**Requirements:** QG-01, QG-02, QG-03, QG-04, QG-05, QG-06, QG-07, QG-08, STRUCT-01, STRUCT-02, SIMPLIFY-01, SIMPLIFY-02, SIMPLIFY-03

**Depends on:** P56 GREEN. **Gate-state precondition:** P56's release/install-paths catalog rows show GREEN in `quality/reports/verdicts/p56/`, AND the verifier subagent's P56 verdict file exists. P57 cannot start on bare "P56 merged" — it starts on "P56 verifier subagent verdict written and GREEN."

**Success criteria:**
1. `quality/{gates,catalogs,reports,runners}/` exists with `quality/catalogs/README.md` documenting the unified catalog schema (every row carries `id`, `dimension`, `cadence`, `kind`, `sources`, `verifier`, `artifact`, `status`, `freshness_ttl`, `waiver`, `blast_radius`, `owner_hint`).
2. `quality/runners/run.py --cadence pre-push` discovers and runs every gate tagged `pre-push`, emits per-gate artifacts to `quality/reports/verifications/`, and `quality/runners/verdict.py` collates them into `quality/reports/verdicts/pre-push/<ts>.md` with non-zero exit on RED. `pre-push` hook delegates to it (no behavioural regression vs current `end-state.py` row-set).
3. `quality/PROTOCOL.md` ships as a single-page contract every phase agent reads at start: gate-routing table, catalog-first rule, pivot triggers, waiver protocol with TTL (every catalog row supports `waiver: {until: <RFC3339>, reason, dimension_owner}`; expired waivers flip back to FAIL), skill-dispatch patterns, "when stuck" rules, anti-bloat rules per surface. `quality/SURPRISES.md` opens with a header explaining its append-only one-line-per-obstacle / one-line-per-resolution convention.
4. The 6 freshness rows from `scripts/end-state.py` move to `quality/gates/structure/` with rows in `quality/catalogs/freshness-invariants.json`; `scripts/end-state.py` is ≤30 lines, delegates to `quality/runners/verdict.py session-end`, and has an anti-bloat header comment naming `quality/gates/<dim>/` as the home for new rules.
5. **SIMPLIFY absorption (P57 dimension = structure):** every script in scope for this dimension is either folded into `quality/gates/structure/` (banned-words, freshness rows), reduced to a one-line shim (`scripts/end-state.py`), or has a waiver row in `quality/catalogs/orphan-scripts.json` with a reason (e.g. `scripts/catalog.py` if its domain doesn't fully overlap with `quality/runners/verdict.py`).
6. QG-08 enforced: a structure-dimension catalog row fails if any `*ROADMAP*.md` or `*REQUIREMENTS*.md` exists at `.planning/` top level outside the active milestone scope, OR at `.planning/milestones/v*-*.md` outside `*-phases/` dirs (extends the existing CLAUDE.md §0.5 / `scripts/end-state.py` `freshness/no-loose-roadmap-or-requirements` claim into the new framework).
7. **Recurring (catalog-first):** Catalog rows for QG-01..08 + STRUCT-01..02 + SIMPLIFY-01..03 land in `quality/catalogs/{freshness-invariants,orphan-scripts,framework-skeleton}.json` BEFORE any implementation commit.
8. **Recurring (CLAUDE.md):** CLAUDE.md updated in the same PR with: (a) new "Quality Gates" section pointing at `quality/PROTOCOL.md`, (b) updated §"Subagent delegation rules" referencing the QG-06 verifier pattern, (c) the QG-07 mandatory-update rule itself documented as a project meta-rule.
9. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p57/<ts>.md`. The same subagent dogfoods the QG-06 pattern PROTOCOL.md documents.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Quality Gates framework" + § "Structure dimension — migrate freshness invariants" + § "Aggressive simplification" SIMPLIFY-01..03, `.planning/research/v0.12.0-naming-and-architecture.md` (directory layout + unified schema), `.planning/research/v0.12.0-autonomous-execution-protocol.md` (catalog-first rule + waiver TTL + verifier dispatch), existing `scripts/end-state.py` (the surface being absorbed).

### Phase 58: Release dimension gates + code-dimension absorption (v0.12.0)

**Goal:** Build the release dimension that would have caught the curl-installer regression within 24h of the release-plz cutover. `quality/gates/release/{gh-assets-present.py, brew-formula-current.py, crates-io-max-version.py, installer-asset-bytes.py}` ship with weekly + post-release runners. Catalog rows for every install URL, brew formula, and crates.io crate live in `quality/catalogs/install-paths.json` and `quality/catalogs/crates-io.json` (the latter migrates from `scripts/end-state.py`'s existing crates.io rows). SIMPLIFY-04 (`scripts/check_clippy_lint_loaded.sh` → `quality/gates/code/clippy-lint-loaded.sh` with a catalog row recording the expected lint set so silent lint removal trips the gate) and SIMPLIFY-05 (`scripts/check_fixtures.py` audit — code-dimension gate or `crates/<crate>/tests/` integration test, move accordingly) close the smaller code-dimension surfaces; the wider code-dimension build (clippy/fmt/test as full gates) is deliberately deferred since the existing pre-push hook + CI already cover it. Operating-principle hooks: **OP-1 close the feedback loop** — `installer-asset-bytes.py` actually GETs the installer URL and asserts non-zero `Content-Length`, no trusting the workflow log; **OP-3 ROI awareness** — the weekly cadence is the cheapest possible insurance against another silent two-release breakage.

**Requirements:** RELEASE-04, SIMPLIFY-04, SIMPLIFY-05

**Depends on:** P57 GREEN. **Gate-state precondition:** P57's structure-dimension catalog shows GREEN in `quality/reports/verdicts/p57/`, AND the framework skeleton (`quality/runners/run.py`, `quality/PROTOCOL.md`, `quality/SURPRISES.md`) is functional — the new dimension gets composed by an existing runner, not a one-off.

**Success criteria:**
1. `quality/gates/release/{gh-assets-present.py, brew-formula-current.py, crates-io-max-version.py, installer-asset-bytes.py}` exist and run successfully against current release state. `quality/catalogs/install-paths.json` and `quality/catalogs/crates-io.json` carry rows for every install URL, brew formula, and crates.io crate.
2. `quality/runners/run.py --cadence weekly` discovers and runs the release-dimension gates; a GitHub Actions weekly cron (NOT nightly — owner cost decision) invokes it and PR-creates a verdict-report update if any row flips RED.
3. `quality/runners/run.py --cadence post-release` is wired into the release-plz publish workflow (or runs immediately after release.yml ships) and proves a fresh user can install from the latest release.
4. **Backstop assertion** (OP-1 dogfooded): the installer-asset-bytes verifier downloads each install asset and asserts non-zero bytes + valid signature/checksum where applicable. Synthetic regression test: temporarily mutate the release.yml tag glob, run the runner, confirm RED verdict; revert and confirm GREEN.
5. **SIMPLIFY absorption (P58 dimensions = release + code):** `scripts/check_clippy_lint_loaded.sh` is folded into `quality/gates/code/clippy-lint-loaded.sh` with a catalog row recording the expected lint set; `scripts/check_fixtures.py` is moved to its appropriate home (code-dimension gate OR `crates/<crate>/tests/` integration test) and removed from `scripts/`. Any remainder gets an `orphan-scripts.json` waiver row with a reason.
6. **Recurring (catalog-first):** Catalog rows for RELEASE-04 + SIMPLIFY-04..05 land in `quality/catalogs/{install-paths,crates-io,clippy-lints,orphan-scripts}.json` BEFORE the verifier-script commits.
7. **Recurring (CLAUDE.md):** CLAUDE.md updated to reflect this phase's contributions: weekly cadence convention (cost-conscious, not nightly), the release-dimension gate inventory, the post-release hook integration. In the same PR.
8. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p58/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Release dimension" RELEASE-04 + § "Aggressive simplification" SIMPLIFY-04..05, `.planning/research/v0.12.0-naming-and-architecture.md` § "release dimension" (gate inventory), `.planning/research/v0.12.0-install-regression-diagnosis.md` (the regression class this dimension prevents), v0.11.x crates.io rows in `scripts/end-state.py` that migrate here.

### Phase 59: Docs-repro dimension + tutorial replay + agent-ux thin-home (v0.12.0)

**Goal:** Make every code snippet in user-facing docs a tracked, container-rehearsed gate. `quality/gates/docs-repro/snippet-extract.py` parses every fenced code block in README.md, docs/index.md, docs/tutorials/*.md and emits catalog rows; the drift detector fails if a doc snippet has no catalog row, or a catalog row's content drifted from its source. `quality/gates/docs-repro/container-rehearse.sh <id>` spins ubuntu:24.04 (default), runs the snippet verbatim, asserts post-conditions. `scripts/repro-quickstart.sh` is promoted to `quality/gates/docs-repro/tutorial-replay.sh` as one container-rehearsal-kind row (SIMPLIFY-06 part 1). Every `examples/0[1-5]-*/run.sh` becomes a docs-repro catalog row (container-rehearsal-kind, post-release cadence — SIMPLIFY-06 part 2); the `examples/` README gains a callout that each example is now a tracked gate. SIMPLIFY-07 moves `scripts/dark-factory-test.sh` to `quality/gates/agent-ux/dark-factory.sh`; the agent-ux dimension is documented as "intentionally sparse — perf and security stubs land in v0.12.1" (since dark-factory is the only gate at v0.12.0 close). SIMPLIFY-11 is **file-relocate only** in this phase (the perf-bench scripts and benchmarks/fixtures move to `quality/gates/perf/` with thin shims at old paths if anything imports them); the actual cross-check logic that compares bench output against headline copy is the v0.12.1 stub per MIGRATE-03. The DRAFT seed `.planning/docs_reproducible_catalog.json` ports row-by-row into `quality/catalogs/docs-reproducible.json` with the unified schema (DOCS-REPRO-04). Operating-principle hooks: **OP-1 close the feedback loop** — container rehearsal is the gold-standard "did the snippet actually work" test, not `mkdocs build --strict`; **OP-6 ground truth obsession** — a snippet's catalog row is the ground truth for what GREEN looks like.

**Requirements:** DOCS-REPRO-01, DOCS-REPRO-02, DOCS-REPRO-03, DOCS-REPRO-04, SIMPLIFY-06, SIMPLIFY-07, SIMPLIFY-11

**Depends on:** P58 GREEN. **Gate-state precondition:** P58's release-dimension catalog shows GREEN in `quality/reports/verdicts/p58/`, AND the post-release runner is operational (docs-repro container rehearsals are a post-release-cadence consumer of the same runner contract).

**Success criteria:**
1. `quality/gates/docs-repro/snippet-extract.py` parses every fenced code block in README.md, docs/index.md, docs/tutorials/*.md; emits a catalog-row stub for any uncatalogued snippet; flips RED if a row's content drifted from its source. Synthetic regression test: edit a snippet and confirm the drift detector fires.
2. `quality/gates/docs-repro/container-rehearse.sh <id>` spins ubuntu:24.04, runs the snippet verbatim, asserts post-conditions defined per row. At least one full tutorial replay passes end-to-end in CI.
3. `scripts/repro-quickstart.sh` is reduced to a one-line shim (or deleted) that calls `quality/gates/docs-repro/tutorial-replay.sh`. Every `examples/0[1-5]-*/run.sh` has a `quality/catalogs/docs-reproducible.json` row (container-rehearsal-kind, post-release cadence). `examples/README.md` includes the "each example is a tracked gate" callout.
4. `quality/catalogs/docs-reproducible.json` exists and contains every row from the DRAFT `.planning/docs_reproducible_catalog.json` ported to the unified schema. The DRAFT file is deleted (or kept only as a header-comment redirect to the new path).
5. `scripts/dark-factory-test.sh` is moved to `quality/gates/agent-ux/dark-factory.sh` with a catalog row; agent-ux dimension's `quality/gates/agent-ux/README.md` documents "intentionally sparse — perf and security stubs land in v0.12.1." Old path is one-line shim or deleted.
6. **SIMPLIFY-11 file-relocate only:** `scripts/bench_token_economy.py`, `scripts/test_bench_token_economy.py`, `scripts/latency-bench.sh`, `benchmarks/fixtures/*` move to `quality/gates/perf/` with thin shims at old paths IF anything imports them (else delete). A perf-targets catalog stub lands as a placeholder; the actual cross-check logic is v0.12.1 stub per MIGRATE-03 — explicitly waived in `quality/catalogs/orphan-scripts.json` with reason "v0.12.1 stub — file-relocate only at v0.12.0."
7. **SIMPLIFY absorption (P59 dimensions = docs-repro + agent-ux + perf-relocate):** every script/example in scope is either folded into `quality/gates/<dim>/`, reduced to a one-line shim, or has a waiver row in `quality/catalogs/orphan-scripts.json` with a reason. **Plus** every `examples/0[1-5]-*/run.sh` has a docs-reproducible catalog row.
8. **Recurring (catalog-first):** Catalog rows for DOCS-REPRO-01..04 + SIMPLIFY-06..07 + SIMPLIFY-11 land in `quality/catalogs/{docs-reproducible,perf-targets,orphan-scripts}.json` BEFORE the verifier-script commits.
9. **Recurring (CLAUDE.md):** CLAUDE.md updated to reflect this phase's contributions: docs-repro container-rehearse convention, the "examples are tracked gates" rule, the agent-ux sparse-dimension note, the SIMPLIFY-11 v0.12.1 carry-forward. In the same PR.
10. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p59/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Docs-repro dimension" + § "Aggressive simplification" SIMPLIFY-06, 07, 11, `.planning/research/v0.12.0-naming-and-architecture.md` § "docs-repro dimension", `.planning/docs_reproducible_catalog.json` (DRAFT seed being ported), existing `scripts/repro-quickstart.sh` + `scripts/dark-factory-test.sh` (surfaces being absorbed).

**Plans:** 6 plans
- [ ] 59-01-PLAN.md — Wave A catalog-first commit: docs-reproducible.json + agent-ux.json + perf-targets.json + 3 dimension READMEs (DOCS-REPRO-04 + SIMPLIFY-06/07/11 contract)
- [ ] 59-02-PLAN.md — Wave B snippet-extract.py drift detector + docs-repro/snippet-coverage row (DOCS-REPRO-01 + DOCS-REPRO-04)
- [ ] 59-03-PLAN.md — Wave C container-rehearse.sh + tutorial-replay.sh + manual-spec-check.sh + repro-quickstart.sh shim/delete (DOCS-REPRO-02/03 + SIMPLIFY-06)
- [ ] 59-04-PLAN.md — Wave D dark-factory.sh migration + ci.yml canonical-path edit (SIMPLIFY-07 + POLISH-AGENT-UX)
- [ ] 59-05-PLAN.md — Wave E perf-dimension file relocate + shims + benchmarks/README pointer (SIMPLIFY-11 v0.12.0 stub)
- [ ] 59-06-PLAN.md — Wave F POLISH-DOCS-REPRO + POLISH-AGENT-UX broaden-and-deepen + CLAUDE.md QG-07 + verifier QG-06 verdict + STATE/SURPRISES advance

### Phase 60: Docs-build migration + composite runner cutover (v0.12.0)

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

**Context anchor:** `.planning/REQUIREMENTS.md` § "Docs-build dimension migration" + § "Aggressive simplification" SIMPLIFY-08..10, `.planning/research/v0.12.0-naming-and-architecture.md` § "docs-build dimension" (gate inventory), existing `scripts/check-docs-site.sh` + `scripts/check-mermaid-renders.sh` + `scripts/check-doc-links.py` + `scripts/green-gauntlet.sh` + `scripts/hooks/pre-push` (surfaces being absorbed).

**Plans:** 8 plans
- [ ] 60-01-PLAN.md — Wave A catalog-first commit: docs-build.json (4 rows) + code.json (+2 rows) + freshness-invariants.json (+1 row + 1 amend) + docs-build dimension README (DOCS-BUILD-01 + BADGE-01 + SIMPLIFY-08/09/10 contract; short-lived waivers per the catalog-first pattern)
- [ ] 60-02-PLAN.md — Wave B docs-build verifier migrations: git mv 3 verifiers (mkdocs-strict.sh + mermaid-renders.sh + link-resolution.py) + path-arithmetic fixes + thin shims at old paths (SIMPLIFY-08; 3 of 4 verifiers)
- [ ] 60-03-PLAN.md — Wave C BADGE-01 verifier (badges-resolve.py) ships + both badges-resolve catalog rows unwaived (BADGE-01)
- [ ] 60-04-PLAN.md — Wave D 3 verifier wrappers (cargo-fmt-check.sh + cargo-clippy-warnings.sh + cred-hygiene.sh) + green-gauntlet shim (SIMPLIFY-09) + 3 catalog rows unwaived
- [ ] 60-05-PLAN.md — Wave E pre-push hook one-liner rewrite + test-pre-push.sh validation (SIMPLIFY-10)
- [ ] 60-06-PLAN.md — Wave F QG-09 publish: docs/badge.json + README + docs/index.md endpoint badge + WAVE_F_PENDING_URLS clear (QG-09 P60 portion)
- [ ] 60-07-PLAN.md — Wave G POLISH-DOCS-BUILD broaden-and-deepen sweep: 4 cadences GREEN; fix every RED row in-phase or carry-forward via MIGRATE-03 (POLISH-DOCS-BUILD)
- [ ] 60-08-PLAN.md — Wave H phase close: CLAUDE.md QG-07 + STATE.md cursor + REQUIREMENTS.md traceability flips + SURPRISES.md update + verifier subagent verdict GREEN (QG-06 + QG-07)

### Phase 61: Subjective gates skill + freshness TTL enforcement (v0.12.0)

**Goal:** Subjective gates (cold-reader hero clarity, install positioning, headline-numbers sanity) become first-class catalog citizens with TTL freshness enforcement. `quality/catalogs/subjective-rubrics.json` ships with seed rubrics — `cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity` — each with a numeric scoring rubric and `freshness_ttl: 30d` default. The `reposix-quality-review` skill ships at `.claude/skills/reposix-quality-review/SKILL.md`: it reads the catalog, dispatches one unbiased subagent per stale/unverified row in parallel (per OP-2), persists JSON artifacts to `quality/reports/verifications/subjective/`, updates the catalog row's `last_verified` timestamp + `status`. It integrates the existing `doc-clarity-review` skill as one rubric implementation (the cold-reader rubric). SUBJ-03 wires the skill into pre-release cadence so subjective gates with TTL ≥ 14d expired auto-dispatch before any milestone tag push — a release cannot ship with stale subjective rows. Operating-principle hooks: **OP-2 aggressive subagent delegation** — one rubric = one isolated subagent, parallel dispatch; **OP-6 ground truth obsession** — TTL is the explicit "this row's freshness is proof, not history" contract.

**Requirements:** SUBJ-01, SUBJ-02, SUBJ-03

**Depends on:** P60 GREEN. **Gate-state precondition:** P60's docs-build-dimension catalog shows GREEN in `quality/reports/verdicts/p60/`, AND the runner cadence cutover is complete (subjective gates compose into the same runner contract as a `pre-release`-cadence consumer).

**Success criteria:**
1. `quality/catalogs/subjective-rubrics.json` exists with at minimum 3 seed rows: `cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity`. Each row carries a numeric scoring rubric, `freshness_ttl` (default 30d), `last_verified` timestamp, and `status`.
2. `.claude/skills/reposix-quality-review/SKILL.md` ships with frontmatter `name: reposix-quality-review` and a one-line description. The skill reads the rubrics catalog, dispatches one subagent per stale/unverified row IN PARALLEL, persists per-row JSON artifacts to `quality/reports/verifications/subjective/<rubric-id>/<ts>.json`, updates catalog row.
3. The skill integrates the existing `doc-clarity-review` skill as the implementation of the `cold-reader-hero-clarity` rubric (rubric → skill mapping documented in the catalog row).
4. `quality/runners/run.py --cadence pre-release` invokes the `reposix-quality-review` skill against any rubric whose `last_verified + freshness_ttl < now`, blocks the runner exit on RED, and writes a verdict to `quality/reports/verdicts/pre-release/<ts>.md`.
5. Synthetic regression test: backdate a rubric's `last_verified` past its TTL, run `--cadence pre-release`, confirm the skill auto-dispatches a fresh review and updates the row.
6. **Recurring (catalog-first):** Catalog rows for SUBJ-01..03 land in `quality/catalogs/subjective-rubrics.json` BEFORE the skill commits.
7. **Recurring (CLAUDE.md):** CLAUDE.md "Cold-reader pass on user-facing surfaces" section updated to point at the new skill + the rubric catalog; the §"Subagent delegation rules" gains the `reposix-quality-review` parallel-dispatch pattern. In the same PR.
8. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p61/<ts>.md`. (Verifier in this case grades catalog mechanics, not the rubrics themselves — the rubrics are graded by the skill it dispatches.)

**Context anchor:** `.planning/REQUIREMENTS.md` § "Subjective gates" SUBJ-01..03, `.planning/research/v0.12.0-naming-and-architecture.md` § "subjective gates" (skill + TTL design), existing `.claude/skills/doc-clarity-review/SKILL.md` (integrated as one rubric implementation), CLAUDE.md "Cold-reader pass" section (the convention the catalog encodes).

### Phase 62: Repo-org-gaps cleanup — close the v0.11.1 audit (v0.12.0)

**Goal:** Audit `.planning/research/v0.11.1-repo-organization-gaps.md` against current state and close out every remaining gap as either a fix + structure-dimension catalog row that prevents recurrence, or an explicit waiver with reason. The repo-organization-gaps doc is a forgotten todo list if not actioned; this phase ensures every gap becomes a tracked catalog row in the new framework (so a future gap audit doesn't have to be a manual document grep). Operating-principle hooks: **OP-4 self-improving infrastructure** — each gap that recurred under v0.11.x is evidence of a missing structure gate, which this phase backfills; **OP-6 ground truth obsession** — "fixed in CLAUDE.md but not enforced" is not a fix; only a catalog row + verifier counts.

**Requirements:** ORG-01

**Depends on:** P57 GREEN (structure dimension must exist), P61 GREEN (so the repo-org gaps can route into the now-mature dimension set, not into a half-built framework). **Gate-state precondition:** P61's subjective-gates catalog shows GREEN in `quality/reports/verdicts/p61/`. (P62 is also independent enough to slot earlier if scheduling demands it, but the natural order is "polish the framework first, then sweep gaps into it.")

**Success criteria:**
1. Every gap in `.planning/research/v0.11.1-repo-organization-gaps.md` has a status: `closed-by-catalog-row` (gap fixed + recurrence prevented by a structure-dimension catalog row), `closed-by-existing-gate` (gap fixed + already covered by an earlier P57/P58/P60 gate), or `waived` (explicit `quality/catalogs/orphan-scripts.json` or `quality/catalogs/waivers.json` row with reason + dimension_owner + RFC3339 `until`).
2. The audit results are committed under `quality/reports/verifications/repo-org-gaps/<ts>.md` with a row per gap and its closure path.
3. The `.planning/research/v0.11.1-repo-organization-gaps.md` document gets a top-banner update naming "fully audited and closed under P62; see `quality/reports/verifications/repo-org-gaps/<ts>.md` for per-gap closure" — the document is no longer a forgotten todo list.
4. New structure-dimension catalog rows added under `quality/catalogs/freshness-invariants.json` (or a new `repo-org.json`) for each gap that needed a recurrence guard.
5. **Recurring (catalog-first):** ORG-01 catalog row + the per-gap closure rows land BEFORE the audit-fix commits.
6. **Recurring (CLAUDE.md):** CLAUDE.md updated to cite the audit closure + new recurrence-guard rows + waivers (if any) in the appropriate freshness-invariant or workspace-layout sections. In the same PR.
7. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p62/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Repo-org cleanup" ORG-01, `.planning/research/v0.11.1-repo-organization-gaps.md` (the audit document being closed), existing `quality/gates/structure/` rows from P57 (the recurrence-guard infrastructure these gaps will route into).

**Plans:** 6 plans
- [ ] 62-01-PLAN.md — Wave 1 catalog-first commit: 3 structure rows + dim README delta (ORG-01 + POLISH-ORG contract)
- [ ] 62-02-PLAN.md — Wave 2 execute audit: render quality/reports/audits/repo-org-gaps.md + scripts/check_repo_org_gaps.py verifier (ORG-01)
- [ ] 62-03-PLAN.md — Wave 3 POLISH-ORG fix wave: relocate top-level audits + archive SESSION-END-STATE + purge __pycache__ + extend structure verifier
- [ ] 62-04-PLAN.md — Wave 4 SURPRISES.md rotation (302→<=200; archive P57+P58 to SURPRISES-archive-2026-Q2.md)
- [ ] 62-05-PLAN.md — Wave 5 CLAUDE.md QG-07 P62 subsection + audit-doc closure banner + STATE/REQUIREMENTS flips
- [ ] 62-06-PLAN.md — Wave 6 verifier subagent dispatch (Path A or Path B) + verdict GREEN

### Phase 63: Retire migrated sources + final CLAUDE.md cohesion + v0.12.1 carry-forward (v0.12.0)

**Goal:** Final close-out. After SIMPLIFY-01..12 complete and the new system has shown parity in `quality/reports/verdicts/` for two pre-push cycles, delete the migrated source files (anything still kept as a thin shim documents the reason in a header comment per OP-5 reversibility — but the reason has to be real). MIGRATE-01 captures this final retirement. MIGRATE-02 is the cohesion pass on CLAUDE.md: the full dimension/cadence/kind taxonomy section + the meta-rule extension ("when an owner catches a miss: fix the issue, update CLAUDE.md, AND tag the dimension"). Cross-references to all `quality/PROTOCOL.md` sections are audited (per-phase QG-07 updates landed throughout — MIGRATE-02 is the final audit, NOT the only update). MIGRATE-03 files the v0.12.1 carry-forward as stub catalog rows + REQUIREMENTS.md placeholders: perf-dimension full implementation (SIMPLIFY-11 stubs become real gates; latency vs headline-copy cross-check, token-economy bench cross-check), security-dimension stubs (allowlist enforcement gate, audit immutability test), cross-platform container rehearsals (windows-2022, macos-14), AND completion of the `Error::Other` 156→144 partial migration (POLISH2-09 carry-forward from v0.11.1). SIMPLIFY-12 is the final scripts/-tree audit: `find scripts/ -maxdepth 1 -type f | grep -v hooks | grep -v install-hooks.sh` returns empty (or every remaining file has an explicit waiver). Operating-principle hooks: **OP-4 self-improving infrastructure** — the framework that started as scripts is now the system that supersedes them; the `scripts/` dir is the leanest it has ever been; **OP-5 reversibility** — every retirement has a one-cycle parallel proof in `quality/reports/verdicts/`; **OP-1 close the feedback loop** — milestone close requires `gh run view` GREEN AND the QG-06 verifier subagent verdict GREEN AND the audit document closed.

**Requirements:** MIGRATE-01, MIGRATE-02, MIGRATE-03, SIMPLIFY-12

**Depends on:** P56, P57, P58, P59, P60, P61, P62 ALL GREEN. **Gate-state precondition:** every prior phase's verdict file exists and shows GREEN; `quality/reports/verdicts/pre-push/` shows two consecutive GREEN runs across the full runner; `quality/reports/verdicts/pre-pr/` and `quality/reports/verdicts/pre-release/` runner contracts are exercised at least once each.

**Success criteria:**
1. **SIMPLIFY-12 audit:** `find scripts/ -maxdepth 1 -type f | grep -v hooks | grep -v install-hooks.sh` returns empty, OR every remaining file has an explicit `quality/catalogs/orphan-scripts.json` waiver row with reason + dimension_owner + RFC3339 `until` date. `examples/*/run.sh` all have catalog rows. The `examples/` README documents that each example is a tracked gate (already landed in P59; this is the final assertion).
2. **MIGRATE-01:** Every source file flagged by SIMPLIFY-01..12 is either deleted, reduced to a one-line shim with a header-comment reason, or has an explicit waiver row. Two pre-push cycles' worth of `quality/reports/verdicts/pre-push/` show parity-or-better vs the v0.11.x baseline (no behaviour regression introduced by the migration).
3. **MIGRATE-02 cohesion pass:** CLAUDE.md gains a full dimension/cadence/kind taxonomy section (cross-referenced from `quality/PROTOCOL.md`) and the meta-rule extension ("when an owner catches a miss: fix the issue, update CLAUDE.md, AND tag the dimension"). All per-phase QG-07 updates land coherently (no orphan paragraphs, no contradictions). A subagent grep audit of CLAUDE.md against `quality/PROTOCOL.md` finds zero stale cross-refs.
4. **MIGRATE-03 v0.12.1 carry-forward:** stub catalog rows ship for perf-dimension (`quality/catalogs/perf-targets.json` with stubs for latency vs headline-copy + token-economy bench cross-check), security-dimension (`quality/catalogs/security-gates.json` with stubs for allowlist enforcement + audit-immutability), cross-platform container rehearsals (`quality/catalogs/cross-platform.json` with stubs for windows-2022 + macos-14). REQUIREMENTS.md (or a v0.12.1-phases file) is updated with placeholders: PERF-*, SEC-*, CROSS-*, and the `Error::Other` 156→144 completion item.
5. **CHANGELOG `[v0.12.0]` finalized:** summarizes Phases 56–63 + lists every shipped REQ-ID + names the v0.12.1 carry-forward.
6. **Tag gate authored:** a milestone-tag script is in place at `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` (or equivalent path) mirroring v0.11.x tag-script safety guards (≥6 guards: clean tree, on `main`, version match, CHANGELOG entry exists, tests green, signed tag). Owner runs the tag — orchestrator does NOT push the tag.
7. **SIMPLIFY absorption (P63 close-out):** every script flagged by SIMPLIFY-01..12 has an actioned status (folded / shimmed / waived); no orphan remainder.
8. **Recurring (catalog-first):** Catalog rows for MIGRATE-01..03 + SIMPLIFY-12 land in `quality/catalogs/{orphan-scripts,perf-targets,security-gates,cross-platform}.json` BEFORE the source-file deletions and the CLAUDE.md cohesion edit.
9. **Recurring (CLAUDE.md):** CLAUDE.md final cohesion pass landed (this IS MIGRATE-02 — the per-phase incremental updates from P56–P62 get audited and stitched here). In the same PR.
10. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN AND audits cross-phase coherence (every phase's verdict file exists and is GREEN, no orphan catalog rows, no expired waivers without follow-up); verdict in `quality/reports/verdicts/p63/<ts>.md`. The milestone-close verdict is the union of P56..P63 verdicts.
11. **Broaden-and-deepen close-out:** Every dimension's catalog has zero NOT-VERIFIED P0+P1 rows; every WAIVED row has a non-expired `until` (RFC3339) and a tracked carry-forward entry under MIGRATE-03 v0.12.1 placeholders; the milestone-close verifier subagent confirms cross-dimension coherence (no dimension shipped a gate without sweeping its first-run REDs per POLISH-* in REQUIREMENTS.md). Owner directive: "after this milestone the codebase is pristine and high quality across all the dimensions."

**Context anchor:** `.planning/REQUIREMENTS.md` § "Migration close-out" MIGRATE-01..03 + § "Aggressive simplification" SIMPLIFY-12 + § "Out of Scope" (the v0.12.1 carry-forward list), `.planning/research/v0.12.0-autonomous-execution-protocol.md` (parallel migration + hard-cut + cohesion-pass design), v0.11.x tag-script precedent at `.planning/milestones/v0.11.0-phases/tag-v0.11.0.sh` (template for v0.12.0 tag script).

---

## Previously planned milestones

Per CLAUDE.md §0.5 / Workspace layout, each shipped/historical milestone's
ROADMAP.md lives inside its `*-phases/` directory. Top-level ROADMAP.md
holds ONLY the active milestone (currently v0.12.0) + this index.

- **v0.11.0** Polish & Reproducibility — `.planning/milestones/v0.11.0-phases/ROADMAP.md` (Phases 50–55, SHIPPED 2026-04-25 → 2026-04-27).
- **v0.10.0** Docs & Narrative Shine — `.planning/milestones/v0.10.0-phases/ROADMAP.md` (Phases 40–45, SHIPPED 2026-04-25).
- **v0.9.0** Architecture Pivot — `.planning/milestones/v0.9.0-phases/ROADMAP.md` (Phases 31–36, SHIPPED 2026-04-24).
- v0.8.0 and earlier — see `.planning/milestones/v0.X.0-phases/ARCHIVE.md` per the POLISH2-21 condensation (8 archives, v0.1.0 → v0.8.0).

## Backlog

### Phase 999.1: Follow-up — missing SUMMARY.md files from prior phases (BACKLOG)

**Goal:** Resolve plans that ran without producing summaries during earlier phase executions
**Deferred at:** 2026-04-16 during /gsd-next advancement to /gsd-verify-work (Phase 29 → milestone completion)
**Plans:**
- [ ] Phase 16: 16-D-docs-and-release (ran, no SUMMARY.md)
- [ ] Phase 17: 17-A-workload-and-cli (ran, no SUMMARY.md)
- [ ] Phase 17: 17-B-tests-and-docs (ran, no SUMMARY.md)
- [ ] Phase 18: 18-02 (ran, no SUMMARY.md)
- [ ] Phase 21: 21-A-audit (ran, no SUMMARY.md)
- [ ] Phase 21: 21-B-contention (ran, no SUMMARY.md)
- [ ] Phase 21: 21-C-truncation (ran, no SUMMARY.md)
- [ ] Phase 21: 21-D-chaos (ran, no SUMMARY.md)
- [ ] Phase 21: 21-E-macos (ran, no SUMMARY.md)
- [ ] Phase 22: 22-A-bench-upgrade (ran, no SUMMARY.md)
- [ ] Phase 22: 22-B-fixtures-and-table (ran, no SUMMARY.md)
- [ ] Phase 22: 22-C-wire-docs-ship (ran, no SUMMARY.md)
- [ ] Phase 25: 25-02 (ran, no SUMMARY.md)
- [ ] Phase 27: 27-02 (ran, no SUMMARY.md)
