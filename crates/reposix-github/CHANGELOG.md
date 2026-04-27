# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.2](https://github.com/reubenjohn/reposix/compare/reposix-github-v0.11.1...reposix-github-v0.11.2) - 2026-04-27

### Other

- §0.3 + §0.4 — rename version-pinned files, hoist token-economy bench into nav

## [0.11.1](https://github.com/reubenjohn/reposix/compare/reposix-github-v0.11.0...reposix-github-v0.11.1) - 2026-04-27

### Added

- *(doctor)* print backend capability matrix row (POLISH2-08, persona-coding-agent fix #3)
- *(33-01)* GithubReadOnlyBackend::list_changed_since with native ?since=
- *(27-03)* add Issue.extensions field + ADR-004 + v0.8.0 + CHANGELOG
- *(13-A-1)* add Issue::parent_id + BackendFeature::Hierarchy + root_collection_name default
- *(10-4)* Tier 5 demo + docs for FUSE-mount-real-GitHub
- *(cli)* `reposix list --backend github` actually reads real GitHub
- *(github)* LR-02 rate-limit backoff — honor x-ratelimit-reset
- *(08-C-2)* GithubReadOnlyBackend implementing IssueBackend + wiremock tests (C-3)
- *(08-C-1)* new crate reposix-github (scaffold)
- *(bench)* token-economy benchmark — 92.3% reduction measured
- scaffold workspace, CI, CLAUDE.md, PROJECT.md with security guardrails

### Fixed

- *(security)* redact GITHUB_TOKEN from GithubReadOnlyBackend Debug impl
- *(clippy)* replace map+unwrap_or(false) with is_ok_and across all test files

### Other

- *(error)* migrate jira/confluence/github 'not supported' to typed Error::NotSupported (POLISH2-09 ext)
- *(planning)* land 5 v0.11.1 audit reports (persona + repo-org)
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
- *(28-03)* ADR-005, jira.md reference, CHANGELOG v0.8.0, fmt clean, tag script
- *(27-02)* propagate BackendConnector rename across workspace
- *(26-02)* clarity review — README, docs/index.md, CHANGELOG
- *(26-01)* fix README version references — v0.3.0 -> v0.7.0 throughout
- *(25-A)* move InitialReport.md + AgenticEngineeringReference.md to docs/research/ (OP-11)
- *(15-B)* CHANGELOG [v0.5.0] + workspace version bump
- *(14-D)* CHANGELOG [Unreleased] + sweep v0.3-era write-path deferral prose
- *(github)* wiremock contract tests for pagination, 429, SSRF, state_reason, User-Agent (OP-6 MEDIUM-13)
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
- *(08-D-1)* contract test proves shape parity SimBackend vs GithubReadOnlyBackend
- *(08-A-13)* README Demo section — Tier 1 table + runnable suite block
- add Docs badge + site link to README
- *(04-02)* README Status / Demo / Security / Honest scope for v0.1 ship
