# Phase 3 — Wave Structure

**Phase:** `03-readonly-fuse-mount-cli`
**Plans:** 2
**Waves:** 2 (serial: 03-01 → 03-02)

## Wave map

```
Wave 1 ────────────────────────────────────────────────────────────
  │
  └── 03-01-fuse-readonly-impl.md
        creates: crates/reposix-fuse/src/{inode,fetch,fs}.rs,
                 crates/reposix-fuse/src/{lib,main}.rs (rewrite),
                 crates/reposix-fuse/tests/{readdir,sim_death_no_hang}.rs
        extends: crates/reposix-core/src/http.rs — adds
                 `HttpClient::request_with_headers(method, url, &[(&str,&str)])`
                 with the same per-call allowlist recheck as `request`;
                 existing wrappers (`request`, `get`, `post`, `patch`,
                 `delete`) delegate to it with an empty header slice. The
                 inner `reqwest::Client` stays sealed (SG-01 invariant
                 survives — compile-fail fixture still locks it).
                 Tests appended to
                 crates/reposix-core/tests/http_allowlist.rs:
                 `request_with_headers_rechecks_allowlist` and
                 `request_with_headers_attaches_header`.
        closes:  ROADMAP Phase 3 SC #1, #2, #4 (via tests/readdir.rs +
                 tests/sim_death_no_hang.rs); partial SC #3 (grep is
                 exercised end-to-end in 03-02); partial SC #5
                 (`cargo test -p reposix-fuse` green)

Wave 2 ────────────────────────────────────────────────────────────  (depends on Wave 1)
  │
  └── 03-02-cli-orchestrator-and-demo-subcmd.md
        creates: crates/reposix-cli/src/{sim,mount,demo,binpath}.rs,
                 crates/reposix-cli/src/main.rs (rewrite),
                 crates/reposix-cli/tests/cli.rs
        extends: workspace-root Cargo.toml — adds the `signal` feature to
                 the shared `tokio` dep so `tokio::signal::ctrl_c()` works
                 without pulling in a separate `ctrlc` crate.
        consumes: the `reposix-fuse` binary produced by 03-01 (spawned as a
                  child process — we do NOT link the lib; CONTEXT.md §CLI
                  orchestrator fixed this choice). Consumes the sealed
                  `HttpClient` newtype for `/healthz` probing — no custom
                  headers needed, so plain `HttpClient::get` suffices.
        closes:  ROADMAP Phase 3 SC #3 (via demo's grep step) and SC #5
                 (help lists sim|mount|demo, full test matrix green).
```

## Why serial, not parallel

Two reasons 03-02 must wait for 03-01:

1. **Contract dependency.** `reposix demo` spawns `reposix-fuse` as a child
   process and depends on that binary's exact CLI surface
   (`reposix-fuse <mount> --backend <origin> --project <slug>`). Plan 03-01
   defines that CLI in its Task 2 (`src/main.rs`). If 03-02 ran in parallel
   against an imagined surface, the first mismatch on merge would either
   break the demo test or force a re-roll.

2. **Test dependency.** Plan 03-02's `demo_exits_zero_within_30s` integration
   test relies on the full 03-01 stack actually working against a live
   Phase-2 simulator. Until 03-01 is green, there is nothing to orchestrate.

There IS a theoretically-parallel slice — 03-02 Task 1 (sim + mount
subcommand plumbing + `--help` test) only needs the binary *names* and could
run in parallel with 03-01. But the contract is thin and the cost of a
serial wave is small (two waves of 2–3 tasks each fits inside a single
execution session), so we pay the extra half-hour for the guarantee. If the
7-hour budget pressure forces parallelism, the fallback is: run 03-02 Task 1
in parallel with 03-01 Task 1, then serialize 03-02 Task 2 after 03-01 Task
2 + 3.

## Parallel-safety check within each wave

**Wave 1 (03-01 only):** Tasks within 03-01 are strictly serial (Task 1
extends `reposix-core::http` AND creates inode + fetch; Task 2 depends on
them to build the Filesystem impl; Task 3 writes an integration test that
exercises Task 2's timeout path). `<files>` lists show no overlap between
tasks other than `src/lib.rs` (which Task 1 adds module decls to, Task 2
extends with `pub use fs::ReposixFs`). Standard read-before-edit protocol
handles it.

**Wave 2 (03-02 only):** Task 1 creates the skeletal subcommand plumbing
plus the sim+mount subprocess wrappers and a `--help` test. Task 2 extends
`main.rs` with the demo wiring and adds `src/demo.rs` + a new integration
test case. The only shared file is `src/main.rs`; Task 2 re-reads before
editing.

## File ownership matrix

| File | 03-01 | 03-02 |
|------|-------|-------|
| `crates/reposix-fuse/**` | writes | reads (binary only; never edits) |
| `crates/reposix-cli/**` | untouched | writes |
| `crates/reposix-core/src/http.rs` | additive: new `request_with_headers` method | reads (consumes `HttpClient::get`) |
| `crates/reposix-core/tests/http_allowlist.rs` | appends 2 tests | untouched |
| `Cargo.toml` (workspace root) | untouched | adds `"signal"` feature to shared `tokio` dep |
| `crates/reposix-core/**` (rest) | untouched | untouched |

No shared-file conflicts between plans → safe to batch-review.

## Decision: filename format — now aligned with ROADMAP

ROADMAP Phase 3 success criteria #1 has been updated separately to use the
numeric-ID scheme (`0001.md`/`/issues/1`) that Phase 1's validator and the
Phase-2 seed already enforce.

**Binding decision for Phase 3 (and Phase 4's demo script):** v0.1 file
names at the FUSE mount are **`0001.md`, `0002.md`, `0003.md`**. The plans
embed this expectation in every `<verify>` block:

- 03-01 Task 2's `tests/readdir.rs` asserts
  `read_dir(mount).sorted_names() == ["0001.md","0002.md","0003.md"]`.
- 03-02 Task 2's demo's `list_sorted` step prints the same.

Phase 4's demo recording will show `0001.md`-style names on screen, which
matches what agents pretraining on GitHub/Jira exports would expect
(zero-padded numeric IDs as filenames is a common convention).

## H-01 follow-through: sealed `HttpClient` newtype assumptions

Phase 1's H-01 fix replaced the free-function `reposix_core::http::request(&client,
method, url)` with a method on a sealed newtype: `HttpClient::request(method,
url)` (plus `get`/`post`/`patch`/`delete` wrappers). The inner
`reqwest::Client` is no longer reachable from outside `reposix-core`. Two
consequences for Phase 3:

1. **Ownership plumbing.** `ReposixFs`, fetch helpers, and the CLI now hold
   `Arc<HttpClient>` / `&HttpClient`, never `Arc<reqwest::Client>`. Every
   `Arc<reqwest::Client>` in the old plan wording has been rewritten.
2. **Header attachment.** The FUSE daemon MUST send
   `X-Reposix-Agent: reposix-fuse-{pid}` on every request so the Phase-2
   audit log attributes. The old plan relied on reaching the inner
   `reqwest::Client`; post-H-01 there is no such escape hatch. Plan 03-01
   Task 1 step 0 adds `HttpClient::request_with_headers(method, url,
   &[(&str,&str)])` to `reposix-core::http` — same allowlist recheck,
   plus header iteration. Existing wrappers stay as-is and delegate to
   `request_with_headers(..., &[])`. This is additive and does not break
   Phase 1's compile-fail fixture.

## Phase exit criterion

The phase is done when this composite command exits 0:

```
cd /home/reuben/workspace/reposix && \
  cargo fmt --all --check && \
  cargo clippy -p reposix-core -p reposix-fuse -p reposix-cli --all-targets -- -D warnings && \
  cargo test -p reposix-core -p reposix-fuse -p reposix-cli && \
  cargo test -p reposix-fuse -p reposix-cli --release -- --ignored --test-threads=1 && \
  [ "$(grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' \
        crates/reposix-fuse/ crates/reposix-cli/ --include='*.rs' | \
        grep -v 'crates/reposix-core/src/http.rs' | wc -l)" = "0" ] && \
  grep -q 'validate_issue_filename' crates/reposix-fuse/src/fs.rs && \
  [ "$(grep -RIn 'AllowOther' crates/reposix-fuse/ --include='*.rs' | wc -l)" = "0" ] && \
  ./target/debug/reposix --help | grep -q '\bsim\b' && \
  ./target/debug/reposix --help | grep -q '\bmount\b' && \
  ./target/debug/reposix --help | grep -q '\bdemo\b'
```

That command is the union of ROADMAP Phase 3 success criteria #1–#5 plus
the ambient CLAUDE.md constraints (no unsafe, no direct reqwest ctor,
`AllowOther` OFF, filename validator on the FUSE boundary).
</content>
</invoke>