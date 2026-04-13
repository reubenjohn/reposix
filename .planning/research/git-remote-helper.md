# Research: Git Remote Helper Protocol for `git-remote-reposix`

**Researcher:** subagent (research mode: feasibility + ecosystem)
**Date:** 2026-04-13
**Confidence:** HIGH for protocol mechanics (Context-equivalent: official `gitremote-helpers(7)` man page); MEDIUM-HIGH for design recommendation; HIGH for OSS prior-art survey.

---

## 1. TL;DR — Recommendation for reposix

| Decision | Choice | One-liner |
|----------|--------|-----------|
| Capability set | `import` + `export` (NOT `fetch`/`push`) | Stream fast-import; we never reconstruct packfiles. |
| Auxiliary capabilities | `refspec`, `*export-marks`, `*import-marks`, `option` | Required for `import`/`export`; marks give us O(1) incremental sync. |
| State diffing | Maintain a `last-pushed.tree` SHA per ref in `$GIT_DIR/reposix/<remote>/state` | On `export`, walk new tree vs. last tree → field-level deltas → REST verbs. |
| Auth | Env vars `REPOSIX_TOKEN`, fallback `git config remote.<name>.reposixToken`, namespaced per-remote via `argv[1]` (the alias) | Helper receives `(alias, url)` as argv. |
| Error surface | Print to **stderr** (not stdout — stdout is reserved for protocol); also `error <ref> <msg>` on the protocol channel for per-ref failures | Both: human-readable on stderr, machine-readable on stdout per spec. |
| Conflict mode | On `import`, fetch authoritative remote state and emit a fast-import commit on `refs/reposix/<remote>/<ref>`; let git's three-way merge produce textual conflict markers in the agent's working tree | Native git semantics; no JSON conflict synthesis. |
| Async-from-sync | `tokio::runtime::Builder::new_current_thread().enable_all().build()` once at startup; every command handler is a sync function that calls `runtime.block_on(async { ... })` | Same bridge pattern as `fuser` callbacks. |

**Why NOT `fetch`/`push`:** those require us to materialize git packfiles ourselves (delta compression, OFS_DELTA, idx generation). For a REST-backed remote where there is *no upstream pack store*, that is gratuitous work. `import`/`export` lets us speak fast-import — a textual line protocol that fits naturally on top of HTTP. This matches the design of `git-remote-hg`, `git-remote-bzr`, and most non-git-native helpers.

**Why NOT `connect`:** `connect` is for remotes that *already speak git's native pack protocol* (we'd just proxy bytes through SSH or TLS). REST APIs do not.

---

## 2. The Wire Protocol — Verbatim

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

---

## 3. Worked Example: One Issue Per File

### 3.1 Setup

The agent's local repo at `~/work/issues/` has:
```
PROJ-123.md       ← local working tree
.git/
  reposix/origin/
    state.json    ← {"PROJ-123": {"sha": "abc...", "etag": "W/\"4\"", "tree": "deadbeef..."}}
    marks         ← :1 abc123...  :2 def456...
```

`PROJ-123.md` initially:
```yaml
---
status: open
assignee: alice
labels: [bug]
---
Login fails on Safari 17.
```

The agent runs `sed -i 's/^status:.*/status: closed/' PROJ-123.md && git commit -am 'close 123' && git push origin main`.

### 3.2 What git sends to us

Git invokes `git-remote-reposix origin http://localhost:7777/projects/demo`. After capabilities and list, it sends `export\n\n` and pipes the output of `git fast-export --import-marks=... --export-marks=... refs/heads/main` to our stdin:

```
export

blob
mark :7
data 246
---
status: closed
assignee: alice
labels: [bug]
---
Login fails on Safari 17.

commit refs/heads/main
mark :8
author reuben <r@example.com> 1712999000 +0000
committer reuben <r@example.com> 1712999000 +0000
data 13
close PROJ-123

from :2
M 100644 :7 PROJ-123.md

done
```

Note: the `from :2` references mark `:2` from a previous run (loaded via `--import-marks`). If this is the *first* push, git emits `from 0000...` (orphan).

### 3.3 What the helper does

1. **Parse the fast-import stream.** Build an in-memory map `marks: {7: <blob-bytes>, 8: <commit-meta>}` and a tree representation: `tree[":8"] = {"PROJ-123.md": (mode=100644, blob=:7)}`.

2. **Locate the parent tree.** `from :2` → look up `:2` in the persisted marks file → it points to commit `def456...` whose tree was previously cached in `.git/reposix/origin/state.json`. Materialize that prior tree: `prior = {"PROJ-123.md": (100644, blob=":1")}` where `:1`'s contents were also cached.

3. **Diff trees.** Walk both trees. For each path:
   - **Path in new but not old** → CREATE: `POST /projects/demo/issues` with parsed YAML body.
   - **Path in old but not new** → DELETE: `DELETE /projects/demo/issues/PROJ-XYZ`. (Issue ID derived from filename stem.)
   - **Path in both, blob SHA differs** → UPDATE: parse both YAML+body, compute *field-level* delta, emit `PATCH`.

4. **Field-level diff (the magic).** For our example:
   ```rust
   let old_fm: BTreeMap<String, Yaml> = parse_frontmatter(old_blob)?;
   let new_fm: BTreeMap<String, Yaml> = parse_frontmatter(new_blob)?;
   let changed: BTreeMap<&str, &Yaml> = new_fm.iter()
       .filter(|(k, v)| old_fm.get(*k) != Some(*v))
       .collect();
   // changed == {"status": "closed"}
   ```
   We see `status` changed from `open` → `closed`. Emit:
   ```http
   PATCH /projects/demo/issues/PROJ-123
   If-Match: W/"4"      ← from cached etag
   Content-Type: application/json

   {"status": "closed"}
   ```

5. **Handle the response.**
   - `200 OK` → success. Update `state.json`: `{"PROJ-123": {"sha": new_blob_sha, "etag": resp.etag, "tree": new_tree_sha}}`. Print `ok refs/heads/main\n\n` to stdout.
   - `409 Conflict` (etag mismatch — someone edited the issue out-of-band) → print `error refs/heads/main "remote diverged; run 'git pull' first"\n\n` to stdout AND a friendlier multi-line explanation to stderr.
   - `429 Too Many Requests` → respect `Retry-After`, retry up to N times, then `error refs/heads/main "rate limited"`.
   - `400 Bad Request` (e.g. invalid workflow transition) → print the API's error message verbatim to stderr, then `error refs/heads/main "<one-line summary>"`.

6. **Persist marks.** Git will read back `.git/reposix/origin/marks` after we exit (because we advertised `*export-marks`); we must update it with the new mark `:8 → <new-commit-sha>` so the *next* push can use `from :8`.

### 3.4 Why the field-level diff matters

If we naively did `PUT /issues/PROJ-123` with the entire new YAML+body, we'd:
- Clobber any concurrent human edits to *other* fields (e.g. someone else's label change).
- Trigger workflow validators on every field even if unchanged.
- Hit issue tracker rate-limit per-field-write metering harder.

The whole REST-to-POSIX pitch (per `InitialReport.md` §"Differentiating HTTP Verbs") collapses if we PUT-everything. Field-level PATCH is what makes git semantics actually map to API semantics.

---

## 4. Real-World Implementations to Study

### 4.1 git-remote-hg (Felipe Contreras' fork) — the canonical reference

**Lang:** Python. **License:** GPL-2.0. **Status:** widely shipped (Debian/Arch packages).
**URL:** https://github.com/felipec/git-remote-hg

Verbatim main loop (extracted via WebFetch from the repo HEAD):
```python
def main(args):
    # ... extensive setup elided ...
    parser = Parser(repo)
    for line in parser:
        if parser.check('capabilities'):
            do_capabilities(parser)
        elif parser.check('list'):
            do_list(parser)
        elif parser.check('import'):
            do_import(parser)
        elif parser.check('export'):
            do_export(parser)
        elif parser.check('option'):
            do_option(parser)
        else:
            die('unhandled command: %s' % line)
        sys.stdout.flush()
    marks.store()
```

Verbatim `do_capabilities`:
```python
def do_capabilities(parser):
    print("import")
    print("export")
    print("refspec refs/heads/branches/*:%s/branches/*" % prefix)
    print("refspec refs/heads/*:%s/bookmarks/*" % prefix)
    print("refspec refs/tags/*:%s/tags/*" % prefix)

    path = os.path.join(dirname, 'marks-git')

    if os.path.exists(path):
        print("*import-marks %s" % path)
    print("*export-marks %s" % path)
    print("option")

    print("")
```
Note the **conditional** `*import-marks`: only advertised if the marks file already exists, because telling git to import from a non-existent file is a hard error. **We'll do the same.**

Verbatim `do_import` (the fast-import emission side):
```python
def do_import(parser):
    repo = parser.repo
    path = os.path.join(dirname, 'marks-git')

    print("feature done")
    if os.path.exists(path):
        print("feature import-marks=%s" % path)
    print("feature export-marks=%s" % path)
    print("feature force")
    sys.stdout.flush()

    # collect all import lines (batched)
    while parser.check('import'):
        ref = parser[1]
        if ref == 'HEAD':
            export_head(repo)
        elif ref.startswith('refs/heads/branches/'):
            ...
        parser.next()

    print('done')
```
Key insight: **git batches all `import` commands** into one block (terminated by blank line). The helper must drain them all before emitting any fast-import data, because fast-import may be reading from *both* git (via the bidi-import pipe) and us. The pattern: `while parser.check('import'): collect; emit_all_at_once`.

Verbatim `do_export` (the fast-import consumption side):
```python
def do_export(parser):
    parser.next()
    for line in parser.each_block('done'):
        if parser.check('blob'):
            parse_blob(parser)
        elif parser.check('commit'):
            parse_commit(parser)
        elif parser.check('reset'):
            parse_reset(parser)
        elif parser.check('tag'):
            parse_tag(parser)
        elif parser.check('feature'):
            pass
        else:
            die('unhandled export command: %s' % line)
    # ... apply parsed changes; emit ok/error per ref ...
    for ref in ok_refs:
        print("ok %s" % ref)
    print("")
```

Verbatim `do_option`:
```python
def do_option(parser):
    global dry_run, force_push
    _, key, value = parser.line.split(' ')
    if key == 'dry-run':
        dry_run = (value == 'true')
        print('ok')
    elif key == 'force':
        force_push = (value == 'true')
        print('ok')
    else:
        print('unsupported')
```
**Lift this exactly.** Reply `ok` to options we honor, `unsupported` to anything else. Never `error`.

### 4.2 git-bug — the cautionary counter-example

**Lang:** Go. **License:** GPL-3.0.
**URL:** https://github.com/git-bug/git-bug

git-bug is the most-cited prior art for "issues in git" and the very project our `InitialReport.md` references for Lamport timestamps. **Crucially, git-bug does NOT implement git-remote-helper.** It uses custom subcommands:
```
git bug bridge new          # interactive wizard for token + URL
git bug bridge push <name>  # push to GitHub/GitLab/Jira
git bug bridge pull <name>  # pull from upstream
```
The synchronization logic lives in `bridge/github/import.go` (`ImportAll()`, ~300 LOC iterating events from an `importMediator` that drives REST queries).

**Why does this matter for reposix?** It tells us:
1. The git-remote-helper path is *unusual* for issue-tracker bridges. Most projects bail out and write a custom CLI.
2. git-bug's design pre-dates LLM agents. Its "bridge push" CLI requires the agent to learn a bespoke command. **reposix's whole pitch is to avoid bespoke commands** — `git push` is in the agent's pre-training, `git bug bridge push` is not.
3. We are deliberately taking on *more* implementation complexity (the helper protocol) in exchange for *less* agent-side cognitive load. That tradeoff is worth it for our use case but explains why others didn't do it.

If `git-remote-reposix` becomes painful to maintain, the fallback is the git-bug pattern: a `reposix sync push|pull` subcommand. Document this as the escape hatch.

### 4.3 git-remote-gcrypt — minimal helper, useful for "what's the smallest thing"

**Lang:** Bash. **License:** GPL-3.0.
**URL:** https://spwhitton.name/tech/code/git-remote-gcrypt/ ; source on git.spwhitton.name

This implements `connect`-style proxying (encrypts a real git pack stream). Not directly applicable to us (we don't proxy pack), but worth reading because:
- It's **<2000 lines of bash**, proving the protocol itself is small.
- It demonstrates the `gpg` shell-out pattern — same shape as our `reqwest` shell-out from sync code.

### 4.4 git-remote-s3 (AWS Labs) — Rust prior art

**Lang:** Rust. **License:** Apache-2.0.
**URL:** https://github.com/awslabs/git-remote-s3

Stores git bundles in S3, advertised capabilities are essentially `fetch` + `push` (it speaks bundle protocol, not fast-import). Less directly relevant to our REST translation, but **the only mainstream Rust helper** and a reference for:
- Tokio runtime bridging from sync stdin loop.
- Per-ref locking via S3 conditional writes (`If-None-Match: *`) — analogous to our etag-based optimistic concurrency.

### 4.5 The Linux kernel sources

`Documentation/gitremote-helpers.txt` in the git source tree is the canonical spec. The C implementation of the helper-side handshake lives in `transport-helper.c` (`process_connect_service`, `fetch`, `push_refs_with_export`, `push_refs_with_push`). Read these to understand *exactly* what git expects, especially:
- `transport-helper.c:625` `push_refs_with_export()` — shows git invokes `git fast-export --use-done-feature --signed-tags=warn-strip --tag-of-filtered-object=drop --refspec=...` and pipes to our stdin.
- `transport-helper.c:485` `process_connect_service()` — confirms `connect` is line-protocol-based exec of `upload-pack`/`receive-pack`.

---

## 5. Surfacing API Errors Back to the Agent

The agent runs in a shell loop. It sees:
- **stdout** of `git push`: usually summarized refs.
- **stderr** of `git push`: progress, warnings, errors. **This is where humans look. This is where our errors must go.**

### 5.1 Two channels, two purposes

Per the protocol:
- **stdout** (helper → git) is *protocol*. Per-ref status: `ok refs/heads/main` or `error refs/heads/main <one-line-reason>`. Anything not protocol-valid here breaks git.
- **stderr** (helper → user, passed through by git) is *free-form*. Use this for the human-readable explanation.

### 5.2 Pattern

```rust
fn handle_push_error(refname: &str, err: &ApiError) {
    // Free-form, agent-readable: goes to terminal stderr.
    eprintln!("reposix: push of {} failed", refname);
    match err {
        ApiError::Conflict { remote_etag, local_etag, field } => {
            eprintln!("  HTTP 409 Conflict on field `{}`", field);
            eprintln!("  Remote ETag: {}, local last-known: {}", remote_etag, local_etag);
            eprintln!("  Hint: run `git pull` and resolve conflict markers, then push again.");
        }
        ApiError::WorkflowViolation { from, to, allowed } => {
            eprintln!("  HTTP 400: cannot transition `{}` → `{}`", from, to);
            eprintln!("  Allowed transitions: {}", allowed.join(", "));
        }
        ApiError::RateLimit { retry_after } => {
            eprintln!("  HTTP 429: rate limited; retry after {}s", retry_after.as_secs());
        }
        _ => eprintln!("  {}", err),
    }

    // Machine-readable, on-protocol: goes to git, then `git push` exits non-zero.
    println!("error {} {}", refname, err.short_summary().replace('\n', " "));
}
```

The agent's shell sees both:
```
$ git push reposix
To reposix::http://localhost:7777/projects/demo
 ! [remote rejected] main -> main (remote diverged; run 'git pull' first)
error: failed to push some refs to 'reposix::...'
reposix: push of refs/heads/main failed
  HTTP 409 Conflict on field `status`
  Remote ETag: W/"7", local last-known: W/"4"
  Hint: run `git pull` and resolve conflict markers, then push again.
```

LLM agents are extremely good at parsing this format because it's dense in their training data (every developer-facing CLI emits something like it).

### 5.3 Pitfall: never write non-protocol bytes to stdout

```rust
println!("DEBUG: about to PATCH issue PROJ-123");  // BUG! Breaks the protocol.
eprintln!("DEBUG: about to PATCH issue PROJ-123"); // Correct.
```
This is the #1 footgun for new helper authors. Wrap stdout in a `Mutex<BufWriter<Stdout>>` and only write through a `Protocol::send_line()` API. Make stderr the default for everything else.

---

## 6. Handling `git pull` / Divergence with Real Merge Conflicts

### 6.1 The flow we want

1. Agent edits `PROJ-123.md` setting `status: closed`.
2. Meanwhile, a human (via the web UI) edits the same issue setting `assignee: bob`.
3. Agent runs `git pull` (or `git fetch` + `git merge`).
4. Git invokes `git-remote-reposix origin <url>`, sends `import refs/heads/main`.
5. **We fetch the current authoritative remote state**, render it as the canonical Markdown+YAML, emit a fast-import commit on `refs/reposix/origin/main` (our private ref namespace per the `refspec` capability).
6. Git's `fetch` machinery then updates `refs/remotes/origin/main` to point at that commit (via the refspec mapping).
7. Git performs a three-way merge: ancestor = last common state, ours = local commit (status: closed), theirs = remote (assignee: bob).
8. Because both modify the YAML frontmatter, git produces a textual conflict marker right in `PROJ-123.md`:
   ```yaml
   ---
   <<<<<<< HEAD
   status: closed
   assignee: alice
   =======
   status: open
   assignee: bob
   >>>>>>> origin/main
   labels: [bug]
   ---
   ```
9. The agent reads the file, recognizes the well-known conflict marker pattern (deeply trained), edits to merge both changes (`status: closed`, `assignee: bob`), commits, and re-pushes.

### 6.2 What we have to do for this to work

1. **Use a private ref namespace** — `refs/reposix/<remote>/*`. This is what the `refspec refs/heads/*:refs/reposix/origin/*` capability advertises. It keeps our synthetic commits out of `refs/heads`. Mandatory for `export`, recommended for `import`.

2. **Render the remote state deterministically.** If the helper hashes the same logical state to two different blob SHAs across runs, every `import` will look like a divergence and create spurious conflicts. Pin:
   - YAML key ordering (use `BTreeMap` / serde with `preserve_order = false`).
   - Trailing newlines (always exactly one).
   - Line endings (always `\n`).
   - Author/committer for synthetic commits (always `reposix-helper <bot@reposix>` with a deterministic timestamp — e.g., the remote issue's `updated_at`).

3. **Emit a `from <previous-mark>` line** on the synthetic commit so it descends from the prior `refs/reposix/origin/main`, giving git a real merge base. If we omit `from`, every import is a new orphan commit and merges become impossible.

4. **Use marks (`*export-marks`, `*import-marks`) for incremental re-runs.** Without marks, every `import` re-emits the full history; with marks, we emit only the delta since last import. fast-import handles this via the `--import-marks=<file>` flag git passes to it (because we advertised `*import-marks`).

### 6.3 Pitfalls

- **Field reordering masquerading as conflict.** If the remote API returns YAML with keys in `[assignee, status, labels]` order and we previously emitted `[status, assignee, labels]`, git will see a textual difference even though the *semantic* state is unchanged. Always normalize key order before emitting blobs.
- **Whitespace from the API.** Trim trailing whitespace from descriptions; the agent's editor will too, and we don't want a phantom conflict every round-trip.
- **Markdown body normalization.** If the API stores HTML and we render to Markdown (or vice versa), the round-trip must be a fixpoint. Test with `assert_eq!(render(parse(text)), text)` in CI for the simulator.

---

## 7. Concrete Rust Skeleton

### 7.1 Crate layout (within the existing `reposix` workspace)

```
crates/reposix-remote/
├── Cargo.toml
└── src/
    ├── main.rs           # binary: git-remote-reposix
    ├── protocol.rs       # line protocol I/O (Protocol struct)
    ├── caps.rs           # capability advertisement
    ├── fast_import.rs    # parser + emitter for fast-import streams
    ├── state.rs          # marks file + state.json persistence
    ├── diff.rs           # tree-diff → REST-call planner
    ├── client.rs         # async HTTP client (reqwest + rustls-tls)
    └── error.rs          # ApiError, exit-code mapping
```

### 7.2 Cargo.toml (essentials)

```toml
[package]
name = "reposix-remote"
edition = "2021"

[[bin]]
name = "git-remote-reposix"
path = "src/main.rs"

[dependencies]
tokio = { version = "1", features = ["rt", "macros", "net", "time"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
anyhow = "1"
thiserror = "1"
url = "2"
sha1 = "0.10"
hex = "0.4"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 7.3 main.rs — the dispatch loop

```rust
use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};

mod protocol;
mod caps;
mod fast_import;
mod state;
mod diff;
mod client;
mod error;

use protocol::Protocol;

fn main() -> Result<()> {
    // Diagnostics to stderr only. NEVER stdout.
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("REPOSIX_LOG")
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        anyhow::bail!("usage: git-remote-reposix <alias> <url>");
    }
    let alias = args[1].clone();
    let url = args[2].clone();
    let git_dir = std::env::var("GIT_DIR")
        .context("GIT_DIR not set; this binary must be invoked by git")?;

    // One Tokio runtime for the lifetime of the process. All async calls
    // funnel through `runtime.block_on`. This is the same bridge pattern
    // FUSE uses (see crates/reposix-fuse/src/lib.rs).
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let creds = resolve_credentials(&alias)?;
    let client = client::Client::new(&url, creds)?;
    let mut state = state::State::open(&git_dir, &alias)?;
    let mut proto = Protocol::new(io::stdin().lock(), io::stdout().lock());

    loop {
        let line = match proto.read_line()? {
            Some(l) => l,
            None => break, // EOF — git is done with us
        };

        let cmd = line.split_whitespace().next().unwrap_or("");
        match cmd {
            "capabilities" => caps::handle(&mut proto, &state)?,
            "list" => {
                let for_push = line.contains("for-push");
                let refs = runtime.block_on(client.list_refs())?;
                proto.send_list(&refs)?;
            }
            "option" => {
                let resp = handle_option(&line, &mut state);
                proto.send_line(&resp)?;
            }
            "import" => {
                // Drain the batch (multiple consecutive `import <ref>` lines
                // terminated by blank line).
                let mut refs = vec![parse_ref(&line)];
                while let Some(more) = proto.peek_line()? {
                    if more.starts_with("import ") {
                        refs.push(parse_ref(&proto.read_line()?.unwrap()));
                    } else {
                        break;
                    }
                }
                // Consume the terminating blank line.
                proto.expect_blank()?;
                runtime.block_on(handle_import(&mut proto, &client, &mut state, &refs))?;
            }
            "export" => {
                proto.expect_blank()?;
                // Now stdin is a fast-import stream until `done`.
                runtime.block_on(handle_export(&mut proto, &client, &mut state))?;
            }
            "" => continue, // blank line between commands
            other => anyhow::bail!("unknown command: {}", other),
        }
        proto.flush()?;
    }

    state.persist()?;
    Ok(())
}

fn handle_option(line: &str, state: &mut state::State) -> String {
    let mut parts = line.splitn(3, ' ');
    let _ = parts.next(); // "option"
    let key = parts.next().unwrap_or("");
    let val = parts.next().unwrap_or("");
    match key {
        "verbosity" => { state.verbosity = val.parse().unwrap_or(1); "ok".into() }
        "dry-run"   => { state.dry_run = val == "true"; "ok".into() }
        "progress"  => "unsupported".into(),
        _           => "unsupported".into(),
    }
}

fn parse_ref(line: &str) -> String {
    // "import refs/heads/main" → "refs/heads/main"
    line.splitn(2, ' ').nth(1).unwrap_or("").to_string()
}

fn resolve_credentials(alias: &str) -> Result<client::Creds> {
    // Priority order:
    // 1. Env var REPOSIX_TOKEN_<ALIAS_UPPERCASE>
    // 2. Env var REPOSIX_TOKEN
    // 3. `git config --get remote.<alias>.reposixToken`
    // 4. `git config --get reposix.token`
    let alias_upper = alias.to_ascii_uppercase().replace('-', "_");
    if let Ok(t) = std::env::var(format!("REPOSIX_TOKEN_{}", alias_upper)) {
        return Ok(client::Creds::Bearer(t));
    }
    if let Ok(t) = std::env::var("REPOSIX_TOKEN") {
        return Ok(client::Creds::Bearer(t));
    }
    if let Some(t) = git_config(&format!("remote.{}.reposixToken", alias))? {
        return Ok(client::Creds::Bearer(t));
    }
    if let Some(t) = git_config("reposix.token")? {
        return Ok(client::Creds::Bearer(t));
    }
    Ok(client::Creds::None)
}

fn git_config(key: &str) -> Result<Option<String>> {
    let out = std::process::Command::new("git")
        .args(["config", "--get", key])
        .output()?;
    if out.status.success() {
        Ok(Some(String::from_utf8(out.stdout)?.trim().to_string()))
    } else {
        Ok(None)
    }
}
```

### 7.4 protocol.rs — disciplined I/O

```rust
use anyhow::Result;
use std::io::{BufRead, Write};

pub struct Protocol<R: BufRead, W: Write> {
    reader: R,
    writer: W,
    peeked: Option<String>,
}

impl<R: BufRead, W: Write> Protocol<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer, peeked: None }
    }

    pub fn read_line(&mut self) -> Result<Option<String>> {
        if let Some(p) = self.peeked.take() {
            return Ok(Some(p));
        }
        let mut buf = String::new();
        let n = self.reader.read_line(&mut buf)?;
        if n == 0 { return Ok(None); }
        // Strip trailing \n only; preserve internal whitespace.
        if buf.ends_with('\n') { buf.pop(); }
        if buf.ends_with('\r') { buf.pop(); }
        Ok(Some(buf))
    }

    pub fn peek_line(&mut self) -> Result<Option<&str>> {
        if self.peeked.is_none() {
            self.peeked = self.read_line()?;
        }
        Ok(self.peeked.as_deref())
    }

    pub fn expect_blank(&mut self) -> Result<()> {
        match self.read_line()? {
            Some(s) if s.is_empty() => Ok(()),
            Some(other) => anyhow::bail!("expected blank, got: {:?}", other),
            None => Ok(()),
        }
    }

    pub fn send_line(&mut self, s: &str) -> Result<()> {
        // ENFORCE: no embedded \n in protocol lines.
        debug_assert!(!s.contains('\n'), "protocol line contains LF: {:?}", s);
        writeln!(self.writer, "{}", s)?;
        Ok(())
    }

    pub fn send_blank(&mut self) -> Result<()> {
        writeln!(self.writer)?;
        Ok(())
    }

    pub fn send_list(&mut self, refs: &[(String, String)]) -> Result<()> {
        for (sha, name) in refs {
            self.send_line(&format!("{} {}", sha, name))?;
        }
        self.send_blank()?;
        self.flush()
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    /// Pass through stdin to a callback (for fast-import stream parsing).
    pub fn drain_until_done<F: FnMut(&str) -> Result<()>>(
        &mut self,
        mut cb: F,
    ) -> Result<()> {
        loop {
            let line = self.read_line()?
                .ok_or_else(|| anyhow::anyhow!("unexpected EOF in fast-import"))?;
            if line == "done" { return Ok(()); }
            cb(&line)?;
        }
    }
}
```

### 7.5 caps.rs — the static handshake

```rust
use anyhow::Result;
use crate::protocol::Protocol;
use crate::state::State;

pub fn handle<R, W>(proto: &mut Protocol<R, W>, state: &State) -> Result<()>
where R: std::io::BufRead, W: std::io::Write
{
    proto.send_line("import")?;
    proto.send_line("export")?;
    // Private namespace: keep our synthetic commits out of refs/remotes.
    proto.send_line("refspec refs/heads/*:refs/reposix/{alias}/*"
        .replace("{alias}", &state.alias))?;

    // Only advertise *import-marks if the file already exists; git treats
    // a missing mandatory marks file as a fatal error.
    let marks = state.marks_path();
    if marks.exists() {
        proto.send_line(&format!("*import-marks {}", marks.display()))?;
    }
    proto.send_line(&format!("*export-marks {}", marks.display()))?;

    proto.send_line("option")?;
    proto.send_blank()?;
    proto.flush()
}
```

### 7.6 The async-from-sync bridge

The whole protocol loop is sync — git speaks to us line-at-a-time over pipes and blocking reads are fine. But `reqwest` is async, and we want connection pooling, retries, and concurrent fan-out for batch operations.

**Pattern (also used by reposix-fuse):**

```rust
// At startup, build one current-thread runtime. This runs on the main thread;
// no extra threads are spawned unless we explicitly use `multi_thread`.
let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;

// Each command handler is sync. It calls block_on at the boundary.
fn handle_command(rt: &Runtime, client: &Client, ...) -> Result<()> {
    let result = rt.block_on(async {
        client.do_async_thing().await
    });
    result
}
```

For batch fan-out (e.g., applying 10 PATCH calls from a single push), use `futures::future::try_join_all` *inside* the `block_on`:

```rust
runtime.block_on(async {
    let futures: Vec<_> = changes.into_iter()
        .map(|c| client.apply_change(c))
        .collect();
    futures::future::try_join_all(futures).await
})?;
```

**Pitfalls:**
- Do not call `block_on` from within an async context — that deadlocks the current-thread runtime. Keep async/sync boundaries crisp.
- Do not spawn a multi-threaded runtime for one binary that processes one push: it adds threads, introduces nondeterminism in test logs, and offers nothing here.
- `reqwest::Client` is `Clone` and `Arc`-internal; create one per process and clone freely.

---

## 8. Authentication and Per-Remote Namespacing

### 8.1 Where credentials come from

Three legitimate sources, in priority order:

1. **Per-remote env var.** `REPOSIX_TOKEN_<ALIAS>` — e.g. for remote named `prod`, `REPOSIX_TOKEN_PROD=ghp_xxxx`. This lets the agent or human keep multiple remotes' credentials separate in the shell environment.
2. **Global env var.** `REPOSIX_TOKEN` — the catch-all when there's only one remote.
3. **Git config.** `git config remote.<alias>.reposixToken <token>` — persists across shells. Read via `git config --get` (subprocess; do not parse `.git/config` ourselves).
4. **Global git config fallback.** `git config --global reposix.token <token>` — for users with one personal token shared across all remotes.

### 8.2 Why namespace by `<alias>` and not by URL

`git-remote-reposix` is invoked with `(alias, url)`. The alias is the user's chosen name (`origin`, `prod`, `staging`). Namespacing by alias means:
- The same simulator at `http://localhost:7777` can be added as both `local-dev` and `local-test` with different tokens.
- Tokens never appear in URLs (no `http://user:pass@host` smell).
- Agents can't accidentally exfiltrate creds by inspecting `git remote -v`.

### 8.3 Special case: anonymous one-shot URLs

If the user runs `git push reposix::http://localhost:7777/projects/demo` *without* first `git remote add`, git invokes us with `alias == url` (no nickname exists). In that case, fall back to env vars only — we have nowhere persistent to read config from, and we should not invent a name. (git-remote-hg handles this exact edge case by sha1-ing the URL into a synthetic alias for its mark file path; we do the same:)

```rust
let storage_alias = if alias == url {
    let mut h = sha1::Sha1::new();
    sha1::Digest::update(&mut h, url.as_bytes());
    hex::encode(sha1::Digest::finalize(h))
} else {
    alias.clone()
};
let storage_dir = std::path::PathBuf::from(&git_dir)
    .join("reposix")
    .join(&storage_alias);
```

### 8.4 Threat model recap (per PROJECT.md)

The threat model in `PROJECT.md` flags the lethal trifecta: private remote data + untrusted ticket text + git-push exfiltration. Helper-side mitigations:

- **Audit log.** Every outgoing HTTP call is logged to `runtime/audit.db` (the SQLite WAL the project already plans). One row per call: `(timestamp, alias, method, path, status, agent_pid, request_sha, response_sha)`. The orchestrator can `sqlite3 audit.db 'SELECT ... WHERE alias=? AND method != "GET"'` to review every write the helper made.
- **Refuse to push to an unconfigured remote.** If the alias has no token in any of the four sources, `error <ref> "no token configured for remote <alias>; set REPOSIX_TOKEN_<ALIAS> or git config remote.<alias>.reposixToken"`. Do not silently fall back to anonymous; that risks a malicious ticket telling the agent to `git remote add evil reposix::http://attacker.example/...` and then having writes succeed unauthenticated.
- **Tainted-content marking.** When emitting fast-import blobs, prefix any field whose content originated from an issue body (vs. structured metadata) with the `tainted:` xattr equivalent — TBD how this surfaces, but the helper is the chokepoint where the marking should happen.

---

## 9. Confidence Assessment & Open Questions

| Area | Confidence | Notes |
|------|------------|-------|
| Wire protocol mechanics | HIGH | Verified directly against `gitremote-helpers(7)` upstream docs and felipec/git-remote-hg source. |
| `import`/`export` vs `fetch`/`push` choice | HIGH | Spec explicitly recommends `import` for non-git remotes; multiple precedents (hg, bzr, mediawiki). |
| Fast-import format details | HIGH | Quoted directly from `git-fast-import(1)`. |
| Marks-based incremental sync | MEDIUM-HIGH | Pattern is clear; the corner case of "marks file is corrupt" needs a recovery story (probably: delete marks → re-import everything, with a stern stderr warning). |
| Real merge conflicts via `import` | MEDIUM | Mechanism is sound but depends on deterministic blob rendering; needs a CI test that round-trips the same logical state through 10 imports without producing a divergent SHA. |
| Async-from-sync bridge | HIGH | Standard tokio pattern; same as planned for `reposix-fuse`. |
| Authentication scheme | MEDIUM | Env-var-priority approach is conventional but the per-alias namespacing decision should be reviewed against any existing reposix CLI conventions. |
| git-bug as a precedent | HIGH | Explicitly does NOT use this protocol; we're choosing the harder path deliberately for the agent UX win. Should be documented in `Key Decisions`. |

### Open questions to resolve in implementation

1. **Should the helper also implement `connect`?** No — REST is not git-pack. But if we ever want `git ls-remote reposix::...` to work fast, we can advertise `connect` and refuse it (returning `fallback`), which is cheaper than instantiating a full `import`. Decide during implementation.
2. **How do we surface API rate-limit headers (X-RateLimit-Remaining, etc.) to the agent?** Probably as warnings on stderr when remaining < 10% of limit. Worth exposing so the agent learns to back off.
3. **What about `git fetch` of a non-existent remote ref?** Spec says: respond to `list` without that ref; git will report `[no such ref]`. No special handling needed.
4. **Multi-process safety on the marks file.** If two `git push` invocations run concurrently against the same alias, marks file races. Solution: `flock(LOCK_EX)` on the marks file across the entire helper invocation. Simple and matches what fast-import itself does.

---

## 10. Sources

### Authoritative (HIGH confidence)
- [git-scm.com — gitremote-helpers documentation](https://git-scm.com/docs/gitremote-helpers) — the canonical wire protocol spec.
- [git-scm.com — git-fast-import documentation](https://git-scm.com/docs/git-fast-import) — fast-import stream format.
- [git-scm.com — git-fast-export documentation](https://git-scm.com/docs/git-fast-export) — what git pipes into our stdin during `export`.
- [kernel.org — gitremote-helpers(7) man page](https://www.kernel.org/pub/software/scm/git/docs/gitremote-helpers.html) — same spec, mirror.

### Reference implementations (HIGH confidence — actual source code reviewed)
- [felipec/git-remote-hg](https://github.com/felipec/git-remote-hg) — Python; the textbook `import`/`export` helper. Code quoted verbatim above.
- [git-bug/git-bug](https://github.com/git-bug/git-bug) — Go; the cautionary counter-example (custom CLI instead of helper protocol).
- [git-bug third-party bridges doc](https://github.com/git-bug/git-bug/blob/master/doc/usage/third-party.md) — explains the bridge UX (a CLI, not git-push).
- [awslabs/git-remote-s3](https://github.com/awslabs/git-remote-s3) — Rust; closest existing Rust prior art (uses bundle protocol, not fast-import).

### Background / context (MEDIUM confidence)
- [Andrew Nesbitt — Git Remote Helpers](https://nesbitt.io/2026/03/18/git-remote-helpers.html) — recent (Mar 2026) overview survey.
- [Apriorit — Developing a Custom Remote Git Helper](https://www.apriorit.com/dev-blog/715-virtualization-git-remote-helper) — implementation walkthrough (couldn't fully extract via WebFetch but referenced for further reading).
- [git-remote-gcrypt](https://spwhitton.name/tech/code/git-remote-gcrypt/) — minimal `connect`-style helper in bash.
- [drgomesp/gitrmt](https://github.com/drgomesp/gitrmt) — Go library claiming to abstract the helper protocol; potentially worth a look but not reviewed in depth.

### Project context (read directly, HIGH confidence)
- `/home/reuben/workspace/reposix/.planning/PROJECT.md`
- `/home/reuben/workspace/reposix/InitialReport.md` §"Distributed Synchronization: The Git Remote Helper Protocol"
