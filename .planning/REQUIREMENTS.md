# Requirements — Active milestone: v0.12.0 Quality Gates

**Active milestone:** v0.12.0 Quality Gates (planning_started 2026-04-27).

**Previously validated milestones — see per-milestone REQUIREMENTS.md (extracted 2026-04-27 to keep this file scoped to active work):**
- v0.11.x Polish & Reproducibility — `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md` (v0.11.0 SHIPPED 2026-04-25; v0.11.1 + v0.11.2 polish passes SHIPPED 2026-04-26 / 2026-04-27 via release-plz). All eight crates published to crates.io at v0.11.2. **Carry-forward NOT closed by v0.11.x:** the curl/PowerShell installer URLs broke on every release after v0.11.0 because `release.yml` tag glob `v*` does not match release-plz's per-crate `reposix-cli-v*` pattern. Diagnosed in `.planning/research/v0.12.0-install-regression-diagnosis.md`; fixed by RELEASE-01 in P56.
- v0.10.0 Docs & Narrative Shine — `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md` (SHIPPED 2026-04-25, Phases 40–45).
- v0.9.0 Architecture Pivot — `.planning/milestones/v0.9.0-phases/REQUIREMENTS.md` (SHIPPED 2026-04-24, Phases 31–36; ARCH-01..19 + the v1 cache/transport/sync requirements that were the v0.9.0 pivot detail).
- v0.8.0 and earlier — see `.planning/milestones/v0.X.0-phases/ARCHIVE.md` per the POLISH2-21 condensation (8 archives, v0.1.0 → v0.8.0).

> **Convention.** Per CLAUDE.md §0.5 / Workspace layout, each milestone's REQUIREMENTS.md lives inside its `*-phases/` directory once shipped. The top-level file holds ONLY the active milestone + this index. v0.12.0 STRUCT-* gates enforce this going forward.

---

## v0.12.0 Requirements — Quality Gates

**Milestone goal:** Replace ad-hoc quality scripts (`scripts/check-*.sh`, the conflated `scripts/end-state.py`, the not-in-CI `scripts/repro-quickstart.sh`) with a coherent **Quality Gates** system that prevents the silent regressions the v0.11.x cycle missed. The §0.8 SESSION-END-STATE framework caught what it was designed for (file-shape invariants, crates.io max_version) but missed the GitHub-release-asset drift that broke the curl-installer URL for two releases. v0.12.0 generalizes §0.8 into a dimension-tagged framework where every regression class has a home, every gate has a verifier, every verifier produces an artifact, and every artifact is graded by an unbiased subagent — not the executing agent's word.

**Mental model.** Each gate answers three orthogonal questions:
- **Dimension** (what is checked): code, docs-build, docs-repro, release, structure, perf, security, agent-ux.
- **Cadence** (when it runs): pre-push, pre-pr, weekly, pre-release, post-release, on-demand. Note: weekly cron not nightly — explicit owner decision (cost-conscious).
- **Kind** (how it's verified): mechanical, container, asset-exists, subagent-graded, manual.

Catalogs are the data; verifiers are the code; reports are the artifacts; runners compose by tag. Adding a new gate is one catalog row + one verifier — no bespoke script, no new pre-push wiring.

**Source-of-truth handover bundle (read these BEFORE planning P56):**
- `.planning/research/v0.12.0-vision-and-mental-model.md` — the dimension/cadence/kind taxonomy, why each is needed, the regression classes the v0.11.x framework missed.
- `.planning/research/v0.12.0-naming-and-architecture.md` — `quality/{gates,catalogs,reports,runners}/` layout; the unified catalog schema; runner/verdict design; what stays in `scripts/` vs migrates to `quality/`.
- `.planning/research/v0.12.0-roadmap-and-rationale.md` — phase-by-phase rationale, dependencies, pivot rules, blast-radius analysis.
- `.planning/research/v0.12.0-autonomous-execution-protocol.md` — `quality/PROTOCOL.md` design, catalog-first phase rule, waiver protocol with TTL, SURPRISES.md journal, mandatory verifier-subagent grading per phase close, **mandatory CLAUDE.md update per phase**.
- `.planning/research/v0.12.0-install-regression-diagnosis.md` — root cause + fix options for the curl-installer regression (RELEASE-01).
- `.planning/research/v0.12.0-decisions-log.md` — owner Q&A from the planning session, decisions taken with rationale.
- `.planning/research/v0.12.0-open-questions-and-deferrals.md` — what is explicitly NOT in v0.12.0 scope and why.
- `.planning/docs_reproducible_catalog.json` — DRAFT seed for the docs-repro catalog; consumed by P59.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**
- **Self-improving infrastructure (OP-4).** The Quality Gates system IS this principle made structural. Every owner-caught miss now has a routing rule: "fix the issue, update CLAUDE.md, AND tag the dimension" — the meta-rule extension that turns ad-hoc bash into a committed gate.
- **Close the feedback loop (OP-1).** Container-rehearsal at post-release cadence is OP-1 for install paths: don't trust that release.yml shipped the asset; fetch the URL from a fresh container and prove the binary lands on PATH.
- **Aggressive subagent delegation (OP-2).** Every phase close MUST dispatch an unbiased verifier subagent that grades the catalog rows against artifacts with zero session context.
- **Reversibility enables boldness (OP-5).** Parallel migration (old + new system run side-by-side) until the new system shows parity; only then hard-cut. Pivots are documented in `quality/SURPRISES.md`, not lost.
- **Ground truth obsession (OP-6).** Catalog-first phase rule: end-state assertions land in git BEFORE the implementation. The verifier knows what GREEN looks like before the code lands. No "agent's word for it."

### Active

#### Release dimension — close the immediate breakage

- [ ] **RELEASE-01**: Restore the curl/PowerShell installer URLs by fixing `release.yml` so it fires on release-plz's per-crate tags. Pick the cleaner of two options (extend `on.push.tags` glob to match `reposix-cli-v*` and key version off the cli tag, OR add a release-plz post-publish step that mirrors a workspace `vX.Y.Z` tag). Cut a fresh `reposix-cli-v0.11.3` release and verify all 5 install paths work end-to-end. **P0 — every documented install path is broken.**
- [ ] **RELEASE-02**: Homebrew tap formula auto-updates with each release (the `upload-homebrew-formula` job in `release.yml` is currently dead because the workflow doesn't fire). Verified by RELEASE-01's release cycle.
- [ ] **RELEASE-03**: `cargo binstall reposix-cli reposix-remote` resolves to a prebuilt binary (currently falls back to source build because no GH binary asset exists). Lifted by RELEASE-01.
- [ ] **RELEASE-04**: Quality Gates `release/` dimension — `quality/gates/release/{gh-assets-present.py, brew-formula-current.py, crates-io-max-version.py, installer-asset-bytes.py}` with weekly + post-release runners. Catalog rows for every install URL, brew formula, and crates.io crate. Would have caught RELEASE-01 within 24h of the regression.

#### Quality Gates framework

- [ ] **QG-01**: `quality/{gates,catalogs,reports,runners}/` directory layout created. `quality/catalogs/README.md` documents the unified catalog schema (every row carries `id`, `dimension`, `cadence`, `kind`, `sources`, `verifier`, `artifact`, `status`, `freshness_ttl`, `waiver`, `blast_radius`, `owner_hint`).
- [ ] **QG-02**: `quality/runners/run.py --cadence X` discovers all gates tagged X and runs them in order; `quality/runners/verdict.py` collates artifacts into `quality/reports/verdicts/<cadence>/<ts>.md` and exits non-zero on RED. Single entry point for pre-push, pre-pr, weekly, pre-release, post-release.
- [ ] **QG-03**: `quality/PROTOCOL.md` — single-page autonomous-mode runtime contract every phase agent reads at start. Contains the gate routing table, the catalog-first rule, pivot triggers, the waiver protocol with TTL, skill-dispatch patterns, "when stuck" rules, and anti-bloat rules per surface.
- [ ] **QG-04**: Waiver mechanism — every catalog row supports `waiver: {until: <RFC3339>, reason, dimension_owner}`. Expired waivers flip the row back to FAIL. Documented in PROTOCOL.md as the principled escape hatch when a phase agent must pivot rather than fix.
- [ ] **QG-05**: `quality/SURPRISES.md` — append-only journal. One line per unexpected obstacle + one line per resolution. Required reading for the next phase agent (so dead ends aren't repeated).
- [ ] **QG-06**: Mandatory verifier-subagent dispatch per phase close. No phase ships without an unbiased subagent grading the catalog rows GREEN. Pattern documented in PROTOCOL.md; same shape as the §0.8 verifier dispatch from v0.11.2.
- [ ] **QG-07**: **Mandatory CLAUDE.md update per phase** as part of definition-of-done. Each phase that introduces a new file, convention, gate, or operational rule MUST update the relevant CLAUDE.md section in the SAME PR. The verifier subagent grades this as a phase-close requirement. Anti-bloat: each phase appends a paragraph + code reference; deletions are encouraged when superseded. Owner-flagged in this planning session.
- [ ] **QG-08**: Top-level `.planning/REQUIREMENTS.md` MUST contain ONLY the active milestone + a "Previously validated" index pointing to per-milestone REQUIREMENTS.md files inside `*-phases/`. Same rule for `.planning/ROADMAP.md`. This catalog row in `quality/gates/structure/` enforces the convention going forward (currently unenforced; the convention is documented in CLAUDE.md §0.5 but historical sections drifted into the top-level file before this gate existed). Owner-flagged in this planning session.

#### Structure dimension — migrate freshness invariants

- [ ] **STRUCT-01**: Migrate the 6 freshness rows from `scripts/end-state.py` into `quality/gates/structure/`. Catalog rows live in `quality/catalogs/freshness-invariants.json`. Wire `quality/runners/run.py --cadence pre-push` as the new entry point.
- [ ] **STRUCT-02**: `scripts/end-state.py` reduced to a thin shim (≤ 30 lines) that delegates to `quality/runners/verdict.py session-end`. Anti-bloat header comment explicitly tells future agents: "this file does not grow; new gates go under `quality/gates/<dim>/`." Owner-flagged concern: agents will bloat this file if not warned.

#### Docs-repro dimension

- [ ] **DOCS-REPRO-01**: `quality/gates/docs-repro/snippet-extract.py` parses every fenced code block in user-facing docs (README, docs/index.md, docs/tutorials/*.md) and emits catalog rows. Drift detector: fail if a doc snippet has no catalog row, or a catalog row's content drifted from its source.
- [ ] **DOCS-REPRO-02**: `quality/gates/docs-repro/container-rehearse.sh <id>` spins ubuntu:24.04 (default), runs the snippet verbatim, asserts post-conditions. Per-persona matrix (linux first; mac/windows runners deferred to v0.12.1).
- [ ] **DOCS-REPRO-03**: Promote `scripts/repro-quickstart.sh` into `quality/gates/docs-repro/tutorial-replay.sh` as one container-rehearsal-kind row. Wire into post-release cadence.
- [ ] **DOCS-REPRO-04**: Catalog seed — port every row from the DRAFT `.planning/docs_reproducible_catalog.json` into `quality/catalogs/docs-reproducible.json` with the unified schema.

#### Docs-build dimension migration

- [ ] **DOCS-BUILD-01**: Move `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` into `quality/gates/docs-build/` with no behaviour change. Pre-push hook delegates to `quality/runners/run.py --cadence pre-push`. Leave shims at old paths if hooks would otherwise break.

#### Subjective gates

- [ ] **SUBJ-01**: `quality/catalogs/subjective-rubrics.json` with seed rubrics — `cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity`. Each row carries a numeric scoring rubric and a `freshness_ttl` (default 30d).
- [ ] **SUBJ-02**: `reposix-quality-review` skill — reads the catalog, dispatches one unbiased subagent per stale/unverified row in parallel, persists JSON artifacts to `quality/reports/verifications/`, updates the catalog. Integrates `doc-clarity-review` as one rubric implementation.
- [ ] **SUBJ-03**: Wire SUBJ-02 into pre-release cadence so subjective gates with TTL ≥ 14d expired auto-dispatch before any milestone tag push.

#### Repo-org cleanup

- [ ] **ORG-01**: Audit `.planning/research/v0.11.1-repo-organization-gaps.md` against current state. Each remaining gap → either fix + add a structure-dimension catalog row that prevents recurrence, OR file an explicit waiver with reason. Ensures the gaps document isn't a forgotten todo list.

#### Aggressive simplification — absorb existing surfaces into the framework

> **Owner directive (this planning session):** "look aggressively for opportunities to simplify if [existing scripts/examples] can be swallowed by this framework as by incorporating them into the various phases/plans of this milestone." The framework is NOT a new layer on top of the old — it REPLACES the ad-hoc surfaces. After v0.12.0, `scripts/` holds only `hooks/` and `install-hooks.sh`; everything else lives in `quality/gates/<dimension>/`.

Each SIMPLIFY-* item names an existing surface, its target home in the new framework, and the phase that absorbs it. The phase's plan MUST end with the source surface either deleted, reduced to a thin shim, or explicitly waived with a reason.

- [ ] **SIMPLIFY-01** (P57): `scripts/banned-words-lint.sh` → `quality/gates/structure/banned-words.sh` with a catalog row in `quality/catalogs/freshness-invariants.json`. Old path becomes a one-line shim or deleted (whichever the pre-push hook tolerates).
- [ ] **SIMPLIFY-02** (P57): `scripts/end-state.py` reduced to ≤30-line shim per STRUCT-02. The 6 existing freshness rows and the crates.io rows migrate to `quality/gates/structure/` and `quality/gates/release/` respectively.
- [ ] **SIMPLIFY-03** (P57): `scripts/catalog.py` audited — does its per-file rendering job overlap with `quality/runners/verdict.py`? If yes, fold and delete; if no (different domain), document the boundary in `quality/catalogs/README.md` and leave alone.
- [ ] **SIMPLIFY-04** (P58): `scripts/check_clippy_lint_loaded.sh` → `quality/gates/code/clippy-lint-loaded.sh`. Catalog row records the expected lint set so silent lint removal trips the gate.
- [ ] **SIMPLIFY-05** (P58): `scripts/check_fixtures.py` → audit if it's a code-dimension gate or belongs as a `crates/<crate>/tests/` integration test. Move accordingly; do not leave at top of `scripts/`.
- [ ] **SIMPLIFY-06** (P59): `scripts/repro-quickstart.sh` → `quality/gates/docs-repro/tutorial-replay.sh` per DOCS-REPRO-03. **Plus:** every `examples/0[1-5]-*/run.sh` becomes a docs-repro catalog row (container-rehearsal-kind, post-release cadence). The `examples/` README gets a callout that each example is now a tracked gate.
- [ ] **SIMPLIFY-07** (P59): `scripts/dark-factory-test.sh` → `quality/gates/agent-ux/dark-factory.sh`. The `agent-ux` dimension is a thin home; if that's the only gate in the dimension at v0.12.0 close, document the dimension as "intentionally sparse — perf and security stubs land in v0.12.1."
- [ ] **SIMPLIFY-08** (P60): `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` → `quality/gates/docs-build/` per DOCS-BUILD-01.
- [ ] **SIMPLIFY-09** (P60): `scripts/green-gauntlet.sh` (composite "run everything" wrapper) is supplanted by `quality/runners/run.py --cadence pre-pr`. Delete or reduce to a one-line shim that calls the runner.
- [ ] **SIMPLIFY-10** (P60): `scripts/hooks/pre-push` body simplified — current logic chains multiple checks; new body is a single `quality/runners/run.py --cadence pre-push` invocation. `scripts/hooks/test-pre-push.sh` updated to test the new entry point. `scripts/install-hooks.sh` kept as-is (developer install of git hooks).
- [ ] **SIMPLIFY-11** (P59 stub, full v0.12.1): `scripts/bench_token_economy.py`, `scripts/test_bench_token_economy.py`, `scripts/latency-bench.sh`, `benchmarks/fixtures/*` → `quality/gates/perf/` with a perf-targets catalog. **v0.12.0 ships the move (file relocations + thin shims at old paths if anything imports them); the actual gate logic that cross-checks bench output against headline copy is the v0.12.1 stub per MIGRATE-03.**
- [ ] **SIMPLIFY-12** (P63): Final audit — `find scripts/ -maxdepth 1 -type f | grep -v hooks | grep -v install-hooks.sh` returns empty (or every remaining file has an explicit `quality/catalogs/orphan-scripts.json` waiver row with reason). `examples/*/run.sh` all have catalog rows. The `examples/` README documents that each example is a tracked gate.

#### Migration close-out

- [ ] **MIGRATE-01**: After SIMPLIFY-01..12 complete and the new system shows parity in `quality/reports/verdicts/` for two pre-push cycles, delete the migrated source files. Anything kept as a thin shim documents the reason in a header comment.
- [ ] **MIGRATE-02**: CLAUDE.md final pass — full dimension/cadence/kind taxonomy section + meta-rule extension ("when an owner catches a miss: fix the issue, update CLAUDE.md, AND tag the dimension"). Cross-references all `quality/PROTOCOL.md` sections. **Note:** per QG-07 each phase already updated CLAUDE.md incrementally; MIGRATE-02 is the final cohesion pass + cross-link audit.
- [ ] **MIGRATE-03**: File v0.12.1 carry-forward — perf-dimension full implementation (SIMPLIFY-11 stubs become real gates; latency vs headline-copy cross-check, token-economy bench cross-check) and security-dimension (allowlist enforcement gate, audit immutability test) as stub catalog rows + REQUIREMENTS.md placeholders. Cross-platform container rehearsals (windows-2022, macos-14) also stubbed. **Plus:** complete the `Error::Other` 156→144 partial migration (POLISH2-09 carry-forward from v0.11.1).

### Out of Scope

- **Perf-dimension full implementation** (latency vs headline-copy cross-check, token-economy bench cross-check). Stubbed in MIGRATE-03; full ship deferred to v0.12.1.
- **Security-dimension full implementation** (allowlist-enforcement gate, audit-immutability test). Stubbed in MIGRATE-03; full ship deferred to v0.12.1.
- **Cross-platform container rehearsals** (windows-2022, macos-14 runners). v0.12.0 ships ubuntu-only matrix; cross-platform deferred to v0.12.1 because windows/mac GH runners are real money on every release.
- **`Error::Other` 156→144 partial migration completion** (POLISH2-09 carry-forward from v0.11.1). Stubbed under MIGRATE-03 v0.12.1 carry-forward.
- **Threat-model rewrite for v0.9.0 architecture** (friction row 20 from v0.11.1). Deferred to a separate security-focused milestone.

### Traceability

Refined 1:1 mapping after roadmap creation (gsd-roadmapper, 2026-04-27). Coverage = 38/38 requirements ✓; no orphans, no duplicates. See `.planning/ROADMAP.md` `## v0.12.0 Quality Gates (PLANNING)` for full phase entries with goal / requirements / depends-on (gate-state preconditions) / success criteria / context anchor.

| REQ-ID | Phase | Status |
|--------|-------|--------|
| RELEASE-01 | P56 | planning |
| RELEASE-02 | P56 | planning |
| RELEASE-03 | P56 | planning |
| RELEASE-04 | P58 | planning |
| QG-01 | P57 | planning |
| QG-02 | P57 | planning |
| QG-03 | P57 | planning |
| QG-04 | P57 | planning |
| QG-05 | P57 | planning |
| QG-06 | P57 | planning |
| QG-07 | P57 | planning |
| QG-08 | P57 | planning |
| STRUCT-01 | P57 | planning |
| STRUCT-02 | P57 | planning |
| DOCS-REPRO-01 | P59 | planning |
| DOCS-REPRO-02 | P59 | planning |
| DOCS-REPRO-03 | P59 | planning |
| DOCS-REPRO-04 | P59 | planning |
| DOCS-BUILD-01 | P60 | planning |
| SUBJ-01 | P61 | planning |
| SUBJ-02 | P61 | planning |
| SUBJ-03 | P61 | planning |
| ORG-01 | P62 | planning |
| SIMPLIFY-01 | P57 | planning |
| SIMPLIFY-02 | P57 | planning |
| SIMPLIFY-03 | P57 | planning |
| SIMPLIFY-04 | P58 | planning |
| SIMPLIFY-05 | P58 | planning |
| SIMPLIFY-06 | P59 | planning |
| SIMPLIFY-07 | P59 | planning |
| SIMPLIFY-08 | P60 | planning |
| SIMPLIFY-09 | P60 | planning |
| SIMPLIFY-10 | P60 | planning |
| SIMPLIFY-11 | P59 (relocate) + v0.12.1 (cross-check stub per MIGRATE-03) | planning |
| SIMPLIFY-12 | P63 | planning |
| MIGRATE-01 | P63 | planning |
| MIGRATE-02 | P63 | planning |
| MIGRATE-03 | P63 | planning |

**Per-phase requirement counts:** P56=3 (RELEASE-01..03) · P57=13 (QG-01..08, STRUCT-01..02, SIMPLIFY-01..03) · P58=3 (RELEASE-04, SIMPLIFY-04..05) · P59=7 (DOCS-REPRO-01..04, SIMPLIFY-06..07, SIMPLIFY-11) · P60=4 (DOCS-BUILD-01, SIMPLIFY-08..10) · P61=3 (SUBJ-01..03) · P62=1 (ORG-01) · P63=4 (MIGRATE-01..03, SIMPLIFY-12). Sum = 38 ✓.

**Recurring success criteria across every phase (P56–P63)** — these are part of the phase's definition-of-done and are NOT separate REQ-IDs (they are recurring expressions of QG-06 + QG-07 + the autonomous-execution protocol):
- Catalog-first: phase's first commit writes catalog rows BEFORE implementation.
- CLAUDE.md update in the same PR (QG-07).
- Unbiased verifier-subagent dispatch on phase close (QG-06).
- SIMPLIFY absorption (where applicable): every script/example in scope is folded into `quality/gates/<dim>/`, reduced to a one-line shim, or has a waiver row in `quality/catalogs/orphan-scripts.json` with reason.
