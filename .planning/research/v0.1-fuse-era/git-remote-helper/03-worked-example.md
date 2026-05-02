← [back to index](./index.md)

# 3. Worked Example: One Issue Per File

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

The whole REST-to-POSIX pitch (per `docs/research/initial-report.md` §"Differentiating HTTP Verbs") collapses if we PUT-everything. Field-level PATCH is what makes git semantics actually map to API semantics.
