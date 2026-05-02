← [back to index](./index.md)

# Conflict detection, POC bugs, recommendation, and open questions

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
