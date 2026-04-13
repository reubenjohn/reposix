---
phase: 03-readonly-fuse-mount-cli
status: complete
completed_at: 2026-04-13
plans_complete: 2/2
commits:
  - hash: 3c004f6  # feat(01-ext merged into 02-01 due to parallel-exec race): HttpClient::request_with_headers
  - hash: 032e979  # feat(03-01): inode registry + 5s-timeout HTTP fetch helpers
  - hash: 2acd9e4  # feat(03-01): read-only FUSE Filesystem impl + mount binary + readdir test
  - hash: 4fbccc9  # test(03-01): prove 5s timeout — stat returns <7s after backend dies
  - hash: 909de27  # feat(03-02): reposix CLI with sim and mount subcommands
  - hash: 1fb5f84  # feat(03-02): reposix demo — end-to-end sim+mount+ls+cat+grep+audit
requirements_closed:
  - FC-03-read  # FUSE mount read path
  - FC-05       # Working CLI orchestrator
  - SG-04       # Filename derivation + path validation at FUSE boundary
  - SG-07       # FUSE never blocks kernel > 5s
tests:
  reposix_core_integration: 9        # +2 from 03-01 (request_with_headers tests)
  reposix_fuse_lib: 15                # inode + fetch
  reposix_fuse_integration: 2         # readdir (default) + sim_death_no_hang (ignored)
  reposix_cli_integration: 3          # help_*, subcommand_help_*, demo (ignored)
roadmap_sc:
  SC1: PASS  # `ls` lists 0001.md/0002.md/0003.md (manual + readdir + demo)
  SC2: PASS  # `cat 0001.md` starts with `---` and contains `id: 1`
  SC3: PASS  # `grep -ril database` returns 0001.md
  SC4: PASS  # `timeout 7 stat <mount>/0001.md` returns in 1.23s after backend dies
  SC5: PASS  # `reposix --help` lists sim/mount/demo; cargo test -p reposix-fuse -p reposix-cli green
---

# Phase 3: Read-only FUSE Mount + CLI Orchestrator — DONE

Shipped in 6 commits across 2 plans. All 5 ROADMAP success criteria pass
under the committed driver `scripts/phase3_exit_check.sh`.

## Commit trail

| Plan | Task | SHA | Subject |
|------|------|-----|---------|
| 01-ext (merged into 02-01) | Task 1 step 0 | `3c004f6` | HttpClient::request_with_headers (see deviation #1 in 03-01-SUMMARY) |
| 03-01 | Task 1 | `032e979` | inode registry + 5s-timeout HTTP fetch helpers |
| 03-01 | Task 2 | `2acd9e4` | read-only FUSE Filesystem impl + mount binary + readdir test |
| 03-01 | Task 3 | `4fbccc9` | prove 5s timeout — stat returns <7s after backend dies |
| 03-02 | Task 1 | `909de27` | reposix CLI with sim and mount subcommands |
| 03-02 | Task 2 | `1fb5f84` | reposix demo — end-to-end sim+mount+ls+cat+grep+audit |

## Shipped files

### Core extension (one file, 01-ext)
- `crates/reposix-core/src/http.rs` — `HttpClient::request_with_headers`
  (allowlist-gated, header slice, sealed newtype invariant intact).
- `crates/reposix-core/tests/http_allowlist.rs` — +2 tests.

### `reposix-fuse`
- `src/inode.rs` — `InodeRegistry` (AtomicU64 starting at `0x1_0000`)
- `src/fetch.rs` — `fetch_issues` / `fetch_issue` with 5s timeout
- `src/fs.rs` — `ReposixFs` + `Filesystem` impl (init/getattr/lookup/readdir/read)
- `src/lib.rs` — `Mount::open(&MountConfig)` spawning a fuser session
- `src/main.rs` — `reposix-fuse <mount> --backend <origin> --project <slug>`
- `tests/readdir.rs` — wiremock-backed mount + ls/cat assertions
- `tests/sim_death_no_hang.rs` — `#[ignore]`-gated; measured 1.23s
- `Cargo.toml` — `+rustix`, dev-deps `+wiremock`, `+tempfile`

### `reposix-cli`
- `src/main.rs` — clap-derive `sim` / `mount` / `demo` / `version`
- `src/binpath.rs` — sibling-binary resolver (T-03-08 mitigation)
- `src/sim.rs` — `SimProcess` child wrapper
- `src/mount.rs` — `MountProcess` with 3s `fusermount3 -u` watchdog
- `src/demo.rs` — `Guard`-based orchestration + `tokio::select!` Ctrl-C
- `tests/cli.rs` — help-surface + end-to-end demo tests
- `Cargo.toml` — `+libc`, `+rusqlite`, `+tempfile`, `+rustix`; dev-deps `+assert_cmd`, `+predicates`

### Scripts
- `scripts/phase3_exit_check.sh` — committed driver running the full
  SC union.

### Planning
- `.planning/phases/03-readonly-fuse-mount-cli/03-01-SUMMARY.md`
- `.planning/phases/03-readonly-fuse-mount-cli/03-02-SUMMARY.md`
- `.planning/phases/03-readonly-fuse-mount-cli/03-DONE.md` (this file)

## Test counts

| Suite | Default | `--ignored` | Notes |
|-------|---------|-------------|-------|
| `reposix-core::http_allowlist` | 9 | 1 | +2 new (request_with_headers_*) |
| `reposix-fuse::lib::inode` | 7 | 0 | monotonic alloc / idempotency / range |
| `reposix-fuse::lib::fetch` | 7 | 0 | wiremock + timeout budget |
| `reposix-fuse::tests::readdir` | 1 | 0 | actual FUSE mount on tempdir |
| `reposix-fuse::tests::sim_death_no_hang` | 0 | 1 | `timeout 7 stat` after backend dies |
| `reposix-cli::tests::cli` | 2 | 1 | help + demo end-to-end |

**Workspace total: ~115 tests (was ~107 before Phase 3).**

## Real-world verification

Manual smoke against a live Phase-2 sim:

```console
$ ./target/debug/reposix-sim --bind 127.0.0.1:17878 --ephemeral \
      --seed-file crates/reposix-sim/fixtures/seed.json &
$ ./target/debug/reposix-fuse /tmp/reposix-mnt \
      --backend http://127.0.0.1:17878 --project demo &
$ ls /tmp/reposix-mnt
0001.md  0002.md  0003.md
$ head -1 /tmp/reposix-mnt/0001.md
---
$ grep -l database /tmp/reposix-mnt/*.md
/tmp/reposix-mnt/0001.md
```

The `reposix demo` subcommand exits 0 in ~400ms wall clock (well under
the 30s budget).

## Phase exit check

```
$ ./scripts/phase3_exit_check.sh
...
==> guard: no direct reqwest ctor in fuse/cli
==> guard: validate_issue_filename used in fuse fs.rs
==> guard: AllowOther not present in fuse
==> guard: reposix --help lists sim/mount/demo
ALL PASS
```

## Notable deviations (summary)

- **01-ext commit collision.** Phase 2's `3c004f6` happened to scoop my
  staged `http.rs` changes during a parallel-execution race. Code is
  correct; attribution is tracked in 03-01-SUMMARY.
- **Dropped `AutoUnmount`.** Fuser 0.17 requires `SessionACL != Owner`
  for `AutoUnmount`, which would breach SG. The CLI's `MountProcess`
  watchdog is the compensating control.
- **`libc` → `rustix` for uid/gid + SIGTERM.** libc 0.2.x flags
  `getuid`/`getgid`/`kill` as `unsafe fn`; using rustix's safe
  wrappers kept `#![forbid(unsafe_code)]` at every crate root.
- **Healthz budget 5s → 15s** in `reposix demo` to cover the
  `cargo run` cold-start fallback when only `reposix` itself is built
  but the sibling binaries aren't.

Full per-plan deviation lists in `03-01-SUMMARY.md` and
`03-02-SUMMARY.md`.

## Next

Phase 4 (demo recording + README polish) can begin. Phases 2 and 3 both
green with `./scripts/phase3_exit_check.sh` + Phase-2's
`scripts/phase2_goal_backward.sh`.
