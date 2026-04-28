---
phase: 60-docs-build-migration
plan: 06
subsystem: quality-gates
tags: [qg-09, badge, mkdocs, shields-io, publish]
requires:
  - quality/reports/badge.json
  - quality/gates/docs-build/badges-resolve.py (Wave C)
provides:
  - docs/badge.json (mkdocs auto-include -> https://reubenjohn.github.io/reposix/badge.json)
  - Quality endpoint badge in README + docs/index.md
affects:
  - public visibility of catalog-rollup health
key-files:
  created:
    - docs/badge.json
  modified:
    - README.md
    - docs/index.md
    - quality/gates/docs-build/badges-resolve.py
  unchanged:
    - mkdocs.yml (mkdocs-material auto-includes; no extra_files needed)
decisions:
  - "Approach (b): commit docs/badge.json as snapshot of quality/reports/badge.json; mkdocs auto-includes"
  - "Manual sync between quality/reports/badge.json (auto-emitted) and docs/badge.json (committed); v0.12.1 automates"
  - "WAVE_F_PENDING_URLS cleared; skip-branch preserved as no-op for future migrations"
metrics:
  badge_count_readme: 8
  badge_count_docs_index: 3
  verifier_urls_pass: 8
  pre_push_exit: 0
  weekly_exit: 0
  duration_minutes: 6
  completed_date: "2026-04-27"
---

# Phase 60 Plan 06: QG-09 publish (Wave F)

## One-liner

Quality Gates rollup badge is now publicly visible: `docs/badge.json` ships in the mkdocs-built site; README + docs/index.md gain the shields.io endpoint badge.

## mkdocs auto-include verification (Task 1)

```bash
cp quality/reports/badge.json docs/badge.json
mkdocs build --strict --site-dir /tmp/p60-wave-f-site
# Result: site contains badge.json with matching content; no mkdocs.yml edit needed
```

## Badge insertions

**README.md** (8 badges, was 7):

```markdown
[![CI](.../CI/badge.svg)](...)
[![Quality (weekly)](.../quality-weekly.yml/badge.svg)](...)
[![Quality](https://img.shields.io/endpoint?url=https%3A%2F%2Freubenjohn.github.io%2Freposix%2Fbadge.json)](https://reubenjohn.github.io/reposix/)
[![Docs](.../Docs/badge.svg)](...)
[![codecov](...)](...)
[![License: MIT](...)](LICENSE-MIT)
[![Rust](...)](rust-toolchain.toml)
[![reposix-cli on crates.io](...)](https://crates.io/...)
```

**docs/index.md** (3 badges, was 2): CI + Quality (weekly) + Quality endpoint.

## badges-resolve.py amend

`WAVE_F_PENDING_URLS` cleared to `set()`. The skip-branch in `main()` is now a no-op preserved for future multi-wave URL migrations. Docstring updated to document the post-Wave-F state.

```
$ python3 quality/gates/docs-build/badges-resolve.py
badges-resolve: 8 PASS, 0 FAIL, 0 pending; exit=0
```

All 8 URLs (including the QG-09 endpoint) resolve to HTTP 200 + image content-type immediately. The shields.io endpoint URL returns 200 even before the inner github.io URL is published (shields.io renders an "endpoint error" badge when the inner URL is 404, but still returns image/svg+xml). Once GH Pages publishes the new docs (next deploy), the badge renders the actual rollup status.

## Runner state

```
pre-push: 19 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
weekly:   14 PASS, 0 FAIL, 0 PARTIAL, 3 WAIVED, 2 NOT-VERIFIED -> exit=0
```

## Commits

- `96b28ca` -- feat(p60): publish QG-09 badge -- docs/badge.json + README + docs/index.md endpoint badge

## Self-Check: PASSED

- File `docs/badge.json` FOUND (3 keys: schemaVersion, label, message, color).
- README.md badge count = 8.
- docs/index.md badge count = 3.
- `WAVE_F_PENDING_URLS = set()` confirmed.
- `python3 quality/gates/docs-build/badges-resolve.py` exit 0.
- Commit `96b28ca` FOUND in git log.
- mkdocs auto-include verified.

## Followups (v0.12.1 carry-forward)

- Auto-sync `docs/badge.json` from `quality/reports/badge.json` on every runner emit (MIGRATE-03 carry-forward; manual sync today).
