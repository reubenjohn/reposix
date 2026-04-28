# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.4](https://github.com/reubenjohn/reposix/compare/reposix-confluence-v0.11.3...reposix-confluence-v0.11.4) - 2026-04-28

### Added

- *(p60)* publish QG-09 badge -- docs/badge.json + README + docs/index.md endpoint badge
- *(p58)* wire QG-09 P58 GH Actions badge — quality-weekly link in README + docs/index.md

## [0.11.3](https://github.com/reubenjohn/reposix/compare/reposix-confluence-v0.11.2...reposix-confluence-v0.11.3) - 2026-04-27

### Other

- *(readme)* fix broken badges + replace outdated tagline

## [0.11.2](https://github.com/reubenjohn/reposix/compare/reposix-confluence-v0.11.1...reposix-confluence-v0.11.2) - 2026-04-27

### Other

- §0.3 + §0.4 — rename version-pinned files, hoist token-economy bench into nav

## [0.11.1](https://github.com/reubenjohn/reposix/compare/reposix-confluence-v0.11.0...reposix-confluence-v0.11.1) - 2026-04-27

### Added

- *(doctor)* print backend capability matrix row (POLISH2-08, persona-coding-agent fix #3)
- *(33-01)* ConfluenceBackend::list_changed_since via CQL search
- *(27-03)* add Issue.extensions field + ADR-004 + v0.8.0 + CHANGELOG
- *(24-01)* add list_attachments, list_whiteboards, download_attachment + translate folder fix
- *(23-01)* add list_comments + list_spaces to ConfluenceBackend
- *(21-C)* list_issues_strict + tenant-URL redaction (HARD-02 HARD-05)
- *(16-C)* add roundtrip integration test (WRITE-04 end-to-end) + cargo fmt
- *(16-C)* add audit unit tests (6) — update/create/delete write audit rows
- *(16-C)* switch get_issue to ADF body format with storage fallback
- *(16-C)* add audit field + with_audit builder; add audit_write helper; wire into create/update/delete
- *(16-C)* add rusqlite+sha2 deps to reposix-confluence
- *(16-B)* add wiremock tests for create/update/delete + supports test
- *(16-B)* add supports(Delete|StrongVersioning) + write_headers helper
- *(16-B)* rename ConfluenceReadOnlyBackend → ConfluenceBackend across workspace
- *(16-A)* implement adf.rs markdown_to_storage + adf_to_markdown + 18 unit tests
- *(16-A)* add pulldown-cmark workspace dep
- *(13-B1)* populate Issue::parent_id from Confluence parentId + supports(Hierarchy) + root_collection_name("pages")
- *(13-A-1)* add Issue::parent_id + BackendFeature::Hierarchy + root_collection_name default
- *(11-A-2)* ConfluenceReadOnlyBackend + wiremock unit tests
- *(11-A-1)* scaffold reposix-confluence crate + wire into workspace
- *(10-4)* Tier 5 demo + docs for FUSE-mount-real-GitHub
- *(bench)* token-economy benchmark — 92.3% reduction measured
- scaffold workspace, CI, CLAUDE.md, PROJECT.md with security guardrails

### Fixed

- *(24)* resolve merge conflict in confluence/src/lib.rs (test string wording)
- *(24)* resolve merge conflict in confluence/src/lib.rs (doc comments only)
- *(clippy)* replace map+unwrap_or(false) with is_ok_and across all test files
- *(21)* update 409-conflict unit test assertion after WR-01 error message change
- *(21)* WR-01 redact url and truncate 409 body in update_issue conflict path
- *(21-C)* redact tenant URLs in all error paths (HARD-05 complete)
- *(13-review)* apply REVIEW.md polish — IN-01 tracing cap, IN-02 slug pre-cap, IN-03 stable mtime, WR-01 tree refresh, WR-02 PATH_MAX guard
- *(11-REVIEW)* IN-01 — comment drift in contract.rs
- *(11-REVIEW)* WR-02 — validate space_id before URL construction
- *(11-REVIEW)* WR-01 — percent-encode space_key query param

### Other

- *(docs)* scrub FUSE-era doc residue + stale phase markers in src/ (P2-1, P2-2)
- *(confluence)* split lib.rs into types/translate/client modules (POLISH2-10, code-quality P1-2)
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
- cargo fmt --all
- *(40-04)* rewrite README hero — numbers, not adjectives + v0.9.0 framing
- *(36-02)* rewrite CLAUDE.md, ship reposix-agent-flow skill, finalize v0.9.0 release artifacts
- *(35-01)* CHANGELOG breaking-change note + README quickstart for `reposix init`
- *(28-03)* ADR-005, jira.md reference, CHANGELOG v0.8.0, fmt clean, tag script
- *(27-02)* propagate BackendConnector rename across workspace
- *(26-02)* clarity review — README, docs/index.md, CHANGELOG
- *(26-01)* fix README version references — v0.3.0 -> v0.7.0 throughout
- *(23-03)* apply rustfmt to all modified files
- *(23-01)* add failing tests for list_comments and list_spaces
- *(25-A)* move InitialReport.md + AgenticEngineeringReference.md to docs/research/ (OP-11)
- *(15-B)* CHANGELOG [v0.5.0] + workspace version bump
- *(14-D)* CHANGELOG [Unreleased] + sweep v0.3-era write-path deferral prose
- *(drive-by)* linkedin.md v0.4 messaging + confluence read-only errors version-neutral (OP-6 MEDIUM-17/12)
- *(confluence)* SSRF regression tests for _links.base and webui_link fields (OP-7)
- *(13-D1)* migrate walkthroughs + architecture diagrams to nested layout
- *(13-D2-4)* README folder-structure section + prebuilt-binaries quickstart (OP-12 fold-in)
- README refresh for v0.3.0 + release.yml workflow for prebuilt binaries
- move social/ → docs/social/ to consolidate media assets
- *(11-REVIEW)* IN-03 — drop unused thiserror dep
- *(11-REVIEW)* IN-02 — drop unused rusqlite dev-dep
- *(11-E-3)* update README, CHANGELOG, architecture, .env.example for Phase 11
- *(11-C-1)* contract test parameterized over sim + wiremock-confluence + live-confluence
- HANDOFF.md for next overnight agent + social assets linked
- README — show 'reposix list --backend github' working against real GitHub
- *(09-6)* README + docs/demos/index.md — Tier 4 swarm row
- *(08)* flip integration-contract CI to strict + add codecov badge
- *(08-D-5)* README mentions GitHub adapter + parity demo
- *(08-A-13)* README Demo section — Tier 1 table + runnable suite block
- add Docs badge + site link to README
- *(04-02)* README Status / Demo / Security / Honest scope for v0.1 ship
