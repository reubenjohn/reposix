//! CLI surface tests: `reposix --help` lists every subcommand.
//!
//! Task 2 appends `demo_exits_zero_within_30s` as `#[ignore]`-gated.

#[test]
fn help_lists_all_subcommands() {
    use assert_cmd::Command;
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    for sub in ["sim", "mount", "demo", "list", "version"] {
        assert!(s.contains(sub), "help missing {sub}: {s}");
    }
}

#[test]
fn subcommand_help_renders() {
    use assert_cmd::Command;
    for sub in ["sim", "mount", "demo", "list"] {
        let out = Command::cargo_bin("reposix")
            .unwrap()
            .arg(sub)
            .arg("--help")
            .output()
            .unwrap();
        assert!(out.status.success(), "{sub} --help failed: {out:?}");
    }
}

/// `reposix list --help` must succeed and mention the three flags the
/// subcommand exposes. Ensures the subcommand is reachable from clap's
/// dispatcher and its args are defined as expected.
#[test]
fn list_help_succeeds_and_documents_flags() {
    use assert_cmd::Command;
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .args(["list", "--help"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "list --help failed: status={:?} stderr={:?}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    for flag in ["--project", "--origin", "--format"] {
        assert!(
            stdout.contains(flag),
            "list --help missing {flag}: {stdout}"
        );
    }
}

/// Regression for H-02 (review 2026-04-13): `reposix sim --no-seed` and
/// `--rate-limit` were silently dropped in the `Cmd::Sim` match arm
/// (`no_seed: _, rate_limit: _`). Clap accepted them and `--help`
/// listed them, but `sim::run` never received the values.
///
/// We can't easily prove rate-limit semantics without spinning up the
/// sim end-to-end (see `crates/reposix-sim/tests/api.rs` for that),
/// but we can prove that clap parses both flags together with `--help`
/// without erroring out — i.e. that they are still defined on the
/// subcommand. A regression that re-`_`s the destructure would still
/// pass *this* check (it's a clap-level test), but combined with the
/// `Cmd::Sim` match-arm now naming both fields, the compiler will
/// reject the `_` reintroduction (unused-variable warning under
/// `-D warnings`). Together: defense in depth against the same regression.
#[test]
fn sim_accepts_no_seed_and_rate_limit_flags() {
    use assert_cmd::Command;
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .args(["sim", "--rate-limit", "7", "--no-seed", "--help"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "sim --rate-limit 7 --no-seed --help failed: status={:?} stderr={:?}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Spot-check both flags are documented.
    assert!(
        stdout.contains("--rate-limit"),
        "sim --help missing --rate-limit: {stdout}"
    );
    assert!(
        stdout.contains("--no-seed"),
        "sim --help missing --no-seed: {stdout}"
    );
}

/// Full end-to-end: spawn sim → mount → ls/cat/grep → audit tail → exit 0.
/// Gated `#[ignore]` so default `cargo test` stays fast and doesn't require
/// fusermount3 on dev machines that lack it. CI's integration job runs it.
#[test]
#[ignore = "requires fusermount3 + a built reposix-sim + reposix-fuse binary"]
fn demo_exits_zero_within_30s() {
    // Build the `reposix` binary path from `current_exe` (test harness
    // lives at `target/<profile>/deps/cli-<hash>`, so grandparent is
    // `target/<profile>/reposix`). Using a plain `std::process::Command`
    // with inherited stdio avoids `assert_cmd`'s pipe-capture behavior,
    // which can deadlock the FUSE child when its stderr inherits
    // through a pipe nothing drains.
    let exe = std::env::current_exe().expect("current_exe");
    let binary = exe
        .parent()
        .and_then(|deps| deps.parent())
        .expect("binary dir")
        .join("reposix");
    assert!(binary.exists(), "reposix binary not at {binary:?}");

    // `cargo test` sets CWD to the crate being tested. The demo hardcodes
    // relative paths (`runtime/`, `crates/reposix-sim/fixtures/seed.json`),
    // so we need to chdir to the workspace root first.
    let workspace_root = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::PathBuf::from(workspace_root)
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf();
    eprintln!("test cwd -> workspace_root={workspace_root:?}");

    let t0 = std::time::Instant::now();
    let mut child = std::process::Command::new(&binary)
        .arg("demo")
        .current_dir(&workspace_root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("spawn demo");

    // Wait up to 30s for the demo to exit.
    let budget = std::time::Duration::from_secs(30);
    loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => {
                let elapsed = t0.elapsed();
                assert!(
                    status.success(),
                    "demo exited with {status:?} in {elapsed:?}"
                );
                assert!(elapsed < budget, "demo took {elapsed:?}");
                return;
            }
            None => {
                if t0.elapsed() >= budget {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("demo did not exit within {budget:?}");
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
