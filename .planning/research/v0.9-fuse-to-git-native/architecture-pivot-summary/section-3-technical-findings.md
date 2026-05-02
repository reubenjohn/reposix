[index](./index.md)

# 3. Confirmed Technical Findings

Two research sessions produced detailed findings documents with working POCs. This section summarises the key results.

## Q1: Can a remote helper act as a promisor remote?

**YES**, via the `stateless-connect` capability.

When git has a partial clone and encounters a missing blob, it runs `git fetch <remote> --filter=blob:none --stdin` with the missing OIDs piped on stdin. This fetch goes through the standard transport-selection path in `transport-helper.c::fetch_refs`:

1. If the helper advertises `stateless-connect` (or `connect`), git routes through the protocol-v2 tunnel.
2. If the helper advertises `fetch`, git uses the simple fetch capability.
3. If the helper advertises `import`, git uses fast-import.

Only branch (1) supports the `filter` argument required for partial clone. The `fetch` capability requires a ref name (not bare OIDs), and `import` has no filter semantics. **`stateless-connect` is the only viable path.**

The POC confirmed this empirically: three distinct helper invocations during a partial clone + two lazy blob fetches, with missing-blob count dropping 6 -> 5 -> 4 across invocations.

**Source:** `promisor-remote.c::fetch_objects`, `transport-helper.c::fetch_refs`.

## Transport routing: `stateless-connect` for fetch, `export` for push (hybrid)

The hybrid approach is confirmed working. A single helper advertises both `stateless-connect` and `export`. Git dispatches:

- **Fetch direction:** `stateless-connect` handles `git-upload-pack` (protocol v2 tunnel).
- **Push direction:** `stateless-connect` is gated by service name and explicitly excludes `git-receive-pack`. Git falls through to `export`, which receives a fast-import stream that the helper parses for per-file changes.

This is not a hack -- it is the intended dispatch logic in `transport-helper.c`. The capabilities are independent bits; there is no either/or enforcement.

**POC confirmation:** `poc-push-trace.log` shows 2 `stateless-connect` invocations (clone + lazy fetch) and 2 `export` invocations (accept + reject), all from the same helper binary.

## Helper can count `want` lines and refuse

The helper sees the full pkt-line stream of each `command=fetch` request before dispatching. A multi-blob fetch (e.g., from sparse-checkout) packs all OIDs into a single RPC turn with multiple `want <oid>` lines. The helper can:

- Count `want` lines and refuse if the count exceeds a threshold.
- Inspect or omit the `filter` argument.
- Log every fetch for audit purposes.
- Return errors via stderr (git surfaces helper stderr to the user).

## Sparse-checkout batches blob requests

Setting sparse-checkout alone does not trigger blob fetches. Only `git checkout` (or any blob-materializing operation) fetches. When checkout does fetch, it batches all missing blobs into a single `command=fetch` RPC with multiple `want` lines.

This is materially different from `git cat-file -p <oid>`, which always sends one want at a time (one helper process per blob).

**Architectural implication:** the recommended agent UX is to configure sparse-checkout *before* checkout, so the helper sees one batched request for exactly the blobs the agent needs.

## Three protocol gotchas (from POC)

These are not documented in `gitremote-helpers.adoc` and required iterative testing to discover:

| Gotcha | Symptom if wrong | Fix |
|---|---|---|
| Initial advertisement does NOT end with response-end (`0002`), only flush (`0000`). | `fatal: expected flush after ref listing` | Send advertisement as-is from `upload-pack --advertise-refs --stateless-rpc`; do not append `0002`. |
| Subsequent RPC responses DO need trailing response-end (`0002`). | Helper hangs or git misframes the next request. | After each response-pack, write bytes from `upload-pack --stateless-rpc` followed by `0002`. |
| Mixing text and binary stdin reads corrupts the stream. | Helper hangs after handshake. | Read the entire helper protocol in binary mode. (In Rust: read from `BufReader<Stdin>` consistently.) |

## Push rejection format

The helper emits `error <refname> <message>` after processing the export stream. If `<message>` is a free-form string (not matching one of git's canned status strings), git renders it verbatim:

```
! [remote rejected] main -> main (reposix: issue was modified on backend since last fetch)
```

If the helper emits `error <ref> fetch first`, git renders the standard "perhaps a `git pull` would help" hint. The recommended approach is to emit the canned `fetch first` status for standard UX, plus a detailed diagnostic on stderr via the existing `diag()` function.

## Conflict detection happens inside `handle_export`

The interception point for push-time conflict detection is between receiving the fast-import stream and emitting the status response. Flow:

1. Parse the fast-import stream in memory.
2. For each changed file path, fetch current backend state (`GET /issues/<id>`).
3. Compare base version to backend's current version.
4. On mismatch: emit `error refs/heads/main <message>`; do not touch the backing cache.
5. On success: apply REST writes, update backing cache, emit `ok refs/heads/main`.

The reject path drains the incoming stream and never touches the bare repo, ensuring no partial state.

## Refspec namespace is non-optional

The helper must advertise a private ref namespace (e.g., `refs/heads/*:refs/reposix/*`). Using `refs/heads/*:refs/heads/*` causes fast-export to emit an empty delta because the private OID matches the local HEAD, making the exclude equal the include. The current `crates/reposix-remote` already uses the correct namespace; this must not regress.
