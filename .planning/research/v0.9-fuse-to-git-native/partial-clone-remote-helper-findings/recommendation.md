← [back to index](./index.md)

# Recommendation, Feasibility, Sources, Open questions

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
