← [back to index](./index.md)

# 2. The Wire Protocol — Verbatim

### 2.1 Invocation

When the user runs:
```bash
git push reposix::http://localhost:7777/projects/demo
```

Git searches `$PATH` for an executable named `git-remote-reposix` and execs:
```
git-remote-reposix <alias> <url>
```
- `<alias>` — the remote's nickname (e.g. `origin`, or for a one-shot URL, a SHA-1 hash of the URL — handled later).
- `<url>` — everything after `reposix::` (here: `http://localhost:7777/projects/demo`).

Environment variables of note:
- `GIT_DIR` — path to the calling repo's `.git` directory. Always set.
- `GIT_TERMINAL_PROMPT`, `GIT_ASKPASS` — for interactive credential prompts.

### 2.2 The Conversation

Git speaks first, on **stdin**. Helper replies on **stdout**. Free-form diagnostics go to **stderr** (which git forwards verbatim to the user's terminal).

**Mandatory opening:**
```
> capabilities
< import
< export
< refspec refs/heads/*:refs/reposix/origin/*
< *export-marks .git/reposix/origin/marks
< *import-marks .git/reposix/origin/marks
< option
< 
```
(Lines prefixed `>` come from git, `<` from helper. The trailing blank line terminates the response.)

The leading `*` makes a capability mandatory — git aborts if it doesn't understand it. We mark `import-marks`/`export-marks` mandatory because incremental sync is *correctness-critical* for us, not just an optimization (without marks, an `export` push would re-create every issue from scratch on every push).

**Then a list query (every operation):**
```
> list
< <sha-or-?> refs/heads/main
< @refs/heads/main HEAD
< 
```
For `import`/`export` flows, the helper typically writes `?` for the SHA (it's unknown — fast-import will compute it). The `@<dest> <name>` form is a symref.

**Then either an import or export batch:**

**Import (fetch from remote):**
```
> import refs/heads/main
> 
< feature done
< feature import-marks=.git/reposix/origin/marks
< feature export-marks=.git/reposix/origin/marks
< feature force
< blob
< mark :1
< data 234
< ---
< status: open
< assignee: alice
< ---
< Login fails on Safari 17.
< 
< commit refs/reposix/origin/main
< mark :2
< committer reposix-helper <bot@reposix> 1712998800 +0000
< data 25
< Sync from REST snapshot
< from refs/reposix/origin/main^0
< M 100644 :1 PROJ-123.md
< 
< done
```

**Export (push to remote):** git invokes `git fast-export` and pipes its output to us:
```
> export
> 
> blob
> mark :1
> data 240
> ---
> status: closed
> assignee: alice
> ---
> Fixed in 1.4.2.
> 
> commit refs/heads/main
> mark :2
> author reuben <r@example.com> 1712999000 +0000
> committer reuben <r@example.com> 1712999000 +0000
> data 18
> Close PROJ-123
> from <previous-commit-sha>
> M 100644 :1 PROJ-123.md
> 
> done
< ok refs/heads/main
< 
```
We emit `ok <ref>` (or `error <ref> <reason>`) per ref, terminated by a blank line.

**Option negotiation (interleaved):**
```
> option verbosity 1
< ok
> option dry-run true
< ok
> option progress true
< unsupported
```
Reply `ok`, `unsupported`, or `error <msg>` per option. **Never crash on an unknown option** — `unsupported` is the polite refusal.

**Connect (we will NOT advertise this):** would establish a bidirectional pipe and have us proxy git's native pack protocol. Not applicable to REST.

### 2.3 Spec-quoted capability semantics

From `gitremote-helpers(7)`:

> **fetch** — Can discover remote refs and transfer objects reachable from them to the local object store.
>
> **import** — Can discover remote refs and output objects reachable from them as a stream in fast-import format.
>
> **push** — Can discover remote refs and push local commits and the history leading up to them to new or existing remote refs.
>
> **export** — Can discover remote refs and push specified objects from a fast-import stream to remote refs.
>
> **refspec** `<refspec>` — For remote helpers that implement _import_ or _export_, this capability allows the refs to be constrained to a private namespace, instead of writing to refs/heads or refs/remotes directly. **It is recommended that all importers providing the _import_ capability use this. It's mandatory for _export_.**
>
> **bidi-import** — This modifies the _import_ capability. The fast-import commands _cat-blob_ and _ls_ can be used by remote-helpers to retrieve information about blobs and trees that already exist in fast-import's memory. This requires a channel from fast-import to the remote-helper.
>
> **export-marks** `<file>` — instructing Git to dump the internal marks table to `<file>` when complete.
>
> **import-marks** `<file>` — instructing Git to load the marks specified in `<file>` before processing any input.

### 2.4 Stream terminators (cheat sheet)

| Stream | How it ends |
|--------|-------------|
| `capabilities` response | blank line |
| `list` response | blank line |
| `option` response | single line (`ok`/`unsupported`/`error ...`) |
| Batched `import` commands from git | blank line |
| Helper's fast-import stream (response to `import`) | literal `done` line (only because we advertised `feature done`) |
| `export` stream from git | literal `done` line (because git emits `--use-done-feature` when the helper advertises `done`) |
| Helper's `ok/error <ref>` response after `export` | blank line |

The `done` marker is critical. Without `feature done` in our import response, the receiving fast-import has no way to know the stream is complete (it would block waiting on stdin EOF, which never comes because git wants to keep the helper alive for more commands). **Always advertise `feature done` first.**
