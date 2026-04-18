# Phase 27 CONTEXT — IssueBackend → BackendConnector rename + Issue.extensions field (v0.8.0)

> Status: Auto-generated from ROADMAP.md Phase 27 entry. Fully specified — no discuss-phase needed.
> Milestone: v0.8.0 (breaking release — trait rename + new Issue field in public API)
> Requirements: RENAME-01, EXT-01

## Phase Boundary

Two simultaneous changes:
1. Rename the `IssueBackend` trait to `BackendConnector` across all crates and call-sites.
2. Add `extensions: BTreeMap<String, serde_yaml::Value>` to the `Issue` struct.
3. Write ADR-004 documenting the rename decision.
4. Bump workspace version to `0.8.0`.

**No behaviour changes** — this is a pure rename + additive field. All existing tests must stay green.

## Implementation Decisions

### Trait rename — locked

- Old name: `IssueBackend` (in `reposix-core/src/lib.rs` or wherever it lives)
- New name: `BackendConnector`
- Impls to update: `SimBackend`, `GithubReadOnlyBackend`, `ConfluenceBackend`
- Call-sites to update:
  - `reposix-fuse/src/fs.rs` — ≈20 sites
  - `reposix-cli/src/list.rs`, `mount.rs`, `refresh.rs`
  - `reposix-remote` (wherever it dispatches to a backend)
  - `reposix-swarm` (wherever it dispatches)
- Verification: `grep -r "IssueBackend" crates/ docs/ --include="*.rs" --include="*.md"` must return zero matches after the rename (excluding planning docs and CHANGELOG history)

### Issue.extensions field — locked

- Field: `extensions: BTreeMap<String, serde_yaml::Value>`
- Default: empty via `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`
- Location: `Issue` struct in `reposix-core`
- Frontmatter impact: flows through `frontmatter::render` and `frontmatter::parse` without schema churn (serde handles it transparently)
- Required test: roundtrip `Issue { extensions: {"foo": Value::Int(42), "bar": Value::String("x")} }` → serialize → parse → assert equality

### ADR-004 — locked

- Path: `docs/decisions/004-backend-connector-rename.md`
- Must document:
  - Rename rationale: `IssueBackend` is a misnomer (Confluence pages are not "issues"; trait must be neutral)
  - Alternatives considered: `RemoteBackend`, `TrackerBackend`, `WorkItemBackend`, `Connector`
  - Why `BackendConnector` won: neutral across issue/page/content domains; aligns with Phase 12's "Connector protocol" vocabulary
- ADR "Decision" section is the authoritative record — do not hedge

### Version bump — locked

- Cargo.toml workspace version: `0.7.0` → `0.8.0`
- Regenerate Cargo.lock via `cargo check --workspace`
- CHANGELOG: promote `[Unreleased]` → `[v0.8.0] — 2026-04-16` (or current date)

### Claude's Discretion

- Plan wave structure (rename in one wave or split rename + extensions into separate waves)
- Whether to update any doc references to `IssueBackend` found by the grep-verify step (Phase 26 clarity review may have introduced some; the plan should sweep them)
- Exact order of changes within the rename (trait definition first, then impls, then call-sites is the natural order)

## Canonical References

- `crates/reposix-core/src/` — trait definition location (find IssueBackend here)
- `crates/reposix-fuse/src/fs.rs` — ≈20 call-sites
- `crates/reposix-cli/src/` — list.rs, mount.rs, refresh.rs dispatch
- `crates/reposix-confluence/src/lib.rs` — ConfluenceBackend impl
- `crates/reposix-github/src/lib.rs` — GithubReadOnlyBackend impl (if exists)
- `crates/reposix-sim/src/` — SimBackend impl
- `docs/decisions/003-nested-mount-layout.md` — predecessor ADR style guide
- `REQUIREMENTS.md §v0.8.0` — RENAME-01 and EXT-01 definitions

## Deferred

- JIRA adapter (`reposix-jira`) — Phase 28
- Write path for JIRA — Phase 29
- `BackendFeature::Hierarchy` usage in JiraBackend — Phase 28
