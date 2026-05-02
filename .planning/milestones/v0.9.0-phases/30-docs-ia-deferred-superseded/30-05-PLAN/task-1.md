← [back to index](./index.md)

# Task 1: Carve filesystem.md + git.md from docs/architecture.md

<task type="auto">
  <name>Task 1: Carve filesystem.md + git.md from docs/architecture.md</name>
  <files>docs/how-it-works/filesystem.md, docs/how-it-works/git.md, docs/how-it-works/index.md</files>
  <read_first>
    - `docs/architecture.md` (source — lines 82-223 cover the content being carved)
    - `docs/how-it-works/filesystem.md` (current skeleton with placeholder mermaid — preserve the mermaid fence verbatim, fill prose around it)
    - `docs/how-it-works/git.md` (current skeleton with placeholder mermaid — preserve mermaid fence verbatim)
    - `docs/how-it-works/index.md` (remove the "Stub" admonition at the end)
    - `.planning/notes/phase-30-narrative-vignettes.md` lines 197-210 (V2 supporting vignette for the merge-conflict example in git.md)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §"docs/how-it-works/filesystem.md" and §"docs/how-it-works/git.md"
  </read_first>
  <action>
    **docs/how-it-works/filesystem.md** — replace the body while preserving the existing mermaid fence verbatim. Target structure:

```markdown
# The filesystem layer

reposix exposes your tracker as a real POSIX filesystem via FUSE. A `cat` or `sed` in your shell becomes a `read`/`write` syscall, which becomes an HTTP call to your tracker, which becomes bytes back.

## Read path

When an agent runs `cat issues/PROJ-42.md`, the kernel dispatches the syscall to the FUSE daemon. The daemon validates the filename (see "Filename validation" below), resolves it to a tracker ID, calls the backend's `get_issue`, serializes the result to YAML frontmatter + markdown body, and returns bytes to the kernel. Ten-second timeout (SG-07) means a dead backend surfaces as `EIO`, not a hang.

[Preserve the existing mermaid fence from the skeleton — the sequence diagram with A/K/F/R participants.]

Key points:

- Filename validation happens at the FUSE boundary. `../../etc/passwd.md` is rejected with `EINVAL` before any HTTP call (SG-04).
- The 5-second timeout means a dead backend cannot hang the kernel; `ls` returns within 5s with `EIO` (SG-07).
- Every request goes through the audit middleware — append-only rows in SQLite (SG-06).

## Write path

Writes are bytes-in, bytes-out. No template expansion, no shell escape. The daemon accepts a full frontmatter + body payload, strips server-controlled fields (id, created_at, version per SG-03), and calls the backend's `update_issue` or `create_issue`.

Carve the 28-line write-path narrative from docs/architecture.md §"Write path" (lines 112-140) here — preserve the SG-03 frontmatter-strip story and the file rename/unlink semantics.

## The async bridge — sync FUSE over async Rust

Carve the async-bridge story from docs/architecture.md §"The async bridge" (lines 194-223). This is where the FUSE-sync-callback ↔ tokio-runtime bridge is explained. Keep the single-tokio-runtime-per-daemon invariant.

## Filename validation is the security boundary

Lift the filename-validation prose from docs/architecture.md surrounding the "Read path" section (the `validate_issue_filename` box). Cite `crates/reposix-fuse/src/fs.rs::validate_issue_filename` as the source of truth.
```

**docs/how-it-works/git.md** — replace the body while preserving the existing mermaid fence verbatim. Target structure:

```markdown
# The git layer

`git push` is the central verb. A commit is a diff; reposix translates the diff into `PATCH` / `POST` / `DELETE` calls against your tracker's REST API. Optimistic concurrency — when two agents race on the same ticket — surfaces as an ordinary text-file merge conflict, not a silent 409.

## git push — the round trip

When you run `git push`, git invokes `git-remote-reposix` as a remote helper subprocess. The helper reads the commit range, computes per-issue diffs, and dispatches backend calls in order. Each dispatch returns `200` (accepted) or `409` (conflict). On `409`, the helper writes a conflict marker into the working tree and returns a non-fast-forward error to git — exactly the failure mode git users already know how to resolve.

[Preserve the existing mermaid fence from the skeleton — the flowchart LR with commit → reposix → tracker → round-trip.]

## Optimistic concurrency as git merge

Carve docs/architecture.md §"Optimistic concurrency as git merge" (lines 169-192) here. Include the moneyline (line 192):

> The agent resolving the conflict never has to parse a JSON 409 error. It never has to hold two versions of the issue in context and synthesize a merge. It uses `sed` on a text file with unambiguous markers — a flow it has seen in every merge-conflict-resolution corpus it was trained on.

### Example — two agents racing on PROJ-42

Lift the V2 supporting vignette from `.planning/notes/phase-30-narrative-vignettes.md` lines 197-210:

\`\`\`bash
# Agent A
git pull
sed -i 's/^labels:.*/labels: [needs-review]/' issues/PROJ-42.md
git commit -am "add needs-review" && git push          # succeeds

# Agent B (simultaneous)
git pull
sed -i 's/^priority: medium/priority: high/' issues/PROJ-42.md
git commit -am "bump priority" && git push             # rejected: non-fast-forward
git pull --rebase    # A touched 'labels:', B touched 'priority:' — clean 3-way merge
git push             # both changes land
\`\`\`

Git is the CRDT. True conflicts — both agents edited `status:` — surface as human-readable merge markers on a text file, not as silent data loss.

## Bulk delete is capped (SG-02)

Carve the SG-02 bulk-delete-cap discussion from docs/architecture.md §"git push" section — a push containing >5 deletes is rejected unless the commit message contains `[allow-bulk-delete]`. Cite `crates/reposix-remote/src/diff.rs::plan` as the source of truth.
```

**docs/how-it-works/index.md** — remove the "Stub" admonition at the end. The bridge paragraph and grid-cards block stay exactly as they were. Target final content:

```markdown
# How reposix works

Under the hood, reposix is three pieces: a FUSE daemon that projects your tracker as a real filesystem, a git remote helper that turns pushes into API calls, and a sandboxed simulator that lets you run the whole thing offline. The three pages below open each of those, one at a time.

<div class="grid cards" markdown>

-   :material-folder-open: **[The filesystem layer](filesystem.md)**

    How `cat issues/PROJ-42.md` becomes an HTTP GET and back. Read path, write path, and filename validation as the security boundary.

-   :material-source-branch: **[The git layer](git.md)**

    How `git push` becomes a series of `PATCH`s, and how optimistic-concurrency conflicts land as text-file merge conflicts instead of silent 409s.

-   :material-shield-lock: **[The trust model](trust-model.md)**

    The lethal-trifecta scenario reposix lives inside (private data, untrusted input, egress) and the cuts — taint typing, outbound allowlist, append-only audit, bulk-delete cap.

</div>
```

(Note: no `!!! note "Stub"` admonition — the bridge paragraph IS the final text.)

After writing, verify mermaid counts are unchanged (still exactly one per file) and that `mkdocs build --strict` still passes. Vale remains skipped on how-it-works (scope exemption).
  </action>
  <verify>
    <automated>test -f docs/how-it-works/filesystem.md && test -f docs/how-it-works/git.md && test -f docs/how-it-works/index.md && grep -c '```mermaid' docs/how-it-works/filesystem.md | grep -q '^1$' && grep -c '```mermaid' docs/how-it-works/git.md | grep -q '^1$' && ! grep -q 'Stub' docs/how-it-works/index.md && grep -q 'SG-03' docs/how-it-works/filesystem.md && grep -q 'SG-02' docs/how-it-works/git.md && grep -q 'sed -i .s/\^labels' docs/how-it-works/git.md && mkdocs build --strict</automated>
  </verify>
  <acceptance_criteria>
    - filesystem.md still has exactly 1 mermaid fence.
    - git.md still has exactly 1 mermaid fence.
    - `grep -c '^## ' docs/how-it-works/filesystem.md` returns `>= 3` (Read path, Write path, async bridge, or similar).
    - `grep -c '^## ' docs/how-it-works/git.md` returns `>= 3`.
    - `grep -c 'SG-0[1-9]' docs/how-it-works/filesystem.md` returns `>= 2` (SG-03, SG-04, SG-06, SG-07 referenced).
    - `grep -c 'SG-02' docs/how-it-works/git.md` returns `>= 1` (bulk-delete cap cited).
    - `grep -c 'needs-review' docs/how-it-works/git.md` returns `>= 1` (V2 vignette present).
    - `docs/how-it-works/index.md` does NOT contain the word "Stub" anywhere.
    - `wc -l docs/how-it-works/filesystem.md` reports `>= 60`.
    - `wc -l docs/how-it-works/git.md` reports `>= 60`.
    - `mkdocs build --strict` exits 0.
    - No P1 "replace" word introduced in either carved file (how-it-works is exempt from P2 but NOT from NoReplace — though NoReplace is scoped to docs/index.md only per `.vale.ini`).
  </acceptance_criteria>
  <done>
    Two how-it-works sub-pages carved with full prose; one mermaid each. how-it-works/index.md transitions from stub to final. `mkdocs build --strict` green.
  </done>
</task>
