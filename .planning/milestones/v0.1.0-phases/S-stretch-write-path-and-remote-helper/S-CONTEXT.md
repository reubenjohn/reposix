# Phase S (STRETCH): write path + git-remote-reposix — Context

**Gathered:** 2026-04-13 03:07 PDT
**Status:** Decision gate at 03:30 PDT — COMMITTED at 03:07 (MVD landed early)
**Source:** Auto-generated from PROJECT.md + ROADMAP.md + research/

<domain>
## Phase Boundary

**In scope:**
- **FUSE write path** (`crates/reposix-fuse/src/fs.rs`):
  - `setattr` (truncate-on-`>` redirect, otherwise no-op)
  - `write` (buffer per-fh, flush on `release`)
  - `create` (POST a new issue, allocate inode, return the file handle)
  - `unlink` (DELETE the issue; journal locally per threat model)
  - `flush` / `release` (push buffered writes upstream as PATCH with `If-Match`)
- **`git-remote-reposix` helper** (`crates/reposix-remote/src/main.rs`):
  - `capabilities` reply: `import`, `export`, `refspec refs/heads/*:refs/reposix/*`
  - `option` replies (default `unsupported`)
  - `import` — fast-export stream of all issues as Markdown blobs, deterministically ordered.
  - `export` — read fast-import stream, diff against last-known tree, translate per-file diffs into PATCH/POST/DELETE calls.
  - `list` — list refs (just `main` for v0.1).
  - Server-field stripper applied to outbound PATCH bodies (consumes Phase 1's `sanitize`).
  - **Bulk-delete cap (SG-02)**: any push that deletes >5 files in one go errors out with `error: refusing to push (would delete N issues; see --allow-bulk-delete)` and does NOT send any DELETE requests.
- **End-to-end smoke test** (`tests/e2e_push.rs`, `#[ignore]`-gated):
  - Boot sim, mount FUSE, init git in mount, set remote, edit a file, commit, push, observe via simulator state.

**Out of scope (defer to v0.2):**
- Adversarial swarm harness — narratively less valuable for the demo than `git push`.
- FUSE-in-CI mount integration — operational completeness; v0.1 CI runs cargo tests + the `#[ignore]`-gated test runs locally and during the demo.
- Marks-file format / bidirectional pulls. v0.1 only supports `import` (sim → git) on first push and `export` (git → sim) thereafter; no incremental marks needed for the demo.
- Conflict-aware merging (the helper rejects pushes that would lose remote changes; full merge story = v0.2).

</domain>

<decisions>
## Implementation Decisions

### FUSE write path
- Per-fh write buffer in a `DashMap<u64, Vec<u8>>` keyed by file handle. `write` appends to buffer. `release` flushes.
- `flush` is called on `close(fd)`. This is when we actually push upstream — single PATCH with `If-Match: "<version>"` header.
- On 409 conflict from PATCH: write `EIO` to kernel (the user can `git pull --rebase` later). The 409 → conflict-marker resolution is the helper's job, not the FUSE daemon's.
- On 5s timeout: `EIO`.
- `create(parent, name, mode, ...)` — name must validate via `validate_issue_filename`. POST to backend with empty body. Allocate inode for the new id. Return the inode + a fresh file handle.
- `unlink(parent, name)` — local journal only (per threat model). Actual DELETE happens on `git push` so the bulk-delete cap can fire. For interactive `rm` on the mount: return success but don't push; the user must run `git commit` + `git push` to materialize the delete.

### git-remote-reposix helper
- Subprocess invoked by git. Stdin/stdout = protocol; stderr = errors visible to user.
- Capabilities: `import`, `export`, `refspec refs/heads/*:refs/reposix/*`. `import-marks`/`export-marks` only if marks file exists (per research).
- `import`: GET all issues from sim, render each as a fast-export blob (`blob`, `data`, `<bytes>`), then a single commit with one tree entry per issue (path = `<id>.md`). The tree is sorted by id.
- `export`: read fast-import stream, parse the new tree, diff against the last imported tree (recompute via the same GET; for v0.1 we don't cache marks). For each delta:
  - New blob, no old blob → POST.
  - Old blob, no new blob → DELETE (subject to bulk-delete cap).
  - Both, content differs → PATCH with `If-Match` from the old version. Body diff includes only the changed YAML keys + body.
- All HTTP via `reposix_core::http::client(ClientOpts::default())?` and the `HttpClient::request_with_headers_and_body` method (allowlist-enforced).
- The remote URL parser uses `reposix_core::parse_remote_url`.
- All outbound PATCH/POST bodies pass through `Tainted::new(...).then(sanitize).inner()` to strip server-controlled fields. (Even though the source is the user's own commit, this defends against attacker-authored issue bodies that previously rendered into the local mount.)

### Bulk-delete cap (SG-02)
- Implemented in the `export` path: count the deletes in the new commit's tree-diff. If > 5, write to stderr `error: refusing to push (would delete N issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)` and exit non-zero.
- Override: if the commit message contains the literal `[allow-bulk-delete]`, the cap is bypassed (lets the user opt in for a real cleanup).
- Test: contrive a commit that deletes 6 files; assert `git push` exit code != 0; assert the simulator state is unchanged.

### Demo-recording integration
- Phase 4 will use the write path + helper for the central narrative: `sed -i 's/status: open/status: in_progress/' 0001.md && git commit -am claim && git push`. This shows the central pitch in the recording.
- The recording must also visibly fire SG-02: `for i in 1 2 3 4 5 6; do rm 000$i.md; done && git commit -am cleanup && git push` → push refused, then `git commit --amend -m '[allow-bulk-delete] cleanup' && git push` → succeeds.

### Tests
- All write-path unit tests live in `crates/reposix-fuse/tests/write.rs` (gated `#[ignore]` if they require a live mount).
- `crates/reposix-remote/tests/protocol.rs` — `capabilities` round-trip (driven by spawning the binary and feeding it stdin).
- `crates/reposix-remote/tests/export.rs` — fast-import stream parse + diff translation. Drive against a `wiremock` backend.
- `crates/reposix-remote/tests/bulk_delete_cap.rs` — synthetic push of 6 deletes is rejected; 5 deletes is accepted.
- `tests/e2e_push.rs` (workspace level) — `#[ignore]`-gated, hits real sim + real FUSE + real git push.

</decisions>

<canonical_refs>
## Canonical References

- `.planning/research/git-remote-helper.md` — primary blueprint. §2 (capability choice), §3 (worked example), §5 (deterministic blob rendering — non-negotiable for clean diffs), §6 (async-from-sync bridge).
- `.planning/research/fuse-rust-patterns.md` — §3.7-3.10 (write path), §4 (reply ack patterns).
- `.planning/research/threat-model-and-critique.md` — SG-02 (bulk-delete cap) language.
- `crates/reposix-core/src/{http,taint,path,issue,remote}.rs` — the contracts to consume.
- [git fast-import](https://git-scm.com/docs/git-fast-import)
- [git fast-export](https://git-scm.com/docs/git-fast-export)
- [gitremote-helpers(7)](https://git-scm.com/docs/gitremote-helpers)

</canonical_refs>

<specifics>
## Specific Ideas

- The blob-rendering function reuses `reposix_core::issue::frontmatter::render` — same code path as the FUSE read path. Critical for stable SHAs.
- The helper writes its diagnostic output to stderr ONLY (never stdout, which is reserved for the protocol).
- For v0.1 we accept that `git pull` from the helper recomputes the world (no marks). This makes pulls slow with many issues but is correct.
- The bulk-delete cap is checked locally in the helper before any HTTP call. Defense in depth: even if the simulator forgot to enforce, the client wouldn't.

</specifics>

<deferred>
## Deferred Ideas

- Marks-file based incremental sync.
- Progress reporting from the helper to the user (fast-import progress events).
- Conflict-aware merging that produces real Git merge conflicts on divergent state.
- Adversarial swarm — explicitly defer; not on the demo's critical path.
- FUSE mount inside the GitHub Actions integration job — defer; cargo tests + manual demo are sufficient for v0.1 credibility.

</deferred>

---

*Phase: S-stretch-write-path-and-remote-helper*
*Context: 2026-04-13 03:07 PDT*
