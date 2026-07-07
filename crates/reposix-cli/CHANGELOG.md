# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.0](https://github.com/reubenjohn/reposix/compare/reposix-cli-v0.12.0...reposix-cli-v0.13.0) - 2026-07-07

### Added

- *(remote)* expose backend_dispatch as lib; delegate attach/sync to shared factory
- *(quality)* P90 90-03 test-name-vs-asserts gate + subagent-graded migration
- *(P89)* banned-production-tokens linter (RBF-FW-04)
- *(cli)* reposix sync --reconcile subcommand (DVCS-PERF-L1-02)
- *(cache)* cache_reconciliation table + reconciliation walk module + 3 new public Cache APIs + audit hook (DVCS-ATTACH-02 + 04 part 1)
- *(cli)* scaffold `reposix attach <spec>` subcommand body (DVCS-ATTACH-01 + Q1.2/Q1.3 wiring)

### Fixed

- *(93)* reposix sync --reconcile forces a full build_from rebuild
- *(cli)* configure remote.origin.fetch so init populates refs/reposix/origin/main
- *(ci)* treat empty-set env as unset in jira/confluence test helpers; harden sim spawn; make pre-pr diagnosable
- *(agent-ux)* add test-name-honesty markers for 9 P91 tests
- *(agent-ux)* doctor helper.binary fix-line matches attach's install teaching (T2 H2 followup)
- *(agent-ux)* doctor drift check is topology-aware; git-version is WARN not ERR (T2 H4/MED6)
- *(agent-ux)* attach routes plain `git push` to SoT + warns on missing helper (T2 H1/H2)
- *(security)* strip embedded credentials from mirror URLs in config + helper stderr (Wave-5.5 MEDIUM intake)
- *(core+remote+cache)* bucket-aware canonical paths + id-keyed diff planner (confluence mass-delete BLOCKER)
- *(cli)* refresh emits unpadded issues/<id>.md + wipe stale-padded duplicates (QL-001 / D91-10)
- *(cli)* resolve reposix remote partialClone-aware so subcommands work on attached trees (QL-004)
- *(binstall)* correct pkg-url to real release asset name (QL-003)
- *(capabilities)* reconcile CAPABILITIES with impl across 4 surfaces (QL-002)
- *(readme)* branch-pin CI/Docs/Quality badges to main
- *(cli)* broaden dark_factory_conflict_teaching_string_present to write_loop.rs
- *(cache,cli)* idempotent Cache::open + build_from + REPOSIX_SIM_ORIGIN env override (P79-03 fix-forward)

### Other

- *(93)* add partial_failure_recovery_real_confluence smoke (RBF-LR-03)
- *(cli)* attach_real_*/sync_real_* real-backend smokes + verifier; ci.yml JIRA_TEST_PROJECT
- *(cache,cli)* resolve ForkAsNew (free) + honest Abort doc; scrub phase-ID tokens
- *(cli)* P90 90-06 5 real MISSING_TEST tests (D90-07)
- *(scripts)* finish scripts/ collapse -- registry, doc refs, inverse gate (D-CONV-3)
- *(cli)* update doctor no-remote assert to new teaching finding (CI fix for 8fccaf8)
- *(cli)* update no-remote asserts to new teaching error (CI fix for 8fccaf8)
- reconcile cold-init latency to canonical 27 ms (QL-027)
- *(cli)* refresh stale worktree_helpers module header (QL-004 follow-through)
- *(cli)* integration tests for `reposix attach` (DVCS-ATTACH-01..04 + OP-3)
- *(testing)* parameterize Confluence space, kill TokenWorld hardcoding ([#33](https://github.com/reubenjohn/reposix/pull/33))

## [0.11.3](https://github.com/reubenjohn/reposix/compare/reposix-cli-v0.11.2...reposix-cli-v0.11.3) - 2026-04-27

### Other

- *(readme)* fix broken badges + replace outdated tagline

## [0.11.2](https://github.com/reubenjohn/reposix/compare/reposix-cli-v0.11.1...reposix-cli-v0.11.2) - 2026-04-27

### Other

- §0.3 + §0.4 — rename version-pinned files, hoist token-economy bench into nav

## [0.11.1](https://github.com/reubenjohn/reposix/compare/reposix-cli-v0.11.0...reposix-cli-v0.11.1) - 2026-04-27

### Added

- *(doctor)* print backend capability matrix row (POLISH2-08, persona-coding-agent fix #3)
- *(gc)* --orphans flag finds caches without a live working tree (POLISH-10 4/4)
- *(cost)* reposix cost --since aggregates token-cost ledger (POLISH-10 3/4)
- *(init)* --since=<RFC3339> bootstraps from a historical sync tag (POLISH-10 2/4)
- *(history)* reposix log --time-travel lists sync tags chronologically (POLISH-10 1/4)
- *(doctor)* complete 10-check diagnostic catalog with copy-pastable fixes (POLISH-09)
- *(install)* cargo binstall metadata for reposix-cli + reposix-remote (POLISH-07)
- *(cli)* reposix gc + reposix tokens subcommands
- *(remote)* backend dispatch via URL scheme — closes Phase 32 carry-forward debt
- *(cli)* reposix history + reposix at subcommands for sync-tag lookup
- *(cli)* add reposix doctor diagnostic subcommand
- *(36-01)* delete reposix-fuse crate + FUSE infrastructure (v0.9.0)
- *(35-03)* real-backend integration tests with skip_if_no_env! gating
- *(35-02)* dark-factory regression test (script + integration tests)
- *(35-01)* add `reposix init <backend>::<project> <path>` subcommand
- *(28-02)* wire JiraBackend into CLI + add contract tests
- *(27-03)* add Issue.extensions field + ADR-004 + v0.8.0 + CHANGELOG
- *(23-02)* wire Cmd::Spaces into main.rs dispatcher
- *(23-02)* add spaces module and promote read_confluence_env to pub(crate)
- *(21-C)* --no-truncate flag on reposix list (HARD-02 CLI surface)
- *(20-A)* implement reposix refresh subcommand (OP-3)
- *(20-A-task1)* add CacheDb SQLite metadata store + 4 unit tests
- *(16-B)* add wiremock tests for create/update/delete + supports test
- *(16-B)* rename ConfluenceReadOnlyBackend → ConfluenceBackend across workspace
- *(11-B-2)* --backend confluence in mount + reposix-fuse binary
- *(11-B-1)* reposix list --backend confluence dispatch
- *(10-4)* Tier 5 demo + docs for FUSE-mount-real-GitHub
- *(10-3)* empirical proof — reposix mount --backend github works end-to-end
- *(10-2)* add --backend {sim,github} to `reposix mount`
- *(cli)* `reposix list --backend github` actually reads real GitHub
- *(08-B-4)* reposix list subcommand
- *(bench)* token-economy benchmark — 92.3% reduction measured
- *(03-02)* reposix demo — end-to-end sim+mount+ls+cat+grep+audit
- *(03-02)* reposix CLI with sim and mount subcommands
- scaffold workspace, CI, CLAUDE.md, PROJECT.md with security guardrails

### Fixed

- *(cargo)* add version field on cross-crate path-deps for crates.io publish (POLISH2-01)
- *(doctor)* use URL-aware backend slug for JIRA worktree dispatch (closes friction row 2 partial, code-quality P0-2)
- *(cli)* JIRA cache routing — backend_slug_from_origin reads URL marker (P0)
- *(release)* more windows + cross-compile fixes for v0.11.0 ship
- *(ci)* clippy map_unwrap_or in gc.rs + banned-words allowlist
- clippy map_unwrap_or in doctor + banned-word in troubleshooting
- *(21)* WR-02 rename no_truncate test to accurately reflect smoke-test scope
- *(20)* WR-01 sanitize project in git author + WR-02 EPERM maps to alive
- *(03-review)* H-01 reposix demo audit-tail now actually runs
- *(03-review)* H-02 plumb sim --no-seed and --rate-limit through

### Other

- *(docs)* scrub FUSE-era doc residue + stale phase markers in src/ (P2-1, P2-2)
- *(doctor)* cargo fmt for POLISH2-08 capability check
- *(cli)* dedup cache_path_from_worktree — fold 3x thin wrappers into inline existence checks (closes friction row 10, code-quality P0-1)
- *(planning)* land 5 v0.11.1 audit reports (persona + repo-org)
- *(cost)* integration tests for help, missing-remote, and seeded data
- *(doctor)* serialise allowed-origins env-var tests with a mutex
- *(cli)* normalize subcommand module privacy
- *(cli)* strip FUSE residue from refresh.rs (POLISH-16)
- *(cache)* delete cli_compat.rs holdover (POLISH-15)
- *(cli)* extract worktree_helpers module (POLISH-13)
- *(deps)* bump rustix to 1.x, rand to 0.9, sha2 to 0.11 ([#15](https://github.com/reubenjohn/reposix/pull/15))
- gc + token cost coverage (cache + cli)
- *(cli)* agent_flow_real expects /confluence/ + /jira/ markers
- cargo fmt --all (post-time-travel drift)
- cache + cli coverage for sync tags and time-travel cli
- cargo fmt doctor.rs after is_ok_and rewrite
- *(cli)* doctor coverage for clean/error/warn/fix paths
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
- *(35-01)* add `init`/`mount` CLI surface tests
- *(31-02)* lift cache_db.rs from reposix-cli to reposix-cache::cli_compat
- *(28-03)* ADR-005, jira.md reference, CHANGELOG v0.8.0, fmt clean, tag script
- *(27-02)* propagate BackendConnector rename across workspace
- *(26-02)* clarity review — README, docs/index.md, CHANGELOG
- *(26-01)* fix README version references — v0.3.0 -> v0.7.0 throughout
- rustfmt pass on spaces.rs after Phase 23
- rustfmt pass after fuse-mount-tests feature gate
- *(25-A)* move InitialReport.md + AgenticEngineeringReference.md to docs/research/ (OP-11)
- *(20-B)* integration tests for reposix refresh + workspace green-gauntlet
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
- *(03)* Phase 3 DONE.md + plan summaries; bump healthz budget for cold cargo-run
