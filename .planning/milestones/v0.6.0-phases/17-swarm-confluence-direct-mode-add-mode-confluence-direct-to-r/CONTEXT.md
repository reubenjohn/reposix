# Phase 17 CONTEXT — Swarm Confluence-Direct Mode

> Status: queued (session 6, 2026-04-14). Not yet planned or executed.
> Follows Phase 16. Milestone: v0.6.0.

## Phase identity

**Name:** Swarm confluence-direct mode — add `--mode confluence-direct` to `reposix-swarm` using `SimDirectWorkload` as template.

**Scope tag:** v0.6.0 (feature scope — adds a new swarm workload against the Confluence backend).

**Addresses:** HANDOFF.md session-4 §"Known open gaps" — "Swarm harness against Confluence (`--mode confluence-direct`) — Phase 11 stretch goal; deferred because rate limits make a 50-client 30s run expensive."

## Goal (one paragraph)

Add a `--mode confluence-direct` workload to `reposix-swarm` that exercises `ConfluenceBackend` directly (no FUSE overhead), mirroring the existing `SimDirectWorkload` pattern. The workload should spawn N async clients, each calling `list_issues` + `get_issue` in a loop for a configurable duration, then print a summary of total operations, errors, and requests/sec. Rate-limit handling must be first-class: the workload should respect `Retry-After` headers rather than hammering through 429s, and the summary should show how many requests were rate-limited. This proves that Phase 14's `IssueBackend` trait truly generalizes across backends under concurrent load.

## Source design context (migrated from HANDOFF.md)

### From session-5 §Cluster C

> **Cluster C — Swarm `--mode confluence-direct`.** Not started (~300 LoC warm-up). Exercises Phase 14's refactor against Confluence + proves the trait truly generalizes. Even cheaper now that rate-limiting is well-understood from Phase 9. Ships v0.5.1 (bugfix-size but feature-flavored) or folds into a bigger release.

### From session-4 §"Known open gaps"

> **Swarm harness against Confluence** (`--mode confluence-direct`) — Phase 11 stretch goal; deferred because rate limits make a 50-client 30s run expensive.

## Design questions to resolve

1. **Rate-limit strategy:** With 50 clients hitting a real Confluence tenant, 429s are likely. Should `ConfluenceDirectWorkload` automatically back off and retry, or just count 429s as errors and continue?
2. **Client count for CI:** 50 clients × 30s is too much for a CI job against a real tenant. Should the CI integration test use 3 clients × 5s (against wiremock) and only run the real-tenant test with `--ignored`?
3. **Write operations:** Should `--mode confluence-direct` include write ops (create/update/delete) if Phase 16 has shipped? Or keep it read-only in Phase 17 and add write-contention in Phase 21 (OP-7 hardening)?
4. **Metrics output format:** Match `SimDirectWorkload` summary format exactly (so scripts that parse swarm output don't need branching)?

## Canonical refs

- `crates/reposix-swarm/src/workloads/sim_direct.rs` — `SimDirectWorkload` implementation; this is the template.
- `crates/reposix-swarm/src/main.rs` — CLI dispatch; add `--mode confluence-direct` arm.
- `crates/reposix-swarm/tests/mini_e2e.rs` — existing mini E2E test (5 clients, 1.5s); add a parallel test for `ConfluenceDirectWorkload` against wiremock.
- Phase 9 (`09-swarm-harness/`) — original swarm harness implementation.
