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
    for sub in ["sim", "mount", "demo", "version"] {
        assert!(s.contains(sub), "help missing {sub}: {s}");
    }
}

#[test]
fn subcommand_help_renders() {
    use assert_cmd::Command;
    for sub in ["sim", "mount", "demo"] {
        let out = Command::cargo_bin("reposix")
            .unwrap()
            .arg(sub)
            .arg("--help")
            .output()
            .unwrap();
        assert!(out.status.success(), "{sub} --help failed: {out:?}");
    }
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
