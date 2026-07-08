# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.1](https://github.com/reubenjohn/reposix/compare/reposix-core-v0.13.0...reposix-core-v0.13.1) - 2026-07-08

### Fixed

- *(v0.13.1)* run `reposix sim` in-process with a builtin offline seed

### Other

- *(v0.13.1)* fix onboarding-gate output-mismatch doc-lies + rebind doc-alignment rows
- *(readme)* fix quick-start doc-lie — no-seed-file run seeds 0 issues

## [0.13.0](https://github.com/reubenjohn/reposix/compare/reposix-core-v0.12.0...reposix-core-v0.13.0) - 2026-07-07

### Added

- *(core)* bucket-aware canonical record paths (issues|pages)
- *(core)* canonical record_path + shared issue_id_from_path (issues/<id>.md); adopt in cache (QL-001 BUG-1)
- *(quality)* P90 90-03 test-name-vs-asserts gate + subagent-graded migration

### Fixed

- *(94)* gate prune_oid_map on connector completeness signal (Fork A) + idempotent delete-NotFound (Fork B)
- *(agent-ux)* add test-name-honesty markers for 9 P91 tests
- *(security)* strip embedded credentials from mirror URLs in config + helper stderr (Wave-5.5 MEDIUM intake)
- *(security)* gate mirror push egress against allowlist (QL-006)
- *(readme)* branch-pin CI/Docs/Quality badges to main

### Other

- *(scripts)* finish scripts/ collapse -- registry, doc refs, inverse gate (D-CONV-3)
- reconcile cold-init latency to canonical 27 ms (QL-027)

## [0.11.3](https://github.com/reubenjohn/reposix/compare/reposix-core-v0.11.2...reposix-core-v0.11.3) - 2026-04-27

### Other

- *(readme)* fix broken badges + replace outdated tagline
- *(http)* cover post/patch/delete allowlist gate — lifts http.rs to ~96%

## [0.11.2](https://github.com/reubenjohn/reposix/compare/reposix-core-v0.11.1...reposix-core-v0.11.2) - 2026-04-27

### Other

- §0.3 + §0.4 — rename version-pinned files, hoist token-economy bench into nav

## [0.11.1](https://github.com/reubenjohn/reposix/compare/reposix-core-v0.11.0...reposix-core-v0.11.1) - 2026-04-27

### Other

- update Cargo.toml dependencies
