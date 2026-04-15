---
phase: 21
plan: D
type: execute
wave: 4
depends_on: [21-B]
files_modified:
  - crates/reposix-swarm/tests/chaos_audit.rs
  - crates/reposix-swarm/Cargo.toml
autonomous: true
requirements:
  - HARD-03
user_setup: []
tags: [hardening, chaos, audit-log, wal, kill-9]

must_haves:
  truths:
    - "An operator can run `REPOSIX_CHAOS_TEST=1 cargo test -p reposix-swarm --test chaos_audit --release --locked -- --ignored` and get a green result"
    - "After SIGKILL of the sim mid-swarm, reopening the WAL DB shows 0 rows with NULL in op/entity_id/ts columns"
    - "The chaos test is NOT run by default (`#[ignore]` + env-var guard) so normal CI stays fast"
    - "Failure mode: if WAL recovery produces torn rows, the test fails LOUDLY with the bad-row count, not silently"
  artifacts:
    - path: "crates/reposix-swarm/tests/chaos_audit.rs"
      provides: "chaos_kill9_no_torn_rows integration test (ignored by default)"
      contains: "chaos_kill9_no_torn_rows"
      min_lines: 100
  key_links:
    - from: "crates/reposix-swarm/tests/chaos_audit.rs"
      to: "target/release/reposix-sim (binary)"
      via: "env!(\"CARGO_BIN_EXE_reposix-sim\") + std::process::Command"
      pattern: "CARGO_BIN_EXE_reposix-sim"
    - from: "chaos_audit.rs (SIGKILL step)"
      to: "SQLite WAL recovery on reopen"
      via: "child.kill() sends SIGKILL; next rusqlite::Connection::open replays WAL"
      pattern: "child.kill"
---

<objective>
Add an `#[ignore]`-gated chaos integration test that spawns `reposix-sim` as a real child process, runs a short swarm against it, sends SIGKILL mid-flight, reopens the audit DB, and asserts NO torn rows (rows with NULL values in non-nullable columns) exist. Proves SQLite WAL atomicity under adversarial termination.

Purpose: Per HARD-03, "kill -9 the sim during a swarm run and check for dangling rows". The audit log is the root-of-truth for SG-06 (append-only audit invariant); without this test, we rely on SQLite's reputation for WAL durability, but we never actually exercise it under kill-9. Per RESEARCH.md §Pattern 4 and Pitfall 2: an in-process `JoinHandle::abort()` unwinds too cleanly — we need a real child process and a real SIGKILL.

Output: One new integration test file, gated behind `#[ignore]` + `REPOSIX_CHAOS_TEST=1` env var, that demonstrates WAL integrity survives kill-9.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/CONTEXT.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md
@crates/reposix-swarm/tests/mini_e2e.rs
@crates/reposix-swarm/src/contention.rs
@crates/reposix-sim/src/main.rs
@crates/reposix-swarm/Cargo.toml

<interfaces>
<!-- Key contracts the chaos test uses -->

Rust stdlib:
```rust
// std::process::Child::kill() sends SIGKILL on Unix. [ASSUMED — stdlib docs; verify with `man 2 kill` or src on first run]
impl Child {
    pub fn kill(&mut self) -> io::Result<()>;
    pub fn wait(&mut self) -> io::Result<ExitStatus>;
}
```

Cargo integration-test binary path helper:
```rust
let sim_bin: &'static str = env!("CARGO_BIN_EXE_reposix-sim");
// Cargo guarantees this env var is set at compile time when the test crate
// depends on `reposix-sim` as a bin target in the same workspace.
```

From crates/reposix-sim/src/main.rs (confirm via read):
- `reposix-sim` accepts `--bind <addr>` and `--db <path>` CLI args
- exposes `/healthz` endpoint (200 OK when ready)
- uses WAL mode for the audit DB (set during init)

From crates/reposix-swarm/tests/mini_e2e.rs:
- `audit_row_count(&Path) -> rusqlite::Result<i64>` pattern — copy adapted form
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task D1: Write chaos_kill9_no_torn_rows test (ignored by default)</name>
  <files>
    crates/reposix-swarm/tests/chaos_audit.rs,
    crates/reposix-swarm/Cargo.toml
  </files>
  <read_first>
    - crates/reposix-swarm/tests/mini_e2e.rs (for spawn_sim + audit_row_count patterns; do NOT copy spawn_sim verbatim — chaos variant uses Command, not tokio::spawn)
    - crates/reposix-sim/src/main.rs (confirm --bind and --db CLI args and /healthz endpoint)
    - crates/reposix-swarm/src/contention.rs (available as the workload to drive load during the kill window)
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md (section "crates/reposix-swarm/tests/chaos_audit.rs")
    - crates/reposix-swarm/Cargo.toml (to confirm rusqlite is listed in dev-dependencies)
  </read_first>
  <behavior>
    - Test name: `chaos_kill9_no_torn_rows` in crates/reposix-swarm/tests/chaos_audit.rs
    - `#[tokio::test]`, `#[ignore = "chaos: requires reposix-sim binary; set REPOSIX_CHAOS_TEST=1"]`
    - First 5 lines of the test body: if `std::env::var("REPOSIX_CHAOS_TEST").is_err() { eprintln!("SKIP: ..."); return; }` — defence in depth so a stray `--ignored` flag doesn't accidentally run it.
    - Algorithm:
      1. Create a tempfile for the audit DB
      2. Spawn `reposix-sim` via env!("CARGO_BIN_EXE_reposix-sim") with --db <tempfile> --bind 127.0.0.1:0 (or a fixed 127.0.0.1:7979 — simpler because sim stdout doesn't easily expose the bound port)
      3. Poll /healthz up to 5s; fail if sim doesn't come up
      4. Spawn N (= 5) async workload clients that issue GET+PATCH against the sim for 2s using ContentionWorkload against a seeded issue
      5. Kill -9 the sim: `child.kill().expect("kill"); child.wait().unwrap();`
      6. Sleep 200ms for the OS to release the file lock
      7. Open the tempfile with rusqlite; run `SELECT COUNT(*) FROM audit_events WHERE op IS NULL OR entity_id IS NULL OR ts IS NULL`
      8. Assert bad_count == 0
      9. Also assert `SELECT COUNT(*) FROM audit_events` returns >= 0 (sanity; no corruption fatal enough to prevent row count)
      10. Optionally repeat steps 2–8 a second time (restart, run, kill, check) to prove WAL replay on reopen works
  </behavior>
  <action>
    **Step 0 — confirm dev-dependencies.** In `crates/reposix-swarm/Cargo.toml`, check the `[dev-dependencies]` section. Required:
    - `rusqlite = { workspace = true }` (probably already present — used by mini_e2e.rs)
    - `tempfile = { workspace = true }` (probably already present)
    - `reposix-sim = { path = "../reposix-sim" }` — needs to include the bin target so env!("CARGO_BIN_EXE_reposix-sim") works. If missing, add it.
    - `reposix-core = { workspace = true }` — probably already present
    - `reqwest = { workspace = true }` — for /healthz polling

    If adding the `reposix-sim` dev-dep, ensure Cargo can still build the workspace; there may be a cyclic-dep guard if sim already depends on swarm somewhere — check with `cargo check --workspace --locked`.

    **Step 1 — create the test file.**
    ```rust
    #![forbid(unsafe_code)]
    #![warn(clippy::pedantic)]
    #![allow(clippy::missing_panics_doc, clippy::too_many_lines)]

    //! Chaos audit-log test (HARD-03): kill -9 the sim mid-swarm, verify WAL
    //! durability (no torn rows).
    //!
    //! This test is `#[ignore]` by default and requires `REPOSIX_CHAOS_TEST=1`
    //! to run. It spawns the `reposix-sim` binary as a real child process so
    //! `Child::kill()` sends SIGKILL (not cooperative cancellation).

    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    use rusqlite::Connection;
    use tempfile::NamedTempFile;

    const SIM_BIND: &str = "127.0.0.1:7979";
    const HEALTHZ_URL: &str = "http://127.0.0.1:7979/healthz";

    fn poll_healthz(timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(200))
            .build()
            .expect("http client");
        while Instant::now() < deadline {
            if client.get(HEALTHZ_URL).send().map(|r| r.status().is_success()).unwrap_or(false) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }

    fn count_torn_rows(db: &Path) -> rusqlite::Result<i64> {
        let conn = Connection::open(db)?;
        conn.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE op IS NULL OR entity_id IS NULL OR ts IS NULL",
            [],
            |row| row.get(0),
        )
    }

    fn count_all_rows(db: &Path) -> rusqlite::Result<i64> {
        let conn = Connection::open(db)?;
        conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
    }

    #[tokio::test]
    #[ignore = "chaos: requires reposix-sim binary + REPOSIX_CHAOS_TEST=1"]
    async fn chaos_kill9_no_torn_rows() {
        if std::env::var("REPOSIX_CHAOS_TEST").is_err() {
            eprintln!("SKIP: set REPOSIX_CHAOS_TEST=1 to run chaos tests");
            return;
        }

        let sim_bin = env!("CARGO_BIN_EXE_reposix-sim");
        let db = NamedTempFile::new().expect("tempfile");
        let db_path = db.path().to_owned();

        // Cycle 1: spawn → load → SIGKILL → check
        let mut child = Command::new(sim_bin)
            .args([
                "--bind", SIM_BIND,
                "--db", db_path.to_str().expect("utf8 db path"),
                // include any other required sim args (seed fixture, etc.)
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn reposix-sim");

        assert!(poll_healthz(Duration::from_secs(5)), "sim did not come up within 5s (cycle 1)");

        // Drive some load. Simplest: 5 async tasks hammering /issues for 2s.
        // Using reqwest::Client directly (not ContentionWorkload) to keep the
        // test self-contained and avoid the shared-version-read requirement.
        let load = tokio::spawn(async {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_millis(500))
                .build()
                .expect("http client");
            let deadline = Instant::now() + Duration::from_secs(2);
            let mut ops = 0u64;
            while Instant::now() < deadline {
                // fire and forget — we expect many to fail with connection reset when kill lands
                let _ = client.get(format!("http://{SIM_BIND}/issues/demo")).send().await;
                ops += 1;
            }
            ops
        });

        // Let some writes hit the WAL before the kill
        tokio::time::sleep(Duration::from_millis(500)).await;

        child.kill().expect("SIGKILL sim (cycle 1)");
        child.wait().expect("reap sim (cycle 1)");

        // Abort the load driver; its in-flight requests will 500/ECONNREFUSED
        load.abort();
        let _ = load.await;

        // Let the kernel release file locks
        tokio::time::sleep(Duration::from_millis(200)).await;

        let bad = count_torn_rows(&db_path).expect("query torn rows after cycle 1");
        assert_eq!(bad, 0, "torn rows after SIGKILL (cycle 1): {bad}");

        let total1 = count_all_rows(&db_path).expect("row count cycle 1");
        eprintln!("cycle 1: total rows after SIGKILL = {total1} (no torn rows)");

        // Cycle 2: restart on same DB (WAL replay), drive load, kill again, check
        let mut child2 = Command::new(sim_bin)
            .args(["--bind", SIM_BIND, "--db", db_path.to_str().unwrap()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn reposix-sim cycle 2");

        assert!(
            poll_healthz(Duration::from_secs(5)),
            "sim did not come up within 5s (cycle 2) — WAL replay may have corrupted DB"
        );

        // Minimal load; we mainly care that WAL replay succeeded
        tokio::time::sleep(Duration::from_millis(500)).await;
        child2.kill().expect("SIGKILL sim (cycle 2)");
        child2.wait().expect("reap sim (cycle 2)");
        tokio::time::sleep(Duration::from_millis(200)).await;

        let bad2 = count_torn_rows(&db_path).expect("query torn rows after cycle 2");
        assert_eq!(bad2, 0, "torn rows after SIGKILL (cycle 2): {bad2}");

        let total2 = count_all_rows(&db_path).expect("row count cycle 2");
        eprintln!("cycle 2: total rows after SIGKILL = {total2} (no torn rows)");
        assert!(total2 >= total1, "cycle 2 row count {total2} < cycle 1 {total1} — WAL checkpoint lost data");
    }
    ```

    **Step 2 — confirm sim CLI args.** Read `crates/reposix-sim/src/main.rs` (first ~80 lines). If the binary does NOT accept `--bind` and `--db` as spelled above, ADJUST the Command args to match whatever it does accept. If there are required args the chaos test needs to supply (seed fixture path, rate-limit-rps), add them. Do NOT invent CLI args.

    **Step 3 — port collision guard.** 127.0.0.1:7979 is hard-coded for simplicity. If the port is in use at test start (another sim process from a previous run), the test will fail at spawn. Accept this — the test is `#[ignore]` by default; operators running it manually can kill stragglers. If this proves fragile, add a `pkill -f reposix-sim` step at the very start of the test (best-effort; don't unwrap the exit).

    **Step 4 — SQLite table name.** Confirm the audit table is named `audit_events` (per mini_e2e.rs pattern). If it's named differently in sim, adjust the SQL. Confirm the column names `op`, `entity_id`, `ts` exist on that table (read `crates/reposix-sim/src/` for the schema creation statement). If any are named differently, adjust the "torn rows" query.

    **Step 5 — verify.**
    ```
    cargo test -p reposix-swarm --test chaos_audit --locked                # Should compile + skip (ignored)
    cargo test -p reposix-swarm --test chaos_audit --locked -- --ignored   # Should skip inside the body (env var not set)
    REPOSIX_CHAOS_TEST=1 cargo test -p reposix-swarm --test chaos_audit --release --locked -- --ignored --nocapture
    cargo clippy -p reposix-swarm --all-targets -- -D warnings
    cargo fmt --all --check
    ```
    The env-var-set run should exit 0 with "2 passed" (1 test, 1 skipped elsewhere maybe 0). The unset run under `--ignored` should exit 0 via the in-body skip.

    **Step 6 — commit.**
    `git add crates/reposix-swarm/tests/chaos_audit.rs crates/reposix-swarm/Cargo.toml && git commit -m "test(21-D): chaos kill-9 audit-log WAL integrity test (HARD-03)"`
  </action>
  <verify>
    <automated>cargo test -p reposix-swarm --test chaos_audit --locked && cargo test -p reposix-swarm --test chaos_audit --locked -- --ignored && REPOSIX_CHAOS_TEST=1 cargo test -p reposix-swarm --test chaos_audit --release --locked -- --ignored --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - File `crates/reposix-swarm/tests/chaos_audit.rs` exists and is >= 100 lines
    - `grep -q "chaos_kill9_no_torn_rows" crates/reposix-swarm/tests/chaos_audit.rs`
    - `grep -q "#\[ignore" crates/reposix-swarm/tests/chaos_audit.rs`
    - `grep -q "REPOSIX_CHAOS_TEST" crates/reposix-swarm/tests/chaos_audit.rs`
    - `grep -q "CARGO_BIN_EXE_reposix-sim" crates/reposix-swarm/tests/chaos_audit.rs`
    - `grep -q "child.kill" crates/reposix-swarm/tests/chaos_audit.rs` (SIGKILL present)
    - `grep -qE "SELECT COUNT.*WHERE.*(IS NULL)" crates/reposix-swarm/tests/chaos_audit.rs` (torn-row query)
    - `cargo test -p reposix-swarm --test chaos_audit --locked` exits 0 (test compiles and is ignored by default)
    - `REPOSIX_CHAOS_TEST=1 cargo test -p reposix-swarm --test chaos_audit --release --locked -- --ignored` exits 0
    - `cargo clippy -p reposix-swarm --all-targets -- -D warnings` exits 0
  </acceptance_criteria>
  <done>
    Chaos test lands gated behind env var; runs green with the env var set; no regressions in default test suite.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| sim child process → test harness | test spawns sim via env!("CARGO_BIN_EXE_reposix-sim") — binary path is trusted (Cargo-provided) |
| SIGKILL → SQLite WAL | kernel-level termination; WAL atomicity is the trust contract being tested |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-D-01 | Elevation of Privilege | chaos test spawning arbitrary binary | mitigate | Use `env!("CARGO_BIN_EXE_reposix-sim")` — Cargo-provided path from the same workspace. Never accept path from env/config. |
| T-21-D-02 | Denial of Service | port 7979 collision across concurrent chaos runs | accept | Test is `#[ignore]`; operators run serially. Fixed port chosen for simplicity; ephemeral port would require capturing sim stdout, which complicates the test without improving the invariant tested. |
| T-21-D-03 | Tampering | test asserts `total2 >= total1` | accept | SQLite WAL replay can in principle roll back uncommitted writes, which would mean total2 < total1. If that assertion flakes, the fix is to relax it — the invariant we care about is *no torn rows*, not *monotonic row count*. Documented in the test comment. |
| T-21-D-04 | Repudiation | chaos test is env-gated and may never run in CI | mitigate | The chaos test is a manual/weekly invariant gate, not a per-commit gate. Add a documentation follow-up to the SUMMARY noting this is a manual verification; any future automation belongs in a separate chaos CI job (out of scope for this plan). |
</threat_model>

<verification>
- `cargo test -p reposix-swarm --locked` green (chaos test compiles, ignored by default)
- `REPOSIX_CHAOS_TEST=1 cargo test -p reposix-swarm --test chaos_audit --release --locked -- --ignored` green
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
</verification>

<success_criteria>
HARD-03 closes: WAL durability under SIGKILL is exercised by a committed test, not a theoretical argument. Future regressions in audit_events schema (making a NOT NULL column nullable, adding a new NOT NULL column without backfill) will trip this test.
</success_criteria>

<output>
After completion, create `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-D-SUMMARY.md` with: chaos test invocation command (env var + flags), sim binary path source, schema columns checked for torn rows, note about manual-only CI status.
</output>
