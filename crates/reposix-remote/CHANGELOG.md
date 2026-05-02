# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.0](https://github.com/reubenjohn/reposix/compare/reposix-remote-v0.12.0...reposix-remote-v0.13.0) - 2026-05-02

### Added

- *(reposix-remote)* bus_handler write fan-out replacing deferred-shipped stub (DVCS-BUS-WRITE-01..05)
- *(remote)* bus_handler + main.rs Route dispatch + capabilities branching + State extension (DVCS-BUS-PRECHECK-01..02 + DVCS-BUS-FETCH-01)
- *(remote)* coarser SoT-drift wrapper precheck_sot_drift_any (DVCS-BUS-PRECHECK-02 substrate)
- *(remote)* bus URL parser — bus_url::parse + Route::Single|Bus enum (DVCS-BUS-URL-01)
- *(cache,remote)* L1 precheck — read_last_fetched_at + precheck.rs + handle_export rewrite (DVCS-PERF-L1-01, DVCS-PERF-L1-03)
- *(remote)* wire mirror-lag refs into handle_export success + conflict-reject paths (DVCS-MIRROR-REFS-02 + DVCS-MIRROR-REFS-03)

### Fixed

- *(remote)* per-test cache_dir isolation in push_conflict tests
- *(remote)* clippy match-wildcard-for-single-variants + doc-markdown SoT/Route in test panics (P82-01 pre-push fix)
- *(remote)* add per-issue GET mocks to push_conflict.rs tests for L1 precheck
- *(remote)* drop incidental .expect(1) GET-count in bulk_delete_cap test
- *(remote)* serialize REPOSIX_CACHE_DIR env-var mutation in perf_l1 tests

### Other

- *(remote)* bus_write_audit_completeness.rs dual-table audit assertion (DVCS-BUS-WRITE-06 audit-completeness)
- *(remote)* SoT-fail tests — mid-stream 5xx + post-precheck 409 (DVCS-BUS-WRITE-06 b+c)
- *(remote)* mirror-fail integration test with #[cfg(unix)] failing-update-hook fixture (DVCS-BUS-WRITE-06 a)
- *(reposix-remote)* bus write happy-path + no-mirror-remote regression integration tests (DVCS-BUS-WRITE-01..05)
- *(reposix-remote)* lift handle_export write loop into write_loop::apply_writes (P83 prelude)
- *(remote)* 4 integration tests — bus_url + bus_capabilities + bus_precheck_a + bus_precheck_b (DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01)
- *(remote)* copy tests/common.rs from reposix-cache (P81 M3 gap)
- *(remote)* N=200 perf regression + positive control + flip catalogs FAIL→PASS + CLAUDE.md update (DVCS-PERF-L1-01..03 close)
- *(remote)* integration tests for mirror-lag refs + verifier flip + CLAUDE.md update + schema migration (DVCS-MIRROR-REFS-01..03 close)

## [0.11.3](https://github.com/reubenjohn/reposix/compare/reposix-remote-v0.11.2...reposix-remote-v0.11.3) - 2026-04-27

### Other

<<<<<<< Updated upstream
=======
- *(remote)* drive stateless-connect e2e — lifts stateless_connect.rs 42% → 75%
>>>>>>> Stashed changes
- *(readme)* fix broken badges + replace outdated tagline

## [0.11.2](https://github.com/reubenjohn/reposix/compare/reposix-remote-v0.11.1...reposix-remote-v0.11.2) - 2026-04-27

### Other

- §0.3 + §0.4 — rename version-pinned files, hoist token-economy bench into nav

## [0.11.1](https://github.com/reubenjohn/reposix/compare/reposix-remote-v0.11.0...reposix-remote-v0.11.1) - 2026-04-27

### Added

- *(install)* cargo binstall metadata for reposix-cli + reposix-remote (POLISH-07)
- *(remote)* token_cost audit per fetch/push (chars/4 estimate)
- *(remote)* backend dispatch via URL scheme — closes Phase 32 carry-forward debt
- *(34-02)* push-time conflict detection + push audit ops
- *(34-01)* blob-limit guardrail enforcement in stateless-connect
- *(33-02)* helper stateless-connect calls Cache::sync before tunnel
- *(32-04)* integration tests + cargo fmt --all
- *(32-03)* stateless-connect tunnel handler + capability wiring
- *(32-01)* pkt-line frame reader/encoder for protocol-v2 tunnel
- *(27-03)* add Issue.extensions field + ADR-004 + v0.8.0 + CHANGELOG
- *(13-A-1)* add Issue::parent_id + BackendFeature::Hierarchy + root_collection_name default
- *(10-4)* Tier 5 demo + docs for FUSE-mount-real-GitHub
- *(bench)* token-economy benchmark — 92.3% reduction measured
- *(S-B-1)* protocol skeleton + capabilities/list/option dispatch
- scaffold workspace, CI, CLAUDE.md, PROJECT.md with security guardrails

### Fixed

- *(cargo)* add version field on cross-crate path-deps for crates.io publish (POLISH2-01)
- *(S-review)* M-03 normalized-compare so unchanged pushes emit zero PATCHes
- *(S-review)* H-03 emit protocol error on backend failures, no torn pipe

### Other

- *(docs)* scrub FUSE-era doc residue + stale phase markers in src/ (P2-1, P2-2)
- *(remote)* demote pub → pub(crate) on internal symbols (POLISH2-13, code-quality P1-7)
- *(remote)* drop 3 unused Cargo deps (POLISH2-12, code-quality P1-6)
- *(planning)* land 5 v0.11.1 audit reports (persona + repo-org)
- *(core)* unify parse_remote_url; remote calls into core (POLISH-14)
- cargo fmt --all after Record rename
- rename remaining Issue-prefixed types to Record (issue.rs → record.rs)
- rename BackendConnector methods *_issue to *_record
- rename Issue type to Record (preserves YAML wire format)
- rename IssueId to RecordId (workspace-wide)
- per-crate description, keywords, categories, readme
- *(45-01)* rewrite README for v0.9.0 surface — drop Tier 1-5 demo blocks
- cargo fmt --all
- *(40-04)* rewrite README hero — numbers, not adjectives + v0.9.0 framing
- *(36-02)* rewrite CLAUDE.md, ship reposix-agent-flow skill, finalize v0.9.0 release artifacts
- *(35-01)* CHANGELOG breaking-change note + README quickstart for `reposix init`
- *(34-02)* integration tests for push conflict + sanitize regression
- *(28-03)* ADR-005, jira.md reference, CHANGELOG v0.8.0, fmt clean, tag script
- *(27-02)* propagate BackendConnector rename across workspace
- *(26-02)* clarity review — README, docs/index.md, CHANGELOG
- *(26-01)* fix README version references — v0.3.0 -> v0.7.0 throughout
- *(25-A)* move InitialReport.md + AgenticEngineeringReference.md to docs/research/ (OP-11)
- *(15-B)* CHANGELOG [v0.5.0] + workspace version bump
- *(14-D)* CHANGELOG [Unreleased] + sweep v0.3-era write-path deferral prose
- *(14-B2)* route reposix-remote through IssueBackend trait
- *(13-D1)* migrate walkthroughs + architecture diagrams to nested layout
- *(13-D2-4)* README folder-structure section + prebuilt-binaries quickstart (OP-12 fold-in)
- README refresh for v0.3.0 + release.yml workflow for prebuilt binaries
- move social/ → docs/social/ to consolidate media assets
- *(11-E-3)* update README, CHANGELOG, architecture, .env.example for Phase 11
- HANDOFF.md for next overnight agent + social assets linked
- README — show 'reposix list --backend github' working against real GitHub
- *(09-6)* README + docs/demos/index.md — Tier 4 swarm row
- *(08)* flip integration-contract CI to strict + add codecov badge
- *(08-D-5)* README mentions GitHub adapter + parity demo
- *(08-A-13)* README Demo section — Tier 1 table + runnable suite block
- add Docs badge + site link to README
- *(04-02)* README Status / Demo / Security / Honest scope for v0.1 ship
