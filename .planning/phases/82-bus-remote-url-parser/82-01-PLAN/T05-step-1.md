← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T05 — 4 integration tests (bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b)

<read_first>
- `crates/reposix-remote/tests/perf_l1.rs` (P81 wiremock fixture
  donor pattern; ~250-300 lines including helpers).
- `crates/reposix-remote/tests/mirror_refs.rs` (P80 helper-driver
  donor pattern: `drive_helper_export`, `render_with_overrides`,
  `sample_issue`, `one_file_export`).
- `scripts/dark-factory-test.sh` (file:// bare-repo fixture donor
  pattern — RESEARCH.md Test Fixture Strategy option (a)).
- `crates/reposix-remote/Cargo.toml` `[dev-dependencies]` —
  `wiremock`, `assert_cmd`, `tempfile` already present.
- `crates/reposix-cache/tests/common/mod.rs` — authoritative wiremock
  helper (`sample_issues`, `seed_mock`, `sim_backend`, `CacheDirGuard`).
  Cargo's test harness does NOT share `mod common;` across crates;
  step 5a-prime copies this file unconditionally to
  `crates/reposix-remote/tests/common.rs` BEFORE step 5d's import
  reaches `cargo check`. The copy is the literal first sub-step of T05.
- `crates/reposix-remote/src/main.rs::handle_export` (post-T04 state
  — confirm bus dispatch is wired and capability branching is in
  place).
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md`
  § Test Fixture Strategy (option a — two local bare repos for
  PRECHECK A; wiremock for PRECHECK B).
</read_first>

<action>
Four concerns: write four test files. Order: bus_url → bus_capabilities
→ bus_precheck_a → bus_precheck_b → cargo nextest + commit.

The four test files share ONE common helper module
(`crates/reposix-remote/tests/common.rs`). Step 5a-prime below copies
it unconditionally from `crates/reposix-cache/tests/common/mod.rs` as
the literal first sub-step (M3 hard-block from P81 plan-check —
cargo's test harness does NOT share `mod common;` across crates).

### 5a-prime. Copy `tests/common.rs` from `reposix-cache` (HARD-BLOCK)

Per P81 plan-check M3: cargo's test harness does NOT share `mod common;`
across crates. The wiremock helpers (`sample_issues`, `seed_mock`,
`sim_backend`, `CacheDirGuard`) live ONLY in
`crates/reposix-cache/tests/common/mod.rs`. Step 5d below imports
`common::{sample_issues, seed_mock, sim_backend, CacheDirGuard}` — if
this copy is skipped, `cargo nextest run --test bus_precheck_b` fails
to compile.

This step is UNCONDITIONAL — confirm-then-copy, do NOT short-circuit
on "maybe P81 already did this". P81's M3 is a documented gap (the
copy was scoped out of P81 plan-time) and v0.13.0 has not yet added
it to `reposix-remote/tests/`. Run the copy verbatim:

```bash
# Sanity: confirm the source exists and the destination does NOT.
test -f crates/reposix-cache/tests/common/mod.rs || {
    echo "FATAL: source common/mod.rs missing; cannot proceed with P82 T05"
    exit 1
}
if test -f crates/reposix-remote/tests/common.rs; then
    echo "WARN: crates/reposix-remote/tests/common.rs already exists; skipping copy"
else
    cp crates/reposix-cache/tests/common/mod.rs crates/reposix-remote/tests/common.rs
fi

cargo check -p reposix-remote --tests
```

The `cargo check -p reposix-remote --tests` MUST exit 0 — it confirms
(a) the copied file compiles in the new crate, (b) no `pub use` shape
broke the import graph, (c) `reposix-cache` and `reposix-core` are
already in `reposix-remote`'s `[dev-dependencies]` (verified during
P81). If any of these fail, fix BEFORE proceeding to step 5a.

```bash
git add crates/reposix-remote/tests/common.rs
git commit -m "$(cat <<'EOF'
test(remote): copy tests/common.rs from reposix-cache (P81 M3 gap)

Cargo's test harness does NOT share `mod common;` across crates. The
wiremock helpers (`sample_issues`, `seed_mock`, `sim_backend`,
`CacheDirGuard`) lived only in `crates/reposix-cache/tests/common/mod.rs`
post-P81; P82 T05's `tests/bus_precheck_b.rs` imports them via
`mod common; use common::{...};`. Without this copy the integration
test would fail to compile (P81 plan-check M3 hard-block, carried
into P82 T05).

Phase 82 / Plan 01 / Task 05 / step 5a-prime / DVCS-BUS-PRECHECK-02 (substrate).
EOF
)"
```

### 5a. `crates/reposix-remote/tests/bus_url.rs`

Author the new file. Tests the bus URL parser via `assert_cmd`-driven
helper invocation (the parser itself is also unit-tested in T02
inline; this file exercises the helper-end-to-end shape).

```rust
//! Integration tests for bus URL parser via the helper binary
//! (DVCS-BUS-URL-01).
//!
//! Asserts the helper's `parse remote url` failure path emits
//! verbatim error messages for the rejected forms; the success
//! path is tested by bus_capabilities.rs and bus_precheck_*.rs
//! which exercise the helper end-to-end.

#![allow(clippy::missing_panics_doc)]

use assert_cmd::Command;

#[test]
fn parses_query_param_form_round_trip() {
    // POSITIVE capability-advertise assertion (HIGH-1 fix from P82
    // plan-check). The original negative assertion
    // `!stderr.contains("parse remote url")` passed if the helper
    // errored at ANY later stage with a different message — masking
    // bugs. We assert the helper REACHED the capabilities arm and
    // emitted the expected lines to stdout.
    //
    // Pattern matches `tests/bus_capabilities.rs::bus_url_omits_stateless_connect`
    // (and P80's `tests/stateless_connect.rs::capabilities_advertises_*`):
    // write `capabilities\n\n` on stdin → helper advertises
    // capabilities → next read_line returns Some("") (continue) →
    // EOF → helper exits cleanly with code 0.
    //
    // The bus URL points at port 9 (closed) but `instantiate_sim` is
    // a no-network constructor (`crates/reposix-remote/src/backend_dispatch.rs:228-232`),
    // so the helper reaches the dispatch loop and the capabilities
    // arm fires regardless of SoT availability.
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo?mirror=file:///tmp/m.git"])
        .write_stdin("capabilities\n\n")
        .output()
        .expect("run helper");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("import") && stdout.contains("export"),
        "expected helper to advertise capabilities; stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn rejects_plus_delimited_bus_url() {
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo+file:///tmp/m.git"])
        .write_stdin("\n")
        .output()
        .expect("run helper");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("`+`-delimited") && stderr.contains("?mirror="),
        "expected stderr to reject `+` form and suggest `?mirror=`; got: {stderr}"
    );
    assert!(!out.status.success(), "expected helper to exit non-zero");
}

#[test]
fn rejects_unknown_query_param() {
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo?priority=high"])
        .write_stdin("\n")
        .output()
        .expect("run helper");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown query parameter") && stderr.contains("priority"),
        "expected stderr to name the unknown key; got: {stderr}"
    );
    assert!(!out.status.success());
}
```

### 5b. `crates/reposix-remote/tests/bus_capabilities.rs`

```rust
//! Integration test: bus URL omits `stateless-connect` from
//! capabilities (DVCS-BUS-FETCH-01 / Q3.4).

#![allow(clippy::missing_panics_doc)]

use assert_cmd::Command;

#[test]
fn bus_url_omits_stateless_connect() {
    // Bus URL — `stateless-connect` MUST be absent.
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo?mirror=file:///tmp/m.git"])
        .write_stdin("capabilities\n\n")
        .output()
        .expect("run helper");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("import"), "expected `import`; got: {stdout}");
    assert!(stdout.contains("export"), "expected `export`; got: {stdout}");
    assert!(stdout.contains("refspec refs/heads/*:refs/reposix/*"), "expected `refspec`; got: {stdout}");
    assert!(stdout.contains("object-format=sha1"), "expected `object-format=sha1`; got: {stdout}");
    assert!(
        !stdout.contains("stateless-connect"),
        "bus URL MUST NOT advertise stateless-connect (DVCS-BUS-FETCH-01); got: {stdout}"
    );
}

#[test]
fn single_backend_url_advertises_stateless_connect() {
    // Regression check: bare `reposix::<sot>` (no `?mirror=`) DOES
    // advertise `stateless-connect`. Without this guard, an off-by-
    // one in capability branching would silently break single-backend
    // fetch (DVCS-DARKFACTORY-* would catch it eventually but the
    // signal is much faster here).
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo"])
        .write_stdin("capabilities\n\n")
        .output()
        .expect("run helper");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("stateless-connect"),
        "single-backend URL MUST advertise stateless-connect; got: {stdout}"
    );
}
```

Continue to [T05 step 2](./T05-step-2.md) for `bus_precheck_a.rs` (step 5c).
</action>
