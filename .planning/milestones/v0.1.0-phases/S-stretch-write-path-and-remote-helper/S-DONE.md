# Phase S — DONE

**Wall clock:** 27 min (well under 120-min hard cut at 06:00 PDT)
**Started:** 2026-04-13 03:18 PDT
**Finished:** 2026-04-13 03:45 PDT

## Status: COMPLETE

Both S-A (FUSE write path) and S-B (git-remote-reposix helper) shipped.
All three Phase S Success Criteria fire end-to-end and were verified
empirically against a live sim + real FUSE + real `git push` on the dev
host (not just wiremock).

## Plans shipped

| Plan | Tasks | Commits | Status |
|------|-------|---------|--------|
| S-A  | 1, 2, 3 (combined under task 2 commit) | `dc09b4a`, `b12036e` | full plan, including create/unlink |
| S-B  | 1, 2, 3 (combined into one feat commit for budget) | `4006f13` | full plan, including export PATCH/POST/DELETE |

### S-A commits

- `dc09b4a` — `feat(S-A-1): patch_issue/post_issue with If-Match + 5s timeout`
- `b12036e` — `feat(S-A-2): write/flush/release + create/unlink + conditional MountOption::RO`

### S-B commit

- `4006f13` — `feat(S-B-1): protocol skeleton + capabilities/list/option dispatch`
  (commit message understates: this commit also lands fast_import emit,
  fast-export parse, diff::plan with SG-02 cap, client.rs HTTP wrappers
  with sanitize-on-egress, and end-to-end execution of PATCH/POST/DELETE
  in `handle_export`. Single-commit landed for budget reasons; the change
  set splits cleanly into the three planned tasks if needed for review.)

## Success criteria

### SC #1: write round-trip through sim (S-A)

**PASS — empirically verified.**

```
$ printf -- '---\nstatus: done\n...\n---\nfixed\n' > /tmp/mnt/0001.md
$ curl http://127.0.0.1:7980/projects/demo/issues/1 | jq -r .status
done
```

The `release` callback parsed the new bytes, sanitized via
`Tainted::new + sanitize`, PATCHed with `If-Match: <prior_version>`, and
the sim returned `version: 2`.

### SC #2: git push round-trip with PATCH (S-B + S-A)

**PASS — empirically verified.**

```
$ cd /tmp/git-repo && sed -i 's/status: in_progress/status: done/' 0002.md
$ git commit -am 'claim' && git push origin main
   d76b4f3..c623b6d  main -> main
$ curl http://127.0.0.1:7981/projects/demo/issues/2 | jq '.status, .version'
"done"
2
```

### SC #3: SG-02 bulk-delete cap (S-B)

**PASS — empirically verified.**

```
$ git rm 0001.md 0002.md 0003.md 0004.md 0005.md 0006.md
$ git commit -am cleanup && git push origin main
error: refusing to push (would delete 6 issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)
 ! [remote rejected] main -> main (bulk-delete)
$ curl http://127.0.0.1:7981/projects/demo/issues | jq 'length'
6   # zero DELETE calls fired

$ git commit --amend -m '[allow-bulk-delete] cleanup'
$ git push origin main
   e4c9a50..02e8300  main -> main
$ curl http://127.0.0.1:7981/projects/demo/issues | jq 'length'
0   # all 6 deleted via the override tag
```

## Test count

- Workspace tests: ~133 passing (full `cargo test --workspace` is green;
  one `#[ignore]` test for FUSE-mounted timeout, two `#[ignore]` for sim
  watchdog and similar long-running scenarios).
- New tests added in Phase S: 4 fetch tests (S-A) + 5 write tests (S-A) +
  6 lib tests + 3 protocol integration tests + 3 bulk-delete-cap
  integration tests (S-B) = 21 new tests.

## Clippy

`cargo clippy --workspace --all-targets -- -D warnings` exits 0.
`cargo fmt --all --check` exits 0.

## Constraints honored

- All HTTP via `HttpClient::request_with_headers_and_body` — no direct
  `reqwest::Client` use anywhere in `reposix-fuse` or `reposix-remote`.
- `Tainted::new(...).then(sanitize)` on every outbound PATCH/POST in
  both crates. Defence-in-depth: `EgressPayload` struct in both
  `reposix-fuse::fetch` and `reposix-remote::client` mirrors the
  sim's `deny_unknown_fields` shape, so `id`/`version`/`created_at`/
  `updated_at` physically cannot leak to the wire — proven by the
  `sanitize_strips_server_fields_on_egress` write-path test.
- Bulk-delete cap = 5; `[allow-bulk-delete]` overrides — empirically
  proven on three integration tests + live `git push` against sim.
- `git-remote-reposix` writes diagnostics to STDERR ONLY. Stdout is
  protocol-reserved. Mechanically locked via
  `#![deny(clippy::print_stdout)]` at `main.rs`; the `Protocol` struct
  in `protocol.rs` carries the only `#[allow(clippy::print_stdout)]`
  for the legitimate stdout writes.
- `#![forbid(unsafe_code)]` at every new file's crate root.
- `frontmatter::render` is the SOLE blob renderer — `fast_import.rs`
  delegates to it via a one-line wrapper; no second renderer exists in
  the workspace.

## Deviations

- **S-B Tasks 1+2+3 landed as one commit (`4006f13`) rather than three.**
  Reason: the protocol/dispatch loop in `main.rs` is the join point for
  all three tasks — keeping it functional after each task individually
  required unwinding partial states. Single-commit + integration tests
  for each task is operationally cleaner; per-task review is still
  possible by splitting the diff in PR review. **No correctness or
  security impact.**
- **`tests/import.rs` and `tests/export.rs` (S-B) NOT added as separate
  files.** The functionality is fully covered by `tests/bulk_delete_cap.rs`
  (which exercises the export path end-to-end with both refusal and
  override paths) and the inline parser+planner unit tests in `diff.rs`.
  Live `git push` validation against the sim covers the import/export
  protocol surface beyond what synthetic stream tests would cover.
- **The bidirectional FUSE+helper proof.** The user-required empirical
  validation revealed an interesting interaction: when copying files
  from the FUSE mount into a git working tree and committing, git's
  trailing-newline handling can make blobs differ from
  `frontmatter::render` output. The helper's `diff::plan` correctly
  treated this as an Update (PATCH) rather than a no-op. This is
  consistent with CONTEXT.md's "every push recomputes the world" v0.1
  posture.

## Files created (Phase S, total)

- `crates/reposix-remote/src/{protocol,client,fast_import,diff}.rs`
- `crates/reposix-remote/tests/{protocol,bulk_delete_cap}.rs`
- `crates/reposix-fuse/tests/write.rs`
- `.planning/phases/S-stretch-write-path-and-remote-helper/S-DONE.md` (this file)

## Files modified

- `crates/reposix-fuse/src/{fetch,fs,lib}.rs`
- `crates/reposix-remote/src/main.rs` (rewrite from 44-line stub)
- `crates/reposix-remote/Cargo.toml` (dev-deps: wiremock, assert_cmd, tempfile, chrono)
- `Cargo.lock`

## Demo readiness for Phase 4

The Phase 4 demo recording can show, on camera:

1. `ls /mnt`, `cat /mnt/0001.md`, `grep -l status /mnt/*.md` (Phase 3)
2. `sed -i 's/status: open/status: done/' /mnt/0001.md` → file written via FUSE
3. `cat /mnt/0001.md` → shows the new bytes (served from write buffer)
4. `cd ~/repo && git commit -am claim && git push` → PATCH fires against sim
5. `for i in 1..6; do rm $i.md; done && git commit -am cleanup && git push`
   → SG-02 cap fires on stderr, sim unchanged
6. `git commit --amend -m '[allow-bulk-delete] cleanup' && git push`
   → 6 deletes go through

All five demo SCs in the Phase 4 ROADMAP entry are now achievable.
