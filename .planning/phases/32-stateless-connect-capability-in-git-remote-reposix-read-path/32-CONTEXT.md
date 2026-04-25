# Phase 32: `stateless-connect` capability in `git-remote-reposix` (read path) - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning
**Mode:** Auto-generated (discuss skipped via workflow.skip_discuss=true)

<domain>
## Phase Boundary

Port the Python POC's `stateless-connect` handler to Rust inside `crates/reposix-remote/`. After this phase, `git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone` succeeds end-to-end with lazy blob loading, tunnelling protocol-v2 traffic to the Phase 31 `reposix-cache` bare repo. The existing `export` capability for push must keep working in the same binary (hybrid).

This is a transport-layer phase. No backend semantics change. No cache semantics change. The only public surface is two new advertised capabilities (`stateless-connect` plus `object-format=sha1`) and the protocol bytes the helper writes to git's stdin/stdout. Push behaviour from v0.8.0 must be unchanged.

The interception point on disk is `crates/reposix-remote/src/main.rs` (capability advertisement) and a new module that proxies protocol-v2 frames to a `git upload-pack --stateless-rpc` subprocess running against the Phase 31 cache. Per architecture-pivot-summary §3, this is the "tunnel pattern" (the helper is a thin pipe; not the "synthesis pattern" where we'd implement protocol-v2 server-side ourselves).

</domain>

<decisions>
## Implementation Decisions

### Operating-principle hooks (non-negotiable — per project CLAUDE.md)

- **Subagent delegation per CLAUDE.md.** Use `gsd-phase-researcher` for the protocol-v2 stateless-connect Rust port — non-trivial, three protocol gotchas, easy to over-research in the orchestrator. Orchestrator routes; subagent codes.
- **Ground truth obsession (OP-6).** Verify against a real `git clone --filter=blob:none` against the Rust helper, not against unit-test mocks. Mocks for protocol framing are fine, but the success criterion is "real `git` binary clones successfully and missing-blob count drops with each `cat-file`."
- **Close the feedback loop (OP-1).** Capture a fresh trace log analogous to POC `poc-helper-trace.log` and commit it under `.planning/research/v0.9-fuse-to-git-native/rust-port-trace.log`. Without the trace artifact, "it works" is unverifiable in future sessions.
- **Audit log non-optional (OP-3).** Every helper invocation writes a `command=fetch` audit row to the cache DB (one row per RPC turn, with the count of `want` lines and total response bytes — useful for the Phase 34 blob-limit telemetry).
- **No hidden state (OP-4).** Helper reads `REPOSIX_CACHE_DIR` (or default `$XDG_CACHE_HOME/reposix/<backend>-<project>.git`) and resolves the cache path deterministically. No session-local /tmp paths.

### Three protocol gotchas — locked from POC

These are the gotchas surfaced in `partial-clone-remote-helper-findings.md` Q2 and `architecture-pivot-summary` §3 "Three protocol gotchas". They are NOT documented in `gitremote-helpers.adoc`; getting any of them wrong produces silent hangs or `fatal: expected flush after ref listing`. Each MUST have a named regression test:

1. **Initial advertisement does NOT terminate with `0002`** — only `0000` (flush). Send the bytes from `upload-pack --advertise-refs --stateless-rpc` verbatim. Do not append a response-end packet. (Test name suggestion: `initial_advertisement_ends_with_flush_only`.)
2. **Subsequent RPC responses DO need trailing `0002`** — after each response pack, write the bytes from `upload-pack --stateless-rpc` followed by the response-end marker. (Test name suggestion: `rpc_response_appends_response_end`.)
3. **Stdin reads in binary mode throughout.** Read the entire helper protocol via a `BufReader<Stdin>` consistently — no mixing of line-mode (`read_line`) and byte-mode reads on the same stream. (Test name suggestion: `stdin_is_binary_throughout`.)

### Hybrid dispatch logic (locked)

Capability advertisement after `capabilities`:

```
import
export
refspec refs/heads/*:refs/reposix/*
stateless-connect
object-format=sha1
```

Dispatch:

- `stateless-connect git-upload-pack` → tunnel to `upload-pack --stateless-rpc` against the Phase 31 cache bare repo.
- `stateless-connect git-upload-archive` → same upstream branch (low priority; can return error if not implemented).
- `stateless-connect git-receive-pack` → never reached. Per `transport-helper.c::process_connect_service` (push-path-stateless-connect-findings.md Q1), git's dispatch is service-name-gated and excludes receive-pack. So push falls through to the existing `export` handler.
- `export` → unchanged from v0.8.0; existing fast-import parsing in `crates/reposix-remote/src/fast_import.rs` is preserved.
- `import` is kept for one release cycle then deprecated (architecture-pivot-summary §7 Q5).

### Refspec namespace (locked)

`refs/heads/*:refs/reposix/*` — non-optional. The empty-delta bug from `push-path-stateless-connect-findings.md` "Bug 1" requires the private namespace. The current `crates/reposix-remote/src/main.rs` already advertises this; regression test must assert it does not flip to `refs/heads/*:refs/heads/*`.

### Test surface

- `protocol_gotcha_1_initial_no_response_end` — captures stdin advertisement bytes and asserts the last 4 bytes are `0000`, not `0002`.
- `protocol_gotcha_2_subsequent_responses_have_response_end` — captures bytes after a `command=fetch` RPC and asserts trailing `0002`.
- `protocol_gotcha_3_stdin_binary` — feed mixed text/binary input; the parser must round-trip without misframing.
- `refspec_namespace_is_reposix` — asserts the advertisement contains `refs/heads/*:refs/reposix/*`.
- `partial_clone_succeeds_against_sim` — integration test using a real `git` binary in CI: `git clone --filter=blob:none reposix::sim/proj-N /tmp/clone` succeeds and `git rev-list --objects --missing=print --all` lists every blob as missing.
- `lazy_fetch_idempotent` — `git cat-file -p <oid>` twice produces exactly one cache audit row (second read is local-only).
- `sparse_batches_wants` — `git sparse-checkout set issues/PROJ-24*; git checkout` produces exactly one `command=fetch` RPC with multiple `want` lines, not N RPCs.
- `existing_v08_push_tests_unchanged` — re-run the v0.8.0 fast-import push test suite; everything passes.

### Claude's Discretion

Internal module layout (one file vs. several) is at Claude's discretion. Choice of `tokio::process::Command` vs `std::process::Command` for spawning `upload-pack` is at Claude's discretion (tokio preferred for async parity with the rest of the workspace, std acceptable if it simplifies the binary I/O). Whether to read `upload-pack`'s stdout in chunks or via a `BufReader` is at Claude's discretion as long as gotcha #3 (binary throughout) is honoured.

</decisions>

<code_context>
## Existing Code Insights

### Reusable assets

- `crates/reposix-remote/src/main.rs` — already has the helper-protocol entry point (`capabilities`, `list`, `import`, `export` dispatch) and the `diag()` stderr helper. Add a `stateless-connect` arm to the existing dispatch match.
- `crates/reposix-remote/src/protocol.rs` — pkt-line framing primitives + the `diag()` re-export. Reusable for the v2 tunnel.
- `crates/reposix-remote/src/fast_import.rs`, `crates/reposix-remote/src/diff.rs` — existing export-path parsing. Phase 32 must NOT touch these except to confirm they still link.
- `reposix-cache::Cache::bare_repo_path()` (Phase 31) — returns the path to the on-disk bare repo. The helper invokes `git -C <path> upload-pack --stateless-rpc .` against this.
- POC reference: `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py` — Python implementation that handles all three gotchas correctly. Read this side-by-side with the Rust port.
- POC trace: `.planning/research/v0.9-fuse-to-git-native/poc/poc-helper-trace.log` — byte-level reference for what wire output looks like for clone + lazy fetch.

### Established patterns

- Helper exits non-zero with stderr message via `diag()` for any fatal error.
- Existing capability advertisement lines end with `\n`; flush is `0000` (4-byte ASCII).
- `#![forbid(unsafe_code)]` + `#![warn(clippy::pedantic)]` at the crate root — additions must comply.
- Tests co-located: `#[cfg(test)] mod tests` at the bottom of each src file; integration tests in `tests/`.

### Integration points

- Phase 31's `reposix-cache` is the upstream of every `command=fetch`. The helper opens (or spawns) a `git upload-pack --stateless-rpc` subprocess against the cache's bare-repo path.
- `reposix_core::http::client()` is NOT used in this phase directly — the helper proxies bytes; cache materialization is Phase 31's job.

</code_context>

<specifics>
## Specific Ideas

- The `upload-pack` subprocess is invoked with `--stateless-rpc` for each request turn. We do NOT keep a long-lived upload-pack alive across turns; that's what "stateless" means in the capability name.
- `uploadpack.allowFilter=true` MUST be set on the cache's bare repo at Phase 31 init time. Phase 32's integration test asserts this is honoured (without it, `--filter=blob:none` is silently dropped — see `partial-clone-remote-helper-findings.md` "Feasibility").
- Minimum runtime git version: `>= 2.34` (architecture-pivot-summary §7 risks). CI must run integration tests against alpine:latest (git 2.52) per POC pattern.
- The fresh trace log committed under `.planning/research/v0.9-fuse-to-git-native/rust-port-trace.log` is the new ground-truth artifact replacing `poc-helper-trace.log` for the Rust port.
- Do not introduce a new `reqwest::Client` in this phase. The helper does NOT make REST calls in the read path — the cache does, via Phase 31's audit-aware client.

</specifics>

<deferred>
## Deferred Ideas

- `import` deprecation (architecture-pivot-summary §7 Q5) — keep for one release cycle past v0.9.0. Phase 36 documents the deprecation in CHANGELOG; actual removal is v0.10.0+.
- Synthesis pattern (helper implements protocol-v2 server-side without an `upload-pack` subprocess) — deferred indefinitely; tunnel is sufficient.
- Read-cache inside the helper for cross-process repeat reads (`partial-clone-remote-helper-findings.md` Q4 "Performance caveat") — deferred. Sparse-checkout batching is the recommended UX.
- `git-upload-archive` support — low priority; phase ships with stubbed-out error if invoked.

</deferred>