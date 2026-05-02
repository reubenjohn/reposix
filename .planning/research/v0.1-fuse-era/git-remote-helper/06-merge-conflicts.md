← [back to index](./index.md)

# 6. Handling `git pull` / Divergence with Real Merge Conflicts

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
