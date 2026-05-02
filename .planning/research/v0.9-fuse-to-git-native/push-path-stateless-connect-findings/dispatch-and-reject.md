← [back to index](./index.md)

# Dispatch and reject

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
