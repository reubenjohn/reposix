# Phase 34: Push path — conflict detection + blob limit guardrail - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning
**Mode:** Auto-generated (discuss skipped via workflow.skip_discuss=true)

<domain>
## Phase Boundary

Make the existing `export` handler conflict-aware and the Phase 32 `stateless-connect` handler scope-bounded. Two independent guardrails ship in this phase:

1. **Push-time conflict detection** — inside `handle_export`, compare the agent's commit base against the current backend version. On mismatch, emit `error refs/heads/main fetch first` (canned status — git renders the standard "perhaps a `git pull` would help" hint) plus a detailed diagnostic via stderr. The reject path drains the incoming stream and never touches the bare cache: no partial state.
2. **Blob-limit guardrail** — count `want <oid>` lines per `command=fetch` request; refuse if the count exceeds `REPOSIX_BLOB_LIMIT` (default 200). The stderr error message is verbatim self-teaching: `"error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with `git sparse-checkout set <pathspec>` and retry."`. This is the dark-factory teaching mechanism — an unprompted agent reads the error, runs `git sparse-checkout`, and self-corrects.

A third invariant rides along: the frontmatter field allowlist (`id`, `created_at`, `version`, `updated_at` stripped before REST) is enforced at the `Tainted -> Untainted` conversion site, so an attacker-authored issue body with `version: 999999` does not change the server version.

</domain>

<decisions>
## Implementation Decisions

### Operating-principle hooks (non-negotiable — per project CLAUDE.md)

- **Tainted-by-default (OP-2).** The frontmatter sanitize step is the explicit `Tainted<Vec<u8>> -> Untainted<Issue>` conversion. The allowlist strip happens inside that conversion; no other code path is permitted to send issue content to a backend without going through this sanitizer. Compile-fail test asserts this.
- **Audit log non-optional (OP-3).** Every push attempt — accept and reject — gets one audit row. Schema: `(ts, backend, project, ref, files_touched, decision, reason)`. The reject path's audit row is written even though the bare cache is not touched.
- **ROI awareness (OP-3 user-global).** The blob-limit error message is the cheapest possible regression net for "agent does naive `git grep`." The message names `git sparse-checkout` literally so the agent's next action is obvious. No system-prompt instructions, no in-context tool docs — the error message is the documentation.
- **Egress allowlist.** REST writes go through the Phase 31 `reposix_core::http::client()` factory. No new `reqwest::Client` in this phase.

### Push-reject status string (locked)

Helper emits **`error refs/heads/main fetch first`** as the status line — the canned string from `transport-helper.c::push_update_ref_status` (push-path-stateless-connect-findings.md Q2). This produces git's standard hint. The detailed diagnostic ("reposix: issue-2444.md was modified on backend; backend version=5, your base=4") goes to stderr via the existing `diag()` channel (`crates/reposix-remote/src/main.rs::diag`).

Rationale: standard-UX takes precedence over reposix-specific cleverness. The agent already knows what "fetch first" means; it does not know what "reposix: ... since last fetch" means without prior context. Both messages are emitted; git surfaces the canned one in the `[remote rejected]` line and the diagnostic on stderr.

### Reject-path atomicity (locked)

Reject path:

1. Parse fast-import stream in memory (existing `crates/reposix-remote/src/fast_import.rs`).
2. For each changed file, call `BackendConnector::get_issue(id)` and compare the base version.
3. On first conflict detected: do NOT fail-fast yet — drain the rest of the incoming stream into `/dev/null` to avoid SIGPIPE on the git side.
4. Emit `error refs/heads/main fetch first` + stderr diagnostic.
5. Bare cache is untouched. `git fsck` clean after reject (assertable).
6. Audit row: `decision="reject", reason="conflict_on_<id>"`.

Accept path:

1. Apply REST writes via the existing `crates/reposix-remote/src/diff.rs::plan` translation.
2. Update bare-repo cache (the new HEAD becomes `refs/reposix/main`).
3. Emit `ok refs/heads/main`.
4. REST + cache update is one atomic step: REST first, then cache. If cache write fails after successful REST, log a WARN and emit a compensating bare-ref rollback (`git update-ref refs/reposix/main <old>`). Architecture-pivot-summary §7 Q2 acknowledges this as an open question; v0.9.0 ships REST-first with WARN-on-divergence — full reconciliation is v0.10.0.
5. Audit row: `decision="accept", reason="<files_touched_summary>"`.

### Frontmatter field allowlist (locked)

Stripped fields: `id`, `created_at`, `updated_at`, `version`. These are server-controlled. The strip happens inside the `Tainted -> Untainted` conversion in `reposix-core`. Test name: `frontmatter_strips_server_controlled_fields`. Asserts that an inbound issue body with `version: 999999` and `id: hijacked` produces a REST write whose `version` and `id` are absent (the server stays authoritative).

### Blob-limit guardrail (locked)

- **Counting locus:** inside the Phase 32 `stateless-connect` handler, after the `command=fetch` request is fully read but before it is forwarded to `upload-pack`. Count occurrences of `want ` (including the trailing space) at the start of pkt-line payloads.
- **Limit value:** `REPOSIX_BLOB_LIMIT` env var, default `200`. Read once at helper startup; cached in a `OnceCell`.
- **Refusal protocol:** write the verbatim stderr message (below), exit non-zero. Do NOT forward the request to `upload-pack` (no partial-pack response).
- **Stderr message (verbatim — including backticks):** `error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with \`git sparse-checkout set <pathspec>\` and retry.`
  - The mention of `git sparse-checkout` is non-optional — it is the dark-factory teaching mechanism. An agent that does not recognize the literal command in the error message will miss the recovery path.
- **Audit row:** `op="blob_limit_refused", files_touched=N, decision="reject", reason="exceeds_limit_M"`.

### Test surface

- `stale_base_emits_fetch_first` — second-writer mutates issue mid-flight; agent push emits `error refs/heads/main fetch first` and the bare cache is untouched.
- `accept_writes_rest_then_cache` — happy-path push updates backend then cache; ordering verified via fault injection.
- `frontmatter_strips_server_controlled_fields` — inbound `version: 999999` does not reach the REST call body.
- `blob_limit_refuses_at_201` — `REPOSIX_BLOB_LIMIT=200`; a 201-want fetch is refused with the verbatim message.
- `blob_limit_below_200_passes` — 5-want fetch with `REPOSIX_BLOB_LIMIT=5` passes; 6-want with same env fails.
- `audit_row_per_push_attempt` — both accept and reject paths produce exactly one audit row.
- `git_fsck_clean_after_reject` — `git fsck` against the cache bare repo returns clean after a rejected push.
- `chaos_kill_between_rest_and_cache` — kill-9 during the REST→cache window; on retry the cache converges to the post-REST state (or compensating ref rollback fires).

### Claude's Discretion

Whether to use a streaming state-machine parser for the fast-import stream (architecture-pivot-summary §7 Q6) or to keep the in-memory parser from v0.8.0 is at Claude's discretion. Streaming is more efficient for large pushes; in-memory is simpler. v0.9.0 may keep in-memory; the streaming parser is a v0.10.0+ optimization.

The exact format of the diagnostic stderr line beyond the canned `fetch first` is at Claude's discretion — must include enough info for an agent to investigate (issue ID, expected version, observed version), but the format is not user-facing tooling input.

</decisions>

<code_context>
## Existing Code Insights

### Reusable assets

- `crates/reposix-remote/src/main.rs` — has the `export` capability dispatch + `diag()` stderr helper. Insert conflict detection between `parse_export_stream` and the REST plan execution.
- `crates/reposix-remote/src/fast_import.rs::parse_export_stream` — existing v0.8.0 parser. Phase 34 must NOT regress this; conflict detection runs *after* parsing on the parsed value.
- `crates/reposix-remote/src/diff.rs::plan` — translates parsed export into REST POST/PATCH/DELETE. Conflict check runs *before* `plan` executes its REST calls.
- Phase 16 audit-log patterns — every backend mutation already writes an audit row in v0.6.0+. Phase 34 adds new `op` values (`push_accept`, `push_reject`, `blob_limit_refused`); no schema migration needed.
- Phase 21 `HARD-03` kill-9 chaos pattern — reuse for the REST→cache atomicity test.
- `reposix_core::Tainted<T>` and `Untainted<T>` newtypes (Phase 31) — the type-system trust boundary the allowlist strip happens at.
- POC reject-path code: `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py` (`REPOSIX_POC_REJECT_PUSH` env-var path); trace `.planning/research/v0.9-fuse-to-git-native/poc/poc-push-trace.log`.

### Established patterns

- Helper protocol errors: stderr via `diag()` + non-zero exit. Git surfaces stderr to user.
- Allowlist enforcement at trust-boundary conversion sites (Phase 27 / Phase 31 patterns).
- `#![forbid(unsafe_code)]` + clippy::pedantic at crate root.

### Integration points

- Phase 31's `Cache` is the bare-repo target for accept-path writes. Cache exposes a `commit_synthesized(refs/reposix/main, new_oid)` method (or equivalent — Phase 31's discretion).
- Phase 32's `stateless-connect` is where the blob-limit counter sits. The two guardrails (conflict on push, blob limit on fetch) live in different code paths but share the audit-row writer.
- Phase 33's `last_fetched_at` is consulted on the accept path to advance the timestamp after a successful push (avoids re-fetching what we just wrote).

</code_context>

<specifics>
## Specific Ideas

- Backticks in the stderr blob-limit message are literal — they render as code formatting in agent terminals that support it, and look correct in plaintext too.
- `REPOSIX_BLOB_LIMIT=0` is reserved as "disable" (no limit). Tests cover `=0`, `=5`, default `=200`.
- The conflict-detection comparison is currently coarse (any field mismatch on the backend issue → reject). A finer-grained per-field merge is v0.10.0+ work; v0.9.0 ships pessimistic.
- Audit row's `files_touched` column is a comma-separated list of issue IDs (or a JSON array — Claude's discretion). Keep deterministic ordering for diff-friendly logs.
- Stderr diagnostic on conflict is NOT prefixed with `reposix:` — it follows git's helper convention of free-form context lines on stderr.
- The verbatim error string (with backticks around `git sparse-checkout set <pathspec>`) is committed as a `const` in the helper crate so a regression test can string-match against it.

</specifics>

<deferred>
## Deferred Ideas

- Per-field merge on conflict (vs. all-or-nothing reject) — v0.10.0+.
- Streaming state-machine parser for fast-import (architecture-pivot-summary §7 Q6) — v0.10.0+.
- Background reconciler for the REST→cache divergence window — v0.10.0+ (architecture-pivot-summary §7 Q2).
- Non-issue file handling on push (e.g., changes to `.planning/`-like paths) — architecture-pivot-summary §7 Q4. v0.9.0 silently commits to bare cache only; no REST write. Documented behaviour, not a bug.
- Custom reject messages with reposix-specific reason codes — kept for v0.10.0 once the agent UX baseline is stable. v0.9.0 uses canned `fetch first` only.

</deferred>
</content>
