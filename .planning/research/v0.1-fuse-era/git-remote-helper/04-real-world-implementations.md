← [back to index](./index.md)

# 4. Real-World Implementations to Study

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

git-bug is the most-cited prior art for "issues in git" and the very project our `docs/research/initial-report.md` references for Lamport timestamps. **Crucially, git-bug does NOT implement git-remote-helper.** It uses custom subcommands:
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
