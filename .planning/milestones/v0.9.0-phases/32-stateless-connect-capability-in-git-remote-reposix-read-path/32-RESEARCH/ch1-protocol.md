# Phase 32 Research — Protocol & Gotchas

← [back to index](./index.md)

## 1. What we are actually building

Port the Python POC's `stateless-connect` handler to Rust in
`crates/reposix-remote/` so that:

```bash
git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone
git cat-file -p <issue-oid>
git sparse-checkout set 'issues/PROJ-24*'
git checkout main
```

all work end-to-end, with blobs lazy-fetched on demand through the
helper. Push via `export` must keep working (hybrid helper).

**Implementation pattern:** Tunnel (NOT synthesis). The helper is a thin
pkt-line pipe between git's stdin/stdout and a
`git upload-pack --stateless-rpc` subprocess running against the Phase
31 `reposix-cache` bare repo.

The helper does NOT make REST calls in the read path. REST translation
happens at the next layer down (`reposix-cache::Cache::build_from` and
`Cache::read_blob`, delivered in Phase 31).

---

## 2. Wire protocol — the bytes on the wire

### 2.1 Helper-protocol handshake (text, line-based, binary-safe)

```
git  -> helper : capabilities\n
helper -> git  : import\nexport\nrefspec refs/heads/*:refs/reposix/*\nstateless-connect\nobject-format=sha1\n\n
git  -> helper : stateless-connect git-upload-pack\n
helper -> git  : \n                       (single empty line = "ready")
helper -> git  : <advertisement bytes from `upload-pack --advertise-refs --stateless-rpc`>
```

After the ready newline, git expects the service's bytes to start
**immediately**. The advertisement is the initial, unsolicited v2 stream.

### 2.2 Protocol-v2 RPC loop (binary, pkt-line framed)

Each RPC turn:

```
git  -> helper : <pkt-line stream, terminated by flush 0000>
helper -> git  : <response bytes from `upload-pack --stateless-rpc`> 0002
```

The helper re-spawns `upload-pack --stateless-rpc` **once per RPC turn**.
That is what "stateless" means in the capability name — there is no
long-lived upload-pack across turns.

### 2.3 pkt-line frame format

4-byte ASCII hex length header + payload:

| Header (hex) | Meaning |
|---|---|
| `0000` | flush |
| `0001` | delim (v2 uses this for request/response section boundaries) |
| `0002` | response-end (v2 only, terminates a helper `stateless-connect` response) |
| `0004` | empty-data |
| `000e..` | data packet; length includes the 4-byte header |

Our framer must pass through all four special shorts (`0000`, `0001`,
`0002`) without wrapping them in another header.

---

## 3. The three protocol gotchas — locked regression tests

From `partial-clone-remote-helper-findings.md` Q2 and
`architecture-pivot-summary.md` §3:

| # | Gotcha | Symptom if wrong | Fix |
|---|---|---|---|
| 1 | Initial advertisement ends with flush `0000` only — NOT response-end `0002`. | `fatal: expected flush after ref listing` on first request. | Send `upload-pack --advertise-refs --stateless-rpc` output verbatim; do NOT append `0002`. |
| 2 | Subsequent RPC responses DO need trailing `0002`. | Helper hangs or git misframes the next request. | After each response, write `upload-pack --stateless-rpc` output followed by `b"0002"`. |
| 3 | Binary stdin throughout. | Helper hangs after handshake (`readline()` over-reads). | In Rust: pull pre-handshake text lines via a single `BufReader<Stdin>` with `read_until(b'\n')`. Do NOT mix `read_line()` with raw reads on separate buffers. |

Concrete test names (per CONTEXT.md):

- `initial_advertisement_ends_with_flush_only` (gotcha 1)
- `rpc_response_appends_response_end` (gotcha 2)
- `stdin_is_binary_throughout` (gotcha 3)
- `refspec_namespace_is_reposix`
- `capability_advertisement_lists_stateless_connect`
