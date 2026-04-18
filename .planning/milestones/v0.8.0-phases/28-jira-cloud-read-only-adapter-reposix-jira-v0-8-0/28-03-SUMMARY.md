---
plan: 28-03
phase: 28
status: complete
commit: b9f397d
wave: 3
---

# Plan 28-03 Summary: ADR-005, docs/reference/jira.md, CHANGELOG v0.8.0, ship prep

## What Was Built

**docs/decisions/005-jira-issue-mapping.md** — ADR-005 covering all 5 decision areas:
1. ID vs Key — numeric `id` used as `IssueId`; `key` preserved in `extensions["jira_key"]`
2. Status + Resolution Mapping — two-field mapping table (`statusCategory.key` + `resolution.name`)
3. Version Synthesis — `fields.updated.timestamp_millis() as u64`; `StrongVersioning: false`
4. ADF Description Stripping — plain-text walker in `adf.rs`; no Markdown conversion
5. Attachments and Comments — deferred to future phase with rationale

**docs/reference/jira.md** — User guide covering:
- Required env vars (`JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`)
- Egress allowlist setup
- `list` and `mount` usage with `--backend jira`
- `--no-truncate` semantics
- Issue frontmatter example with `extensions` block
- Phase 28 limitations

**CHANGELOG.md** — Phase 28 entries added to existing `[v0.8.0]` section:
- `reposix-jira` crate, `list/mount --backend jira`, ADR-005, `docs/reference/jira.md`, extensions
- Rate-limit backoff documented under `### Changed`
- `StrongVersioning: false` and write-stub notes under `### Notes`

**scripts/tag-v0.8.0.sh** — 7-guard annotated tag script (user-driven, not invoked)

**cargo fmt --all** — import ordering and struct literal formatting fixed across
`reposix-jira`, `reposix-cli`, `reposix-confluence`, `reposix-fuse`, `reposix-swarm`

**.planning/STATE.md** — cursor advanced to Phase 28 complete

## Green Gauntlet Results

| Check | Result |
|-------|--------|
| `cargo test --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all --check` | PASS |

Workspace version: 0.8.0 (already correct from Phase 27)

## Deviations

- CHANGELOG [v0.8.0] section already existed (Phase 27 started it); Phase 28 entries
  were appended to the existing section rather than creating a new one.
- `cargo fmt --all` was needed to fix pre-existing style issues introduced in Wave 1/2
  (struct literals on one line, import ordering). All fixes are mechanical.

## Self-Check: PASSED
