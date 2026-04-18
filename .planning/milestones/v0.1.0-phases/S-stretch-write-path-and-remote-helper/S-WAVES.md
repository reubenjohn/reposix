# Phase S — Wave Structure

**Phase:** `S-stretch-write-path-and-remote-helper`
**Plans:** 2
**Waves:** 2 (serial: S-A → S-B)
**Budget:** 120 minutes total, hard cut at **06:00 PDT**.

## Wave map

```
Wave 1 ────────────────────────────────────────────────────────────
  │
  └── S-A-fuse-write-path.md
        Budget: 60 min wall clock
        Creates:  crates/reposix-fuse/tests/write.rs
        Extends:  crates/reposix-fuse/src/fs.rs        (setattr/write/flush/release/create/unlink)
                  crates/reposix-fuse/src/fetch.rs     (patch_issue, post_issue, FetchError variants)
                  crates/reposix-fuse/src/lib.rs       (conditional MountOption::RO)
                  crates/reposix-fuse/Cargo.toml       (minor dev-dep adjustment)
        Closes:   ROADMAP Phase S SC #1 (write round-trip through sim)
        Min-viable: Tasks 1 + 2 only (PATCH on existing inode via release).

Wave 2 ────────────────────────────────────────────────────────────  (depends on Wave 1)
  │
  └── S-B-git-remote-reposix-helper.md
        Budget: 60 min wall clock
        SKIP entirely if S-A overran by >15 min.
        Creates:  crates/reposix-remote/src/{protocol,fast_import,diff,client}.rs
                  crates/reposix-remote/tests/{protocol,bulk_delete_cap,import,export}.rs
        Rewrites: crates/reposix-remote/src/main.rs (capabilities-only stub → full helper)
        Extends:  crates/reposix-remote/Cargo.toml    (dev-deps: wiremock, assert_cmd)
        Closes:   ROADMAP Phase S SC #2 (git push round-trip; Task 3 only),
                  ROADMAP Phase S SC #3 (SG-02 bulk-delete cap fires on `git push`;
                                          Tasks 1+2 sufficient).
        Min-viable: Tasks 1 + 2 (capabilities + import emit + bulk-delete cap on
                    export; export stub does NOT yet execute PATCH/POST/DELETE).
```

## Why serial, not parallel

Two reasons S-B must wait for S-A:

1. **Shared-contract dependency.** S-A introduces `patch_issue` / `post_issue`
   helpers in `crates/reposix-fuse/src/fetch.rs` that establish the egress
   shape (trimmed `EgressPayload` struct honoring simulator's
   `deny_unknown_fields`, `If-Match` header format, 5s timeout, sanitize
   invocation). S-B's `crates/reposix-remote/src/client.rs` mirrors that
   exact shape against the same sim endpoints. Running them in parallel
   risks two divergent egress payloads that both claim to be canonical.
   (We do NOT make `reposix-remote` depend on `reposix-fuse` as a crate —
   that would couple a binary to a daemon. But the *pattern* is copied, and
   the pattern lands first in S-A.)

2. **Demo-narrative dependency.** The Phase 4 demo recording is
   `sed -i 's/status: open/status: done/' 0001.md && git commit -am claim && git push`.
   That requires both the FUSE write path (so `sed -i` on the mount mutates
   something) AND the helper (so `git push` has a receiver). S-A shipping
   first means that even if S-B is SKIPPED, the demo can still show `sed` +
   `cat` working end-to-end through the FUSE write path (just without the
   `git push` flourish). S-B shipping without S-A would be useless — there
   would be no locally-editable mount to push FROM.

There IS no parallel slice worth chasing: S-B Task 1 (protocol skeleton)
*could* technically run in parallel with S-A Task 1, but the file-ownership
matrix (below) makes the serialization cost zero — neither plan touches the
other's files, so the only cost of serialization is wall-clock time, not
waiting-on-dependency time. Given the hard 120-min budget, we prefer to
commit fully to S-A first, decide at T+60 whether the write path is solid
enough to push to, and THEN start S-B.

## File ownership matrix

| File                                                 | S-A                              | S-B                       |
|------------------------------------------------------|----------------------------------|---------------------------|
| `crates/reposix-fuse/**`                             | writes                           | untouched                 |
| `crates/reposix-remote/**`                           | untouched                        | writes                    |
| `crates/reposix-core/**`                             | untouched                        | untouched                 |
| `crates/reposix-sim/**`                              | untouched                        | untouched                 |
| Workspace root (`Cargo.toml`, `clippy.toml`, etc.)   | untouched                        | untouched                 |

**Zero file overlap.** The serialization is narrative/budget-driven, not
technical.

## Time budget & decision gates

| Clock time      | Expected state                                                  | Action if not met                               |
|-----------------|-----------------------------------------------------------------|-------------------------------------------------|
| T+0 (start)     | S-A kicks off.                                                  | —                                               |
| T+30            | S-A Task 1 done (patch_issue/post_issue + tests green).         | Abort Task 3; Tasks 1+2 are the min-viable cut. |
| T+60            | S-A done (minimum = Tasks 1+2).                                 | If >T+75, SKIP S-B entirely; commit S-A.        |
| T+60 → T+75     | Orchestrator decides: start S-B, or stop at S-A for demo.       | —                                               |
| T+90            | S-B Task 2 done (SG-02 cap provably fires).                     | Abort Task 3; the demo has SG-02 proof already. |
| T+120 (06:00)   | HARD CUT — whatever is committed ships.                         | —                                               |

## What the demo can show at each checkpoint

- **Only S-A done, Tasks 1+2:** `sed -i` on a mounted file → `cat` shows the
  new bytes → optional: `cat /tmp/reposix-mnt/0001.md` round-trips. NO
  `git push` demo. SC #1 fires.
- **Only S-A done, all 3 tasks:** Above plus `touch new.md`; `rm 0001.md`
  (local only). Still no `git push`. SC #1 fires.
- **S-A + S-B Tasks 1+2:** Above plus `git init && git commit && git push`
  succeeds capabilities/list, but `export` is a stub so the sim state does
  NOT change on push. `for i in 1..=6; do rm $i.md; done && git commit &&
  git push` fires SG-02 error on stderr. **SC #3 fires.**
- **S-A + S-B all 3 tasks:** Full central demo. `sed && git commit && git
  push` actually PATCHes the sim. SG-02 still fires on 6-delete. The
  `[allow-bulk-delete]` tag lets a real cleanup through. **SC #1 + #2 + #3
  all fire.**

## Parallel-safety check within each wave

**Wave 1 (S-A only):** Internally serial.
- Task 1 adds `patch_issue`/`post_issue` to `fetch.rs` → Task 2 consumes them
  in `fs.rs::release` → Task 3 consumes them again in `create`. Task 2
  depends on Task 1; Task 3 depends on Task 2. No internal parallelism.

**Wave 2 (S-B only):** Internally serial.
- Task 1 defines the `Protocol` struct that Tasks 2 & 3 both use. Task 3's
  export-execution path builds on Task 2's `diff::plan` and `ParsedExport`
  types. No internal parallelism.

## Phase exit criterion

The phase is done when this composite exits 0 (lines gated by completion):

```
cd /home/reuben/workspace/reposix && \
  cargo fmt --all --check && \
  cargo clippy --workspace --all-targets -- -D warnings && \
  cargo test --workspace && \
  # S-A minimum:
  grep -q 'fn write' crates/reposix-fuse/src/fs.rs && \
  grep -q 'fn release' crates/reposix-fuse/src/fs.rs && \
  grep -q 'patch_issue' crates/reposix-fuse/src/fs.rs && \
  grep -q 'if cfg.read_only' crates/reposix-fuse/src/lib.rs && \
  # S-B minimum (only if S-B shipped):
  ( [ ! -f crates/reposix-remote/src/protocol.rs ] || (
    grep -q 'frontmatter::render' crates/reposix-remote/src/fast_import.rs && \
    grep -q '\[allow-bulk-delete\]' crates/reposix-remote/src/diff.rs && \
    test -x ./target/debug/git-remote-reposix
  ) )
```

The `[ ! -f ... ] ||` clause makes the S-B check conditional on S-B having
shipped — a Phase S where only S-A landed is still a VALID phase exit (per
CONTEXT.md: "the Phase 4 demo will use whatever's available").

## Constraints restated (from CONTEXT.md + user brief)

- All HTTP via `HttpClient::request_with_headers_and_body`. Allowlist enforced.
- `Tainted::new(...).then(sanitize)` on every outbound PATCH/POST body.
- Bulk-delete cap: count > 5 refused, `[allow-bulk-delete]` in commit message
  overrides.
- Deterministic blob rendering via `reposix_core::issue::frontmatter::render`
  — no second renderer anywhere in the workspace.
- `git-remote-reposix` writes diagnostics to stderr ONLY. Stdout is
  protocol-reserved. Mechanical lock via `#[deny(clippy::print_stdout)]`.
- `#![forbid(unsafe_code)]` in every new file. Clippy pedantic on. `# Errors`
  doc section on every Result-returning function.
