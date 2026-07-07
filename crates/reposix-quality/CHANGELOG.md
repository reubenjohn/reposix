# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.0](https://github.com/reubenjohn/reposix/compare/reposix-quality-v0.12.0...reposix-quality-v0.13.0) - 2026-07-07

### Added

- *(quality)* P90 90-03 test-name-vs-asserts gate + subagent-graded migration
- *(quality)* walk verb skips catalog save on last_walked-only delta (D-CONV-5)
- *(quality)* docs-alignment per-row waiver verb (time-boxed, loud, tracked)
- *(P89)* banned-production-tokens linter (RBF-FW-04)
- *(reposix-quality)* bind --dimension agent-ux (closes 80% of GOOD-TO-HAVES-01)

### Fixed

- *(96)* key doc-alignment drift-skip on bind-state + heal legacy same-file-multi rows
- *(quality)* bind replaces same-file cite instead of appending phantom

### Other

- *(docs-alignment)* walker AND-compares per-source hashes (MULTI-SOURCE-WATCH-01)
