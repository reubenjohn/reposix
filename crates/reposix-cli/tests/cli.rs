//! CLI surface tests: `reposix --help` lists every subcommand.

#[test]
fn help_lists_all_subcommands() {
    use assert_cmd::Command;
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    // v0.9.0: `mount` and `demo` removed; `init` is the canonical entry point.
    for sub in ["sim", "init", "list", "version"] {
        assert!(s.contains(sub), "help missing {sub}: {s}");
    }
    // mount/demo must NOT appear — they were deleted in v0.9.0.
    for removed in ["mount", "demo"] {
        assert!(
            !s.contains(removed),
            "help should not list removed subcommand `{removed}`: {s}"
        );
    }
}

#[test]
fn subcommand_help_renders() {
    use assert_cmd::Command;
    for sub in ["sim", "init", "list"] {
        let out = Command::cargo_bin("reposix")
            .unwrap()
            .arg(sub)
            .arg("--help")
            .output()
            .unwrap();
        assert!(out.status.success(), "{sub} --help failed: {out:?}");
    }
}

/// v0.9.0 breaking change: `reposix mount` was removed entirely (no stub).
/// Running it must exit non-zero with clap's "unrecognized subcommand"
/// error so stale CI scripts surface the change loudly.
#[test]
fn mount_subcommand_is_removed() {
    use assert_cmd::Command;
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .args(["mount", "/tmp/nowhere"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "mount should error out (subcommand removed), got success: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `reposix init --help` must document the `<backend>::<project>` spec
/// argument so a fresh agent reading help text learns the form without
/// in-context training.
#[test]
fn init_help_documents_spec_argument() {
    use assert_cmd::Command;
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .args(["init", "--help"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "init --help failed: status={:?} stderr={:?}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The spec form must appear so a help-reading agent can copy the
    // pattern without reading source.
    assert!(
        stdout.contains("BACKEND::PROJECT") || stdout.contains("backend>::<project"),
        "init --help missing `<backend>::<project>` spec hint: {stdout}"
    );
    // Mention each supported backend so the agent learns the four options.
    for backend in ["sim", "github", "confluence", "jira"] {
        assert!(
            stdout.contains(backend),
            "init --help missing backend `{backend}`: {stdout}"
        );
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

// `demo_exits_zero_within_30s` removed in v0.9.0 — the FUSE-backed `reposix
// demo` subcommand was deleted alongside `crates/reposix-fuse/`. The
// dark-factory regression replaces it: `scripts/dark-factory-test.sh sim`
// + `crates/reposix-cli/tests/agent_flow.rs`.
