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
