# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.4](https://github.com/reubenjohn/reposix/compare/reposix-jira-v0.11.3...reposix-jira-v0.11.4) - 2026-04-28

### Added

- *(p60)* publish QG-09 badge -- docs/badge.json + README + docs/index.md endpoint badge
- *(p58)* wire QG-09 P58 GH Actions badge — quality-weekly link in README + docs/index.md

## [0.11.3](https://github.com/reubenjohn/reposix/compare/reposix-jira-v0.11.2...reposix-jira-v0.11.3) - 2026-04-27

### Other

- *(readme)* fix broken badges + replace outdated tagline

## [0.11.2](https://github.com/reubenjohn/reposix/compare/reposix-jira-v0.11.1...reposix-jira-v0.11.2) - 2026-04-27

### Other

- §0.3 + §0.4 — rename version-pinned files, hoist token-economy bench into nav

## [0.11.1](https://github.com/reubenjohn/reposix/compare/reposix-jira-v0.11.0...reposix-jira-v0.11.1) - 2026-04-27

### Added

- *(doctor)* print backend capability matrix row (POLISH2-08, persona-coding-agent fix #3)
- *(33-01)* JiraBackend::list_changed_since via JQL updated>=
- *(29-03)* delete_or_close via transitions + supports() + contract write tests
- *(29-02)* implement create_issue + update_issue write path
- *(29-01)* ADF write encoder + issuetype cache infrastructure
- *(28-02)* wire JiraBackend into CLI + add contract tests
- *(28-01)* add reposix-jira crate — JiraBackend + BackendConnector impl + 17 tests
- *(10-4)* Tier 5 demo + docs for FUSE-mount-real-GitHub
- *(bench)* token-economy benchmark — 92.3% reduction measured
- scaffold workspace, CI, CLAUDE.md, PROJECT.md with security guardrails

### Fixed

- *(jira)* clippy map_unwrap_or → is_ok_and in BodyContains matcher

### Other

- *(error)* migrate jira/confluence/github 'not supported' to typed Error::NotSupported (POLISH2-09 ext)
- *(jira)* split lib.rs into types/translate/client modules (POLISH2-11, code-quality P1-2)
- *(planning)* land 5 v0.11.1 audit reports (persona + repo-org)
- *(deps)* bump rustix to 1.x, rand to 0.9, sha2 to 0.11 ([#15](https://github.com/reubenjohn/reposix/pull/15))
- cargo fmt --all after Record rename
- update Record/RecordId rename across user-facing docs
- rename remaining Issue-prefixed types to Record (issue.rs → record.rs)
- rename BackendConnector methods *_issue to *_record
- rename Issue type to Record (preserves YAML wire format)
- rename IssueId to RecordId (workspace-wide)
- per-crate description, keywords, categories, readme
- *(45-01)* rewrite README for v0.9.0 surface — drop Tier 1-5 demo blocks
- *(40-04)* rewrite README hero — numbers, not adjectives + v0.9.0 framing
- *(36-02)* rewrite CLAUDE.md, ship reposix-agent-flow skill, finalize v0.9.0 release artifacts
- *(35-01)* CHANGELOG breaking-change note + README quickstart for `reposix init`
- *(28-03)* ADR-005, jira.md reference, CHANGELOG v0.8.0, fmt clean, tag script
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
