← [back to index](./index.md)

## PART B — Architectural Holes

### B1. YAML frontmatter that overrides server-controlled fields

Covered in A2. Reiterating because it is also a non-security correctness bug: even a benign user can accidentally type `status: closed` in the body of a comment-style line and lose data on the next round-trip. The current design (`docs/research/initial-report.md` §"Modeling Hierarchical Directory Structures") is silent on the canonical → projection direction.

**Hole:** there is no defined schema for which YAML keys are server-authoritative vs. agent-writable.

### B2. `git push` interrupted halfway — atomicity story

The remote helper translates a single `git push` into N HTTP calls (one per modified file, possibly more for transitions). PROJECT.md and `docs/research/initial-report.md` both elide this. Failure modes:

- Push 50 issues, network drops after issue 30. Local git thinks 50 succeeded; remote has 30. Next push reapplies 1–50 but the simulator now 409s on the 30 already-applied ones.
- Push 1 issue with status transition + body update + label change as 3 API calls. Body update succeeds, transition fails on a workflow validator. Local state diverges from remote: body advances, status doesn't.
- Process killed mid-push. Audit log entry for "push started" exists; no entry for "push finished". On restart, no recovery logic exists.

**Holes:**

- No transaction journal in the remote helper.
- No idempotency keys on outbound API calls (the simulator is being designed without them per PROJECT.md L21–22 — "rate limits, 409 conflicts, workflow rules, RBAC" but no mention of idempotency).
- No two-phase apply: stage all changes locally, validate against a dry-run endpoint, then commit.

**Mitigation:** see PART D #3.

### B3. Two FUSE writers race on the same file

POSIX gives no application-level lock on `write()`; you need `flock` or `fcntl` advisory locks. With two agents writing to the same `PROJ-123.md`, the FUSE daemon will see two `write()` calls interleaved at arbitrary 4KB boundaries (FUSE default `max_write` is 4096 or 131072 depending on kernel/version). The result is byte-level interleaving; YAML frontmatter parse will fail or produce a garbage frontmatter that gets pushed.

**Hole:** PROJECT.md L27 explicitly mentions an "adversarial swarm harness" — i.e., reposix is **designed to be hammered concurrently** — but neither doc addresses write coordination.

**Mitigations:**

- Per-file mutex in the FUSE daemon, held across the entire write-to-flush cycle.
- Reject `O_APPEND` writes that would race with an in-flight body update.
- Test: see PART D #5.

### B4. Simulator offline — does FUSE block forever?

PROJECT.md L23 says FUSE is "backed by an async client to the simulator". Open questions:

- What does `read()` do when the simulator's HTTP socket is `ECONNREFUSED`?
- What's the timeout?
- Does it return cached data? If so, with what staleness contract?
- Does `getattr` (called constantly by every `ls`) stall the kernel waiting for a network round-trip?

If FUSE blocks indefinitely on `getattr`, the mount becomes unkillable without `umount -l` (lazy unmount), and any process that has touched the mount (including `bash` doing tab-completion) hangs in `D` state. The agent's terminal becomes unresponsive. This is a **demo-day showstopper** because the simulator is the most likely thing to crash during a recording.

**Holes:**

- No documented timeout policy.
- No documented behavior on simulator failure.
- No "fail fast" mode for the demo recording.

**Mitigations:**

- Set a hard timeout on every FUSE op (suggest 2s for `getattr`, 5s for `read`).
- On timeout, return `EAGAIN` or `ETIMEDOUT` (`docs/research/initial-report.md` §"Mitigating Rate Limits and API Exhaustion" says this is the design — make sure it's enforced for the offline case too).
- Cache `getattr` results for 30s to keep `ls` snappy.
- Test: see PART D #4.

### B5. Path traversal via issue title with `..`, `/`, NUL

The mapping is "Issue → Markdown File" (PROJECT.md L22, `docs/research/initial-report.md` table in §"Modeling Hierarchical Directory Structures"). What is the filename derivation?

- If filename = `slug(title) + '.md'`, an attacker creates an issue titled `../../etc/passwd.md` and the FUSE daemon happily exposes a path that, when written to, escapes the mount.
- A title with embedded `/` creates a directory the agent didn't expect.
- A title with embedded NUL byte (`\0`) terminates the path early in the C-string boundary between Rust and the FUSE kernel module.
- A title with control characters (newline, BEL) shows up in `ls` output and breaks any `xargs`-style pipeline.
- Two issues with the same slug after normalization → filename collision. Which one wins on write?

**Holes:**

- No documented filename derivation function.
- No documented collision policy.
- No input validation at the FUSE boundary.

**Mitigations:**

- Filename = issue ID (e.g., `PROJ-123.md`), never the title. Title goes in YAML frontmatter only.
- Reject any path component containing `/`, `\0`, or matching `^\.{1,2}$` at the FUSE boundary with `EINVAL`.
- Test: see PART D #6.

### B6. `rm -rf /mnt/reposix`

What does this do? The FUSE `unlink` op gets called for every file. Each `unlink` translates to a `DELETE /rest/api/3/issue/PROJ-N`. **In 30 seconds the agent has deleted the entire backlog.**

`docs/research/initial-report.md` §"Differentiating HTTP Verbs" actually anticipates this:

> "Deletion presents a unique edge case... If an agent simply executes `rm PROJ-123.md`, the system lacks the context to provide this metadata."

The InitialReport's proposed solution (parse commit message for delete reason) **does not protect against `rm -rf`** — `rm` doesn't go through git at all. It hits FUSE `unlink` directly.

PROJECT.md L166 talks about `chmod -R -w` for "Bulk change" admin permissions but doesn't operationalize it.

**Holes:**

- No bulk-delete throttle.
- No "delete confirmation" gate.
- No distinction between `rm` (filesystem op, immediate) and `git rm` + `commit` + `push` (deferred, batched, reversible).

**Mitigations:**

- FUSE `unlink` does NOT immediately call the API. It marks the file as deleted in a local journal. Only `git push` triggers the API DELETE.
- Push-time guard: refuse to push a single commit that deletes more than N files (configurable, default 5) without `--force-bulk-delete`.
- Audit log entry for every `unlink`, with caller PID.
- Test: see PART D #7.

### B7. Symlinks, hardlinks, and `setuid` bits

POSIX has more than `read`/`write`. What happens when the agent runs:

- `ln -s /etc/passwd /mnt/reposix/PROJ-pwned.md` — does the FUSE daemon follow the symlink? Does `cat PROJ-pwned.md` exfiltrate `/etc/passwd` into an issue body on the next push?
- `chmod u+s /mnt/reposix/PROJ-123.md` — does the FUSE daemon honor setuid? If the mount is on a real disk via writeback caching, yes.
- `mknod /mnt/reposix/foo c 1 3` — device file creation.

PROJECT.md and `docs/research/initial-report.md` don't mention any of these.

**Mitigations:**

- FUSE daemon refuses `symlink`, `link`, `mknod`, `chmod` ops with `EPERM`.
- Mount with `nosuid,nodev,noexec` flags.

### B8. The `.git/` directory is a confused deputy

The whole architecture relies on the agent running `git` in a working tree that is **itself** the FUSE mount? Or in a sibling working tree backed by the FUSE mount as a remote? PROJECT.md is ambiguous. Both modes have problems:

- **Mount-as-worktree:** `.git/` lives on FUSE. Every git operation does ~hundreds of `read`/`stat` calls. Performance dies. Worse, `.git/config` is now writable from inside the mount, meaning the lethal-trifecta attack A1 can rewrite it.
- **Mount-as-remote:** the agent has a local clone. The "git push" UX is preserved, but now there are two sources of truth (local clone vs. mount) and conflict resolution stories multiply.

**Hole:** the relationship between the FUSE mount and the git working tree is not documented.

**Mitigation:** Pick one explicitly. Recommend mount-as-remote. Document. Test that `.git/config` is not writable through the mount.

### B9. Codecov / CI badges leak repo metadata

PROJECT.md L28 says "Codecov coverage. Badges in README." A coverage badge URL is a public endpoint that, on a private repo, requires a token in the URL. Badges that include tokens get committed to README. Standard footgun.

**Mitigation:** Public repo (PROJECT.md L67 confirms this is `reubenjohn/reposix`, public), so badge URLs are fine. **But** re-verify before pushing.
