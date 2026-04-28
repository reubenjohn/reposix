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

- [x] **RELEASE-01**: Restore the curl/PowerShell installer URLs by fixing `release.yml` so it fires on release-plz's per-crate tags. Pick the cleaner of two options (extend `on.push.tags` glob to match `reposix-cli-v*` and key version off the cli tag, OR add a release-plz post-publish step that mirrors a workspace `vX.Y.Z` tag). Cut a fresh `reposix-cli-v0.11.3` release and verify all 5 install paths work end-to-end. **P0 — every documented install path is broken.**
- [x] **RELEASE-02**: Homebrew tap formula auto-updates with each release (the `upload-homebrew-formula` job in `release.yml` is currently dead because the workflow doesn't fire). Verified by RELEASE-01's release cycle.
- [x] **RELEASE-03**: `cargo binstall reposix-cli reposix-remote` resolves to a prebuilt binary (currently falls back to source build because no GH binary asset exists). Lifted by RELEASE-01.
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
- [ ] **QG-09**: Quality Gates summary badge — `quality/runners/verdict.py` emits `quality/reports/badge.json` in [shields.io endpoint format](https://shields.io/badges/endpoint-badge): `{"schemaVersion": 1, "label": "quality gates", "message": "<N>/<M> GREEN", "color": <green|yellow|red>}`. Color thresholds: green if all P0+P1 PASS or WAIVED; yellow if any P2 RED; red if any P0+P1 RED. **P57 ships:** the verdict.py JSON emit. **P60 ships:** mkdocs publishes it as `docs/badge.json` → `https://reubenjohn.github.io/reposix/badge.json`; README + docs/index.md add the badge: `![Quality](https://img.shields.io/endpoint?url=https://reubenjohn.github.io/reposix/badge.json)`. **Plus** the cheaper standard badge `![Quality (weekly)](https://github.com/reubenjohn/reposix/actions/workflows/quality-weekly.yml/badge.svg)` lands in P58 alongside the workflow. Owner-flagged in this planning session.

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
- [ ] **BADGE-01**: Validate every README + docs-page badge URL renders. New gate `quality/gates/docs-build/badges-resolve.py` HEADs each badge URL and asserts HTTP 200 + content-type contains `image`. Catalog row per badge in `quality/catalogs/freshness-invariants.json` (or a new `quality/catalogs/badges.json` if the count grows). Catches: shields.io drift, codecov project rename, badge-URL typos, broken endpoint URLs. Cadence: weekly + pre-push (cheap HEAD; ~1s for all 6 badges). Includes the new QG-09 endpoint badge URL once published.

#### Subjective gates

- [ ] **SUBJ-01**: `quality/catalogs/subjective-rubrics.json` with seed rubrics — `cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity`. Each row carries a numeric scoring rubric and a `freshness_ttl` (default 30d).
- [ ] **SUBJ-02**: `reposix-quality-review` skill — reads the catalog, dispatches one unbiased subagent per stale/unverified row in parallel, persists JSON artifacts to `quality/reports/verifications/`, updates the catalog. Integrates `doc-clarity-review` as one rubric implementation.
- [ ] **SUBJ-03**: Wire SUBJ-02 into pre-release cadence so subjective gates with TTL ≥ 14d expired auto-dispatch before any milestone tag push.

#### Repo-org cleanup

- [x] **ORG-01**: Audit `.planning/research/v0.11.1-repo-organization-gaps.md` against current state. Each remaining gap → either fix + add a structure-dimension catalog row that prevents recurrence, OR file an explicit waiver with reason. Ensures the gaps document isn't a forgotten todo list. **SHIPPED P62.** Audit at `quality/reports/audits/repo-org-gaps.md` (99 items; 13 closed-by-deletion, 26 closed-by-relocation, 50 closed-by-existing-gate, 8 out-of-scope; zero open Wave-3 items). 3 new structure-dimension rows in `quality/catalogs/freshness-invariants.json` lock recurrence guards.

#### Polish passes (per-dimension RED-fix sweeps)

> **Owner directive (this planning session):** "I'm really hoping that after this milestone the codebase is pristine and high quality across all the dimensions." The POLISH-* items below are the broaden-and-deepen pass: every dimension that ships a gate in v0.12.0 ALSO ships a sweep that fixes the RED rows the gate's first run flags. The milestone is not about instrumenting the codebase — it is about leaving the codebase pristine. Each POLISH-* row is P0/P1 blast radius and gates milestone close. Anything that cannot be fixed in-phase is WAIVED (with TTL ≤ 90d + dimension_owner) or filed as a v0.12.1 carry-forward via MIGRATE-03.

- [ ] **POLISH-STRUCT** (P57): After structure-dim gates ship, audit every freshness invariant and fix any drift. Specifically: confirm no version-pinned filenames outside `CHANGELOG.md` and `*-phases/`; confirm install path leads with package manager (cargo binstall / brew install BEFORE any clone+build snippet) on `README.md` + `docs/index.md`; confirm benchmarks under `benchmarks/` and `docs/benchmarks/` appear in `mkdocs.yml` `nav:`; confirm no loose `*ROADMAP*.md` or `*REQUIREMENTS*.md` at `.planning/milestones/` top-level. Fix any flagged drift in the same phase (cite commit).
- [ ] **POLISH-RELEASE** (P58): After release-dim gates ship, audit and fix any release-asset drift not already covered by P56. Specifically: every install URL in user-facing docs HEADs to HTTP 200; brew formula version is current with the latest reposix-cli release; `crates.io` `max_version` per published crate matches the latest tag; `cargo binstall` metadata resolves to a prebuilt binary for every published binary crate (no source-fallback). Fix any drift in the same phase.
- [ ] **POLISH-DOCS-REPRO** (P59): After docs-repro gates ship, every fenced code block in user-facing docs (`README.md`, `docs/index.md`, `docs/tutorials/*`, `docs/guides/*`) has a catalog row AND a passing container rehearsal OR is explicitly marked manual/illustrative (with rationale in the catalog row). Fix any broken/stale snippets in the same phase (cite commit). Every `examples/0[1-5]-*/run.sh` passes its container rehearsal.
- [ ] **POLISH-DOCS-BUILD** (P60): After docs-build gates ship, every badge URL in `README.md` + docs renders (BADGE-01 fix-the-REDs); every link in user-facing docs resolves (no link rot); `mkdocs build --strict` is GREEN; every mermaid block on every nav page renders without errors (assertion: `document.querySelectorAll('pre.mermaid svg').length > 0` per page, zero browser-console rendering errors). Fix any flagged failures in the same phase.
- [ ] **POLISH-SUBJECTIVE** (P61): After subjective rubrics seed (SUBJ-01..03), dispatch the unbiased subagent for `cold-reader-hero-clarity`, `install-positioning`, and `headline-numbers-sanity` AT LEAST ONCE; fix any P0/P1 findings in the same phase; remaining P2 findings either fixed, waived (with TTL), or filed as v0.12.1 carry-forward.
- [x] **POLISH-ORG** (P62): Every gap in `.planning/research/v0.11.1-repo-organization-gaps.md` gets a status (`closed-by-catalog-row`, `closed-by-existing-gate`, or `waived` with reason + dimension_owner + RFC3339 `until`). This is already P62's scope per ORG-01 — listed here for cohesion across the polish-pass family. **SHIPPED P62.** All 99 audit items have explicit dispositions; zero `closed-by-Wave-3-fix` items remain open. Fixes shipped in commits `eaf7068` (Wave 1 catalog), `4584fca` (Wave 2 audit), `8842d48` (Wave 3 relocations), `9011e91` (Wave 3 verifier extension), `2413f13` (Wave 4 SURPRISES rotation).
- [ ] **POLISH-AGENT-UX** (P59): The `dark-factory-test.sh` migration to `quality/gates/agent-ux/dark-factory.sh` runs end-to-end against the simulator; any regressions found vs. the v0.9.0 baseline are fixed in the same phase (cite commit).
- [x] **POLISH-CODE** (P58 stub, P63 final): **SHIPPED P63.** `cargo clippy --workspace --all-targets -- -D warnings` passes (code/clippy-lint-loaded + code/cargo-clippy-warnings PASS); cargo fmt verified via direct invocation (commit 16c4cbb -- code/cargo-fmt-clean wired to quality/gates/code/cargo-fmt-clean.sh, status NOT-VERIFIED -> PASS, read-only ~5s honoring CLAUDE.md ONE cargo at a time rule). code/cargo-test-pass intentionally remains as ci-job-status canonical wrapper per CLAUDE.md memory-budget rule (cargo nextest workspace 6-15 min violates ONE cargo at a time + pre-pr 10-min cadence cap); CI is the canonical enforcement venue, tracked-forward to v0.12.1 MIGRATE-03 for per-row local cargo enforcement alternatives. The Error::Other 156->144 migration completion is filed as v0.12.1 ERR-OTHER-01 per MIGRATE-03; no NEW Error::Other(String) sites introduced in v0.12.0.

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
- [x] **SIMPLIFY-12** (P63): **SHIPPED P63** (commit 4950cdd). 22 scripts in audit set; 5 DELETE (`_patch_plan_block.py`, `check-p57-catalog-contract.py`, `check_crates_io_max_version_sweep.sh`, `check_install_rows_catalog.py`, `test-runner-invariants.py`) + 13 SHIM-WAIVED + 4 KEEP-AS-CANONICAL. Per-script audit at `quality/reports/audits/scripts-retirement-p63.md` (caller-scan + decision + rationale). Surviving scripts have rows in `quality/catalogs/orphan-scripts.json` (17 rows asserting shim-shape contract); verifier at `quality/gates/structure/orphan-scripts-audit.py` mechanizes re-grading.

#### Migration close-out

- [x] **MIGRATE-01**: **SHIPPED P63** (commit 4950cdd). 5 source-file deletions of P57-superseded helpers; 13 shim-waivers anchored at `quality/catalogs/orphan-scripts.json`; shim-shape contract enforced by `quality/gates/structure/orphan-scripts-audit.py` (17/17 rows PASS at commit time). KEEP-AS-CANONICAL scripts gained header `# KEEP-AS-CANONICAL (P63 SIMPLIFY-12)` markers documenting the no-canonical-home rationale.
- [x] **MIGRATE-02**: **SHIPPED P63** (commit 1c316e7). Cross-link audit verifier at `quality/gates/structure/cross-link-audit.py` walks CLAUDE.md + `quality/PROTOCOL.md` + 8 dim READMEs, asserts every relative path mention exists. 100 paths verified, 0 stale. Per-dim READMEs normalized to verifier-table + conventions only (runtime detail cross-linked to PROTOCOL.md). New `quality/gates/security/README.md` fills missing security home. CLAUDE.md gained P63 H3 subsection summarizing SIMPLIFY-12 + POLISH-CODE final + v0.12.1 carry-forward + meta-rule extension.
- [x] **MIGRATE-03**: **SHIPPED P63** (commit 9c13843). File v0.12.1 carry-forward — perf-dimension full implementation (SIMPLIFY-11 stubs become real gates; latency vs headline-copy cross-check, token-economy bench cross-check) and security-dimension (allowlist enforcement gate, audit immutability test) as stub catalog rows + REQUIREMENTS.md placeholders. Cross-platform container rehearsals (windows-2022, macos-14) also stubbed. **Plus:** complete the `Error::Other` 156→144 partial migration (POLISH2-09 carry-forward from v0.11.1). **Plus (added 2026-04-27 from P56 Wave 4):** (a) `gh release create --latest` (or release-plz config) to pin the `releases/latest/download/...` pointer to the cli release after every per-crate release sequence — without it, a non-cli per-crate release published after the cli release moves the pointer and re-breaks the curl URL; (b) release-plz workflow uses fine-grained PAT (or adds a post-tag `gh workflow run` step) so GITHUB_TOKEN-pushed tags trigger `release.yml` instead of being silently dropped by the GH loop-prevention rule; (c) `[package.metadata.binstall]` blocks in `crates/reposix-cli/Cargo.toml` and `crates/reposix-remote/Cargo.toml` rewritten to match the actual release.yml archive shape (`reposix-cli-v${version}` tag prefix, `reposix-v${version}-${target}.tar.gz` archive basename, x86_64-unknown-linux-musl + aarch64-unknown-linux-musl target overrides) — ~10 LOC, lifts install/cargo-binstall PARTIAL → PASS; (d) Rust MSRV bump 1.82 → 1.85 (or cap transitive `block-buffer` at `<0.12`) so `cargo install reposix-cli` from crates.io works against the project MSRV — currently broken because block-buffer-0.12.0 requires edition2024. **Plus (added 2026-04-28 from P61 Wave G):** (e) Subjective dispatch-and-preserve runner invariant -- the runner's run_row currently overwrites `quality/reports/verifications/subjective/<id>.json` on every cadence sweep (waiver branch writes a WAIVED-shape stub; subprocess branch writes a Path-B-stub). The Path A scored verdict produced from a Claude session is therefore not durable across runner sweeps. Fix path: extend run_row so that a row with `kind=subagent-graded` AND a recent artifact whose `dispatched_via` starts with `Wave-G-Path-A` or `Path-A` is treated as authoritative -- the runner reads score + verdict from the artifact, sets row.status from the score-vs-threshold mapping, and does NOT overwrite the artifact. (f) Auto-dispatch from CI (would require Anthropic API auth on GH Actions runners) -- left as plain v0.12.1 follow-up (deferred per `.planning/research/v0.12.0-open-questions-and-deferrals.md` line 124). (g) Hard-gate chaining for release.yml waiting on quality-pre-release.yml verdict -- composite workflow OR workflow_run trigger; v0.12.0 ships parallel-execution soft-gate per P56 SURPRISES row 1 GH Actions cross-workflow chaining limitation.

### Out of Scope

- **Perf-dimension full implementation** (latency vs headline-copy cross-check, token-economy bench cross-check). Stubbed in MIGRATE-03; full ship deferred to v0.12.1.
- **Security-dimension full implementation** (allowlist-enforcement gate, audit-immutability test). Stubbed in MIGRATE-03; full ship deferred to v0.12.1.
- **Cross-platform container rehearsals** (windows-2022, macos-14 runners). v0.12.0 ships ubuntu-only matrix; cross-platform deferred to v0.12.1 because windows/mac GH runners are real money on every release.
- **`Error::Other` 156→144 partial migration completion** (POLISH2-09 carry-forward from v0.11.1). Stubbed under MIGRATE-03 v0.12.1 carry-forward.
- **Threat-model rewrite for v0.9.0 architecture** (friction row 20 from v0.11.1). Deferred to a separate security-focused milestone.

### Traceability

Refined 1:1 mapping after roadmap creation (gsd-roadmapper, 2026-04-27). Coverage = 46/46 requirements ✓ (38 original + 8 POLISH-* per the broaden-and-deepen directive added 2026-04-27); no orphans, no duplicates. See `.planning/ROADMAP.md` `## v0.12.0 Quality Gates (PLANNING)` for full phase entries with goal / requirements / depends-on (gate-state preconditions) / success criteria / context anchor.

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
| QG-09 | P57 (verdict.py emit) + P58 (GH Actions badge) + P60 (mkdocs publish + README badge) | planning (P57 + P58 + P60 portions shipped; row closes at v0.12.0 milestone end / P63) |
| STRUCT-01 | P57 | planning |
| STRUCT-02 | P57 | planning |
| DOCS-REPRO-01 | P59 | planning |
| DOCS-REPRO-02 | P59 | planning |
| DOCS-REPRO-03 | P59 | planning |
| DOCS-REPRO-04 | P59 | planning |
| DOCS-BUILD-01 | P60 | shipped (P60) |
| BADGE-01 | P60 | shipped (P60) |
| SUBJ-01 | P61 | shipped (P61) |
| SUBJ-02 | P61 | shipped (P61) |
| SUBJ-03 | P61 | shipped (P61) |
| ORG-01 | P62 | shipped (P62) |
| POLISH-STRUCT | P57 | planning |
| POLISH-RELEASE | P58 | planning |
| POLISH-DOCS-REPRO | P59 | planning |
| POLISH-DOCS-BUILD | P60 | shipped (P60) |
| POLISH-SUBJECTIVE | P61 | shipped (P61) |
| POLISH-ORG | P62 | shipped (P62) |
| POLISH-AGENT-UX | P59 | planning |
| POLISH-CODE | P58 (stub) + P63 (final) | planning |
| SIMPLIFY-01 | P57 | planning |
| SIMPLIFY-02 | P57 | planning |
| SIMPLIFY-03 | P57 | planning |
| SIMPLIFY-04 | P58 | planning |
| SIMPLIFY-05 | P58 | planning |
| SIMPLIFY-06 | P59 | planning |
| SIMPLIFY-07 | P59 | planning |
| SIMPLIFY-08 | P60 | shipped (P60) |
| SIMPLIFY-09 | P60 | shipped (P60) |
| SIMPLIFY-10 | P60 | shipped (P60) |
| SIMPLIFY-11 | P59 (relocate) + v0.12.1 (cross-check stub per MIGRATE-03) | planning |
| SIMPLIFY-12 | P63 | planning |
| MIGRATE-01 | P63 | planning |
| MIGRATE-02 | P63 | planning |
| MIGRATE-03 | P63 | planning |

**Per-phase requirement counts:** P56=3 (RELEASE-01..03) · P57=15 (QG-01..09, STRUCT-01..02, SIMPLIFY-01..03, POLISH-STRUCT) · P58=5 (RELEASE-04, SIMPLIFY-04..05, POLISH-RELEASE, POLISH-CODE-stub) · P59=9 (DOCS-REPRO-01..04, SIMPLIFY-06..07, SIMPLIFY-11, POLISH-DOCS-REPRO, POLISH-AGENT-UX) · P60=6 all SHIPPED (DOCS-BUILD-01, BADGE-01, SIMPLIFY-08..10, POLISH-DOCS-BUILD) · P61=4 all SHIPPED (SUBJ-01, SUBJ-02, SUBJ-03, POLISH-SUBJECTIVE) · P62=2 (ORG-01, POLISH-ORG) · P63=5 (MIGRATE-01..03, SIMPLIFY-12, POLISH-CODE-final). Sum = 49 (40 original + 8 POLISH-* + POLISH-CODE counted in BOTH P58-stub and P63-final per its dual home) ✓. (QG-09 spans P57+P58+P60; counted in P57 as primary owner. POLISH-CODE spans P58+P63; counted in both.)

**Recurring success criteria across every phase (P56–P63)** — these are part of the phase's definition-of-done and are NOT separate REQ-IDs (they are recurring expressions of QG-06 + QG-07 + the autonomous-execution protocol):
- Catalog-first: phase's first commit writes catalog rows BEFORE implementation.
- CLAUDE.md update in the same PR (QG-07).
- Unbiased verifier-subagent dispatch on phase close (QG-06).
- SIMPLIFY absorption (where applicable): every script/example in scope is folded into `quality/gates/<dim>/`, reduced to a one-line shim, or has a waiver row in `quality/catalogs/orphan-scripts.json` with reason.
- **Fix every RED row the dimension's gates flag.** When a phase ships a new gate, the gate's first run almost always finds NOT-VERIFIED or FAIL rows. Those rows MUST be either (a) FIXED in the same phase (cite commit), (b) WAIVED with explicit `until` + `reason` + `dimension_owner` per the waiver protocol (capped at 90 days), or (c) filed as a v0.12.1 carry-forward via MIGRATE-03. **The milestone does NOT close on NOT-VERIFIED P0+P1 rows.** Goal: after v0.12.0 closes, every dimension's catalog is all-GREEN-or-WAIVED. Owner directive: "I'm really hoping that after this milestone the codebase is pristine and high quality across all the dimensions."
