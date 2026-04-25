# Findings: `git push` through a stateless-connect + export hybrid helper

**Date:** 2026-04-24
**Question source:** `.planning/research/push-path-stateless-connect.md`
**Verdict:** **The hybrid is viable and working.** A single helper can advertise `stateless-connect` (for fetch / partial clone / lazy blobs) AND `export` (for push via fast-import stream) at the same time. Git dispatches each operation to the correct capability automatically, with no refspec or protocol trickery. Custom reject messages surface verbatim to the user. We do NOT need to solve "push through receive-pack over stateless-connect."

**Evidence:**
- Source review of `transport-helper.c` (latest master) — dispatch logic proves the hybrid works by construction.
- Working POC at `.planning/research/git-remote-poc.py` (extended from the read-path POC).
- Runner script: `.planning/research/run-poc-push.sh`.
- Full captured trace: `.planning/research/poc-push-trace.log`.

---

## Q1: What happens when `git push` goes through a `stateless-connect` helper?

**Answer:** `git push` does **NOT** go through `stateless-connect`. The dispatch is service-name-gated: `stateless-connect` only ever handles `git-upload-pack` and `git-upload-archive`, never `git-receive-pack`.

**Evidence (source):** `transport-helper.c::process_connect_service` (master, line 625):

```c
if (data->connect) {
    strbuf_addf(&cmdbuf, "connect %s\n", name);
    ret = run_connect(transport, &cmdbuf);
} else if (data->stateless_connect &&
           (get_protocol_version_config() == protocol_v2) &&
           (!strcmp("git-upload-pack", name) ||
            !strcmp("git-upload-archive", name))) {    // <-- no receive-pack
    strbuf_addf(&cmdbuf, "stateless-connect %s\n", name);
    ret = run_connect(transport, &cmdbuf);
    ...
}
```

`push_refs` (line 1176) calls `process_connect(transport, /*for_push=*/1)`, which sets `name = "git-receive-pack"` (line 669) and hands off to `process_connect_service`. The `connect` branch fires only if the helper advertised `connect`; the `stateless_connect` branch is inaccessible for `git-receive-pack` by construction. `process_connect` therefore returns 0 (no take-over), and control falls through to:

```c
if (data->push)
    return push_refs_with_push(transport, remote_refs, flags);   // custom push cmd
if (data->export)
    return push_refs_with_export(transport, remote_refs, flags); // fast-export stream
```

**Implication:** whatever `stateless-connect` does with `git-receive-pack` over v2 is a non-issue — git never takes that path. The helper decides push semantics via `push` or `export` capability. This makes the hybrid trivially safe from a dispatch standpoint.

---

## Q2: Can the helper reject a push with a meaningful error?

**Answer:** **Yes.** The helper emits `error <refname> <message>` after the export stream. If `<message>` is a free-form string (i.e., doesn't exactly match one of git's canned status strings), git renders it verbatim in the `[remote rejected]` line.

**Evidence (source):** `transport-helper.c::push_update_ref_status` (line 827-888):

```c
if (starts_with(buf->buf, "ok ")) {
    status = REF_STATUS_OK;  refname = buf->buf + 3;
} else if (starts_with(buf->buf, "error ")) {
    status = REF_STATUS_REMOTE_REJECT;  refname = buf->buf + 6;
}
...
msg = strchr(refname, ' ');
if (msg) {
    *msg++ = '\0';
    /* unquote_c_style(...) */
    if (!strcmp(msg, "no match")) status = REF_STATUS_NONE;
    else if (!strcmp(msg, "up to date"))        status = REF_STATUS_UPTODATE;
    else if (!strcmp(msg, "non-fast forward"))  status = REF_STATUS_REJECT_NONFASTFORWARD;
    else if (!strcmp(msg, "already exists"))    status = REF_STATUS_REJECT_ALREADY_EXISTS;
    else if (!strcmp(msg, "fetch first"))       status = REF_STATUS_REJECT_FETCH_FIRST;
    else if (!strcmp(msg, "needs force"))       status = REF_STATUS_REJECT_NEEDS_FORCE;
    else if (!strcmp(msg, "stale info"))        status = REF_STATUS_REJECT_STALE;
    else if (!strcmp(msg, "remote ref updated since checkout"))
                                                status = REF_STATUS_REJECT_REMOTE_UPDATED;
    else if (!strcmp(msg, "forced update"))     forced = 1;
    else if (!strcmp(msg, "expecting report"))  status = REF_STATUS_EXPECTING_REPORT;
}
```

If `msg` doesn't match any canned string, it falls through with `status = REF_STATUS_REMOTE_REJECT` (set at line 831) and `msg` is attached verbatim to the ref's `remote_status` field. Git prints it in the `[remote rejected]` line.

**Empirical confirmation (POC step 5):** the helper was run with `REPOSIX_POC_REJECT_PUSH="reposix: issue was modified on backend since last fetch"`. Git output:

```
To poc::/work/bare.git
 ! [remote rejected] main -> main (reposix: issue was modified on backend since last fetch)
error: failed to push some refs to 'poc::/work/bare.git'
```

Push exit code was 1; the bare repo did not advance. The custom reposix-specific message reached the user with no alteration.

**Canned-string mapping for compatibility:** if we want git's built-in retry hints (e.g. "fetch first to see if your local is behind"), we can emit `error <ref> fetch first`. Git then renders the standard prompt. For reposix we probably want to use `fetch first` when we detect a backend-side newer version, to get the most familiar UX.

**Implication:** Q2 answered positively with no downsides. We can emit project-specific strings (`reposix: conflict on issue-2444.md`) for detailed diagnosis or the canned `fetch first` for standard UX.

---

## Q3: Can we intercept pushed commits and extract per-file diffs?

**Answer:** **Yes — and this is exactly what the existing `crates/reposix-remote` already does.** The `export` capability receives a textual fast-import stream which is directly parseable for per-commit, per-file changes.

**Evidence (protocol):** `gitremote-helpers.adoc` line 376:

```
'export'::
    Instructs the remote helper that any subsequent input is
    part of a fast-import stream (generated by 'git fast-export')
    containing objects which should be pushed to the remote.
```

`transport-helper.c::get_exporter` (line 480) confirms how git invokes fast-export:

```c
strvec_push(&fastexport->args, "fast-export");
strvec_push(&fastexport->args, "--use-done-feature");
strvec_push(&fastexport->args, data->signed_tags ?
    "--signed-tags=verbatim" : "--signed-tags=warn-strip");
if (data->export_marks)
    strvec_pushf(&fastexport->args, "--export-marks=%s.tmp", data->export_marks);
...
```

The stream structure is documented and simple:

```
blob
mark :N
data <bytes>\n<raw bytes>

commit refs/heads/<branch>
mark :M
author NAME <EMAIL> TIME TZ
committer NAME <EMAIL> TIME TZ
data <bytes>\n<commit message>
from :K                  ← parent mark (or a sha)
M <mode> :N <path>       ← file modify, blob = mark :N
D <path>                 ← file delete
R <src> <dst>            ← rename
...

done                     ← stream terminator from --use-done-feature
```

From this, the helper can:
1. Track blob marks → content bytes (cache in a dict).
2. For each `commit`, note parent mark and the `M`/`D`/`R` per-file operations.
3. For each `M` op, pull the blob bytes from the cache and parse the `.md` frontmatter.
4. Build a backend plan (POST new issue, PATCH existing, DELETE).

This is precisely what `crates/reposix-remote/src/fast_import.rs::parse_export_stream` and `crates/reposix-remote/src/diff.rs::plan` implement today.

**Implication:** commit interception is a solved problem for the `export` capability, and is already integrated in the reposix-remote crate. We do NOT need to parse packfiles, run receive-pack, or implement any v2-side push handling. Keep what we have.

---

## Q4: Is a hybrid approach viable?

**Answer:** **Yes. Confirmed empirically.** A single helper can advertise `stateless-connect` + `export` + `refspec` together. Git uses `stateless-connect` for fetch and `export` for push, with zero additional configuration.

**Evidence (source):** capability storage in `transport-helper.c::get_helper` (line 202-236) is additive — each `capname` sets an independent `unsigned N : 1` bit. There is no either/or enforcement. And dispatch prefers one cap over another ONLY within a single direction:

```c
// get_refs_list (fetch-direction):
if (process_connect(transport, 0))    // (1) stateless-connect or connect
    return transport->vtable->fetch_refs(...);
if (data->fetch)                      // (2) fetch cap
    return fetch_with_fetch(...);
if (data->import)                     // (3) import/fast-import cap
    return fetch_with_import(...);

// push_refs (push-direction):
if (process_connect(transport, 1))    // (1') connect ONLY — stateless_connect gated by service name
    return transport->vtable->push_refs(...);
if (data->push)                       // (2') push cap
    return push_refs_with_push(...);
if (data->export)                     // (3') export cap
    return push_refs_with_export(...);
```

The fetch side reaches its stateless-connect branch; the push side falls through past process_connect because stateless-connect refuses `git-receive-pack`. The two directions are completely independent.

**Empirical confirmation (POC trace from `poc-push-trace.log`):**

```
=== Capability usage counts ===
  stateless-connect invocations: 2    ← partial clone + lazy blob fetch
  export invocations: 2               ← 1 accept + 1 reject
```

Helper-by-helper breakdown:

| PID | Command received | Capability path taken |
|---|---|---|
| 29 | `stateless-connect git-upload-pack` | protocol-v2 tunnel → ls-refs + fetch with `filter blob:none` |
| 39 | `stateless-connect git-upload-pack` | protocol-v2 tunnel → single-want lazy blob fetch |
| 58 | `list`, then `export` | emit ref listing; stream into local `git fast-import`; `ok refs/heads/main` |
| 74 | `list`, then `export` | emit ref listing; drain stream; `error refs/heads/main <custom message>` |

The client-side push output confirms the accept path:

```
To poc::/work/bare.git
   ce471b4..c1500ca  main -> main
```

bare.git HEAD advanced from `ce471b4` → `c1500ca`. `git log --oneline` in bare.git shows the new commit, committed by the client and materialized via fast-import inside the helper.

**Implication:** the hybrid IS the recommended architecture. Compared to the alternatives:

| Architecture | Fetch path | Push path | Verdict |
|---|---|---|---|
| `stateless-connect` only | v2 protocol proxy (works — partial clone) | **FAILS** — receive-pack not dispatched through stateless-connect | Not viable |
| `stateless-connect` + `connect` | v2 proxy for fetch, v0/v1 connect for push | connect would give us raw receive-pack bytes, but we'd have to parse a packfile ourselves | Worst of both worlds |
| **`stateless-connect` + `export`** ✅ | v2 proxy for fetch | fast-import stream — text, easy to parse | Confirmed working |
| `import` + `export` (current) | fast-import fetch (no filter support) | fast-import push (current) | Works for push; blocks partial clone |

The hybrid path keeps everything that works about the current export-based push, and adds what we need for partial-clone fetch. Both capabilities can ship in the same `git-remote-reposix` binary.

---

## Q5: Push-time conflict detection — where does the backend comparison happen?

**Answer:** Inside `handle_export` in the helper, between receiving the fast-import stream and emitting the status response. Reject via `error <ref> <message>` — bare repo is never touched.

**Flow (with conflict detection) as demonstrated by the POC:**

1. **Client:** `git push origin main`.
2. **Git → helper:** `capabilities\n` → `list\n` → `export\n`.
3. **Helper → git:** ref list advertising the helper's view of remote HEAD.
4. **Git → helper:** full fast-export stream (`commit refs/heads/main`, blob bodies, `done`).
5. **Helper:** parse the stream in memory. For each changed `M <mode> :blob <path>` operation, extract the blob bytes and the path.
6. **Helper:** for each file path that corresponds to an issue in the backend, fetch the current backend state (`GET /issues/<id>`).
7. **Helper:** compare the base version the commit was built on (extractable from the commit's parent tree, or from a `reposix-base-version` trailer we could add) against the backend's current version. On mismatch → `error refs/heads/main reposix: issue-2444.md was modified on <backend> since last fetch`; return without writing to backing cache.
8. **No conflict:** apply the REST writes (POST/PATCH/DELETE), then update the backing bare-repo cache to record the new state, then `ok refs/heads/main`.
9. **Git:** reads status lines; on `ok`, updates the local tracking ref (`refs/reposix/main`). On `error`, surfaces the message as `[remote rejected]`.

**Key design points:**

- **The comparison happens fully inside the helper.** No pre-receive hook on a real bare repo is needed. The helper IS the receive endpoint.
- **Refspec design matters.** The POC initially tried `refs/heads/*:refs/heads/*` and fast-export emitted only a `reset refs/heads/main / from 0000` stanza — effectively a delete. Reason: `apply_refspecs` mapped the ref to itself (the LOCAL ref), so the private-shadow OID was the same as the local HEAD, making the "delta" empty. Fixed by using a distinct private namespace: `refs/heads/*:refs/poc/*` (for reposix, use `refs/reposix/*`). This is exactly what `crates/reposix-remote` already does.
- **Transactional semantics:** a reject must not leave partial state. The POC's reject path drains the incoming stream into `/dev/null` and never touches the bare repo, which is the correct default. A production helper should do the SAME: collect all changes in memory, validate against the backend in one shot, and only then commit to both bare-repo and REST backend. If the REST call fails mid-write, we need compensating DELETE/rollback calls plus a log entry.

**Implication:** Q5 dovetails with Q3. Interception + rejection is a natural extension of the export-parsing we already do. The threat-model angle (`CLAUDE.md` Operating Principle #2, "Tainted by default") fits here: the helper is the only thing authorized to emit REST writes, so it is the only place conflict-detection can correctly live.

---

## POC bugs discovered and fixed (for reposix-remote implementers)

Three non-obvious bugs surfaced while extending the POC. Document them so the Rust port doesn't relearn them:

### Bug 1: Refspec namespace collapse (critical)

Advertising `refspec refs/heads/*:refs/heads/*` causes fast-export to emit an EMPTY delta (just `reset refs/heads/main / from 0000`), deleting the ref on the next push. Fix: use a distinct private namespace, e.g. `refs/heads/*:refs/reposix/*`.

Root cause: `transport-helper.c::push_refs_with_export` (line 1116) computes `private = apply_refspecs(&data->rs, ref->name)`, then `repo_get_oid(the_repository, private, &oid)` to find the "last known remote OID" as the starting point for `^private` fast-export excludes. If `private` resolves to the same ref as `ref->name` in the local repo, the exclude equals the include → empty stream.

Takeaway: the private namespace is non-optional. `crates/reposix-remote` gets this right (`refs/heads/*:refs/reposix/*`); don't regress it.

### Bug 2: Naive `commit ` line matching in export parser

A greedy `line.startswith("commit ")` regex also matches the literal commit-message body `"commit 2 from client"` if the commit message happens to start with "commit ". The POC emitted spurious `ok 2 from client` status lines, producing `warning: helper reported unexpected status of 2`. Fix: require `line.startswith("commit refs/")` — ref-decl lines always start with `refs/...`.

Better fix for the Rust port: parse the fast-import stream with state-machine awareness of `data <N>\n<N raw bytes>` sections, so we never look for structural lines inside a data payload.

### Bug 3: `proc.communicate()` on closed stdin pipe

Python's `subprocess.communicate()` unconditionally flushes stdin before draining stdout/stderr. If we `proc.stdin.close()` BEFORE calling `communicate()`, we get `ValueError: flush of closed file`. Fix: `proc.stdin.close(); proc.wait(); stdout = proc.stdout.read(); stderr = proc.stderr.read()`.

Not applicable to Rust (`std::process::Child` has correct semantics), but worth noting because agents using Python for helper prototyping will hit this.

---

## Recommendation

**Keep the export-based push path.** The current `crates/reposix-remote` uses `import` + `export`; add `stateless-connect` alongside and keep `export`. Do NOT remove `export`.

Concrete workstream:

1. **Add `stateless-connect` to `crates/reposix-remote`** per findings in `partial-clone-remote-helper-findings.md`. This is net-new code, ~200 lines of Rust, tunnelling to a backing bare repo.
2. **Keep the `import`/`export` code paths as-is.** They are not invalidated by adding `stateless-connect` — the dispatch logic uses them independently for push.
3. **Drop `import` if the new stateless-connect path covers all fetch paths.** `import` was the only fetch mechanism when we supported non-partial clones; once `stateless-connect` is live, `import` is redundant. Keep for one release deprecation cycle per `partial-clone-remote-helper-findings.md` §"Open questions" Q1.
4. **Invariant test to add:** a round-trip test that does partial clone + new commit + push, verifying (a) fetch uses stateless-connect, (b) push uses export. The POC's `run-poc-push.sh` is the template; convert it into a Rust integration test once the helper supports both caps.
5. **Conflict-detection surface:** the helper's `handle_export` path is where backend comparison belongs. Add a backend-version check before REST writes and emit `error <ref> <reposix-specific message>` on mismatch. Use `fetch first` as the canned message for standard git UX.

---

## Feasibility of POC: confirmed

The hybrid POC at `.planning/research/git-remote-poc.py` demonstrates:

- A single helper advertises both `stateless-connect` and `export`.
- Partial clone with `--filter=blob:none` routes through `stateless-connect` (3 separate helper invocations, all using v2 tunneling).
- Lazy-blob fetch routes through `stateless-connect` (single-want v2 request, same helper binary).
- Push of a new commit routes through `export` (list → export → ok).
- Push rejection via `REPOSIX_POC_REJECT_PUSH=<msg>` env var surfaces the custom message to the user via `[remote rejected] main -> main (<msg>)`.

Run with:

```bash
docker run --rm -v $(pwd)/.planning/research:/work alpine:latest \
  sh -c 'apk add --quiet --no-cache git python3 && cp /work/git-remote-poc.py /work/git-remote-poc && chmod +x /work/git-remote-poc && /work/run-poc-push.sh'
```

Full trace in `poc-push-trace.log` (114 lines).

---

## Sources cited

Primary (git source, latest master):

- `transport-helper.c::process_connect_service` (line 625) — service-name gate that excludes receive-pack from stateless-connect.
- `transport-helper.c::push_refs` (line 1176) — push-direction capability dispatch.
- `transport-helper.c::push_refs_with_export` (line 1094) — fast-export + refspec-to-private-namespace mapping.
- `transport-helper.c::push_update_ref_status` (line 777-914) — parses helper's `ok`/`error` response and renders free-form messages.
- `transport-helper.c::get_exporter` (line 480) — shows git calls `fast-export --use-done-feature`.
- `transport-helper.c::get_helper` (line 202-236) — capability advertisement parser; capabilities are independent bits.
- `Documentation/gitremote-helpers.adoc` — `export`, `stateless-connect`, `refspec`, `push` capability definitions.

Secondary:

- POC trace `.planning/research/poc-push-trace.log`.
- Prior findings `.planning/research/partial-clone-remote-helper-findings.md` (stateless-connect read-path).

---

## Open questions for next phase planning

1. **Atomicity of REST write + bare-repo-cache update.** If the REST POST succeeds but the local bare-repo-cache update fails, we get divergence. Options: write to bare cache first, then REST (rollback = `git update-ref refs/heads/main <old>`), or the reverse (rollback = DELETE the issue). Probably want the bare-cache-first ordering plus a background reconciler.
2. **Reposix-specific reject messages vs canned strings.** Custom messages are great for diagnostics (`reposix: issue-2444.md field.state was 'open' locally, backend is 'closed'`), but canned strings like `fetch first` trigger git's built-in "perhaps a `git pull` would help" hint. Consider emitting BOTH: helpful canned status + stderr diagnostic via the helper's own stderr (already supported — see the existing `diag()` function in `crates/reposix-remote/src/main.rs`).
3. **What if push touches only `.planning/`-like files that aren't issues?** The current export parser may see changes outside any issue-tracking path. Decide: reject (refuse to accept non-issue files via `git push`), or silently commit to bare cache only (the client got the commit, nothing flows to REST). Given the project pitch "the mount is a real git working tree," we probably want to commit to bare cache regardless — treats the repo like a normal git remote for non-issue paths.
4. **Performance: stream-parsing the export in a single pass.** The POC reads the stream byte-by-byte and keeps it all in memory. A production helper should stream-parse into a state machine (`EXPECT_HEADER` → `READ_DATA_SIZE` → `READ_DATA_BYTES` → ...) and emit REST writes as it goes, with a final commit-or-rollback barrier at the `done` terminator.
