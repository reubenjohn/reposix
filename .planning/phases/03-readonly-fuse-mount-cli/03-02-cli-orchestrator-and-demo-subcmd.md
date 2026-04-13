---
phase: 03-readonly-fuse-mount-cli
plan: 02
type: execute
wave: 2
depends_on:
  - "03-01"
files_modified:
  - Cargo.toml
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
    - "All HTTP in the CLI goes through `reposix_core::http::client()` (returning the sealed `HttpClient` newtype); no direct `reqwest::Client::new` anywhere in reposix-cli."
    - "Ctrl-C during `reposix demo` unmounts the FUSE tempdir and kills the sim child before the process exits — no orphaned mounts, no zombie sims."
    - "If `fusermount3 -u` ever blocks (lazy unmount / stuck kernel), the demo kills the fuse child with SIGKILL within 3s rather than hanging forever."
  artifacts:
    - path: "crates/reposix-cli/src/main.rs"
      provides: "clap-derive top-level dispatcher: sim | mount | demo"
    - path: "crates/reposix-cli/src/sim.rs"
      provides: "child-process wrapper around `reposix-sim` binary"
    - path: "crates/reposix-cli/src/mount.rs"
      provides: "child-process wrapper around `reposix-fuse` binary; `MountProcess::drop` wraps `fusermount3 -u` in a 3-second watchdog"
    - path: "crates/reposix-cli/src/demo.rs"
      provides: "End-to-end orchestration: spawn sim, healthz-wait, mount, ls/cat/grep, audit tail; single `Guard` struct owns sim+mount+tempdir so Drop order is deterministic; `tokio::signal::ctrl_c` handler ensures SIGINT unmounts cleanly"
    - path: "crates/reposix-cli/tests/cli.rs"
      provides: "Integration test: `--help` contains all subcommands; `reposix demo` exits 0 in <30s"
  key_links:
    - from: "crates/reposix-cli/src/demo.rs :: Guard"
      to: "crates/reposix-cli/src/{sim,mount}.rs + tempfile::TempDir"
      via: "Guard drops in reverse order: mount first, then sim, then tempdir"
      pattern: "impl Drop for Guard"
    - from: "crates/reposix-cli/src/mount.rs :: MountProcess::drop"
      to: "fusermount3 -u"
      via: "spawn + 3s polling try_wait watchdog; SIGKILL child if unmount hangs"
      pattern: "fusermount3"
    - from: "crates/reposix-cli/src/demo.rs :: run"
      to: "tokio::signal::ctrl_c"
      via: "select! race between scripted steps and Ctrl-C so drop still fires"
      pattern: "ctrl_c"
    - from: "crates/reposix-cli/src/demo.rs :: wait_for_healthz"
      to: "reposix_core::http::client + HttpClient::get"
      via: "5s-timeout GET loop on /healthz"
      pattern: "http_client\\.(get|request)"
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
@Cargo.toml

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

`reposix_core::http` (POST-H-01 sealed newtype):
```rust
pub fn client(opts: ClientOpts) -> Result<HttpClient>;     // ONLY legal ctor

impl HttpClient {
    pub async fn get<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response>;
    pub async fn request<U: IntoUrl>(&self, m: Method, url: U)
        -> Result<reqwest::Response>;
    // ... post/patch/delete/request_with_headers (added by 03-01 Task 1)
}
```
There is NO public accessor for the inner `reqwest::Client`. `wait_for_healthz`
uses `http_client.get(&url).await` — it does NOT need custom headers, so it
stays on the plain `get` wrapper.

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
- **Healthz wait loop.** After spawning sim, `reposix demo` builds an
  `HttpClient` via `reposix_core::http::client(ClientOpts::default())` and
  loops `http_client.get(&format!("http://{bind}/healthz")).await` every
  100ms for up to 5s. On first 200 → proceed. On timeout → bail with a clear
  error ("sim did not become ready"). No `sleep` heuristics.
- **Mount readiness wait.** After spawning the fuse child, poll
  `std::fs::read_dir(mount_point)` every 100ms for up to 3s; consider
  ready when it returns Ok with ≥ 1 entry. Absent a shell `mountpoint` we
  use stdlib.
- **Demo steps (matches CONTEXT.md §CLI orchestrator):**
  1. Start sim (ephemeral DB at `runtime/demo-sim-{pid}.db`, clean slate).
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
  6. Drop in reverse order via a single top-level `Guard` (see below).
- **Guard-based cleanup (new).** Introduce `struct Guard { mount:
  Option<MountProcess>, sim: Option<SimProcess>, tempdir: Option<tempfile::TempDir> }`
  in `demo.rs`. `run()` constructs the Guard in the same sequence as setup
  (sim → mount → tempdir populated as they succeed). `impl Drop for Guard`
  calls `self.mount.take()`, then `self.sim.take()`, then `self.tempdir.take()`
  — dropping each in turn. This makes the teardown order deterministic
  regardless of where in the scripted flow an error surfaces, and ensures
  both child processes AND the tempdir go away on panic/early-return.
- **Ctrl-C handling (new).** At the top of `run()`, spawn a
  `tokio::signal::ctrl_c()` future and race it against the scripted body via
  `tokio::select!`. If Ctrl-C wins, the scripted body is cancelled, control
  falls back to `run()`'s normal return, the `Guard` drops, everything
  unmounts. No extra crate needed — `tokio::signal` is already pulled in by
  the workspace `tokio` dep (features: `signal`).
- **Unmount watchdog (new).** `MountProcess::drop` must not block forever if
  `fusermount3 -u` hangs on a lazy kernel unmount (the grandchild fuse
  process is still holding state). Implementation:
  ```rust
  // SIGTERM the fuse child first
  let _ = self.child.kill();
  // Spawn fusermount3 -u <mount>
  let mut um = match std::process::Command::new("fusermount3")
      .arg("-u").arg(&self.mount)
      .spawn() {
        Ok(c) => c, Err(_) => { let _ = self.child.wait(); return; }
      };
  let t0 = Instant::now();
  loop {
      match um.try_wait() {
          Ok(Some(_)) => break,                 // unmounted
          Ok(None) if t0.elapsed() >= Duration::from_secs(3) => {
              let _ = um.kill();                // fusermount3 hung → SIGKILL
              let _ = um.wait();
              break;
          }
          Ok(None) => std::thread::sleep(Duration::from_millis(50)),
          Err(_) => break,
      }
  }
  let _ = self.child.wait();
  ```
  The 3-second cap is the same budget `Mount::open` drop has in 03-01.
- **No new workspace dep.** We use `tokio::signal` (feature `signal` on the
  `tokio` workspace dep — add it in the root `Cargo.toml` if absent) rather
  than the `ctrlc` crate. The CLI already runs under `#[tokio::main]`, so
  there is no blocking-vs-async impedance mismatch. Adding `ctrlc = "3"`
  would be a second signal mechanism for no benefit.
- **No daemonization.** `reposix mount` is foreground + blocking (so Ctrl-C
  unmounts). `reposix demo` owns its own process tree; the fuse mount inside
  it is a grandchild that the `Guard` explicitly kills.
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
| T-03-09 | Tampering | user-supplied `--backend` URL could target non-loopback | mitigate | `HttpClient::request` rejects non-allowlisted origins; the demo subcommand hard-codes `http://127.0.0.1:7878`. `reposix mount` forwards user input to the child; the child's fetch layer re-enforces the allowlist. |
| T-03-10 | Denial of Service | sim never becomes ready | mitigate | 5s healthz timeout → clean bail with non-zero exit. |
| T-03-11 | Information Disclosure | audit tail could leak sensitive rows | accept | v0.1 sim is local, seeded with demo data only; the audit log is precisely what we want to display on camera. |
| T-03-12 | Elevation of Privilege | zombie fuse child / orphaned mount survives demo exit | mitigate | Top-level `Guard` struct owns sim + mount + tempdir and drops them in reverse order on any exit path (normal return, `?` propagation, panic, Ctrl-C). `MountProcess::drop`'s unmount is bounded by a 3-second watchdog (SIGKILLs `fusermount3` if it hangs). `tokio::signal::ctrl_c()` is raced against the scripted body so SIGINT still runs Drop. |
</threat_model>

<tasks>

<task type="auto">
  <name>Task 1: Subcommand plumbing (sim + mount) with clap-derive</name>
  <files>
    Cargo.toml,
    crates/reposix-cli/Cargo.toml,
    crates/reposix-cli/src/main.rs,
    crates/reposix-cli/src/sim.rs,
    crates/reposix-cli/src/mount.rs,
    crates/reposix-cli/tests/cli.rs
  </files>
  <action>
    1. Workspace root `Cargo.toml`: ensure the shared `tokio` dep carries the
       `signal` feature (alongside the existing `macros`, `rt-multi-thread`,
       `time` features). If `tokio` in `[workspace.dependencies]` is written as
       a features list, add `"signal"` to it. No new workspace deps beyond
       this.
    2. `crates/reposix-cli/Cargo.toml`: add `[dependencies]` if missing:
       `libc` (workspace — for process-group plumbing in task 3), `rusqlite =
       { workspace = true, features = ["bundled"] }` (for audit tail in task
       2), `tempfile = { workspace = true }`, `tokio = { workspace = true,
       features = ["macros", "rt-multi-thread", "time", "signal"] }`. Add
       `[dev-dependencies]`: `assert_cmd = "2"`, `predicates = "3"`.
    3. `src/sim.rs` — `pub struct SimProcess(std::process::Child)`:
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
    4. `src/mount.rs` — `pub struct MountProcess { child: Child, mount: PathBuf }`:
       - `MountProcess::spawn(mount_point: &Path, backend: &str, project: &str)`
         returns `anyhow::Result<Self>`. Uses
         `resolve_sibling_bin("reposix-fuse")`. Use `Command::process_group(0)`
         (stable since Rust 1.64 on Unix via `CommandExt::process_group` — no
         unsafe required) to put the child in its own process group so a
         negative-pgid kill can reap the whole tree.
       - After spawn, poll `std::fs::read_dir(mount_point)` every 100ms up
         to 3s; return `Err` if never ready.
       - `impl Drop for MountProcess`: implement the unmount watchdog from
         `<decisions>` — SIGTERM the child, spawn `fusermount3 -u <mount>`,
         poll `try_wait` every 50ms with an `Instant::now()` check, SIGKILL
         `fusermount3` if 3s elapses. Final `self.child.wait()` to reap the
         zombie. Never panic.
    5. `src/main.rs` — rewrite the current placeholder:
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
    6. `tests/cli.rs`:
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
    7. `cargo fmt`, `cargo clippy -p reposix-cli --all-targets -- -D warnings`,
       `cargo test -p reposix-cli`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-cli --test cli help_lists_all_subcommands</automated>
  </verify>
  <done>
    `reposix --help`, `reposix sim --help`, `reposix mount --help`,
    `reposix demo --help` all render. Sim and Mount subcommands spawn the
    respective child binaries and propagate exit codes. `MountProcess::drop`
    unmount is bounded by 3s (watchdog implemented). No unsafe; no direct
    reqwest ctor; no new workspace deps (only the `signal` feature flag on
    the existing `tokio` dep). Commit subject:
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
       -> anyhow::Result<()>`. The body introduces a top-level `Guard` struct
       and races scripted work against `tokio::signal::ctrl_c`:

       ```rust
       #[derive(Default)]
       struct Guard {
           mount: Option<MountProcess>,
           sim: Option<SimProcess>,
           tempdir: Option<tempfile::TempDir>,
       }

       impl Drop for Guard {
           fn drop(&mut self) {
               // Reverse order: unmount first, then stop sim, then wipe tempdir.
               self.mount.take();   // MountProcess::drop runs fusermount3 -u
                                    // bounded by the 3s watchdog.
               self.sim.take();     // SimProcess::drop SIGTERMs then SIGKILLs.
               self.tempdir.take(); // TempDir::drop rm -rfs the mount point.
           }
       }

       pub async fn run(keep_running: bool) -> Result<()> {
           let mut guard = Guard::default();
           let body = async {
               // 1/6: Spawn sim on ephemeral DB in runtime/demo-sim-{pid}.db
               let db = PathBuf::from("runtime")
                   .join(format!("demo-sim-{}.db", std::process::id()));
               std::fs::create_dir_all("runtime").ok();
               let seed = PathBuf::from("crates/reposix-sim/fixtures/seed.json");
               let bind = "127.0.0.1:7878";
               guard.sim = Some(SimProcess::spawn(bind, &db, Some(&seed))?);

               // 2/6: Wait for /healthz (uses HttpClient::get)
               wait_for_healthz(&format!("http://{bind}/healthz"),
                                Duration::from_secs(5)).await?;

               // 3/6: Mount on a tempdir
               let td = tempfile::Builder::new().prefix("reposix-demo-")
                           .tempdir()?;
               let mount_path = td.path().to_path_buf();
               guard.tempdir = Some(td);
               guard.mount = Some(MountProcess::spawn(
                   &mount_path, &format!("http://{bind}"), "demo",
               )?);

               // 4/6: Scripted ls / cat / grep
               let listing = list_sorted(&mount_path)?;
               info!("ls: {listing:?}");
               let first = mount_path.join(listing.first().unwrap());
               let body = std::fs::read_to_string(&first)?;
               info!("cat {first:?} (first 3 lines):\n{}",
                     body.lines().take(3).collect::<Vec<_>>().join("\n"));
               let hits = grep_ril(&mount_path, "database")?;
               info!("grep -ril database: {hits:?}");

               // 5/6: Audit tail
               print_audit_tail(&db, 5)?;

               // 6/6: Cleanup is implicit via Guard::drop at function exit.
               if keep_running { wait_for_ctrl_c().await?; }
               Ok::<_, anyhow::Error>(())
           };

           tokio::select! {
               res = body => res?,
               _ = tokio::signal::ctrl_c() => {
                   tracing::info!("Ctrl-C received, cleaning up");
                   // Fall through to Guard::drop.
               }
           }
           drop(guard);  // explicit for clarity; drop order preserved
           Ok(())
       }
       ```

       Helpers (all in `demo.rs`):
       - `async fn wait_for_healthz(url: &str, timeout: Duration) -> Result<()>`
         — uses `reposix_core::http::client(ClientOpts::default())?` (returns
         `HttpClient`) and polls with `http_client.get(url).await` every
         `Duration::from_millis(100)`. Wrap the whole loop in
         `tokio::time::timeout(timeout, ...)` → `anyhow::bail!("sim did not
         become ready within {timeout:?}")` on elapsed.
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
       `#[tokio::main]` (it is in the current skeleton). The `#[tokio::main]`
       attribute auto-pulls the `signal` feature via the workspace Cargo.toml
       change in Task 1 step 1.

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
      Separately: `./target/release/reposix demo --keep-running` in a
      terminal, press Ctrl-C mid-run, then check `mount | grep reposix-demo`
      is empty AND `pgrep -af reposix-fuse` returns nothing.
    </manual>
  </verify>
  <done>
    `reposix demo` end-to-end: spawns sim, health-gates, mounts, runs
    ls/cat/grep on the live mount, tails 5 audit rows, unmounts, exits 0 in
    <30s. Ctrl-C during the demo unmounts the tempdir and reaps the fuse +
    sim children via the `Guard`. Integration test asserts the happy path in
    CI's `--ignored` gate. SC #3 (`grep -r database` returns ≥ 1 hit)
    satisfied by the demo's grep step succeeding. Commit subject:
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
  `reposix-fuse-{pid}` (verified by `print_audit_tail`; the attribution
  comes from the `X-Reposix-Agent` header the FUSE daemon attaches via
  `request_with_headers`).
- Ctrl-C during `reposix demo` leaves no orphaned mounts (`mount | grep
  reposix-demo` is empty) and no zombie children (`pgrep -af reposix-fuse`
  returns nothing).
</success_criteria>

<output>
After completion, create `.planning/phases/03-readonly-fuse-mount-cli/03-02-SUMMARY.md`
capturing: subprocess-vs-library choice, the actual `Guard` struct drop
sequence, audit query SQL, the unmount watchdog's measured wall-clock
behaviour (does `fusermount3 -u` ever hit the 3s bound in practice?), and
any signal-handling edge cases hit.
</output>
</content>
</invoke>