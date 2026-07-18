# Error codes (`RPX-xxxx`)

Every reposix error carries a stable `RPX-xxxx` code — a four-digit identifier
that never changes across releases. When a command fails, the code rides the
first line of the message in square brackets, and the last line tells you how to
read the full explanation:

```text
$ git push
… push rejected — the record changed on the backend since your last fetch [RPX-0505]
Fix: Fetch the newer backend state, rebase your change on top, then push again.
…
Explain: reposix explain RPX-0505
```

## Look up any code — `reposix explain`

`reposix explain <code>` prints the extended cause, fix, alternative, and a
copy-paste recovery command for that code — the same idea as `rustc --explain
E0308`:

```bash
reposix explain RPX-0505
```

```text
RPX-0505: push rejected — the record changed on the backend since your last fetch

A record you are pushing was modified on the backend after your last fetch, so
your push would overwrite a newer version. `git-remote-reposix` rejects with
git's standard `fetch first` status to protect the remote change; the
accompanying diag names the conflicting record, both versions, and a mirror-lag
hint. This is ordinary distributed-VCS drift, not a reposix fault.

Fix: Fetch the newer backend state, rebase your change on top, then push again.
Alternative: Run `reposix sync` to update the local cache from the backend directly, then `git rebase`.
Recovery:
  git pull --rebase   # bring in the newer backend state, replay your change
  git push            # re-drive the push once rebased
```

To list every code with its one-line summary:

```bash
reposix explain --list
```

**`reposix explain --list` is the always-current source of truth.** The tables
below mirror the built-in registry for browsing and search, but the CLI is what
ships in your binary — trust it if the two ever disagree.

## Two tiers, by design

reposix's error UX has two deliberately different surfaces, exactly like rustc's
compact `E0308` message versus the longer `rustc --explain E0308`:

- **The inline error** (what you see on stderr) is terse: a one-line headline
  with the `[RPX-xxxx]` tag, a `Fix:` line, an optional `Alternative:`, and a
  `Recovery:` block of copy-paste commands.
- **`reposix explain <code>`** prints the *extended* explanation — the mental
  model behind the failure and why it happens.

Only the code is shared between the two. Read the inline error first; run
`reposix explain` when you want the deeper "why".

## Code families

Codes are grouped into families by their numeric prefix. The family tells you
which part of reposix raised the error:

| Family     | Codes       | What it covers |
| ---------- | ----------- | -------------- |
| `RPX-00xx` | `0001`      | Backend spec parsing — the `<backend>::<project>` spec on `init` / `attach` / `sync` / `refresh`. |
| `RPX-01xx` | `0101`–`0102` | Missing real-backend credentials or tenant environment variables (the CLI and the git remote helper). |
| `RPX-02xx` | `0201`–`0204` | Local cache and working-tree binding — cache build, no synced cache, not-a-reposix-tree, unparseable stored remote URL. |
| `RPX-03xx` | `0301`–`0311` | Subcommand and flag preconditions — `log` / `spaces` / `refresh` mode gates, `--since` parsing and time-travel rewind, git-on-`PATH`, non-UTF-8 path. |
| `RPX-04xx` | `0401`–`0406` | `init` / `attach` working-tree bootstrap — existing-repo refusal, initial fetch failure, the `attach` reconciliation preconditions, and the `init` nested-in-worktree refusal (`0406` — a conservative pre-check plus a precise post-`git init` git-dir self-check; run `reposix explain RPX-0406` for the mechanism). |
| `RPX-05xx` | `0501`–`0508` | Git remote helper transport — upload-pack, protocol EOF, blob limit, backend unreachable (push and fetch), push conflict, unfiltered fetch, import-parent resolve. |
| `RPX-06xx` | `0601`–`0603` | DVCS bus / mirror and helper invocation — malformed bus URL, helper misuse, mirror unreachable. |
| `RPX-09xx` | `0900`      | `reposix explain` itself — an unknown or unregistered code was looked up. |

## All codes

The full index, mirroring `reposix explain --list`. Run `reposix explain <code>`
for the extended explanation of any row.

| Code | Summary |
| ---- | ------- |
| `RPX-0001` | invalid backend spec |
| `RPX-0101` | a real-backend credential or tenant environment variable is unset |
| `RPX-0102` | the git remote helper is missing backend credentials |
| `RPX-0201` | reposix could not build its local cache from the backend |
| `RPX-0202` | no synced reposix cache yet — nothing to read |
| `RPX-0203` | this directory is not a reposix working tree |
| `RPX-0204` | this tree's stored reposix remote URL could not be parsed |
| `RPX-0301` | `reposix log` currently requires the `--time-travel` flag |
| `RPX-0302` | `reposix spaces` supports only the Confluence backend |
| `RPX-0303` | `reposix refresh --offline` is not implemented yet |
| `RPX-0305` | invalid `--since` value |
| `RPX-0306` | reposix could not invoke `git` |
| `RPX-0307` | the `reposix init` target path is not valid UTF-8 |
| `RPX-0308` | `reposix init --since` has no cache to rewind into |
| `RPX-0309` | no sync tag at-or-before the `--since` timestamp |
| `RPX-0310` | the `--since` sync tag resolved but its commit could not be fetched |
| `RPX-0311` | could not move the working tree's refs to the `--since` snapshot |
| `RPX-0401` | refusing to `reposix init` over an existing git repository |
| `RPX-0402` | `reposix init` configured the tree but the initial fetch failed |
| `RPX-0403` | `reposix attach` needs an existing git working tree |
| `RPX-0404` | duplicate record `id` across local files — attach aborted |
| `RPX-0405` | this tree is already attached to a different system of record |
| `RPX-0406` | refusing to `reposix init` a target nested inside another git repository |
| `RPX-0501` | the reposix cache could not serve `git upload-pack` |
| `RPX-0502` | unexpected EOF mid-request (protocol desync) |
| `RPX-0503` | refusing to materialize more blobs than the configured limit |
| `RPX-0504` | push rejected — the backend was unreachable during the pre-push check |
| `RPX-0505` | push rejected — the record changed on the backend since your last fetch |
| `RPX-0506` | the reposix cache cannot serve an unfiltered fetch |
| `RPX-0507` | fetch rejected — the backend was unreachable while listing records to import |
| `RPX-0508` | fetch/import could not resolve the client's tracking tip |
| `RPX-0601` | malformed reposix bus URL |
| `RPX-0602` | `git-remote-reposix` was invoked with too few arguments |
| `RPX-0603` | the reposix bus mirror is unreachable or misconfigured |
| `RPX-0900` | no such reposix error code |

## Unknown code?

If `reposix explain <code>` reports `RPX-0900` ("no such reposix error code"),
the code is mistyped, from a newer reposix version, or not an RPX code at all.
Codes are always four digits. Run `reposix explain --list` to see every code
this build knows about.

## See also

- [CLI reference](cli.md) — the `reposix explain` subcommand and every other command.
- [Exit codes](exit-codes.md) — the process exit codes (`0` / `1` / `2`) that pair with these messages.
- [Troubleshooting](../guides/troubleshooting.md) — recovery playbooks for the common push/pull failures.
