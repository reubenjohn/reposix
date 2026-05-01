# v0.12.0 Requirements — Quality Gates (SHIPPED 2026-04-28)

**Milestone goal:** Replace ad-hoc quality scripts (`scripts/check-*.sh`, the conflated `scripts/end-state.py`, the not-in-CI `scripts/repro-quickstart.sh`) with a coherent **Quality Gates** system that prevents the silent regressions the v0.11.x cycle missed. The §0.8 SESSION-END-STATE framework caught what it was designed for (file-shape invariants, crates.io max_version) but missed the GitHub-release-asset drift that broke the curl-installer URL for two releases. v0.12.0 generalizes §0.8 into a dimension-tagged framework where every regression class has a home, every gate has a verifier, every verifier produces an artifact, and every artifact is graded by an unbiased subagent — not the executing agent's word.

**Mental model.** Each gate answers three orthogonal questions:
- **Dimension** (what is checked): code, docs-build, docs-repro, release, structure, perf, security, agent-ux.
- **Cadence** (when it runs): pre-push, pre-pr, weekly, pre-release, post-release, on-demand. Note: weekly cron not nightly — explicit owner decision (cost-conscious).
- **Kind** (how it's verified): mechanical, container, asset-exists, subagent-graded, manual.

Catalogs are the data; verifiers are the code; reports are the artifacts; runners compose by tag. Adding a new gate is one catalog row + one verifier — no bespoke script, no new pre-push wiring.

**Source-of-truth handover bundle (consumed during v0.12.0 planning):**
- `.planning/research/v0.12.0/vision-and-mental-model.md` — the dimension/cadence/kind taxonomy, why each is needed, the regression classes the v0.11.x framework missed.
- `.planning/research/v0.12.0/naming-and-architecture.md` — `quality/{gates,catalogs,reports,runners}/` layout; the unified catalog schema; runner/verdict design; what stays in `scripts/` vs migrates to `quality/`.
- `.planning/research/v0.12.0/roadmap-and-rationale.md` — phase-by-phase rationale, dependencies, pivot rules, blast-radius analysis.
- `.planning/research/v0.12.0/autonomous-execution-protocol.md` — `quality/PROTOCOL.md` design, catalog-first phase rule, waiver protocol with TTL, SURPRISES.md journal, mandatory verifier-subagent grading per phase close, **mandatory CLAUDE.md update per phase**.
- `.planning/research/v0.12.0/install-regression-diagnosis.md` — root cause + fix options for the curl-installer regression (RELEASE-01).
- `.planning/research/v0.12.0/decisions-log.md` — owner Q&A from the planning session, decisions taken with rationale.
- `.planning/research/v0.12.0/open-questions-and-deferrals.md` — what is explicitly NOT in v0.12.0 scope and why.
- `.planning/docs_reproducible_catalog.json` — DRAFT seed for the docs-repro catalog; consumed by P59.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**
- **Self-improving infrastructure (OP-4).** The Quality Gates system IS this principle made structural. Every owner-caught miss now has a routing rule: "fix the issue, update CLAUDE.md, AND tag the dimension" — the meta-rule extension that turns ad-hoc bash into a committed gate.
- **Close the feedback loop (OP-1).** Container-rehearsal at post-release cadence is OP-1 for install paths: don't trust that release.yml shipped the asset; fetch the URL from a fresh container and prove the binary lands on PATH.
- **Aggressive subagent delegation (OP-2).** Every phase close MUST dispatch an unbiased verifier subagent that grades the catalog rows against artifacts with zero session context.
- **Reversibility enables boldness (OP-5).** Parallel migration (old + new system run side-by-side) until the new system shows parity; only then hard-cut. Pivots are documented in `quality/SURPRISES.md`, not lost.
- **Ground truth obsession (OP-6).** Catalog-first phase rule: end-state assertions land in git BEFORE the implementation. The verifier knows what GREEN looks like before the code lands. No "agent's word for it."

## Active

### Release dimension — close the immediate breakage

- [x] **RELEASE-01**: Restore the curl/PowerShell installer URLs by fixing `release.yml` so it fires on release-plz's per-crate tags. Pick the cleaner of two options (extend `on.push.tags` glob to match `reposix-cli-v*` and key version off the cli tag, OR add a release-plz post-publish step that mirrors a workspace `vX.Y.Z` tag). Cut a fresh `reposix-cli-v0.11.3` release and verify all 5 install paths work end-to-end. **P0 — every documented install path is broken.**
- [x] **RELEASE-02**: Homebrew tap formula auto-updates with each release (the `upload-homebrew-formula` job in `release.yml` is currently dead because the workflow doesn't fire). Verified by RELEASE-01's release cycle.
- [x] **RELEASE-03**: `cargo binstall reposix-cli reposix-remote` resolves to a prebuilt binary (currently falls back to source build because no GH binary asset exists). Lifted by RELEASE-01.
- [x] **RELEASE-04** (shipped P58, 2026-04-28): Quality Gates `release/` dimension — `quality/gates/release/{gh-assets-present.py, brew-formula-current.py, crates-io-max-version.py, installer-asset-bytes.py}` with weekly + post-release runners. Catalog rows for every install URL, brew formula, and crates.io crate. Would have caught RELEASE-01 within 24h of the regression.

### Quality Gates framework

- [x] **QG-01** (shipped P57, 2026-04-28): `quality/{gates,catalogs,reports,runners}/` directory layout created. `quality/catalogs/README.md` documents the unified catalog schema (every row carries `id`, `dimension`, `cadence`, `kind`, `sources`, `verifier`, `artifact`, `status`, `freshness_ttl`, `waiver`, `blast_radius`, `owner_hint`).
- [x] **QG-02** (shipped P57, 2026-04-28): `quality/runners/run.py --cadence X` discovers all gates tagged X and runs them in order; `quality/runners/verdict.py` collates artifacts into `quality/reports/verdicts/<cadence>/<ts>.md` and exits non-zero on RED. Single entry point for pre-push, pre-pr, weekly, pre-release, post-release.
- [x] **QG-03** (shipped P57, 2026-04-28): `quality/PROTOCOL.md` — single-page autonomous-mode runtime contract every phase agent reads at start. Contains the gate routing table, the catalog-first rule, pivot triggers, the waiver protocol with TTL, skill-dispatch patterns, "when stuck" rules, and anti-bloat rules per surface.
- [x] **QG-04** (shipped P57, 2026-04-28): Waiver mechanism — every catalog row supports `waiver: {until: <RFC3339>, reason, dimension_owner}`. Expired waivers flip the row back to FAIL. Documented in PROTOCOL.md as the principled escape hatch when a phase agent must pivot rather than fix.
- [x] **QG-05** (shipped P57, 2026-04-28): `quality/SURPRISES.md` — append-only journal. One line per unexpected obstacle + one line per resolution. Required reading for the next phase agent (so dead ends aren't repeated).
- [x] **QG-06** (shipped P57, 2026-04-28): Mandatory verifier-subagent dispatch per phase close. No phase ships without an unbiased subagent grading the catalog rows GREEN. Pattern documented in PROTOCOL.md; same shape as the §0.8 verifier dispatch from v0.11.2.
- [x] **QG-07** (shipped P57, 2026-04-28): **Mandatory CLAUDE.md update per phase** as part of definition-of-done. Each phase that introduces a new file, convention, gate, or operational rule MUST update the relevant CLAUDE.md section in the SAME PR. The verifier subagent grades this as a phase-close requirement. Anti-bloat: each phase appends a paragraph + code reference; deletions are encouraged when superseded. Owner-flagged in this planning session.
- [x] **QG-08** (shipped P57, 2026-04-28): Top-level `.planning/REQUIREMENTS.md` MUST contain ONLY the active milestone + a "Previously validated" index pointing to per-milestone REQUIREMENTS.md files inside `*-phases/`. Same rule for `.planning/ROADMAP.md`. This catalog row in `quality/gates/structure/` enforces the convention going forward (currently unenforced; the convention is documented in CLAUDE.md §0.5 but historical sections drifted into the top-level file before this gate existed). Owner-flagged in this planning session.
- [x] **QG-09** (shipped P57/P60, 2026-04-28): Quality Gates summary badge — `quality/runners/verdict.py` emits `quality/reports/badge.json` in [shields.io endpoint format](https://shields.io/badges/endpoint-badge): `{"schemaVersion": 1, "label": "quality gates", "message": "<N>/<M> GREEN", "color": <green|yellow|red>}`. Color thresholds: green if all P0+P1 PASS or WAIVED; yellow if any P2 RED; red if any P0+P1 RED. **P57 ships:** the verdict.py JSON emit. **P60 ships:** mkdocs publishes it as `docs/badge.json` → `https://reubenjohn.github.io/reposix/badge.json`; README + docs/index.md add the badge: `![Quality](https://img.shields.io/endpoint?url=https://reubenjohn.github.io/reposix/badge.json)`. **Plus** the cheaper standard badge `![Quality (weekly)](https://github.com/reubenjohn/reposix/actions/workflows/quality-weekly.yml/badge.svg)` lands in P58 alongside the workflow. Owner-flagged in this planning session.

### Structure dimension — migrate freshness invariants

- [x] **STRUCT-01** (shipped P57, 2026-04-28): Migrate the 6 freshness rows from `scripts/end-state.py` into `quality/gates/structure/`. Catalog rows live in `quality/catalogs/freshness-invariants.json`. Wire `quality/runners/run.py --cadence pre-push` as the new entry point.
- [x] **STRUCT-02** (shipped P57, 2026-04-28): `scripts/end-state.py` reduced to a thin shim (≤ 30 lines) that delegates to `quality/runners/verdict.py session-end`. Anti-bloat header comment explicitly tells future agents: "this file does not grow; new gates go under `quality/gates/<dim>/`." Owner-flagged concern: agents will bloat this file if not warned.

### Docs-repro dimension

- [x] **DOCS-REPRO-01** (shipped P59, 2026-04-28): `quality/gates/docs-repro/snippet-extract.py` parses every fenced code block in user-facing docs (README, docs/index.md, docs/tutorials/*.md) and emits catalog rows. Drift detector: fail if a doc snippet has no catalog row, or a catalog row's content drifted from its source.
- [x] **DOCS-REPRO-02** (shipped P59, 2026-04-28): `quality/gates/docs-repro/container-rehearse.sh <id>` spins ubuntu:24.04 (default), runs the snippet verbatim, asserts post-conditions. Per-persona matrix (linux first; mac/windows runners deferred to v0.12.1).
- [x] **DOCS-REPRO-03** (shipped P59, 2026-04-28): Promote `scripts/repro-quickstart.sh` into `quality/gates/docs-repro/tutorial-replay.sh` as one container-rehearsal-kind row. Wire into post-release cadence.
- [x] **DOCS-REPRO-04** (shipped P59, 2026-04-28): Catalog seed — port every row from the DRAFT `.planning/docs_reproducible_catalog.json` into `quality/catalogs/docs-reproducible.json` with the unified schema.

### Docs-build dimension migration

- [x] **DOCS-BUILD-01** (shipped P60, 2026-04-28): Move `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` into `quality/gates/docs-build/` with no behaviour change. Pre-push hook delegates to `quality/runners/run.py --cadence pre-push`. Leave shims at old paths if hooks would otherwise break.
- [x] **BADGE-01** (shipped P60, 2026-04-28): Validate every README + docs-page badge URL renders. New gate `quality/gates/docs-build/badges-resolve.py` HEADs each badge URL and asserts HTTP 200 + content-type contains `image`. Catalog row per badge in `quality/catalogs/freshness-invariants.json` (or a new `quality/catalogs/badges.json` if the count grows). Catches: shields.io drift, codecov project rename, badge-URL typos, broken endpoint URLs. Cadence: weekly + pre-push (cheap HEAD; ~1s for all 6 badges). Includes the new QG-09 endpoint badge URL once published.

### Subjective gates

- [x] **SUBJ-01** (shipped P61, 2026-04-28): `quality/catalogs/subjective-rubrics.json` with seed rubrics — `cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity`. Each row carries a numeric scoring rubric and a `freshness_ttl` (default 30d).
- [x] **SUBJ-02** (shipped P61, 2026-04-28): `reposix-quality-review` skill — reads the catalog, dispatches one unbiased subagent per stale/unverified row in parallel, persists JSON artifacts to `quality/reports/verifications/`, updates the catalog. Integrates `doc-clarity-review` as one rubric implementation.
- [x] **SUBJ-03** (shipped P61, 2026-04-28): Wire SUBJ-02 into pre-release cadence so subjective gates with TTL ≥ 14d expired auto-dispatch before any milestone tag push.

### Repo-org cleanup

- [x] **ORG-01**: Audit `.planning/research/v0.11.1/repo-organization-gaps.md` against current state. Each remaining gap → either fix + add a structure-dimension catalog row that prevents recurrence, OR file an explicit waiver with reason. Ensures the gaps document isn't a forgotten todo list. **SHIPPED P62.** Audit at `quality/reports/audits/repo-org-gaps.md` (99 items; 13 closed-by-deletion, 26 closed-by-relocation, 50 closed-by-existing-gate, 8 out-of-scope; zero open Wave-3 items). 3 new structure-dimension rows in `quality/catalogs/freshness-invariants.json` lock recurrence guards.

### Polish passes (per-dimension RED-fix sweeps)

> **Owner directive (this planning session):** "I'm really hoping that after this milestone the codebase is pristine and high quality across all the dimensions." The POLISH-* items below are the broaden-and-deepen pass: every dimension that ships a gate in v0.12.0 ALSO ships a sweep that fixes the RED rows the gate's first run flags. The milestone is not about instrumenting the codebase — it is about leaving the codebase pristine. Each POLISH-* row is P0/P1 blast radius and gates milestone close. Anything that cannot be fixed in-phase is WAIVED (with TTL ≤ 90d + dimension_owner) or filed as a v0.12.1 carry-forward via MIGRATE-03.

- [x] **POLISH-STRUCT** (shipped P57, 2026-04-28) (P57): After structure-dim gates ship, audit every freshness invariant and fix any drift.
- [x] **POLISH-RELEASE** (shipped P58, 2026-04-28) (P58): After release-dim gates ship, audit and fix any release-asset drift not already covered by P56.
- [x] **POLISH-DOCS-REPRO** (shipped P59, 2026-04-28) (P59): After docs-repro gates ship, every fenced code block has catalog row + passing rehearsal OR explicit manual marker.
- [x] **POLISH-DOCS-BUILD** (shipped P60, 2026-04-28) (P60): Every badge URL renders; every link resolves; mkdocs strict GREEN; mermaid blocks render.
- [x] **POLISH-SUBJECTIVE** (shipped P61, 2026-04-28) (P61): Cold-reader + install-positioning + headline-numbers rubrics dispatched; P0/P1 findings fixed.
- [x] **POLISH-ORG** (P62): All 99 audit items have explicit dispositions; zero open Wave-3 items.
- [x] **POLISH-AGENT-UX** (shipped P59, 2026-04-28) (P59): `dark-factory-test.sh` migration runs end-to-end against simulator.
- [x] **POLISH-CODE** (P58 stub, P63 final): Clippy GREEN; cargo fmt verified; cargo-test wrapper kept as ci-job-status canonical per memory-budget rule.

### Aggressive simplification

> **Owner directive (this planning session):** "look aggressively for opportunities to simplify if [existing scripts/examples] can be swallowed by this framework as by incorporating them into the various phases/plans of this milestone."

- [x] **SIMPLIFY-01..12** (shipped P57–P63, 2026-04-28): Twelve scripts/examples absorbed into `quality/gates/<dim>/`. Per-script audit at `quality/reports/audits/scripts-retirement-p63.md`. Surviving scripts gated by `quality/catalogs/orphan-scripts.json` (17 rows enforcing shim-shape contract).

### Migration close-out

- [x] **MIGRATE-01..03** (shipped P63, 2026-04-28): 5 source-file deletions; 13 shim-waivers; cross-link audit verifier (100 paths verified, 0 stale); v0.12.1 carry-forward filed (perf-dim full impl, security-dim full impl, cross-platform rehearsals, Error::Other 156→144 completion).

### Docs-alignment dimension (added 2026-04-28; P64 + P65)

- [x] **DOC-ALIGN-01..10** (shipped P64+P65, 2026-04-28): `crates/reposix-quality/` workspace crate; `reposix-quality doc-alignment` CLI surface; hash-test-fn syn-based body hashing; catalog + skill + slash commands; pre-push wired; backfill executed (388 rows: 181 BOUND / 166 MISSING_TEST / 41 RETIRE_PROPOSED); PUNCH-LIST.md generated; milestone-close verifier GREEN.

## Out of Scope

- **Perf-dimension full implementation** (latency vs headline-copy cross-check, token-economy bench cross-check). Stubbed in MIGRATE-03; full ship deferred to v0.12.1.
- **Security-dimension full implementation** (allowlist-enforcement gate, audit-immutability test). Stubbed in MIGRATE-03; full ship deferred to v0.12.1.
- **Cross-platform container rehearsals** (windows-2022, macos-14 runners). v0.12.0 ships ubuntu-only matrix; cross-platform deferred to v0.12.1.
- **`Error::Other` 156→144 partial migration completion** (POLISH2-09 carry-forward from v0.11.1). Stubbed under MIGRATE-03 v0.12.1 carry-forward.
- **Threat-model rewrite for v0.9.0 architecture** (friction row 20 from v0.11.1). Deferred to a separate security-focused milestone.

## Traceability

Coverage = 46/46 requirements ✓ (38 original + 8 POLISH-*); no orphans, no duplicates. Full traceability table preserved in v0.12.0-shipped state. Phase entries: P56 (3) · P57 (15) · P58 (5) · P59 (9) · P60 (6) · P61 (4) · P62 (2) · P63 (5).

## Recurring success criteria across every phase (P56–P65)

- Catalog-first: phase's first commit writes catalog rows BEFORE implementation.
- CLAUDE.md update in the same PR (QG-07).
- Unbiased verifier-subagent dispatch on phase close (QG-06).
- SIMPLIFY absorption (where applicable).
- Fix every RED row the dimension's gates flag (or WAIVE with TTL or carry-forward via MIGRATE-03).
