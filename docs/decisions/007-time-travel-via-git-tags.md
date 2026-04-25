---
status: accepted
date: 2026-04-25
supersedes: nothing
---

# ADR-007: Time-travel via private git tags per `Cache::sync`

- **Status:** Accepted
- **Date:** 2026-04-25
- **Deciders:** reposix core team (v0.11.0 milestone)
- **Supersedes:** nothing
- **Superseded by:** none
- **Scope:** `reposix-cache` (tag emission, list/at API), `reposix-cli`
  (`history` and `at` subcommands), the cache's bare-repo `git config`.

## Context

`.planning/research/v0.11.0-vision-and-innovations.md` §3b is the design
brief: every `Cache::sync` should write a deterministic ref pointing at
the synthesis commit for that sync, so an agent or human can
`git checkout <ref>` and inspect what reposix observed at any past
moment. The §6 originality audit flagged this as the brainstorm's
**highest-novelty** entry — no prior art surfaces for "tag every external
sync as a first-class git ref." git-bug uses Lamport timestamps
internally; jirafs has snapshot tarballs. Neither exposes the per-sync
state as a checkable git ref.

Without sync tags, "what did issue PROJ-42 look like last Tuesday" required
manual reconstruction from the audit log plus a backend HTTP call, neither
of which composes with `git diff`. The audit log answers *what reposix did*;
sync tags answer *what reposix observed*. Together they are a fully
replayable history.

## Decision

1. **Namespace.** Sync tags live at `refs/reposix/sync/<ISO8601-no-colons>`
   inside the cache's bare repo (e.g.
   `refs/reposix/sync/2026-04-25T01-13-00Z`). Colons are illegal in git ref
   names; we substitute `-` and round-trip via
   `parse_sync_tag_timestamp`.

2. **Emission.** `Cache::tag_sync(commit, ts)` writes the ref via gix's
   `edit_reference` API with `PreviousValue::Any` (so re-tagging the same
   timestamp is idempotent). It is invoked from both branches of
   `Cache::sync`: the seed-path (`build_from`) and the delta path. Best-
   effort — a tag-write failure logs WARN but does not poison the sync
   (the SQL transaction with `meta.last_fetched_at` has already
   committed).

3. **Audit pairing.** Every `tag_sync` call appends a row to
   `audit_events_cache` with `op='sync_tag_written'`, `oid=<commit>`,
   `reason=<full-ref-name>`. The CHECK constraint in
   `cache_schema.sql` is extended to include the new op. Append-only
   triggers unchanged.

4. **Privacy from the agent.** The cache's bare repo gets
   `transfer.hideRefs = refs/reposix/sync/` set on `Cache::open` (idempotent
   `git config --add` if not already present). This hides the namespace
   from `git upload-pack --advertise-refs`, so the helper's protocol-v2
   advertisement does NOT propagate sync tags to the agent's working tree.
   The agent only ever sees `refs/heads/main`. Verified by integration
   test (`helper_does_not_export_sync_tags`).

5. **CLI surface.** `reposix history [<path>] [--limit N]` lists sync
   tags most-recent first. `reposix at <ts> [<path>]` returns the closest
   tag at-or-before the target timestamp. Both subcommands resolve the
   cache path from `remote.origin.url` of the working tree (same logic
   `reposix doctor` uses).

## Consequences

**Positive.**
- Time-travel via `git checkout` is the dark-factory teaching mechanism in
  action: an agent that knows git already knows how to inspect history.
  Zero reposix-specific learning required.
- Audit log + sync tags = fully replayable observation history. Forensics
  on "when did this field flip" becomes a `git log` walk over the bare
  repo's `refs/reposix/sync/*` namespace.
- Generalisable beyond reposix — any partial-clone promisor remote could
  do this. ADR-007 may end up cited from outside the project.

**Negative / cost.**
- Cache size grows linearly with sync count. Each tag is 41 bytes loose
  (less when packed). A repo synced hourly for a year accumulates ~360 KB
  of refs. Below the noise floor for normal use, but a future
  `reposix gc` (planned for v0.12.0) should support TTL-based pruning of
  old sync tags.
- One extra `git config --add` invocation per `Cache::open` (idempotent
  fast path skips on subsequent opens). Negligible — `Cache::open` is
  already a multi-syscall bootstrap.

**Security.** No new HTTP. No new threat surface — the namespace is
private to the cache's bare repo, not a target the helper exports.
The append-only triggers on `audit_events_cache` continue to apply to
the new `sync_tag_written` rows.

## Alternatives considered

- **`refs/tags/...`** — would expose sync points to the agent's `git tag -l`
  and to `git upload-pack`'s default advertisement. Rejected: the agent
  has no business with these; they're cache forensics state.
- **Commit messages with timestamp** — already done (the synthesis commit
  message embeds the RFC-3339 ts). Rejected as the *primary* mechanism
  because there's no efficient lookup; finding "the sync at-or-before T"
  would require scanning the entire commit history.
- **Separate tag DB (e.g. a SQLite table mapping ts → commit)** — works
  but duplicates state. Git's ref store IS a tag DB; using it directly
  means `git diff <tag1> <tag2>` works without a custom resolver.
- **One ref per sync at a different namespace** (`refs/sync/...`,
  `refs/reposix-sync/...`) — bikeshed; we picked
  `refs/reposix/sync/` because it sits inside the existing
  `refs/reposix/origin/main` namespace already used by Phase 32's helper
  refspec.

## Validation

- 6 unit/integration tests in `crates/reposix-cache/tests/sync_tags.rs`
  (tag creation, multi-sync ordering, sort, at-lookup, audit row,
  helper-doesn't-export proof).
- 2 end-to-end CLI tests in `crates/reposix-cli/tests/history.rs`
  (`reposix history`, `reposix at`).
- Hidden-from-helper invariant verified by spawning real
  `git upload-pack --advertise-refs` against the cache's bare repo and
  asserting no `refs/reposix/sync/` line appears in the advertisement.

## References

- `.planning/research/v0.11.0-vision-and-innovations.md` §3b — design
  intent, novelty audit (§6), and prior-art search.
- `crates/reposix-cache/src/sync_tag.rs` — module home.
- `docs/how-it-works/time-travel.md` — user-facing explanation.
- `CHANGELOG.md` `[Unreleased]` — release-note line.
