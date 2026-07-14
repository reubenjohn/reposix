← [back to index](./index.md) · phase 80 plan 01 · [continued in 06b →](./06b-T04-integration-close.md)

## Task 80-01-T04 — Integration tests + verifier flip + CLAUDE.md update + per-phase push (terminal)

> **Split note:** this chapter covers § 4a (integration tests — `mirror_refs.rs`
> test file + M1 dev-dependencies + H3 fallback).
> §§ 4b–4d (verifier flip, CLAUDE.md update, per-phase push) continue in
> [06b-T04-integration-close.md](./06b-T04-integration-close.md).

<read_first>
- `crates/reposix-cli/tests/agent_flow.rs` (entire file — integration
  test pattern; how a test starts a sim subprocess + runs CLI binary
  against a tempdir; the new
  `crates/reposix-remote/tests/mirror_refs.rs` mirrors this pattern).
- `crates/reposix-cli/tests/attach.rs` (entire file — P79 integration
  test precedent; SimSubprocess helper + tempdir isolation).
- `crates/reposix-remote/Cargo.toml` `[dev-dependencies]` — confirm
  `tempfile`, `tokio`, etc. are available; add `predicates` if missing
  (P79's attach.rs uses them).
- `quality/runners/run.py` lines 50-100 — `--cadence pre-pr` runner
  invocation shape (P79 precedent verified valid).
- `CLAUDE.md` § "Commands you'll actually use" + § Architecture or
  Threat model (locate the natural insertion point for the
  `refs/mirrors/...` namespace note).
- `crates/reposix-remote/src/main.rs::handle_export` (the wiring from
  T03 — to know what stderr tokens to assert against in tests).
- `crates/reposix-cache/src/mirror_refs.rs` (the module from T02 — to
  know the function signatures and tag-message-body shape).
</read_first>

<action>
Three concerns in this task; keep ordering: integration tests →
runner flip → CLAUDE.md edit + commit + push.

### 4a. Integration tests

Author `crates/reposix-remote/tests/mirror_refs.rs` (≤ 350 lines).
The shape mirrors `crates/reposix-cli/tests/attach.rs` (P79 precedent):
SimSubprocess helper, tempdir isolation, env-var-based cache routing.

```rust
//! Integration tests for mirror-lag refs (DVCS-MIRROR-REFS-01..03).
//!
//! Each test starts a fresh `reposix-sim` subprocess on a unique port,
//! drives a single-backend `git push` via the existing reposix init +
//! edit + push flow, then asserts on the cache's bare-repo refs +
//! audit table + helper stderr.
//!
//! Pattern donor: crates/reposix-cli/tests/attach.rs (P79).

use std::path::Path;
use std::process::{Command, Stdio};

mod helpers {
    use std::net::TcpListener;
    use std::process::{Child, Command};
    use std::time::{Duration, Instant};

    pub struct SimSubprocess {
        pub child: Child,
        pub port: u16,
    }

    impl SimSubprocess {
        pub fn start() -> Self {
            let port = pick_free_port();
            let child = Command::new(env!("CARGO_BIN_EXE_reposix-sim"))
                .args(["--bind", &format!("127.0.0.1:{port}"), "--ephemeral"])
                .spawn()
                .expect("spawn reposix-sim");
            wait_for_bind(port, Duration::from_secs(5));
            Self { child, port }
        }
    }

    impl Drop for SimSubprocess {
        fn drop(&mut self) {
            let _ = self.child.kill();
        }
    }

    fn pick_free_port() -> u16 {
        TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
    }

    fn wait_for_bind(port: u16, timeout: Duration) {
        let start = Instant::now();
        loop {
            if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
                return;
            }
            if start.elapsed() > timeout {
                panic!("sim did not bind on port {port} within {timeout:?}");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

use helpers::SimSubprocess;

/// Helper: drive a full reposix-init → edit → git-push cycle.
/// Returns the cache-dir tempdir + the working-tree tempdir for
/// post-conditions to inspect.
fn drive_init_edit_push(
    sim_port: u16,
    cache_dir: &Path,
    work_dir: &Path,
    edit_content: &str,
) -> std::process::Output {
    // reposix init.
    //
    // H1 NOTE: `reposix init` does NOT honor REPOSIX_SIM_ORIGIN — it
    // hardcodes DEFAULT_SIM_ORIGIN to http://127.0.0.1:7878 in
    // crates/reposix-cli/src/init.rs:23-55. Only `reposix attach` reads
    // the env var. We re-point remote.origin.url AFTER init so the
    // subsequent `git push` reaches our test sim. Precedent:
    // crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path
    // (lines 115-146).
    let init_out = Command::new(env!("CARGO_BIN_EXE_reposix"))
        .args(["init", "sim::demo", &work_dir.to_string_lossy()])
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .output()
        .expect("reposix init");
    assert!(init_out.status.success(), "init failed: {}", String::from_utf8_lossy(&init_out.stderr));

    // Re-point remote.origin.url so subsequent git ops reach our test sim.
    let target_url = format!("reposix::http://127.0.0.1:{sim_port}/projects/demo");
    Command::new("git").current_dir(work_dir)
        .args(["config", "remote.origin.url", &target_url])
        .status()
        .expect("git config remote.origin.url");

    // git checkout origin/main -B main
    Command::new("git").current_dir(work_dir)
        .args(["checkout", "origin/main", "-B", "main", "-q"])
        .status().unwrap();

    // Edit one fixture file. The sim seeds at least one issue at
    // issues/0001.md; appending a trivial change drives a single-
    // record push.
    let issue_path = work_dir.join("issues/0001.md");
    let existing = std::fs::read_to_string(&issue_path).unwrap();
    std::fs::write(&issue_path, format!("{existing}\n{edit_content}")).unwrap();

    Command::new("git").current_dir(work_dir).args(["add", "."]).status().unwrap();
    Command::new("git").current_dir(work_dir).args(["commit", "-q", "-m", "test push"]).status().unwrap();

    // git push (drives handle_export through the helper).
    // REPOSIX_SIM_ORIGIN no longer needed — the remote URL re-pointing
    // above carries the port for the helper.
    Command::new("git").current_dir(work_dir)
        .args(["push", "origin", "main"])
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .expect("git push")
}

/// Locate the cache's bare repo under cache_dir.
fn find_cache_bare(cache_dir: &Path) -> std::path::PathBuf {
    walkdir::WalkDir::new(cache_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().is_dir() && e.path().extension().map(|x| x == "git").unwrap_or(false))
        .expect("cache bare repo not found")
        .path()
        .to_path_buf()
}

#[test]
fn write_on_success_updates_both_refs() {
    let sim = SimSubprocess::start();
    let cache_dir = tempfile::tempdir().unwrap();
    let work_dir = tempfile::tempdir().unwrap();

    let push_out = drive_init_edit_push(sim.port, cache_dir.path(), work_dir.path(), "trivial-change");
    assert!(push_out.status.success(), "push failed: {}", String::from_utf8_lossy(&push_out.stderr));

    let cache_bare = find_cache_bare(cache_dir.path());

    // Both refs resolvable.
    let head_out = Command::new("git").arg("-C").arg(&cache_bare)
        .args(["for-each-ref", "refs/mirrors/"]).output().unwrap();
    let head_str = String::from_utf8_lossy(&head_out.stdout);
    assert!(head_str.contains("refs/mirrors/sim-head"), "missing sim-head: {head_str}");
    assert!(head_str.contains("refs/mirrors/sim-synced-at"), "missing sim-synced-at: {head_str}");

    // Tag message body's first line.
    let msg_out = Command::new("git").arg("-C").arg(&cache_bare)
        .args(["log", "refs/mirrors/sim-synced-at", "-1", "--format=%B"]).output().unwrap();
    let msg = String::from_utf8_lossy(&msg_out.stdout);
    let first_line = msg.lines().next().unwrap_or("");
    assert!(
        first_line.starts_with("mirror synced at ") &&
        chrono::DateTime::parse_from_rfc3339(first_line.trim_start_matches("mirror synced at ")).is_ok(),
        "tag message body first line malformed: {first_line:?}",
    );

    // audit_events_cache row.
    let db_path = cache_bare.join("cache.db");  // M2: cache.db lives INSIDE cache_dir per crates/reposix-cache/src/db.rs:35-37 + cache.rs:115-117
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let count: i64 = conn
        .prepare("SELECT count(*) FROM audit_events_cache WHERE op = 'mirror_sync_written'")
        .unwrap()
        .query_row([], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1, "expected exactly 1 mirror_sync_written audit row; got {count}");
}

#[test]
fn vanilla_fetch_brings_mirror_refs() {
    let sim = SimSubprocess::start();
    let cache_dir = tempfile::tempdir().unwrap();
    let work_dir = tempfile::tempdir().unwrap();

    let _ = drive_init_edit_push(sim.port, cache_dir.path(), work_dir.path(), "v-fetch-test");
    let cache_bare = find_cache_bare(cache_dir.path());

    // Vanilla `git clone --bare` from the cache repo — no reposix.
    let clone_dir = tempfile::tempdir().unwrap();
    let clone_path = clone_dir.path().join("mirror.git");
    let clone_out = Command::new("git")
        .args(["clone", "--bare", "-q", &cache_bare.to_string_lossy(), &clone_path.to_string_lossy()])
        .output().unwrap();
    assert!(clone_out.status.success(), "clone failed: {}", String::from_utf8_lossy(&clone_out.stderr));

    let refs_out = Command::new("git").arg("-C").arg(&clone_path)
        .args(["for-each-ref", "refs/mirrors/"]).output().unwrap();
    let refs_str = String::from_utf8_lossy(&refs_out.stdout);
    assert!(refs_str.contains("refs/mirrors/sim-head"), "vanilla clone missing sim-head");
    assert!(refs_str.contains("refs/mirrors/sim-synced-at"), "vanilla clone missing sim-synced-at");
}

#[test]
fn reject_hint_cites_synced_at_with_age() {
    let sim = SimSubprocess::start();
    let cache_dir = tempfile::tempdir().unwrap();
    let work1 = tempfile::tempdir().unwrap();
    let work2 = tempfile::tempdir().unwrap();

    // First successful push from work1 — populates refs/mirrors/*.
    let _ = drive_init_edit_push(sim.port, cache_dir.path(), work1.path(), "first-push");
    std::thread::sleep(std::time::Duration::from_millis(100));  // ensure non-zero age math

    // Second push from work2 with a stale prior — conflict.
    // Re-init work2 against the same cache (cache now has sim's post-first-push state);
    // edit and push: sim sees version mismatch, helper rejects.
    let push2_out = drive_init_edit_push(sim.port, cache_dir.path(), work2.path(), "stale-prior");
    let stderr = String::from_utf8_lossy(&push2_out.stderr);

    assert!(
        stderr.contains("refs/mirrors/sim-synced-at"),
        "reject stderr missing refs/mirrors/sim-synced-at citation: {stderr}",
    );
    let rfc3339_re = regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.*Z").unwrap();
    assert!(rfc3339_re.is_match(&stderr), "reject stderr missing RFC3339 timestamp: {stderr}");
    let ago_re = regex::Regex::new(r"\d+ minutes ago").unwrap();
    assert!(ago_re.is_match(&stderr), "reject stderr missing '(N minutes ago)' rendering: {stderr}");
}

#[test]
fn reject_hint_first_push_omits_synced_at_line() {
    // H3 fix (per PLAN-CHECK.md): engineer a REAL first-push conflict
    // by seeding the sim with a record at version > 1 BEFORE the working
    // tree's first push (which uses version 1 as the prior). The
    // conflict-reject path fires; we then assert (a) it IS the
    // conflict path (stderr cites "fetch first" or "modified on
    // backend"), AND (b) the synced-at hint is cleanly omitted (no
    // "synced at" / "minutes ago" lines) — because refs/mirrors/* do
    // NOT exist yet at first-push time.
    //
    // This is the strong form: it actually exercises the
    // conflict-reject branch's None-case behavior at the helper layer
    // (where the synced-at hint is composed). The previous "weaker
    // form" drove a successful push and asserted no "minutes ago" —
    // vacuously true (success branch never composes the hint).
    //
    // PATH B FALLBACK (prescribed if sim-seeding is brittle at executor
    // time): move this assertion to a unit test in
    // `crates/reposix-remote/src/main.rs` `#[cfg(test)] mod tests` that
    // exercises the conflict-reject branch directly with a stubbed
    // cache returning Ok(None) from read_mirror_synced_at — bypasses
    // the sim entirely. Document the move in the T04 commit message
    // body if Path B is taken.
    let sim = SimSubprocess::start();
    let cache_dir = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();

    // Pre-mutate the sim's record version BEFORE any reposix init runs.
    // The sim's PATCH endpoint at /projects/demo/issues/<id> requires
    // an If-Match header matching the current version (see
    // crates/reposix-sim/src/routes/issues.rs:413-421); each PATCH
    // bumps version by 1. We patch issue 0001 once to advance it from
    // version 1 -> version 2. The working tree's subsequent push will
    // try to send issue 0001 with prior_version=1, sim returns 409,
    // helper enters conflict-reject branch with NO refs/mirrors/*
    // populated yet.
    let patch_url = format!(
        "http://127.0.0.1:{}/projects/demo/issues/0001",
        sim.port,
    );
    let patch_resp = std::process::Command::new("curl")
        .args([
            "-sS",
            "-X", "PATCH",
            &patch_url,
            "-H", "Content-Type: application/json",
            "-H", "If-Match: 1",
            "-d", r#"{"title":"seeded-bump-for-h3"}"#,
        ])
        .output()
        .expect("curl PATCH to seed sim version > 1");
    assert!(
        patch_resp.status.success(),
        "sim PATCH failed (cannot seed version bump): stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&patch_resp.stdout),
        String::from_utf8_lossy(&patch_resp.stderr),
    );

    // Now drive the first-ever push from the working tree. The push
    // sends issue 0001 with prior_version=1; sim is at version=2 →
    // conflict-reject path fires; refs/mirrors/* are absent (this is
    // the cache's first push to this SoT).
    let push_out = drive_init_edit_push(sim.port, cache_dir.path(), work.path(), "first-push-stale");
    let stderr = String::from_utf8_lossy(&push_out.stderr);

    // Assertion 1: this IS the conflict-reject path (proves the test
    // is non-vacuous).
    assert!(
        stderr.contains("fetch first") || stderr.contains("modified on backend"),
        "expected conflict-reject stderr (fetch first / modified on backend); got: {stderr}",
    );

    // Assertion 2: synced-at hint is cleanly omitted on first push
    // (no refs/mirrors/* yet → read_mirror_synced_at returns None →
    // helper's reject hint composition skips the synced-at lines per
    // RESEARCH.md pitfall 7).
    assert!(
        !stderr.contains("synced at"),
        "first-push conflict stderr should NOT contain 'synced at' hint: {stderr}",
    );
    assert!(
        !stderr.contains("minutes ago"),
        "first-push conflict stderr should NOT contain '(N minutes ago)' rendering: {stderr}",
    );
}
```

**M1 — Declare missing dev-dependencies (concrete sub-step, BEFORE
the integration-test build).** The integration test uses
`walkdir::WalkDir` and `regex::Regex` (PATH A also adds curl
shell-out — no Rust dep). At planning time
`crates/reposix-remote/Cargo.toml` `[dev-dependencies]` carried only
`wiremock`, `assert_cmd`, `tempfile`, `tokio`, `chrono`, `reposix-sim`
— `walkdir` and `regex` are missing. Add them concretely before the
test build:

```bash
cargo add --dev --package reposix-remote walkdir regex
```

If a `[workspace.dependencies]` entry exists for either crate, the
`cargo add` invocation will pin to the workspace default automatically;
otherwise it pulls the latest crates.io semver-compatible release.
The Cargo.toml change rides into the same T04 integration-test commit.

**Note on `reject_hint_first_push_omits_synced_at_line` (H3 fix).** The
test above engineers a REAL first-push conflict by patching the sim's
issue 0001 to version 2 BEFORE the working tree's first push (which
sends prior_version=1). The conflict-reject path fires while
`refs/mirrors/*` are still absent (cache's first push to this SoT) —
this is the strong form that exercises the helper's None-case behavior
at the conflict-reject branch.

**Path B fallback (prescribed if sim-seeding is brittle).** If the
sim's PATCH-via-curl seeding at execution time proves flaky (port
not yet listening, curl missing, sim race) OR if the working tree's
init+push sequence races the sim PATCH, fall back to a unit test in
`crates/reposix-remote/src/main.rs` `#[cfg(test)] mod tests` that
exercises the conflict-reject branch directly with a stubbed cache
returning `Ok(None)` from `read_mirror_synced_at`. Path B sidesteps
the sim entirely and asserts:

```rust
// crates/reposix-remote/src/main.rs - inside #[cfg(test)] mod tests
#[test]
fn conflict_reject_omits_synced_at_hint_when_cache_returns_none() {
    // Stub a Cache that returns Ok(None) from read_mirror_synced_at;
    // drive the conflict-reject composition path directly; capture
    // the diag(...) output via a test-only StderrSink. Assert
    // absence of "synced at" and "minutes ago" tokens in the captured
    // output.
}
```

Both paths cover SC4's first-push None-case behavior at the helper
layer. The unit test `mirror_refs::tests::read_mirror_synced_at_returns_none_when_absent`
from T02 covers the cache-layer None contract independently. Pick
Path A (sim-seeded integration) as primary; document Path B in the
T04 commit message body if Path B is taken.

Build serially:

```bash
cargo check -p reposix-remote --tests
cargo nextest run -p reposix-remote --test mirror_refs
```

If any test fails: diagnose; fix-forward in `crates/reposix-cache/src/mirror_refs.rs`
or `crates/reposix-remote/src/main.rs` (bounded to what the failing
test requires). Larger-scope drifts → file as SURPRISES-INTAKE entry
per OP-8.

Stage and commit (NO push yet):

```bash
git add crates/reposix-remote/tests/mirror_refs.rs \
        crates/reposix-remote/Cargo.toml  # if dev-deps changed
git commit -m "$(cat <<'EOF'
test(remote): integration tests for mirror-lag refs (DVCS-MIRROR-REFS-01..03 behavior coverage)

- crates/reposix-remote/tests/mirror_refs.rs (new) — 4 integration tests
  - write_on_success_updates_both_refs: refs resolvable + tag message + audit row
  - vanilla_fetch_brings_mirror_refs: fresh `git clone --bare` brings refs along
  - reject_hint_cites_synced_at_with_age: conflict reject stderr cites refs/mirrors/<sot>-synced-at + RFC3339 + (N minutes ago)
  - reject_hint_first_push_omits_synced_at_line: sim-seeded first-push conflict (sim version=2, push prior=1 → conflict-reject branch fires while refs/mirrors/* absent); asserts conflict-path stderr AND clean omission of `synced at` / `minutes ago` lines per RESEARCH.md pitfall 7 (H3 fix per PLAN-CHECK.md; Path B unit-test fallback documented inline if sim-seeding brittle)

- SimSubprocess helper (mirrors P79's attach.rs pattern); tempdir isolation per test
- crates/reposix-remote/Cargo.toml [dev-dependencies] — added regex, walkdir if missing

Phase 80 / Plan 01 / Task 04 part A / DVCS-MIRROR-REFS-01..03 integration tests.
EOF
)"
```
