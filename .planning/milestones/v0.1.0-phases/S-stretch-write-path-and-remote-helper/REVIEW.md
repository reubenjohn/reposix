---
phase: S-stretch-write-path-and-remote-helper
reviewed: 2026-04-13T04:10:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/fetch.rs
  - crates/reposix-fuse/src/lib.rs
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/protocol.rs
  - crates/reposix-remote/src/diff.rs
  - crates/reposix-remote/src/fast_import.rs
  - crates/reposix-remote/src/client.rs
findings:
  blocker: 0
  high: 3
  medium: 4
  low: 4
  total: 11
status: issues_found
verdict: FIX-REQUIRED
---

# Phase S: Code Review Report

**Reviewed:** 2026-04-13T04:10:00Z
**Depth:** standard
**Commits in range:** `dc09b4a`, `b12036e`, `4006f13`
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase S ships two substantial capabilities (FUSE write path and
git-remote-reposix helper) with SG-02 bulk-delete cap, SG-03
tainted/sanitize discipline via `EgressPayload`, SG-07 5s timeouts, and
SG-05 audit-header attribution all intact end-to-end. Security invariants
hold: no direct `reqwest::Client` anywhere in the two crates, all outbound
PATCH/POST go through `sanitize`, server-controlled fields are physically
unreachable on the wire (verified by the
`sanitize_strips_server_fields_on_egress` test), `#![forbid(unsafe_code)]`
at every crate root, `#![deny(clippy::print_stdout)]` in `reposix-remote`.

No BLOCKER issues. However, there are THREE HIGH-severity correctness
issues in the helper's `ProtoReader` / stream-plumbing path that need
attention before the helper can be trusted for non-trivial content, and
ONE HIGH in the FUSE `create` path where server-assigned IDs create
kernel/backend inconsistency. The happy-path demo-scripted flow
(ASCII-only, LF newlines, empty-tree delete) is solid and matches what
SC #1–3 in `S-DONE.md` verified. The failure-path protocol contract is
weak — backend errors during `handle_import_batch` / `handle_export`
exit the process mid-protocol without sending a clean
`error refs/heads/main ...` line, leaving git to see a torn pipe.

Verdict: **FIX-REQUIRED** (demo can proceed; fixes should land before
any attempt to push a repo with CRLF, binary-ish bytes, or concurrent
helpers against a flaky backend).

## HIGH Issues

### H-01: `ProtoReader::read` strips `\r` from CRLF blobs, silently corrupting bytes

**Files:** `crates/reposix-remote/src/main.rs:329-350`, `crates/reposix-remote/src/protocol.rs:33-46`
**Category:** Correctness / data integrity
**Issue:**
`Protocol::read_line` (protocol.rs:40-44) strips both trailing `\n` and
any preceding `\r` (the classic CRLF handler). `ProtoReader::read`
(main.rs:336-342) re-adds only `\n` — never the stripped `\r`. For blob
bytes that contain CRLF (e.g., a Markdown body authored on Windows or
copied through a CRLF-normalizing tool), every `\r` is silently dropped.
`parse_export_stream::read_exact(&mut buf[..len])` then either
(a) under-reads because the reassembled stream is shorter than declared,
causing the parser to cross into the next directive line and consume it
as blob content, or (b) reads `len` bytes that differ from what git
committed.

Downstream: `frontmatter::parse` may or may not survive the corrupted
bytes; `bytes_match` in `diff::plan` (diff.rs:115-124) will definitely
diverge from `frontmatter::render`'s LF output, triggering spurious
PATCH calls, and the PATCH body will have LF-only where the blob had
CRLF. Silent lossy round-trip — exactly the "every push recomputes the
world" footgun S-DONE.md partially acknowledges but doesn't quantify.

**Fix:**
Either (a) preserve byte-exactness by not going through `String::read_line`
at all for the blob body — refactor `parse_export_stream` to read
`data N\n` headers line-wise but then take a raw `&mut dyn Read` for the
`read_exact(len)` call, or (b) track whether `read_line` stripped `\r`
and re-emit it in `ProtoReader::read`. Option (a) is safer:

```rust
// In parse_export_stream, after detecting `data <len>`:
let mut buf = vec![0u8; len];
proto.read_bytes_exact(&mut buf)?;  // new method on Protocol that
                                     // pulls from BufReader directly
```

Add a test with embedded CRLF in a blob body: `"hello\r\nworld\r\n"` →
round-trip through `parse_export_stream` → assert `blobs[1]` equals the
original 14 bytes.

### H-02: `ProtoReader::read` fails on non-UTF-8 blob bytes

**Files:** `crates/reposix-remote/src/main.rs:329-350`, `crates/reposix-remote/src/protocol.rs:33-46`
**Category:** Robustness / Correctness
**Issue:**
`Protocol::read_line` uses `String::read_line`, which returns
`io::ErrorKind::InvalidData` if any bytes are not valid UTF-8. For issue
blobs that legitimately contain non-UTF-8 bytes (a pasted binary, a
Latin-1 legacy body, an attachment reference), the helper surfaces a
misleading `"stream did not contain valid UTF-8"` error and fails the
push — git sees a torn protocol exchange.

The v0.1 contract says issues are Markdown + YAML (implicitly UTF-8), so
this is a known limitation. But the helper should either (a) hard-reject
non-UTF-8 at the planner with a clear "push rejected: invalid UTF-8 in
<path>" message on stderr + the protocol, or (b) actually plumb bytes
through (as in H-01's fix).

**Fix:**
Same root cause as H-01 — the byte-level reader can handle non-UTF-8
fine; only the line-reader chokes. Option (a) from H-01 fixes both.

Add a test with a blob containing `0xFF` (invalid UTF-8) to prove the
helper either accepts it (with byte fix) or rejects it with a clear
diagnostic.

### H-03: `handle_import_batch` / `handle_export` exit mid-protocol on backend errors (torn pipe for git)

**Files:** `crates/reposix-remote/src/main.rs:167, 194, 291, 304`
**Category:** Protocol robustness
**Issue:**
When `api::list_issues` (import batch) or `api::list_issues` (export
prior) or `api::patch/post/delete` fails with a `?`-propagated anyhow
error, `real_main` returns `Err(e)`, exits with code 2, and the
diagnostic prints to stderr via `diag()`. But no protocol response is
sent — git sees `git-remote-reposix` simply close its stdout. For import,
git has already sent the `import refs/heads/main` request and is waiting
for the fast-import stream + `done`; the torn pipe manifests as a
confusing upstream error (usually "fast-import failed"). For export, git
may hang or error unclearly.

The existing `execute_action` loop (main.rs:222-231) correctly catches
per-action errors and converts them to a clean
`error refs/heads/main some-actions-failed`. The bug is only the
pre-action pipeline: listing issues, parsing the export stream.

**Fix:**
Wrap the `.context(...)?` calls in a pattern that emits a protocol error
before bailing:

```rust
let prior = match state.rt.block_on(api::list_issues(&state.http, &state.origin, &state.project, &state.agent)) {
    Ok(v) => v,
    Err(e) => {
        diag(&format!("error: cannot list prior issues: {e:#}"));
        proto.send_line("error refs/heads/main backend-unreachable")?;
        proto.send_blank()?;
        proto.flush()?;
        state.push_failed = true;
        return Ok(());
    }
};
```

Same pattern for `parse_export_stream` and `handle_import_batch`'s
`list_issues`. Consider extracting a helper `fail_push(proto, state,
reason, err)` to DRY.

Add a test: wiremock returns 500 for `GET /projects/demo/issues`; assert
stdout contains `error refs/heads/main ...`, process exits non-zero,
git sees clean protocol closure (not a torn pipe).

### H-04: `create()` uses user-supplied name's id, but sim reassigns — kernel/backend divergence

**Files:** `crates/reposix-fuse/src/fs.rs:651-717`, `crates/reposix-sim/src/routes/issues.rs:208-253`
**Category:** Correctness / UX
**Issue:**
`create()` parses `0007.md` → `IssueId(7)`, builds a placeholder with
`id=7`, POSTs. The sim's `CreateIssueBody` has no `id` field; the sim
assigns `max_id + 1` regardless of what the client asked for (line 224).
The FUSE code then uses `new_issue.id` (the server-assigned id, say 6)
for the inode registry — but the kernel has already been told that
`0007.md` is a new file via the `ENTRY_TTL` cache line from `reply.created`.

Consequence: `ls /mnt` may now show both `0006.md` (server's view) and
`0007.md` (kernel's stale dirent with inode pointing at issue 6). `cat
/mnt/0007.md` serves issue 6's body including its frontmatter id of 6.
Very confusing; easy for agents to mis-attribute edits.

No security break (the sanitize step overwrites any client-controlled id
with server id), but a meaningful correctness/UX gap.

**Fix:**
Choice of:
- **(preferred) Kill client-specified ids on `create`.** Reject `touch
  /mnt/<name>.md` at the FUSE layer with `ENOTSUP` and document "use
  `git push` to create issues; FUSE `create` is not supported in v0.1."
  The FUSE code already has the machinery to return errno cleanly.
- **Rename the kernel's dirent.** After POST, if `new_issue.id != id`,
  invalidate the kernel's entry for the user's chosen name (fuser has a
  `notify_inval_entry` facility). Best done via a readdir refresh.
- **Teach the sim to honor a client-supplied id.** This may clash with
  unique-id invariants; probably not worth it for v0.1.

Add a test: `create /mnt/0042.md` when sim currently has ids 1..5; assert
either (a) `ENOTSUP` or (b) the dirent visible post-create matches the
server-assigned id.

## MEDIUM Issues

### M-01: `fast-export` parser rejects `M 100644 <SHA1> <path>` (non-mark tree entries)

**Files:** `crates/reposix-remote/src/fast_import.rs:173-183`
**Category:** Correctness
**Issue:**
The parser only strips the prefix `"M 100644 :"` (colon-prefixed mark).
Real `git fast-export` can emit `M 100644 <40-char-SHA1> <path>` when
invoked without `--marked` (the non-mark form). For remote-helper usage
git typically emits marks, so in practice this is fine — but a future
change to git's helper invocation flags, or certain edge cases during
incremental export with an `import-marks` file, could emit SHA form.

Similarly, `M 100755` (exec bit), `M 120000` (symlink), and `M 160000`
(gitlink) are silently ignored — they fall through the unhandled-line
path. An issue blob committed with exec mode (from a misconfigured
`chmod +x`) would be silently dropped, then plan() would classify it as
a delete, and on ≥6 concurrent such drops SG-02 would fire — which is
actually fail-safe, but a surprising error message for the user.

**Fix:**
Either (a) tolerate mode variations (accept any `M <octal> :MARK <path>`
or `M <octal> <sha> <path>`), or (b) reject unknown mode with a clear
error: `"refusing push: unsupported tree entry <line>"`.

Add a test with a synthetic fast-export stream containing `M 100755
:1 0001.md` and assert the helper produces a well-defined result.

### M-02: `ProtoReader` line-at-a-time loop means blob boundary alignment depends on blob-ends-with-LF

**Files:** `crates/reposix-remote/src/main.rs:329-350`, `crates/reposix-remote/src/fast_import.rs:147-164`
**Category:** Correctness
**Issue:**
After `read_exact(len)` consumes the blob bytes via line-glue, the parser
does `read_line(&mut maybe_nl)` to consume the optional trailing LF that
git emits after `data N\n<bytes>`. In the LF-only case this works
because `ProtoReader` has already fed a pristine stream terminated by
`\n`. But if the blob does NOT end with LF (unusual; frontmatter::render
always ends with LF so all emitted blobs from the FUSE side do; git may
emit differently), the `read_exact` call spans multiple upstream
`read_line` calls and the alignment is off by the `\n` that
`ProtoReader` appended spuriously.

Tied closely to H-01. The same underlying fix (byte-level read instead of
line-glue) closes this.

**Fix:**
See H-01. Not a separate fix; listed for traceability.

### M-03: Spurious Updates on no-op `git push` (frontmatter::render vs git's normalized blob can differ)

**Files:** `crates/reposix-remote/src/diff.rs:115-124`, `S-DONE.md` §"Deviations"
**Category:** Correctness / UX
**Issue:**
S-DONE.md explicitly calls this out: "when copying files from the FUSE
mount into a git working tree and committing, git's trailing-newline
handling can make blobs differ from `frontmatter::render` output. The
helper's `diff::plan` correctly treated this as an Update (PATCH) rather
than a no-op."

"Correctly" is doing a lot of work there. Consequence: a `git push` with
no semantic changes PATCHes every issue, bumping server versions,
generating audit-log entries, and in the presence of rate limiting
(SG-08 future) could hit quotas. If another client PATCHes concurrently,
a no-op push from client A may emit N PATCHes, of which several 409
because client B's versions are newer — cascading merge conflicts for
zero actual changes.

Not a security issue, but a footgun the demo should not rely on.

**Fix:**
Before computing `bytes_match`, normalize both sides through `parse +
render` — compare `render(parse(new_bytes))` against
`render(prior_issue)`. This makes "syntactically equivalent but
byte-different" pushes no-ops. Downside: more work per push. Alternative:
store the exact byte-serialized form server-side and byte-compare. For
v0.1 the first approach is cheaper.

Add a test: prior is `{id=1, body="foo"}`; new-blob bytes are
`render({...}) + "\n"` (extra trailing LF from git); assert plan yields
zero actions (not an Update).

### M-04: `five_deletes_passes_cap` mock test doesn't assert DELETE count expectation

**File:** `crates/reposix-remote/tests/bulk_delete_cap.rs:101-143`
**Category:** Test quality
**Issue:**
`six_deletes_refuses_and_calls_no_delete` uses `.expect(0)` on the DELETE
mock (line 64) — perfect, it proves zero DELETEs fire when the cap
refuses. But `five_deletes_passes_cap` (line 110) and
`six_deletes_with_allow_tag_actually_deletes` (line 155) set the DELETE
mock without `.expect(N)` and `.expect(6)` respectively. The allow-tag
test correctly asserts `.expect(6)`; the 5-deletes test does NOT assert
`.expect(5)` — so it doesn't prove that the 5 deletes actually fired.
The test only checks stdout contains `ok refs/heads/main`.

Given the `ok` line would still fire if plan() mis-counted and emitted
four deletes (or zero), this test is weaker than it looks.

**Fix:**
Add `.expect(5)` to the DELETE mock in `five_deletes_passes_cap`. Cheap
fix, tightens the invariant.

## LOW Issues

### L-01: Dead match arm `"list for-push"`

**File:** `crates/reposix-remote/src/main.rs:115`
**Category:** Dead code
**Issue:**
The dispatch code does `trimmed.splitn(2, char::is_whitespace)` at line
101 and takes `cmd = parts.next()`, so for input `"list for-push"`, `cmd`
is `"list"` and `parts.next()` would be `"for-push"`. The match arm
`"list" | "list for-push"` is therefore half-dead — `"list for-push"`
literal can never match because `cmd` never contains a space. The
intended behavior (both forms route to the same handler) is
coincidentally preserved because `"list"` matches both plain `list` and
the first token of `list for-push`.

**Fix:**
Remove the dead alternative or, if you want to actually honor
`list for-push` distinctly (git uses it to signal push-context), match
against `trimmed` not `cmd`:

```rust
match trimmed {
    "list" | "list for-push" => { ... }
    ...
}
```

### L-02: `patch_issue` in `reposix-remote::client` swallows body of non-success responses

**File:** `crates/reposix-remote/src/client.rs:170, 204, 232`
**Category:** Diagnostics
**Issue:**
On 4xx/5xx, the error variant `ClientError::Status(status, body)` captures
the server-returned body string. Good. But the `body` extraction uses
`resp.text().await.unwrap_or_default()`, which silently substitutes an
empty string if text decoding fails. The error message then reads
`"backend status 500: "` — opaque.

**Fix:**
Use `.ok()` + explicit fallback: `body.ok_or_else(|| "<body unreadable>")`
or log the underlying error via `tracing::warn!`. Tiny, diagnostic-only.

### L-03: `PlannedAction::Delete::prior_version` is unused

**File:** `crates/reposix-remote/src/diff.rs:27-45`, `crates/reposix-remote/src/main.rs:294`
**Category:** Dead code
**Issue:**
`Delete` carries `prior_version` but `execute_action` pattern-matches
`Delete { id, .. }` and the sim's DELETE is unconditional (no
`If-Match`). The `#[allow(dead_code)]` at diff.rs:27 acknowledges this
for now. Plumbing is pre-wired for If-Match DELETE support; fine as
future-proofing.

**Fix:**
No action needed for v0.1; either actually use `prior_version` for
`If-Match` on DELETE (the sim currently doesn't enforce it anyway) or
remove the field to trim surface area. Leave for now.

### L-04: `flush` is a no-op; release is the sole push point

**File:** `crates/reposix-fuse/src/fs.rs:528-539`
**Category:** Design clarity
**Issue:**
The comment at line 536-537 explains: "flush can fire multiple times
(e.g. on dup/dup2) and we'd PATCH repeatedly if we flushed here."
Correct reasoning. But: an agent running `sync` after a write expects
data to hit the backend. With only `release` pushing, `sync` on FUSE is
a no-op — the backend still has the old version until the file handle is
closed. `fsync` callback is not implemented (falls to fuser's ENOSYS
default), which is the strictest kernel behavior here.

Not a correctness bug — the v0.1 contract is "release pushes" — but an
agent writing a script that does `sed -i ... && sync` will be confused
when `curl $origin/projects/...` shows the old version. An experience
worth documenting in the README.

**Fix:**
Option A: add a doc sentence to `Mount::open` rustdoc ("PATCH fires on
`close()`/`release`, not on `sync`/`fsync`. Agents should close the file
before expecting the backend to reflect the change.").
Option B: implement `fsync` to push the buffer (semantically equivalent
to release-without-remove, with appropriate concurrency care).

v0.1 demo can ship with option A.

## Cross-cutting observations (not findings)

**SG-01 allowlist:** All outbound HTTP in `reposix-remote` goes through
`HttpClient::request_with_headers_and_body`, which re-checks the
allowlist at send time (http.rs:281). The test
`fetch_issue_origin_rejected` (fetch.rs:422) proves an
`http://evil.example` URL trips `FetchError::Origin`; same path is live
in `reposix-remote` since both use the same sealed `HttpClient`. An
analogous integration test for the remote helper would be nice but not
required — the code path is identical.

**SG-02 bulk-delete cap:** Correctly enforced BEFORE any HTTP call
(`plan()` returns `PlanError::BulkDeleteRefused` before actions execute).
The `six_deletes_refuses_and_calls_no_delete` test proves zero DELETEs
fire via `.expect(0)`. The override tag check is case-sensitive (the
test `six_deletes_with_allow_tag_actually_deletes` uses
`[allow-bulk-delete]` literally). Per SG-02 spec, case-sensitivity is
fine — the tag is a machine contract, not user prose.

**SG-03 tainted/sanitize:** `EgressPayload` in both
`reposix-fuse::fetch` and `reposix-remote::client` mirror the sim's
`PatchIssueBody` (`title`, `status`, `assignee`, `labels`, `body`). The
`sanitize_strips_server_fields_on_egress` test captures a real wiremock
request and asserts `version`, `created_at`, `updated_at`, `id` are all
absent from the body string. Solid.

**SG-05 audit attribution:** Every outbound call attaches
`X-Reposix-Agent: <crate>-<pid>`. Covered for GET, PATCH, POST, DELETE
in both crates.

**SG-07 5s timeout:** `FETCH_TIMEOUT = 5s` and `REQ_TIMEOUT = 5s` are
layered on top of the `HttpClient`'s `total_timeout`. Both have
`..._times_out_within_budget` tests that assert `<5.5s` elapsed. Solid.

**Unsafe code:** `#![forbid(unsafe_code)]` at every crate root. Verified.

**Stdout discipline:** `#![deny(clippy::print_stdout)]` in
`reposix-remote/src/main.rs`. The sole `#[allow(clippy::print_stdout)]`
is on the `Protocol` struct, which owns the protocol writes. `diag()` in
main.rs uses `eprintln!` (stderr). No stdout leak path exists — proven
by the `unknown_command_writes_to_stderr_not_stdout` test.

## Verdict

**FIX-REQUIRED** for H-01, H-02, H-03, H-04 before the helper is trusted
for anything beyond the happy-path demo scenario (ASCII + LF + UTF-8 + empty-delete-tree). For the Phase-4 demo recording specifically, the happy
path is fine and the SG-02 refusal fires empirically. But the review
items above should be tracked as Phase 4 / v0.2 follow-ups.

MEDIUM items (M-01..M-04) are recommended pre-demo polish:
- M-04 (tighten `.expect(5)` assertion) is cheap and buys real confidence.
- M-03 (no-op push optimization) meaningfully improves the demo UX
  ("`git push` on an unchanged tree is a no-op, not a PATCH storm").

LOW items are housekeeping; fix opportunistically.

---

_Reviewed: 2026-04-13T04:10:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Commits: dc09b4a, b12036e, 4006f13_
