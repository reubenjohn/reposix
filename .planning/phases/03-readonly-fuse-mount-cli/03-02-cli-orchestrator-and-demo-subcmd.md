---
phase: 03-readonly-fuse-mount-cli
plan: 02
type: execute
wave: 2
depends_on:
  - "03-01"
files_modified:
  - crates/reposix-cli/Cargo.toml
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/src/sim.rs
  - crates/reposix-cli/src/mount.rs
  - crates/reposix-cli/src/demo.rs
  - crates/reposix-cli/tests/cli.rs
autonomous: true
requirements:
  - FC-05
  - SG-04
  - SG-07

must_haves:
  truths:
    - "`cargo run -p reposix-cli -- --help` lists all three subcommands: sim, mount, demo."
    - "`reposix sim` runs the Phase-2 simulator as a child process on the configured bind addr."
    - "`reposix mount <dir> --backend <origin> --project <slug>` delegates to the Phase 3-01 `reposix-fuse` binary in the foreground."
    - "`reposix demo` spawns sim → waits for `/healthz` 200 → mounts → runs ls, cat, grep -ril database → tails audit rows → cleans up, all inside 30s, exit 0."
    - "All HTTP in the CLI goes through `reposix_core::http::client()` (for `/healthz` probing); no direct `reqwest::Client::new` anywhere in reposix-cli."
  artifacts:
    - path: "crates/reposix-cli/src/main.rs"
      provides: "clap-derive top-level dispatcher: sim | mount | demo"
    - path: "crates/reposix-cli/src/sim.rs"
      provides: "child-process wrapper around `reposix-sim` binary"
    - path: "crates/reposix-cli/src/mount.rs"
      provides: "child-process wrapper around `reposix-fuse` binary"
    - path: "crates/reposix-cli/src/demo.rs"
      provides: "End-to-end orchestration: spawn sim, healthz-wait, mount, ls/cat/grep, audit tail, cleanup"
    - path: "crates/reposix-cli/tests/cli.rs"
      provides: "Integration test: `--help` contains all subcommands; `reposix demo` exits 0 in <30s"
  key_links:
    - from: "crates/reposix-cli/src/demo.rs"
      to: "crates/reposix-cli/src/sim.rs :: SimProcess"
      via: "Drop impl kills the sim child"
      pattern: "impl Drop"
    - from: "crates/reposix-cli/src/demo.rs"
      to: "crates/reposix-cli/src/mount.rs :: MountProcess"
      via: "Drop impl kills the fuse child, then `fusermount3 -u <mount>`"
      pattern: "fusermount3 -u"
    - from: "crates/reposix-cli/src/demo.rs :: wait_for_healthz"
      to: "reposix_core::http::client + request"
      via: "5s-timeout GET loop on /healthz"
      pattern: "reposix_core::http::(client|request)"
---

<objective>
Ship the user-facing `reposix` binary: the orchestrator that ties sim +
FUSE together into a single `reposix demo` command. Closes ROADMAP Phase 3
SC #5 (`cargo run -p reposix-cli -- --help` lists sim|mount|demo; tests
green) and enables the Phase 4 asciinema recording.

Purpose: the project's elevator pitch is "one binary, one command" —
this plan delivers it.

Output: `crates/reposix-cli` serves three subcommands, and `reposix demo`
is the single command the Phase-4 demo recording will invoke.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/03-readonly-fuse-mount-cli/03-CONTEXT.md
@.planning/phases/03-readonly-fuse-mount-cli/03-01-fuse-readonly-impl.md
@.planning/phases/02-simulator-audit-log/02-CONTEXT.md
@CLAUDE.md
@crates/reposix-core/src/http.rs
@crates/reposix-cli/src/main.rs
@crates/reposix-cli/Cargo.toml

<interfaces>
<!-- Binaries we delegate to (produced by 03-01 and Phase 2). -->

`reposix-sim` binary (from Phase 2):
```
reposix-sim --bind <addr> --db <path> [--seed-file <path>] [--rate-limit <rps>]
            [--no-seed] [--ephemeral]
```
Exposes `GET http://<bind>/healthz` (returns 200 once listening) and the
project issue endpoints under `/projects/demo/...`. Writes audit rows to the
DB at `<path>`.

`reposix-fuse` binary (from 03-01):
```
reposix-fuse <mount_point> --backend <origin> --project <slug>
```
Foreground FUSE mount; SIGTERM / SIGKILL + `AutoUnmount` cleans up.

`reposix_core::http`:
```rust
pub fn client(opts: ClientOpts) -> Result<reqwest::Client>;        // ONLY legal ctor
pub async fn request(client: &reqwest::Client, method: Method, url: &str)
    -> Result<reqwest::Response>;                                  // allowlisted send
```

Audit DB schema (from Phase 1, `reposix_core::audit::SCHEMA_SQL`): table
`audit_events` with columns including `ts`, `agent`, `method`, `path`,
`status`. Read-only query from the CLI via `rusqlite` is acceptable
(rusqlite is already a workspace dep).
</interfaces>

<decisions>
- **Subprocess over lib-link (v0.1).** `reposix sim` and `reposix mount`
  `cargo run -p reposix-sim`/`cargo run -p reposix-fuse` via
  `std::process::Command`. Rationale (CONTEXT.md §CLI orchestrator): lower
  coupling, matches how the demo recording will look on camera, and makes
  signal handling trivial. Pre-built binaries are resolved by looking in
  `$CARGO_TARGET_DIR/{debug,release}/` (defaulting to `target/`) for
  `reposix-sim` / `reposix-fuse`; if absent, fall back to `cargo run -q -p
  <crate>`. The fallback is slower the first time but acceptable.
- **Healthz wait loop.** After spawning sim, `reposix demo` opens a
  `reqwest::Client` via `reposix_core::http::client(ClientOpts::default())`
  and loops `GET http://127.0.0.1:7878/healthz` every 100ms for up to 5s.
  On first 200 → proceed. On timeout → bail with a clear error ("sim did
  not become ready"). No `sleep` heuristics.
- **Mount readiness wait.** After spawning the fuse child, poll
  `std::fs::read_dir(mount_point)` every 100ms for up to 3s; consider
  ready when it returns Ok with ≥ 1 entry. Absent a shell `mountpoint` we
  use stdlib.
- **Demo steps (matches CONTEXT.md §CLI orchestrator):**
  1. Start sim (ephemeral DB at `runtime/demo-sim.db`, clean slate).
  2. Wait for `/healthz`.
  3. Mount at `$TMPDIR/reposix-demo-{pid}` (auto-created, removed on exit).
  4. Run scripted interactions, capturing stdout into the CLI's trace log:
     - `std::fs::read_dir(mount)` → print sorted names.
     - `std::fs::read_to_string(mount.join("0001.md"))` → print head -3.
     - Walk mount dir and grep each file's body for "database" case-insensitive
       → print matching filenames.
  5. Open the sim DB via `rusqlite::Connection::open_with_flags(...,
     OpenFlags::SQLITE_OPEN_READ_ONLY)` and `SELECT ts, agent, method, path,
     status FROM audit_events ORDER BY id DESC LIMIT 5` → print table.
  6. Drop in reverse order. `MountProcess::drop` SIGTERMs the fuse child,
     waits 2s, then runs `fusermount3 -u <mount>` as belt-and-braces.
     `SimProcess::drop` SIGTERMs the sim child. `tempfile` handles dir
     cleanup.
- **No daemonization.** `reposix mount` is foreground + blocking (so Ctrl-C
  unmounts). `reposix demo` owns its own process tree; the fuse mount inside
  it is a grandchild that we explicitly kill.
- **Colors / banners.** Use `tracing::info!` with a consistent
  `"[step N/6] ..."` prefix. No custom color crate.
- **`--read-only` alias on `mount`**: accept but ignore (forward-compat,
  CONTEXT.md §specifics).
</decisions>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| user shell → reposix CLI | User-supplied mount path and backend origin. |
| reposix CLI → child process | We spawn `reposix-sim` and `reposix-fuse` with attacker-free args (we build them). |
| reposix CLI → /healthz | Outbound HTTP to the sim. Must honor allowlist. |

## STRIDE Threat Register

| ID | Category | Component | Disposition | Mitigation |
|----|----------|-----------|-------------|------------|
| T-03-08 | Spoofing | `reposix demo` could invoke a malicious binary on `$PATH` | mitigate | Resolve `reposix-sim`/`reposix-fuse` via `std::env::current_exe()`'s parent directory (our target dir) before PATH lookup; fall back only if the target-dir variant does not exist. Never `sh -c`. |
| T-03-09 | Tampering | user-supplied `--backend` URL could target non-loopback | mitigate | `reposix_core::http::request` rejects non-allowlisted origins; the demo subcommand hard-codes `http://127.0.0.1:7878`. `reposix mount` forwards user input to the child; the child's fetch layer re-enforces the allowlist. |
| T-03-10 | Denial of Service | sim never becomes ready | mitigate | 5s healthz timeout → clean bail with non-zero exit. |
| T-03-11 | Information Disclosure | audit tail could leak sensitive rows | accept | v0.1 sim is local, seeded with demo data only; the audit log is precisely what we want to display on camera. |
| T-03-12 | Elevation of Privilege | zombie fuse child survives demo exit, leaving a mounted tempdir | mitigate | `MountProcess::drop` sends SIGTERM + waits + runs `fusermount3 -u`. Additionally set the fuse child's process group so a `kill -- -pgid` can nuke the whole tree if drop misses. |
</threat_model>

<tasks>

<task type="auto">
  <name>Task 1: Subcommand plumbing (sim + mount) with clap-derive</name>
  <files>
    crates/reposix-cli/Cargo.toml,
    crates/reposix-cli/src/main.rs,
    crates/reposix-cli/src/sim.rs,
    crates/reposix-cli/src/mount.rs,
    crates/reposix-cli/tests/cli.rs
  </files>
  <action>
    1. `Cargo.toml`: add `[dependencies]` if missing: `libc` (workspace — for
       process-group plumbing in task 3), `rusqlite = { workspace = true,
       features = ["bundled"] }` (for audit tail in task 2). Add
       `[dev-dependencies]`: `assert_cmd = "2"`, `predicates = "3"`.
    2. `src/sim.rs` — `pub struct SimProcess(std::process::Child)`:
       - `SimProcess::spawn(bind: &str, db: &Path, seed: Option<&Path>) ->
         anyhow::Result<Self>`. Resolves `reposix-sim` binary path via
         `resolve_sibling_bin("reposix-sim")?` helper (see below).
       - `Command::new(path).arg("--bind").arg(bind).arg("--db").arg(db)`
         (+ seed if Some), `.stdout(Stdio::piped()).stderr(Stdio::piped())`,
         `.spawn()`.
       - Helper `resolve_sibling_bin(name)` inside a small `src/binpath.rs`
         module: `std::env::current_exe()?.parent()?.join(name)` — if that
         file exists, use it; else construct a fallback command that runs
         `cargo run -q -p <crate> --` (inferred from name). Document that
         `reposix demo` is expected to be run from a built target dir.
       - `impl Drop for SimProcess`: SIGTERM the child, wait up to 2s, then
         SIGKILL; do NOT panic on errors (we are in drop).
    3. `src/mount.rs` — `pub struct MountProcess { child: Child, mount: PathBuf }`:
       - `MountProcess::spawn(mount_point: &Path, backend: &str, project: &str)`
         returns `anyhow::Result<Self>`. Uses
         `resolve_sibling_bin("reposix-fuse")`. On Linux only, call
         `pre_exec(|| { libc::setsid(); Ok(()) })` via
         `std::os::unix::process::CommandExt::pre_exec` — but this IS unsafe.
         Instead use `libc::setsid` inside the child from a thin wrapper we
         compile with `#[forbid(unsafe_code)]` relaxed ONLY in a single
         `#[allow(unsafe_code)]` line… **actually**: keep
         `#![forbid(unsafe_code)]` crate-wide; instead use
         `Command::process_group(0)` (stable since Rust 1.64 on Unix via
         `CommandExt::process_group` — no unsafe required). This puts the
         child in its own process group so we can target the whole tree with
         a negative-pgid kill if needed.
       - After spawn, poll `std::fs::read_dir(mount_point)` every 100ms up
         to 3s; return `Err` if never ready.
       - `impl Drop for MountProcess`: SIGTERM child → wait 2s →
         `std::process::Command::new("fusermount3").arg("-u").arg(&self.mount)
         .status()?` (ignore failure). This covers the case where the child
         already died but the mount remains.
    4. `src/main.rs` — rewrite the current placeholder:
       - Top-level `Cli { cmd: Cmd }` with `Cmd::{Sim{...}, Mount{...},
         Demo{...}, Version}`. Sim args: `--bind`, `--db`,
         `--seed-file`, `--rate-limit`, `--no-seed`, `--ephemeral` (match
         Phase 2 CLI surface). Mount args: `<mount_point>`, `--backend
         <url>`, `--project <slug>` (default `"demo"`), `--read-only` flag
         (accepted, unused). Demo args: `--keep-running` flag (if set, stay
         up after scripted steps until Ctrl-C — for manual asciinema driving).
       - `Cmd::Sim` calls `SimProcess::spawn(...)` and then
         `child.wait()` inline (foreground). `Cmd::Mount` similarly. `Cmd::Demo`
         stub: `bail!("demo lands in task 2")` until task 2. `Cmd::Version`
         prints `reposix {}`.
    5. `tests/cli.rs`:
       ```rust
       #[test]
       fn help_lists_all_subcommands() {
           use assert_cmd::Command;
           let out = Command::cargo_bin("reposix").unwrap().arg("--help")
                       .output().unwrap();
           let s = String::from_utf8_lossy(&out.stdout);
           for sub in ["sim","mount","demo","version"] {
               assert!(s.contains(sub), "help missing {sub}: {s}");
           }
       }
       ```
    6. `cargo fmt`, `cargo clippy -p reposix-cli --all-targets -- -D warnings`,
       `cargo test -p reposix-cli`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-cli --test cli help_lists_all_subcommands</automated>
  </verify>
  <done>
    `reposix --help`, `reposix sim --help`, `reposix mount --help`,
    `reposix demo --help` all render. Sim and Mount subcommands spawn the
    respective child binaries and propagate exit codes. No unsafe; no direct
    reqwest ctor. Commit subject:
    `feat(03-02): reposix CLI with sim and mount subcommands`.
  </done>
</task>

<task type="auto">
  <name>Task 2: `reposix demo` orchestration + integration test</name>
  <files>
    crates/reposix-cli/src/demo.rs,
    crates/reposix-cli/src/main.rs,
    crates/reposix-cli/tests/cli.rs
  </files>
  <action>
    1. Create `src/demo.rs`. Public entry: `pub async fn run(keep_running: bool)
       -> anyhow::Result<()>`. Structure — EACH STEP prints
       `tracing::info!("[step {n}/6] {label}")` before doing it:

       ```rust
       pub async fn run(keep_running: bool) -> Result<()> {
           // 1/6: Spawn sim on ephemeral DB in runtime/demo-sim.db
           let db = PathBuf::from("runtime").join(format!("demo-sim-{}.db", std::process::id()));
           std::fs::create_dir_all("runtime").ok();
           let seed = PathBuf::from("crates/reposix-sim/fixtures/seed.json");
           let bind = "127.0.0.1:7878";
           let _sim = SimProcess::spawn(bind, &db, Some(&seed))?;

           // 2/6: Wait for /healthz
           wait_for_healthz(&format!("http://{bind}/healthz"),
                            Duration::from_secs(5)).await?;

           // 3/6: Mount on a tempdir
           let mount = tempfile::Builder::new().prefix("reposix-demo-")
                       .tempdir()?;
           let _mount = MountProcess::spawn(mount.path(),
                           &format!("http://{bind}"), "demo")?;

           // 4/6: Scripted ls / cat / grep
           let listing = list_sorted(mount.path())?;
           info!("ls: {listing:?}");
           let first = mount.path().join(listing.first().unwrap());
           let body = std::fs::read_to_string(&first)?;
           info!("cat {first:?} (first 3 lines):\n{}",
                 body.lines().take(3).collect::<Vec<_>>().join("\n"));
           let hits = grep_ril(mount.path(), "database")?;
           info!("grep -ril database: {hits:?}");

           // 5/6: Audit tail
           print_audit_tail(&db, 5)?;

           // 6/6: Cleanup (implicit via Drop on _mount then _sim)
           if keep_running { wait_for_ctrl_c().await?; }
           Ok(())
       }
       ```

       Helpers (all in `demo.rs`):
       - `async fn wait_for_healthz(url, timeout) -> Result<()>` — uses
         `reposix_core::http::client(ClientOpts::default())?` (the ONLY legal
         ctor) and `reposix_core::http::request(&c, Method::GET, url).await`
         in a poll loop with `tokio::time::sleep(Duration::from_millis(100))`.
         Wrap the whole thing in `tokio::time::timeout(timeout, ...)` →
         `anyhow::bail!("sim did not become ready within {timeout:?}")` on
         elapsed.
       - `fn list_sorted(dir) -> Result<Vec<String>>` — `read_dir`, collect
         filenames, sort.
       - `fn grep_ril(dir, needle) -> Result<Vec<PathBuf>>` — walk one level,
         read each file, case-insensitive contains.
       - `fn print_audit_tail(db: &Path, n: u32) -> Result<()>` —
         `rusqlite::Connection::open_with_flags(db,
         OpenFlags::SQLITE_OPEN_READ_ONLY)` then
         `SELECT ts, agent, method, path, status FROM audit_events ORDER BY
         id DESC LIMIT ?1`. Print each row as `info!("audit: {ts} {agent} {method}
         {path} → {status}")`.
       - `async fn wait_for_ctrl_c() -> Result<()>` — `tokio::signal::ctrl_c`.

    2. `src/main.rs`: wire `Cmd::Demo { keep_running }` →
       `demo::run(keep_running).await?`. `main` must already be
       `#[tokio::main]` (it is in the current skeleton).

    3. Extend `tests/cli.rs` with:
       ```rust
       #[test]
       #[ignore] // default-off; runs under --ignored in CI's integration job
       fn demo_exits_zero_within_30s() {
           let t0 = std::time::Instant::now();
           let out = assert_cmd::Command::cargo_bin("reposix").unwrap()
                       .arg("demo")
                       .timeout(std::time::Duration::from_secs(30))
                       .output().unwrap();
           assert!(out.status.success(),
                   "stdout={} stderr={}",
                   String::from_utf8_lossy(&out.stdout),
                   String::from_utf8_lossy(&out.stderr));
           assert!(t0.elapsed() < std::time::Duration::from_secs(30));
       }
       ```

    4. `cargo fmt`, `cargo clippy -p reposix-cli --all-targets -- -D warnings`,
       `cargo test -p reposix-cli`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-cli --release --test cli demo_exits_zero_within_30s -- --ignored --test-threads=1</automated>
    <manual>
      After `cargo build --release -p reposix-sim -p reposix-fuse -p reposix-cli`,
      run `./target/release/reposix demo` in a terminal and confirm banner
      output for steps 1–6, a non-empty `ls:` listing, and 5 audit rows.
    </manual>
  </verify>
  <done>
    `reposix demo` end-to-end: spawns sim, health-gates, mounts, runs
    ls/cat/grep on the live mount, tails 5 audit rows, unmounts, exits 0 in
    <30s. Integration test asserts this in CI's `--ignored` gate.
    SC #3 (`grep -r database` returns ≥ 1 hit) satisfied by the demo's
    grep step succeeding. Commit subject:
    `feat(03-02): reposix demo — end-to-end sim+mount+ls+cat+grep+audit`.
  </done>
</task>

</tasks>

<verification>
Phase-local exit check:

```
cd /home/reuben/workspace/reposix && \
  cargo fmt --all --check && \
  cargo clippy -p reposix-cli --all-targets -- -D warnings && \
  cargo test -p reposix-cli && \
  [ "$(grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' crates/reposix-cli/ --include='*.rs' | wc -l)" = "0" ] && \
  ./target/debug/reposix --help | grep -q '\bsim\b' && \
  ./target/debug/reposix --help | grep -q '\bmount\b' && \
  ./target/debug/reposix --help | grep -q '\bdemo\b'
```

Full SC #5 (`cargo test -p reposix-fuse -p reposix-cli` green, plus ignored
integration tests) is covered by the union of this verification and
03-01's.
</verification>

<success_criteria>
- SC #5: `reposix --help` lists `sim`, `mount`, `demo`; `cargo test -p
  reposix-fuse -p reposix-cli` green; `cargo test --release -- --ignored`
  exercises sim-death-no-hang (from 03-01) and demo end-to-end (from this
  plan).
- SC #1..#3 exercised via the demo: `ls` (step 4a), `cat 0001.md` shows
  `---` + `id: 1` (step 4b), `grep -ril database` returns ≥ 1 hit (step 4c).
  The DEMO script is the executable proof of the ROADMAP bash assertions.
- Audit log shows the FUSE's HTTP traffic attributed to
  `reposix-fuse-{pid}` (verified by `print_audit_tail`).
</success_criteria>

<output>
After completion, create `.planning/phases/03-readonly-fuse-mount-cli/03-02-SUMMARY.md`
capturing: subprocess-vs-library choice, actual child-process cleanup
strategy used, audit query SQL, and any signal-handling edge cases hit.
</output>
