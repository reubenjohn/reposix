---
status: accepted
date: 2026-04-16
supersedes: nothing
---

# ADR-004: Rename IssueBackend → BackendConnector

- **Status:** Accepted
- **Date:** 2026-04-16
- **Deciders:** reposix core team (v0.8.0 milestone planning)
- **Supersedes:** nothing
- **Superseded by:** none
- **Scope:** The `BackendConnector` trait in `reposix-core` and all
  implementing crates (`reposix-sim`, `reposix-confluence`, `reposix-github`,
  `reposix-jira` from Phase 28). Code lives in
  `crates/reposix-core/src/backend.rs`.

## Context

The trait was introduced in Phase 8 as `IssueBackend` — an apt name when
the only concrete implementation was `SimBackend` talking to a simulated
GitHub-style issue tracker. By v0.7.0 the codebase hosts two additional
implementations: `ConfluenceBackend` (pages, not issues) and
`GithubReadOnlyBackend`. Phase 28 adds `JiraBackend`. The trait name
`IssueBackend` is a misnomer: Confluence pages and JIRA items are called
"pages" and "issues" in their respective APIs, but the reposix trait is
a neutral normalization boundary that any content-tracker backend can
implement. The name misleads future contributors about the trait's scope.

## Decision

Rename `IssueBackend` → `BackendConnector` (v0.8.0, breaking release).

**Why `BackendConnector`?**

- *Neutral across domains.* "Connector" implies a pluggable adapter for any
  remote system, not specifically an issue tracker.
- *Vocabulary alignment.* Phase 12 ("Connector protocol") introduced the
  concept of a `reposix-connector-<name>` subprocess ABI. `BackendConnector`
  harmonises the in-process trait with that planned external protocol name,
  making the conceptual model consistent whether a backend is compiled-in
  or subprocess-based.
- *Single-word clarity.* `BackendConnector` is unambiguous; `Connector`
  alone is too generic (conflicts with `TcpConnector`, `DbConnector`, etc.
  in the broader ecosystem).

## Alternatives considered

| Name | Why rejected |
|------|-------------|
| `RemoteBackend` | "Remote" connotes network-only; sim is local; also conflicts with the git-remote-helper concept already in the codebase |
| `TrackerBackend` | "Tracker" implies issue tracking specifically; rules out docs, wikis, linear boards |
| `WorkItemBackend` | Microsoft DevOps vocabulary; unfamiliar to GitHub/Jira/Confluence users |
| `Connector` | Too generic; conflicts with common adapter patterns in async runtimes |
| `IssueBackend` (keep) | Misleads on Confluence/JIRA adapters; every new backend would need a "this isn't really an issue" mental footnote |

## Consequences

- **Breaking change.** All implementing crates (`reposix-confluence`,
  `reposix-github`, future `reposix-jira`) and all call-sites
  (`reposix-fuse`, `reposix-cli`, `reposix-remote`, `reposix-swarm`) must
  update the symbol. No backward-compat alias is provided (pre-1.0
  precedent: `ConfluenceReadOnlyBackend` was renamed to `ConfluenceBackend`
  in Phase 16 without an alias).
- **Mechanical.** No logic changes accompany the rename. A project-wide
  `IssueBackend` → `BackendConnector` substitution in `.rs` files completes
  the migration.
- **Grep-verifiable.** `grep -r "IssueBackend" crates/ --include="*.rs"`
  must return zero matches after the rename (enforced in Phase 27 CI).
