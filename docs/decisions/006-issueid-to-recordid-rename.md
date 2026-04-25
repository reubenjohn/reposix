---
status: accepted
date: 2026-04-25
supersedes: nothing
---

# ADR-006: Rename `IssueId` / `Issue` → `RecordId` / `Record`

- **Status:** Accepted
- **Date:** 2026-04-25
- **Deciders:** reposix core team (v0.11.0 milestone, opening phase)
- **Supersedes:** nothing
- **Superseded by:** none
- **Scope:** All public types, trait methods, and module names in
  `reposix-core` plus every dependent crate
  (`reposix-sim`, `reposix-confluence`, `reposix-github`, `reposix-jira`,
  `reposix-remote`, `reposix-cli`, `reposix-swarm`). YAML wire format is
  out of scope — the on-disk frontmatter field `id` is unchanged.

## Context

The `Issue` vocabulary in `reposix-core` is a vestige of v0.1, when the
only backend was a GitHub-issues simulator. By v0.10.0 reposix routes
Confluence pages, JIRA tickets, GitHub issues, and (in v0.11.0+) comments
and attachments through the same canonical type. The struct was always a
generic record; only the name lagged.

The `IssueId(u64)` newtype, the `Issue` struct, and the `IssueStatus`
enum each appeared verbatim in error messages, IDE tooltips, doc comments,
and connector authors' search-and-rename muscle memory. New backend
authors hit a "this isn't really an issue" mental footnote on every
Confluence/JIRA call site — the same misnomer that motivated ADR-004's
`IssueBackend` → `BackendConnector` rename one milestone earlier.

The rename was first flagged in `.planning/CATALOG.md` §"Naming
generalization candidates" as a v0.11.0 candidate — deferred out of
v0.10.0 because landing it during a docs milestone would have invalidated
every code example mid-milestone.

## Decision

Hard rename across the workspace. No backward-compatibility aliases
(precedent: ADR-004). The YAML wire format is unchanged — frontmatter
still serializes `id`, `title`, `status`, `parent_id`, etc. exactly as
before, because `RecordId` keeps the `#[serde(transparent)]` `u64` shape
of the prior `IssueId`.

| Before | After |
|---|---|
| `IssueId` | `RecordId` |
| `Issue` | `Record` |
| `IssueStatus` | `RecordStatus` |
| `Error::InvalidIssue` | `Error::InvalidRecord` |
| `path::validate_issue_filename` | `path::validate_record_filename` |
| module `reposix-core/src/issue.rs` | `reposix-core/src/record.rs` |
| `BackendConnector::list_issues` | `list_records` |
| `BackendConnector::get_issue` | `get_record` |
| `BackendConnector::create_issue` | `create_record` |
| `BackendConnector::update_issue` | `update_record` |
| `BackendConnector::delete_issue` | `delete_record` |
| `list_issues_strict` | `list_records_strict` |

`list_issues_strict` was scope-creep folded into the same pass — the
function is callable from the same code paths as the trait methods, and
splitting the rename across two phases would have left the API surface
internally inconsistent.

## Alternatives considered

| Name | Why rejected |
|---|---|
| `EntryId` / `Entry` | "Entry" is overloaded across CS — log entry, journal entry, hashmap entry; collides with `std::collections::HashMap::Entry` in tooltips |
| `DocId` / `Document` | "Doc" implies long-form text; reposix records can be one-line tickets or short comments |
| `ItemId` / `Item` | Too generic; loses the "this is a discrete record from a system of record" signal |
| `WorkItemId` / `WorkItem` | Microsoft DevOps vocabulary; unfamiliar to GitHub/JIRA/Confluence users; same objection ADR-004 raised |
| Keep `IssueId` with a doc-comment caveat | Type names appear in IDE tooltips, error messages, and training data — a misleading name compounds across every connector author's first hour with the codebase |

## Consequences

- **Breaking change.** Out-of-tree connector crates must rename type
  references and trait methods in lockstep. The CHANGELOG `[Unreleased]`
  block documents the migration recipe.
- **No alias layer.** Same precedent as ADR-004. A `pub use Record as
  Issue;` re-export was considered and rejected — it would have left the
  old name discoverable in `cargo doc`, defeating the rename's purpose.
- **YAML round-trip is preserved.** The frontmatter `Frontmatter` DTO is
  internal; its field names (`id`, `title`, etc.) match the backend's
  domain, not the Rust type name.
- **Out of scope (intentional).**
  - On-disk path conventions (`issues/`, `pages/`) remain
    backend-shaped. Those directory names are user-facing and tied to
    each backend's domain language; `issues/` for GitHub communicates
    intent better than a generic `records/` would.
  - `crates/reposix-jira/` prose like `//! Issue → Issue mapping` is
    left intact because the LEFT side IS JIRA's domain language, not
    reposix's.
  - The JIRA API-error string `"Issue Does Not Exist"` is JIRA's, not
    ours, and is preserved verbatim.

## Implementation

Six atomic commits across five waves, plus a fmt drift catch-up:

| Wave | Commit | Description |
|---|---|---|
| A | `2af5491` | rename `IssueId` → `RecordId` (workspace-wide) |
| B | `847cc26` | rename `Issue` type → `Record` (preserves YAML wire format) |
| C | `af39e62` | rename `BackendConnector` methods `*_issue` → `*_record` |
| D | `2dd06a1` | rename remaining `Issue`-prefixed types to `Record` (`issue.rs` → `record.rs`) |
| E | `13da2ca` | update `Record`/`RecordId` rename across user-facing docs |
| F | `4ad8e2a` | `cargo fmt --all` after `Record` rename |

The `trybuild` compile-fail fixture in `reposix-core` had to be re-blessed
in Wave A: rustc's diagnostic squiggle dashes are character-aligned to the
type-name length, so `IssueId` (7 chars) → `RecordId` (8 chars) shifted
the `^^^^^^^^` underline by one column. The expected `.stderr` was
regenerated and re-committed in the same wave.

## Verification

All checks performed at `4ad8e2a`:

- `git grep -nE '\bIssueId\b' crates/` — 0 matches.
- `git grep -nE '\b(IssueStatus|InvalidIssue|validate_issue_filename)\b' crates/`
  — 0 matches.
- `git grep -nE '\b(list_issues|get_issue|create_issue|update_issue|delete_issue)\b' crates/`
  — 0 matches in active code (`reposix-jira` doc comments excluded — see
  Consequences §out-of-scope).
- `cargo test --workspace --locked` — 436 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings` — green.
- `cargo fmt --all --check` — green (after Wave F).
- `bash scripts/banned-words-lint.sh` — green.

Roughly 690 `IssueId` occurrences across `crates/` cleared between
`2af5491^` and `4ad8e2a`. Zero stragglers.

## References

- `.planning/CATALOG.md` §"Naming generalization candidates" — original
  proposal and risk-tier table.
- `.planning/research/v0.11.0-vision-and-innovations.md` — the
  post-rename milestone brainstorm cites this rename as a foundational
  naming move that unblocks the v0.11.0 backend-coverage work.
- `CHANGELOG.md` `[Unreleased]` — `### Changed` entry with the migration
  recipe for out-of-tree connector authors.
- ADR-004 — precedent for hard rename without compatibility aliases
  (`IssueBackend` → `BackendConnector`).
- Commits: `2af5491`, `847cc26`, `af39e62`, `2dd06a1`, `13da2ca`,
  `4ad8e2a`.
