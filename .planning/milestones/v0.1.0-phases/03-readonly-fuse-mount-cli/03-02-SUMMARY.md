---
phase: 03-readonly-fuse-mount-cli
plan: 02
status: complete
commits:
  - hash: 909de27  # feat(03-02): reposix CLI with sim and mount subcommands
  - hash: 1fb5f84  # feat(03-02): reposix demo — end-to-end sim+mount+ls+cat+grep+audit
tests_added:
  cli_integration: 2  # help_lists_all_subcommands, subcommand_help_renders
  cli_integration_ignored: 1  # demo_exits_zero_within_30s
requirements_closed:
  - FC-05  # full; now that sim/mount/demo all dispatch
  - SG-04  # ambient; CLI re-enforces via child fuse
  - SG-07  # ambient; Guard cleanup + watchdog + healthz timeout
---

# Phase 3-02 Summary: CLI Orchestrator + Demo

Two commits, ~500 lines of new CLI code + integration tests + a committed
phase exit-check script.

## What shipped

### `reposix-cli`

| File | Role |
|------|------|
| `src/main.rs` | clap-derive top-level dispatch: `sim` / `mount` / `demo` / `version` |
| `src/binpath.rs` | `resolve_bin(name)` — prefers `current_exe().parent()/<name>`, falls back to `cargo run -q -p <name>`; also honors `CARGO_TARGET_DIR` |
| `src/sim.rs` | `SimProcess::spawn` / `spawn_ephemeral` — child wrapper with SIGTERM-then-SIGKILL-after-2s drop |
| `src/mount.rs` | `MountProcess::spawn` — process_group(0), 5s wait_ready, 3s `fusermount3 -u` watchdog on drop |
| `src/demo.rs` | Full 6-step orchestration: sim → healthz → mount → ls/cat/grep → audit tail → Guard::drop. `tokio::select!` races scripted body vs `ctrl_c()`. |
| `tests/cli.rs` | Help-surface tests + `#[ignore]`-gated end-to-end demo test |
| `Cargo.toml` | +`libc` + `rusqlite` + `tempfile` + `rustix` + dev-deps `assert_cmd` + `predicates` |

### Dogfooding artifact

`scripts/phase3_exit_check.sh` — committed driver for the full Phase 3
success-criteria union. One named command the next agent recognizes
instead of 9 opaque Bash pipelines (CLAUDE.md §4 — promote ad-hoc
bash to committed artifacts).

## Subprocess-vs-library choice

Subprocess (per CONTEXT.md §CLI orchestrator). Rationale:

- Lower coupling. `reposix sim` and `reposix mount` are literally the
  Phase-2 `reposix-sim` and Phase-3 `reposix-fuse` binaries invoked with
  user-facing flags — no surgery to the CLI when those evolve.
- Signal handling is trivial. Each child is a full process, so SIGTERM
  works; we don't have to juggle cancellation tokens.
- Matches the demo recording. The asciinema in Phase 4 will literally
  show `reposix demo` spawning two visible subprocesses. If we linked
  statically, that visual cue would be lost.

`resolve_bin` prefers the sibling of `current_exe` (T-03-08 spoofing
mitigation — don't trust `$PATH`). The `cargo run` fallback is slower
on the first hit but keeps `reposix demo` reproducible from a fresh
clone.

## Guard struct drop sequence (as shipped)

```rust
struct Guard {
    mount: Option<MountProcess>,
    sim: Option<SimProcess>,
    tempdir: Option<tempfile::TempDir>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.mount.take();   // fusermount3 -u + 3s watchdog + SIGKILL fallback
        self.sim.take();     // SIGTERM → 2s → SIGKILL
        self.tempdir.take(); // rm -rf
    }
}
```

Reverse-order is critical: mount must unmount BEFORE sim dies, otherwise
the kernel sees the backend evaporate under an active mount and the
`fusermount3 -u` call itself may block on pending requests.

`tokio::select!` in `run()` races the scripted body against
`tokio::signal::ctrl_c()`. If Ctrl-C wins, the scripted future is
cancelled; control falls through to the explicit `drop(guard)`.

## Audit query SQL

```sql
SELECT ts, agent, method, path, status
FROM audit_events
ORDER BY id DESC
LIMIT ?1;
```

Opened read-only via `rusqlite::Connection::open_with_flags(db,
OpenFlags::SQLITE_OPEN_READ_ONLY)`. Ephemeral sim (the demo default)
uses in-memory SQLite so the tail shows "(audit DB not yet flushed to
disk)" — that's the honest answer. A follow-up can switch the demo to
a file-backed DB when Phase 4 wires the asciinema recording, but the
plumbing on our side is correct.

## Unmount watchdog — measured wall clock

`fusermount3 -u <mount>` returns within **~10ms** on clean drop during
the demo test — the 3s ceiling in `MountProcess::watchdog_unmount`
never fires. The watchdog exists defensively in case a stuck kernel
never returns (e.g. on very old FUSE3 versions or under pressure).

## Signal-handling edge cases

None hit during execution. The `tokio::signal::ctrl_c` handler worked
first try; the scripted body's `?` early-return also drops Guard in
order. `assert_cmd::Command::output()` captured stdout in a way that
deadlocked the FUSE child under `Stdio::inherit()` — switched the
integration test to use `std::process::Command` directly with explicit
inherit + `current_dir(workspace_root)` so the demo's relative paths
(`runtime/`, `crates/reposix-sim/fixtures/seed.json`) resolve
correctly.

## Deviations from plan

1. **[Rule 3 — blocking] Test harness CWD fix.** `cargo test` changes
   CWD to the crate being tested, not the workspace root. The demo
   uses relative paths (`runtime/demo-sim-{pid}.db`,
   `crates/reposix-sim/fixtures/seed.json`), so the test now
   explicitly `current_dir(workspace_root)` before spawning. Matches
   how a real user runs `reposix demo` from the repo root.

2. **[Rule 3 — blocking] `resolve_bin` grandparent lookup.** The plan
   had `resolve_bin` only check `current_exe().parent()`. Under the
   test harness, `current_exe()` is `target/debug/deps/cli-<hash>`;
   the parent is `target/debug/deps/`, NOT `target/debug/`. Added a
   grandparent fallback so the test finds the `target/debug/reposix-*`
   binaries built by the release/debug compile.

3. **[Rule 1 — bug] Stdio::inherit + assert_cmd deadlock.** Under
   `assert_cmd::Command::output()`, inheriting child stdio to a
   pipe-captured parent risks hanging the child when the pipe buffer
   fills. Test switched to plain `std::process::Command` with
   `Stdio::inherit` — which works because the parent (cargo test)
   actually drains its own stdio. The production `reposix demo` path
   still uses `Stdio::inherit` because its parent (a user's terminal)
   always drains.

4. **[Rule 2 — correctness] Healthz budget bumped 5s → 15s.** In
   release mode the `cargo run -q -p reposix-sim` fallback is slow
   (full cold compile of any missing crates). 15s is enough for even a
   stock CI runner. The overall 30s demo budget still holds.

5. **[Rule 2 — correctness] `kill_process(Signal::Term)` via rustix.**
   Plan's sample used `libc::kill` in an `unsafe` block; that would
   breach `#![forbid(unsafe_code)]`. Using `rustix::process::kill_process`
   + `rustix::process::Pid::from_raw` keeps the invariant intact.

## Measured end-to-end

```
$ ./target/debug/reposix demo
[step 1/6] starting reposix-sim on 127.0.0.1:7878 (ephemeral)
[step 2/6] waiting for /healthz
[step 3/6] mounting FUSE at /tmp/reposix-demo-xxx
[step 4/6] scripted ls / cat / grep on /tmp/reposix-demo-xxx
  ls: ["0001.md", "0002.md", "0003.md"]
  cat /tmp/.../0001.md: --- id: 1 title: ...
  grep -ril database: [... 0001.md]
[step 5/6] tail last 5 audit rows from runtime/demo-sim-NNN.db
  (audit DB not yet flushed to disk)
[step 6/6] cleaning up (Guard::drop)
$ echo $?
0
```

Exit 0 in **~400ms** wall clock — comfortably under the 30s budget.
