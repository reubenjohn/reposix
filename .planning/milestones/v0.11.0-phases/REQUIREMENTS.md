# Requirements — v0.11.x Polish & Reproducibility (HISTORICAL)

> **Status:** SHIPPED. v0.11.0 (2026-04-25) + v0.11.1 (2026-04-26 polish pass) + v0.11.2 (2026-04-27 polish pass) all closed; all 8 crates published to crates.io at v0.11.2.
>
> Extracted from the top-level `.planning/REQUIREMENTS.md` on 2026-04-27 during v0.12.0 milestone scaffolding to prevent further monolith growth. Convention reference: `CLAUDE.md` §0.5 / Workspace layout.
>
> **Carry-forward NOT closed by v0.11.x:** the curl/PowerShell installer URLs broke on every release after v0.11.0 because `release.yml` tag glob `v*` does not match release-plz's per-crate `reposix-cli-v*` pattern. Diagnosed in `.planning/research/v0.12.0/install-regression-diagnosis.md`; fixed by RELEASE-01 in v0.12.0 P56.

---

## v0.11.1 Requirements — Polish & Reproducibility (second pass)

**Milestone goal:** Close the carry-forward set surfaced by the autonomous §7 sweep on 2026-04-26 (HANDOVER.md §7-G). Two threads run in parallel: (a) **finish v0.11.0 ship-gates** (crates.io publish after owner email verification, linux-aarch64-musl in dist matrix, bench-latency-cron Authorization-header fix, JIRA latency cells once secrets land), and (b) **work the v0.12.0-deferred P1 list early** so the v0.11.x line gives harness authors and the security-lead persona enough machine-readability + typed-error hygiene to recommend reposix. Persona audits + code-quality gaps + repo-org gaps drive every requirement; nothing here is speculative.

**Source of truth:** `HANDOVER.md` §3 (release follow-ups) + §4 (friction matrix, 23 rows) + §7-G (next-coordinator carry-forward list); `.planning/research/v0.11.1/code-quality-gaps.md` P1 list (rows 1–9) + selected P2 (1, 7); `.planning/research/v0.11.1/persona-{mcp-user,harness-author,security-lead,skeptical-oss-maintainer,coding-agent}.md` (5 persona audits); `.planning/research/v0.11.1/repo-organization-gaps.md` recs #5 + orphan list. CATALOG-v3 JSON tracker at `.planning/CATALOG-v3.json` (rendered MD at `.planning/CATALOG-v3.md`) tracks per-file completion status.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Self-improving infrastructure (OP-4).** `reposix doctor` becomes the canonical capability-matrix surface (POLISH2-08), removing the need for ad-hoc `gh secret list` + `cargo tree -p reposix-jira` reasoning when an agent debugs "why doesn't JIRA work for me." Audit-log schema unification (POLISH2-22) closes the dual-table gap that currently makes `doctor` blind to half the audit history.
- **Close the feedback loop (OP-1).** JIRA latency cells (POLISH2-04) + bench-cron Authorization fix (POLISH2-03) keep the latency table honest end-to-end; without the cron run, the table goes stale and `docs/benchmarks/v0.9.0-latency.md` rots.
- **Numbers, not adjectives.** The capability matrix (POLISH2-06) and `--json` output (POLISH2-18) give harness authors machine-checkable data; the v1.0 stability ADR (POLISH2-17) gives them a written commitment instead of "trust the changelog."
- **Ground truth obsession (OP-6).** Code-quality P1 work (typed errors, file splits, dep prune, `pub→pub(crate)`, sim typed-error) closes the stringly-typed protocol in `sim.rs:566-572` that the red-team analysis flagged as an attacker-controlled `Error::Other(String)` round-trip.

### Active

- [ ] **POLISH2-01**: crates.io email verification + first publish of all 9 reposix-* crates. Owner action: verify email at https://crates.io/settings/profile then `gh workflow run release-plz.yml`. Source: HANDOVER.md §3a.
- [shipped] **POLISH2-02**: linux-aarch64-musl binary added to `release.yml` dist matrix. Source: HANDOVER.md §3b. shipped via `feat(release): aarch64-unknown-linux-musl via cargo-zigbuild (POLISH2-02, closes §3b)` — added matrix entry with `zig: true`, conditional `cargo-zigbuild` install step, and branched build step in `.github/workflows/release.yml`.
- [shipped] **POLISH2-03**: `bench-latency-cron` Authorization-header duplicate fixed by setting `persist-credentials: false` on the `actions/checkout` step in `.github/workflows/bench-latency-cron.yml`. Source: bench-cron run 24965024010 failure. Shipped via b5fbcc6.
- [ ] **POLISH2-04**: JIRA real-backend latency cells populated. Owner-action prereq: provision `JIRA_EMAIL` + `JIRA_API_TOKEN` + `REPOSIX_JIRA_INSTANCE` secrets, then re-dispatch `bench-latency-v09`. Source: HANDOVER.md §4 row 5 partial.
- [shipped] **POLISH2-05**: Methodology callout added to `docs/concepts/reposix-vs-mcp-and-sdks.md` naming the synthesized MCP fixture (per HANDOVER §4 row 1). Source: friction row 1 partial. Shipped via c1f3614.
- [shipped] **POLISH2-06**: Connector capability matrix added to landing page (per friction row 7 follow-up + persona-coding-agent F-1). Source: friction row 7 follow-up. Shipped via b5fbcc6.
- [shipped] **POLISH2-07**: Comments-shape callout added to `docs/tutorials/first-run.md` ("sim round-trips; jira/github/confluence connector-specific"). Source: persona-coding-agent F-2. Shipped via c1f3614.
- [shipped] **POLISH2-08**: `reposix doctor` prints a configured-backend capability-matrix row. Source: persona-coding-agent fix #3. Shipped via `feat(doctor): print backend capability matrix row (POLISH2-08, persona-coding-agent fix #3)` — `BackendCapabilities` / `CommentSupport` / `VersioningModel` in `reposix-core`, `pub const CAPABILITIES` in each backend crate, `check_backend_capabilities` Info finding in `crates/reposix-cli/src/doctor.rs`.
- [partial-shipped] **POLISH2-09**: Code-quality P1 — `Error::Other(String)` rewritten to typed `Error::NotFound` / `Error::NotSupported` / `Error::VersionMismatch` variants in `reposix-core`; closes the stringly-typed protocol in `crates/reposix-core/src/backend/sim.rs:566-572`. Source: code-quality P1-1 + P1-5. Shipped: 3 typed variants + sim.rs migrated; github read-only-backend disambiguator migrated to `Error::NotSupported { operation }` (3 production sites in `crates/reposix-github/src/lib.rs`). JIRA/Confluence 'not supported' `Other(…)` sites already removed during the POLISH2-10/-11 `lib.rs` splits. Display-string back-compat preserved. Remaining 'not found' / 'invalid origin' / generic stringly-typed `Error::Other` sites in backend adapters still pending v0.12.1 (deferred from v0.12.0 — see v0.12.0 REQUIREMENTS Out of Scope).
- [shipped] **POLISH2-10**: Code-quality P1 — `crates/reposix-confluence/src/lib.rs` (3 989 LOC) split into `types.rs` / `translate.rs` / `client.rs` modules.
- [shipped] **POLISH2-11**: Code-quality P1 — `crates/reposix-jira/src/lib.rs` (1 957 LOC) split into matching modules (mirror of POLISH2-10).
- [shipped] **POLISH2-12**: Code-quality P1 — drop 3 unused Cargo deps from `reposix-remote` (`serde`, `serde_yaml`, `clap`). Shipped via 48fcd4c.
- [shipped] **POLISH2-13**: Code-quality P1 — demote `pub` → `pub(crate)` on 49 `reposix-remote` symbols. Shipped via dba89c5.
- [shipped] **POLISH2-14**: Code-quality P1 — typed `SimError` introduced in `reposix-sim`; dropped `anyhow` from the library API. Shipped via fc459d0+649da8c.
- [shipped] **POLISH2-15**: `docs/development/roadmap.md` rewritten as 1-screen stub pointing at `.planning/ROADMAP.md`. Shipped via c088cd1.
- [shipped] **POLISH2-16**: Internal ADR-002 + ADR-003 nav cleanup. Shipped via 48859ae.
- [shipped] **POLISH2-17**: ADR-009 v1.0 stability commitment authored. Shipped via 48859ae.
- [shipped] **POLISH2-18**: `docs/reference/exit-codes.md` authored. Shipped via 47aabf0. `--json`/`--format=json` flag deferred to v0.12.0+.
- [ ] **POLISH2-19**: `.claude/skills/reposix-banned-words/SKILL.md` path refs at L9+L64 updated from `.planning/notes/` → `.planning/archive/notes/`. Owner-approval gated.
- [shipped] **POLISH2-20**: Upstream issue filed at https://github.com/squidfunk/mkdocs-material/issues/8584. Shipped via 0375ffc.
- [shipped] **POLISH2-21**: `.planning/` tree condensed per repo-org rec #5: 8 `v0.X.0-phases` dirs (v0.1.0–v0.8.0) folded into 8 `ARCHIVE.md` stubs.
- [shipped (option B)] **POLISH2-22**: Two parallel audit-log schemas unified per code-quality CC-3. Dual-schema endorsed in module docs; physical unification deferred to v0.12.0+ (CC-3).

### Out of Scope

- Full glossary polish + persona-driven page rewrites — separate doc-quality milestone, not v0.11.1.
- Supply chain hardening (cosign, SBOM, SLSA, Apple notarization, Authenticode) — friction row 19; deferred to v0.12.0+ per security-lead priority guidance.
- `research/threat-model-and-critique.md` rewrite for v0.9.0 architecture — friction row 20; the FUSE-era file at `.planning/research/v0.1-fuse-era/threat-model-and-critique.md` stays as-is for v0.11.1; the rewrite ships in v0.12.0+.

### Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| POLISH2-01 | inline (no GSD phase) | planning (owner-action gate) |
| POLISH2-02..08 | inline | shipped |
| POLISH2-09 | inline | partial-shipped (typed variants + sim.rs migrated; backends pending v0.12.1) |
| POLISH2-10..18 | inline | shipped (POLISH2-18 partial — `--json` deferred) |
| POLISH2-19 | inline | planning (owner-approval gate) |
| POLISH2-20..22 | inline | shipped (POLISH2-22 option B) |

---

## v0.11.0 Requirements — Polish & Reproducibility

**Milestone goal:** Close the long tail that v0.10.0 surfaced. Polish the docs site (jargon glosses + glossary + mermaid render hygiene + ADR cleanup), kill four codebase duplicates flagged by `simplify` (worktree helpers, `parse_remote_url`, `cli_compat.rs`, FUSE residue in `refresh.rs`), and ship reproducibility infrastructure: a fresh-clone tutorial runner, pre-built binaries via `dist`, `cargo binstall` metadata, `reposix doctor` / `reposix log --time-travel` / `reposix gc --orphans` / `reposix cost` surfaces, and a real-backend latency table for sim + GitHub + Confluence + JIRA.

**Source of truth:** `.planning/research/v0.11.0/vision-and-innovations.md` (vision spec) plus the v0.11.0 audit family: `v0.11.0-gsd-hygiene-report.md`, `v0.11.0-mkdocs-site-audit.md`, `v0.11.0-jargon-inventory.md`, `v0.11.0-latency-benchmark-plan.md`, `v0.11.0-release-binaries-plan.md`, `v0.11.0-cache-location-study.md`, `v0.11.0-CATALOG-v2.md`. Framing principles inherited from `.planning/notes/phase-30-narrative-vignettes.md`.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Self-improving infrastructure (OP-4).** `scripts/repro-quickstart.sh`, `scripts/check-docs-site.sh`, and the dist release pipeline are committed-and-CI-wired, not session memory.
- **Close the feedback loop (OP-1).** Mermaid diagrams render via mcp-mermaid AND playwright-screenshot the live site for every page touched. `mkdocs build --strict` is green by definition; pre-push hook runs `check-docs-site.sh` so a broken site never reaches `main`.
- **Numbers, not adjectives.** Latency table populated for sim + github + confluence + jira with record counts and 3-sample medians; doctor output has copy-pastable fix strings, not narrative.
- **Ground truth obsession (OP-6).** Tutorial reproducibility is asserted by a script that runs against a fresh `/tmp/clone`, not by reading a doc.

### Active (all SHIPPED in Phases 50–55, see Traceability table below)

- [shipped] **POLISH-01**: All jargon terms have inline gloss + external link at first occurrence per page (Phase 52).
- [shipped] **POLISH-02**: `docs/reference/glossary.md` exists; every other page links to it on first jargon term (≥24 entries, Phase 52).
- [shipped] **POLISH-03**: All mermaid diagrams render without console errors on the live site (Phase 52, F1+F2+F3 from `.planning/research/v0.11.0/mkdocs-site-audit.md`).
- [shipped] **POLISH-04**: `mkdocs build --strict` is green; ADR-008 in nav; blog post in `not_in_nav`; `pymdownx.emoji` configured.
- [shipped] **POLISH-05**: Tutorial reproducible from fresh clone — `bash scripts/repro-quickstart.sh` runs the 7-step tutorial and asserts each step passes (Phase 53).
- [shipped] **POLISH-06**: Pre-built binaries published to GitHub Releases for linux musl x86/arm64, macOS x86/arm64, windows msvc on every git tag (Phase 53).
- [shipped] **POLISH-07**: `cargo binstall reposix-cli` works (Phase 53). Cargo metadata for binstall configured.
- [shipped] **POLISH-08**: Latency table populated for sim + github (real-backend cells) with record counts and 3-sample medians (Phase 54). Confluence + JIRA cells `pending-secrets` until v0.11.1 secret provisioning.
- [shipped] **POLISH-09**: `reposix doctor` runs the full v3a checklist with copy-pastable fix strings (Phase 55).
- [shipped] **POLISH-10**: `reposix log --time-travel`, `reposix init --since=<RFC3339>`, `reposix cost`, `reposix gc --orphans` all surfaced (Phase 55).
- [shipped] **POLISH-11**: ADR-004 + ADR-006 deleted; `agentic-engineering-reference.md` carries a disclaimer; archival doc sweep complete.
- [shipped] **POLISH-12**: `Cargo.toml` workspace version bumped through `0.11.0-dev` → `0.11.0` → `0.11.1` → `0.11.2` lifecycle.
- [shipped] **POLISH-13..16**: 4-way CLI duplication consolidated, `parse_remote_url` unified, `cli_compat.rs` deleted, FUSE residue stripped from `refresh.rs` (Phase 51).
- [shipped] **POLISH-17**: CLAUDE.md adds: any docs-site work MUST be playwright-validated; `scripts/check-docs-site.sh` exists and is wired into pre-push (Phase 53).

### Out of Scope

- New backend connectors. v0.11.0 stays on the existing 4 (sim, github, confluence, jira).
- `cargo install reposix-cli` from crates.io publishing — kept as a Phase 53 stretch goal but not gated on it; `cargo binstall` is the ship requirement. (Both shipped via release-plz in v0.11.1+.)
- Full v0.12.0 agent-SDK recipes (Claude Code / Cursor / Aider). v0.11.0 keeps `docs/guides/integrate-with-your-agent.md` as a pointer page.

### Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| POLISH-01..04 | 52 | shipped |
| POLISH-05..07 | 53 | shipped |
| POLISH-08 | 54 | shipped (sim + github; confluence + jira pending-secrets) |
| POLISH-09..10 | 55 | shipped |
| POLISH-11..12 | 50 + tag-time | shipped |
| POLISH-13..16 | 51 | shipped |
| POLISH-17 | 53 | shipped |
