# Findings: git remote helpers CAN serve as promisor remotes for partial clone

**Date:** 2026-04-24
**Question source:** `.planning/research/partial-clone-remote-helper.md`
**Verdict:** **YES** — a remote helper that advertises `stateless-connect` and proxies protocol-v2 traffic to `git upload-pack` can act as a fully functional promisor remote. We can delete `crates/reposix-fuse` if we accept the architectural shift.
**Evidence:** working POC at `.planning/research/git-remote-poc.py`, run script `.planning/research/run-poc.sh`, captured helper trace `.planning/research/poc-helper-trace.log`. The POC clones with `--filter=blob:none --no-checkout`, then proves lazy fetches happen through the helper by reading individual blobs and observing the helper being re-spawned with single-want `command=fetch` requests.

---

## Q1: Can a `git-remote-<scheme>` helper act as a promisor remote?

**Answer:** **Yes**, if it advertises the `stateless-connect` capability.

**Evidence (source code):** `promisor-remote.c::fetch_objects` literally invokes `git fetch` as a subprocess and lets standard transport selection do the rest:

```c
// git/promisor-remote.c (master)
static int fetch_objects(struct repository *repo,
                         const char *remote_name,
                         const struct object_id *oids,
                         int oid_nr)
{
    struct child_process child = CHILD_PROCESS_INIT;
    ...
    child.git_cmd = 1;
    strvec_pushl(&child.args, "-c", "fetch.negotiationAlgorithm=noop",
                 "fetch", remote_name, "--no-tags",
                 "--no-write-fetch-head", "--recurse-submodules=no",
                 "--filter=blob:none", "--stdin", NULL);
    // OIDs are then written to child.in
}
```

So a missing-blob fetch is just `git fetch <name> --filter=blob:none --stdin` with the missing OIDs piped on stdin. That fetch goes through the normal transport-selection path in `transport-helper.c::fetch_refs`:

```c
// git/transport-helper.c (master), simplified
if (process_connect(transport, 0))
    return transport->vtable->fetch_refs(transport, ...);  // (1) connect/stateless-connect
if (data->fetch)
    return fetch_with_fetch(transport, ...);                // (2) simple fetch cap
if (data->import)
    return fetch_with_import(transport, ...);               // (3) fast-import
return -1;
```

Three branches, in priority order. For partial clone we **must** take branch (1) because the simple `fetch <sha1> <name>` form (branch 2) requires a ref name and only fetches ref-reachable objects, and `import`/`export` (branch 3) is a fast-import stream that doesn't carry filter semantics. **Branch (1) with `stateless-connect` is the only viable path** because filter-spec is a protocol-v2-only feature.

**Empirical confirmation:** see `poc-helper-trace.log`. Three distinct helper invocations during the POC:

| Helper PID | Trigger | Request size | Response | Effect |
|------------|---------|--------------|----------|--------|
| 34 | initial `clone --filter=blob:none --no-checkout` | 243 B (1 `want HEAD`) | 385 B (1 commit + 2 trees, no blobs) | partial clone |
| 54 | `git cat-file -p <big1.txt-blob>` | 193 B (1 `want <oid>`) | 205 KB (just that blob) | lazy fetch #1 |
| 71 | `git show <docs/d1.md-blob>` | 193 B (1 `want <oid>`) | 76 B (just that blob) | lazy fetch #2 |

The "missing blobs" count drops 6 → 5 → 4 across these invocations, proving real lazy materialization through the helper.

**Implication:** our existing `crates/reposix-remote` (which uses `import`/`export` only) is **insufficient** — it implements branch (3). We need to add `stateless-connect` to make it serve partial clones.

---

## Q2: What capabilities does the helper need?

**Answer:** It needs `stateless-connect` plus a way to proxy protocol-v2 to a real `upload-pack` (or to synthesize protocol-v2 responses itself).

**Evidence:** `Documentation/gitremote-helpers.adoc`:

> `stateless-connect` — Experimental; for internal use only. Can attempt to connect to a remote server for communication using git's wire-protocol version 2.

`stateless-connect <service>`:

> Connects to the given remote service for communication using git's wire-protocol version 2. Valid replies to this command are empty line (connect successful), or error message. ... After line feed terminating the positive (empty) response, the output of the service starts. Messages (both request and response) must consist of zero or more PKT-LINEs, terminating in a flush packet. Response messages will then have a response end packet after the flush packet to indicate the end of a response.

The `connect` capability is **insufficient** — it pipes protocol v0/v1, which has no `filter` argument. Empirical proof: an earlier POC iteration that advertised only `connect` succeeded at fetching but git printed `warning: filtering not recognized by server, ignoring`, and *all* blobs were transferred.

The `filter` capability itself does **not** appear in the helper protocol — it's a protocol-v2 fetch-command argument, not a helper capability. The helper just needs to tunnel v2.

**Two implementation patterns:**

1. **Tunnel pattern (POC uses this):** helper proxies v2 traffic to an actual `git upload-pack --stateless-rpc` running on a backing bare repo. Best when reposix can keep a local materialized git repo as cache; the helper is then a thin pipe.

2. **Synthesis pattern:** helper itself implements the protocol-v2 server side of `command=ls-refs` and `command=fetch`, generating refs and on-demand packs from REST responses. More work but lets us avoid maintaining a shadow repo. This is what `git-remote-http` does conceptually — though under the hood it just talks HTTP to a real upload-pack.

**Three protocol gotchas the POC discovered the hard way (not in the docs but confirmed empirically):**

| Gotcha | Symptom if you get it wrong | Fix |
|---|---|---|
| The unsolicited initial advertisement does **not** end with a response-end (`0002`) packet, only with flush (`0000`). | `fatal: expected flush after ref listing` after the FIRST request. | Send advertisement bytes as-is from `upload-pack --advertise-refs --stateless-rpc`; do not append `0002`. |
| Subsequent RPC responses **do** need a trailing response-end (`0002`). | Helper hangs or git misframes the next request. | After each response-pack, write the bytes from `upload-pack --stateless-rpc` followed by `0002`. |
| Mixing Python `sys.stdin.readline()` and `sys.stdin.buffer.read()` corrupts the stream because TextIOWrapper over-reads. | Helper hangs after handshake. | Read the entire helper protocol in binary mode (POC reads pre-handshake lines byte-at-a-time via `STDIN.read(1)`). |

These three behaviours are not stated explicitly in `gitremote-helpers.adoc` and required reading source / iterative testing to discover. Document them in any reposix-remote rework.

**Implication:** the work to add `stateless-connect` to `crates/reposix-remote` is non-trivial but bounded — ~200 lines of Rust, similar shape to what the Python POC does. Pattern 1 (tunneling to a local bare repo) is simpler than pattern 2 (synthesis from REST). For a reposix design where a local cache is acceptable, pattern 1 is the obvious choice.

---

## Q3: Are there alternatives if helpers can't be promisors?

Q1 answers "yes, helpers can be promisors", so this question is **not blocking** any longer. For completeness:

- **Local HTTP server (alternative path C):** would also work; smart-HTTP is itself a stateless-connect-style transport. `git-remote-http` is in fact a remote-helper binary that supports `stateless-connect` and proxies to a local HTTP socket. So this is functionally equivalent to our pattern 1, with one extra hop (TCP localhost loopback). Architecturally more moving parts; no win over the helper-tunnel pattern.
- **FUSE + caching (path D):** still a viable fallback. SQLite blob cache means each issue body is fetched at most once per session. But it loses the "git diff = change set" property that motivates the project, because the working tree is FUSE-backed rather than a real git checkout.
- **Hybrid (path A+C):** helper for push/pull (tree sync), local HTTP for blob fetches. Strictly worse than pure helper for our case since we get nothing from splitting.

**Decision:** **path B (stateless-connect tunnel)**. It is the architecturally cleanest path that uses git's own machinery for partial clone, requires no shadow infrastructure beyond a local bare repo cache, and leaves us with a real git working tree on the user's filesystem.

---

## Q4: Can a remote helper see the request batch and refuse it?

**Answer:** **Yes, with caveats.** The helper sees the full pkt-line stream of each `command=fetch` request and can parse it before dispatching. Errors propagate to git via stderr.

**Evidence — request inspection:**

In `poc-helper-trace.log`, every fetch request is logged with its full byte stream. The format is plaintext-ish (capability lines + `want <oid>` lines, all in pkt-line frames). Sample (single-blob lazy fetch):

```
command=fetch
agent=git/2.52.0-Linux
object-format=sha1
[delim]
thin-pack
no-progress
ofs-delta
filter blob:none
want 0be4da742bc818db0c722389149757d92e894b59
[flush]
```

A request with multiple wants (sparse-checkout driven checkout, 443 bytes vs 193 for single-want) packs them all into one RPC turn:

```
command=fetch
...
filter blob:none
want <oid1>
want <oid2>
want <oid3>
want <oid4>
want <oid5>
[flush]
```

So the helper can:
- count `want` lines in a single RPC and refuse if > N
- inspect the `filter` argument (or its absence)
- log every fetch for audit-trail purposes (REQUIRED per project CLAUDE.md item 3 "Audit log is non-optional")
- selectively allowlist/denylist OIDs based on internal knowledge of which paths they correspond to

**Evidence — error reporting:**

`gitremote-helpers.adoc`: "If a fatal error occurs, the program writes the error message to stderr and exits."

Plain stderr is sufficient; git surfaces helper stderr to the user. No `ERR` pkt-line is required at the helper protocol level. (Inside the v2 stream, `ERR` pkt-lines do exist for protocol-level errors, but our existing `crates/reposix-remote/src/main.rs::diag()` already does the right thing by writing to stderr and exiting non-zero.)

**Performance caveat — lazy fetches are NOT auto-batched:** the POC observed that `git cat-file -p <oid>` triggers an INDEPENDENT helper invocation (new process, new ls-refs, new advertisement) per blob. `git grep` across the working tree similarly invokes the helper once per missing blob. **This is a known git behavior, not something our helper can fix from the inside.** Mitigations:

- **Read-cache in the helper:** maintain a session-scoped cache of fetched blobs so repeat reads don't re-invoke. Doesn't help cross-process concurrency but smooths repeat reads within a single agent task.
- **Batch-fetch at agent layer:** before launching `git grep` etc., the agent runs `git fetch origin --refetch --filter=blob:none $(git ls-tree -r HEAD | awk '{print $3}')` to pre-warm. Klunky.
- **Refuse and educate:** the helper inspects each request's `want` count; if a single-blob lazy fetch is happening AND a heuristic suggests "agent is enumerating", refuse with a stderr message like `error: detected blob enumeration; use 'git cat-file --batch' or 'git fetch --filter=blob:none HEAD' to pre-warm`. Risky — false positives bad.

**Implication:** the helper has full request introspection, which is exactly what the threat model in `CLAUDE.md` ("Tainted by default", "Audit log is non-optional") needs. The lazy-fetch fan-out is a real but mitigable performance problem; it does not block the architecture.

---

## Q5: How does sparse-checkout interact with partial clone via the helper?

**Answer:** Sparse-checkout config alone doesn't fetch; only `git checkout` (or any operation that materializes blobs) fetches. When checkout *does* fetch, it BATCHES the missing wants into a single `command=fetch` RPC, which is excellent for our use case.

**Evidence — config alone is free:**

In `run-poc.sh` step 5b: after `clone --filter=blob:none --no-checkout` (6 blobs missing), running `git sparse-checkout init --cone; git sparse-checkout set docs` does **not** invoke the helper. The missing-blobs count stays at 6.

**Evidence — checkout batches:**

Then `git checkout main` invokes the helper exactly once. The single `v2 request` is **443 bytes** vs the typical single-want 193 bytes. The 250-byte difference is roughly 5 extra `want <oid>` pkt-lines (each ≈50 bytes). One RPC, multiple wants, one packfile response.

This is materially different from `git cat-file -p`, which always sends one want at a time.

**Caveat on the test:** the POC's sparse pattern `set docs` in cone mode includes root files by default, so `big1.txt`/`big2.txt`/`big3.txt`/`small.txt` were ALSO checked out and fetched. To exclude root files, use `set --no-cone 'docs/**'` or `set --skip-checks`. This wrinkle doesn't change the core finding: **sparse-checkout boundaries DO determine what gets checked out, and checkout DOES batch wants through the helper**. We just have to set the sparse pattern correctly.

**Architectural opportunity:** for an agent workflow, we can pre-configure `sparse-checkout` to `docs/**` (or `issues/PROJ-*/`, etc.), do `git checkout HEAD`, and the helper sees one batched fetch with exactly the OIDs the agent will need. No drip-by-drip lazy fetches. This is the recommended UX.

**Implication:** sparse-checkout is the right "scope guard" for a reposix mount. Combined with the helper's per-request introspection (Q4), we can enforce both filesystem-level and protocol-level locality of access.

---

## Recommendation

**Path B — `stateless-connect` tunnel.** Rewrite `crates/reposix-remote` to advertise `stateless-connect` (in addition to current `import`/`export`), proxy protocol v2 to a backing bare-repo cache that reposix maintains.

### Architecture

```
┌─ agent (cat / grep / git) ─┐
│                            │
│  /mnt/reposix/issues/*  ←  this is now a real git working tree, not FUSE
│           │
│  .git/objects (partial clone)
│           │
│  remote.origin.promisor=true
│  remote.origin.partialclonefilter=blob:none
│           │
│  remote.origin.url=reposix::github/orgname/projectname
│           │
└── git-remote-reposix (helper binary, has stateless-connect) ──┐
                                                                 │
                                              proxies protocol-v2│traffic to:
                                                                 │
                                                ┌─ local bare repo cache ─┐
                                                │  built-up by reposix    │
                                                │  from REST API responses│
                                                └──┬──────────────────────┘
                                                   │ on-demand REST calls
                                                   │ when cache is stale
                                                   ↓
                                       ┌── GitHub / Confluence / Jira REST ──┐
                                       │  (gated by REPOSIX_ALLOWED_ORIGINS) │
                                       └──────────────────────────────────────┘
```

### What we delete

- `crates/reposix-fuse` entirely.
- `fuser` dependency (we wanted to drop this anyway because of pkg-config / libfuse-dev pain).
- All FUSE-related runtime concerns: mount/unmount lifecycle, /dev/fuse permissions, WSL2 quirks, the integration tests gated on `fuse-mount-tests`.

### What we add

- `stateless-connect` capability in `git-remote-reposix` (~200 lines of Rust, similar to POC).
- A backing-cache crate (`reposix-cache`?) that materializes REST responses into a local bare git repo. This is where sync logic lives.
- New CLI flow: `reposix init <backend>::<project> <path>` → does `git init`, configures `extensions.partialClone`, sets up `remote.origin.url=reposix::...`, then `git fetch --filter=blob:none origin` to bootstrap.

### What stays the same

- `BackendConnector` trait and all backend impls (`SimBackend`, `GithubBackend`, etc.) — they're now consumed by `reposix-cache` instead of by FUSE.
- Audit-log requirements: the helper still writes to the audit table for every protocol-level fetch.
- Tainted-by-default policy: data flowing through the helper is still tainted; same allowlist guards apply.

### Risks

1. **Lazy-fetch fan-out (Q4 caveat):** agents that grep across the whole repo will trigger one helper invocation per blob. We mitigate via sparse-checkout UX guidance and possibly an "agent pre-warm" command.
2. **Stateless-connect is documented as "experimental, for internal use only"** in the helper docs. The capability has been stable in practice since git 2.21 (≈ 2019) and is what `git-remote-http` uses internally, so the risk of breakage is low. We should pin a minimum git version (≥ 2.27 for full `filter` support over v2; ≥ 2.34 to be safe) in `CLAUDE.md` and `README`.
3. **No CI test of the helper-promisor path on git ≤ 2.25** — we *literally cannot* test it on the dev host's system git. Need to add a docker-based CI job using alpine + git ≥ 2.34 for this path (the POC validates that pattern works).

---

## Feasibility of POC: confirmed

The POC at `.planning/research/git-remote-poc.py` demonstrates the recommended path end-to-end. Key constraints solved:

- **Local git is 2.25.1** (no v2 filter support on the server side). Solution: run the POC inside `alpine:latest` (git 2.52). Documented in `run-poc.sh`.
- **Server-side filter advertisement requires `uploadpack.allowFilter=true`** — without it, `--filter=blob:none` is silently dropped. Set on the bare repo before testing.
- **`--no-checkout` is needed during clone** to keep blobs missing for subsequent lazy-fetch demonstration. A normal clone fetches the entire HEAD tree's blobs during checkout.

Run with:

```bash
docker run --rm -v $(pwd)/.planning/research:/work alpine:latest \
  sh -c 'apk add --quiet --no-cache git python3 && cp /work/git-remote-poc.py /work/git-remote-poc && chmod +x /work/git-remote-poc && /work/run-poc.sh'
```

Or just `bash run-poc.sh` after `cp git-remote-poc.py git-remote-poc; chmod +x git-remote-poc` if your local git is ≥ 2.27.

The POC produces a complete protocol trace (`poc-helper-trace.log`) showing exactly which `command=fetch` requests git issues and how the helper responds, which is the reference for re-implementing this in Rust.

---

## Sources cited

Primary (git source, latest master):

- `Documentation/gitremote-helpers.adoc` — capabilities (`fetch`, `connect`, `stateless-connect`, etc.) and their wire format.
- `Documentation/technical/partial-clone.adoc` — promisor remote fetch is "done by invoking a 'git fetch' subprocess".
- `Documentation/technical/protocol-v2.txt` — `fetch` command, `want <oid>`, `filter <filter-spec>`.
- `promisor-remote.c::fetch_objects` — the subprocess call site.
- `transport-helper.c::fetch_refs` — the routing logic between `connect`/`fetch`/`import`.

Secondary:

- POC empirical run (`.planning/research/poc-helper-trace.log`).
- `git-remote-http` itself uses `stateless-connect` internally (transport-helper.c::process_connect_service), confirming the pattern.

---

## Open questions for next phase planning

1. Should we keep `import`/`export` (current capabilities) alongside `stateless-connect` for backward compatibility, or fully migrate? Recommend: keep both for one release, deprecate `import`/`export` in v0.10.
2. Cache eviction policy in `reposix-cache`. LRU? TTL? Per-project quota?
3. How does `git push` from the working tree get translated back to REST `PATCH /issues/<n>`? The current `export` path handles this; can `stateless-connect` do it via `git receive-pack`? Need to investigate (the helper protocol DOES support receive-pack via `connect git-receive-pack`).
4. Threat-model implications: `git push` through a tunnel-helper means the helper must validate every commit's content against the same allowlist of frontmatter fields. Update `research/threat-model-and-critique.md`.
