---
phase: 33
plan: 01
title: "BackendConnector::list_changed_since — trait method + 4 backend impls"
status: complete
---

# Phase 33 Plan 01 — Summary

## What shipped

Added `BackendConnector::list_changed_since(project, since) -> Vec<IssueId>`
as a default-provided trait method, taught the simulator's
`GET /projects/:slug/issues` handler to honor a `?since=<RFC3339>` query
parameter, and overrode the trait method on all four backends
(`SimBackend`, `GithubReadOnlyBackend`, `ConfluenceBackend`, `JiraBackend`)
to use each backend's native incremental query.

## Tasks

- **01-T01** — Default trait method + test (`reposix-core`).
- **01-T02** — Sim route accepts `?since=`; absent/empty/malformed cases tested.
- **01-T03** — `SimBackend` override sends `?since=` on the wire.
- **01-T04** — `GithubReadOnlyBackend` override uses GitHub's native `?since=`.
- **01-T05** — `ConfluenceBackend` override uses CQL `lastModified > "<datetime>"`.
- **01-T06** — `JiraBackend` override uses JQL `updated >= "<datetime>"`.
- **01-T07** — Workspace gate (check / clippy / test) clean.

## Tests added

- `reposix-core::backend::tests::default_list_changed_since_filters_via_list_issues`
- `reposix-core::backend::sim::tests::list_changed_since_sends_since_query_param`
- `reposix-core::backend::sim::tests::list_changed_since_returns_ids_only`
- `reposix-sim::routes::issues::tests::list_issues_with_since_filters_correctly`
- `reposix-sim::routes::issues::tests::list_issues_absent_since_returns_all`
- `reposix-sim::routes::issues::tests::list_issues_malformed_since_returns_400`
- `reposix-github::tests::github_list_changed_since_sends_since_param_and_returns_ids`
- `reposix-confluence::tests::confluence_list_changed_since_sends_cql_lastmodified`
- `reposix-confluence::tests::confluence_list_changed_since_strips_quotes_from_project`
- `reposix-jira::tests::jira_list_changed_since_sends_updated_jql`
- `reposix-jira::tests::jira_list_changed_since_strips_quotes_from_project`

Net new tests: **+11**.

## Key decisions

- **Default impl is belt-and-suspenders**: forwards to `list_issues` and
  filters in memory. All four target backends override.
- **IDs only on the wire, not full Issues**: callers (Plan 02's `Cache::sync`)
  decide whether to materialize blobs eagerly or lazily — symmetric with the
  Phase 31 lazy-blob design.
- **Sim returns 400 on malformed `since`** (not 500). Matches existing
  validation patterns in `routes/issues.rs::create_issue`.
- **CQL/JQL injection defense**: project slug is `.replace('"', "")`-stripped
  before interpolation into CQL/JQL string literals on Confluence and JIRA.
  JSON request bodies are serde-serialized rather than concatenated.
- **No new Cargo dependencies** added. Confluence URL-encodes via
  `url::Url::query_pairs_mut()` (the existing pattern in `resolve_space_id`).

## Commits

- `feat(33-01): list_changed_since trait method with default impl` — 5512124
- `feat(33-01): sim list_issues honors ?since=<RFC3339> query param` — 1211ddf
- `feat(33-01): SimBackend overrides list_changed_since with ?since= wire call` — 446688b
- `feat(33-01): GithubReadOnlyBackend::list_changed_since with native ?since=` — 0924738
- `feat(33-01): ConfluenceBackend::list_changed_since via CQL search` — 2989e4c
- `feat(33-01): JiraBackend::list_changed_since via JQL updated>=` — fc85b5e

## Verification commands

```bash
cargo check --workspace                                   # exits 0
cargo clippy --workspace --all-targets -- -D warnings     # exits 0
cargo test --workspace                                    # all green
```

## Hand-off to Plan 02

Plan 02 (`Cache::sync`) calls `self.backend.list_changed_since(project, last_fetched_at)`
inside the cache's delta-sync flow. The trait method is dyn-compatible
(verified by `reposix-core::backend::tests::_assert_dyn_compatible`), so
existing `Arc<dyn BackendConnector>` callers work unmodified.
