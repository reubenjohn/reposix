# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.4](https://github.com/reubenjohn/reposix/compare/reposix-cache-v0.11.3...reposix-cache-v0.11.4) - 2026-04-28

### Added

- *(p60)* publish QG-09 badge -- docs/badge.json + README + docs/index.md endpoint badge
- *(p58)* wire QG-09 P58 GH Actions badge — quality-weekly link in README + docs/index.md

## [0.11.3](https://github.com/reubenjohn/reposix/compare/reposix-cache-v0.11.2...reposix-cache-v0.11.3) - 2026-04-27

### Other

- *(readme)* fix broken badges + replace outdated tagline
- *(cache-audit)* cover 5 helper-RPC audit fns — closes NICE-TO-HAVE #1

## [0.11.2](https://github.com/reubenjohn/reposix/compare/reposix-cache-v0.11.1...reposix-cache-v0.11.2) - 2026-04-27

### Other

- §0.3 + §0.4 — rename version-pinned files, hoist token-economy bench into nav

## [0.11.1](https://github.com/reubenjohn/reposix/compare/reposix-cache-v0.11.0...reposix-cache-v0.11.1) - 2026-04-27

### Added

- *(cache)* Cache::gc with LRU/TTL/all strategies + cache_gc audit op
- *(cache)* add helper_backend_instantiated audit op
- *(cache)* time-travel via git tags — refs/reposix/sync/<ISO8601> per Cache::sync
- *(34-01)* blob-limit guardrail enforcement in stateless-connect
- *(34-01)* extend cache audit ops with blob_limit_exceeded + helper_push_*
- *(33-02)* Cache::sync — atomic delta materialization
- *(33-02)* log_delta_sync_tx — transaction-scoped audit helper
- *(33-02)* extend audit_events_cache CHECK to include 'delta_sync'
- *(32-04)* integration tests + cargo fmt --all
- *(32-02)* helper_* audit log surface on Cache
- *(31-03)* trybuild compile-fail fixtures lock Tainted discipline
- *(31-02)* wire audit+meta+oid_map SQLite hardening + read_blob
- *(31-01)* implement Cache::build_from — lazy-blob tree + commit
- *(31-01)* scaffold reposix-cache crate with gix 0.82 API smoke test
- *(10-4)* Tier 5 demo + docs for FUSE-mount-real-GitHub
- *(bench)* token-economy benchmark — 92.3% reduction measured
- scaffold workspace, CI, CLAUDE.md, PROJECT.md with security guardrails

### Fixed

- *(cargo)* add version field on cross-crate path-deps for crates.io publish (POLISH2-01)
- *(release)* unblock v0.11.0 ship — windows compile + drop arm64-musl
- *(cache)* set default git identity in gix_api_smoke test
- *(cache)* default git identity in Cache::open so build_from commits on bare hosts

### Other

- *(docs)* scrub FUSE-era doc residue + stale phase markers in src/ (P2-1, P2-2)
- *(audit)* backtick SQLite in dual-schema docstrings (clippy::doc_markdown)
- *(audit)* endorse dual-schema design (POLISH2-22 option B, friction row 12)
- *(planning)* land 5 v0.11.1 audit reports (persona + repo-org)
- *(cache)* delete cli_compat.rs holdover (POLISH-15)
- gc + token cost coverage (cache + cli)
- cargo fmt --all (post-time-travel drift)
- cache + cli coverage for sync tags and time-travel cli
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
- *(33-02)* integration test — end-to-end delta sync against reposix-sim
- *(31-02)* lift cache_db.rs from reposix-cli to reposix-cache::cli_compat
- *(31-02)* materialize_one + egress_denied_logs integration tests
- *(26-02)* clarity review — README, docs/index.md, CHANGELOG
- *(26-01)* fix README version references — v0.3.0 -> v0.7.0 throughout
- *(25-A)* move InitialReport.md + AgenticEngineeringReference.md to docs/research/ (OP-11)
- *(15-B)* CHANGELOG [v0.5.0] + workspace version bump
- *(14-D)* CHANGELOG [Unreleased] + sweep v0.3-era write-path deferral prose
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
