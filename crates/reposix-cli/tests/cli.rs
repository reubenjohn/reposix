//! CLI surface tests: `reposix --help` lists every subcommand.

use std::path::{Path, PathBuf};

/// Resolve the workspace root from `CARGO_MANIFEST_DIR` (which points at
/// `crates/reposix-cli`). Mirrors the identical helper in
/// `agent_flow.rs` / `attach.rs` — kept local (not shared via a common
/// module) since each `tests/*.rs` file compiles as its own binary.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

/// docs/reference/cli.md/subcommands_exist — `reposix --help` must list
/// every one of the 15 subcommands (clap `Cmd` enum,
/// `crates/reposix-cli/src/main.rs:39-343`). Previously asserted only
/// 4/15 (sim, init, list, version) — a false-BOUND catch (R2 § F): the
/// doc-alignment claim text itself also enumerated only 13 names, missing
/// `attach` and `sync` (both shipped + documented at cli.md:14,19). Fixed
/// at rebind (2026-07-04); this test now grounds the full-15 claim.
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
    // Full 15-subcommand set per main.rs `Cmd` enum + cli.md's own
    // Commands block (cli.md:5-29).
    for sub in [
        "sim", "init", "attach", "list", "refresh", "spaces", "sync", "doctor", "history", "log",
        "at", "gc", "tokens", "cost", "version",
    ] {
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

/// docs/reference/cli.md/env_vars — every env var documented at
/// cli.md:334-343 has a real consuming call site somewhere in the CLI's
/// own source (not merely named in a doc comment). Static-grep coverage:
/// this proves the wiring exists, not that the runtime read path behaves
/// correctly under a given value (that's covered by the backend-specific
/// contract tests, e.g. `reposix-confluence/tests/contract.rs`,
/// `reposix-jira/tests/contract.rs`).
#[test]
fn env_vars_are_consumed_by_binary() {
    let root = workspace_root();
    // (var name, source file with a real `std::env::var("<VAR>")` call site)
    let cases: &[(&str, &str)] = &[
        (
            "REPOSIX_ALLOWED_ORIGINS",
            "crates/reposix-cli/src/doctor.rs",
        ),
        (
            "REPOSIX_CONFLUENCE_TENANT",
            "crates/reposix-cli/src/init.rs",
        ),
        ("REPOSIX_JIRA_INSTANCE", "crates/reposix-cli/src/init.rs"),
        ("GITHUB_TOKEN", "crates/reposix-cli/src/list.rs"),
        ("ATLASSIAN_EMAIL", "crates/reposix-cli/src/refresh.rs"),
        ("ATLASSIAN_API_KEY", "crates/reposix-cli/src/refresh.rs"),
    ];
    for (var, rel_path) in cases {
        let src = std::fs::read_to_string(root.join(rel_path))
            .unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
        let needle = format!("std::env::var(\"{var}\")");
        assert!(
            src.contains(&needle),
            "{var}: expected `{needle}` consuming call site in {rel_path}, not found"
        );
    }
    // RUST_LOG is consumed implicitly: `tracing_subscriber`'s
    // `EnvFilter::try_from_default_env()` reads `RUST_LOG` by the crate's
    // own documented convention rather than a literal
    // `env::var("RUST_LOG")` call, so we assert the API call site exists
    // instead of the literal var name.
    let main_src =
        std::fs::read_to_string(root.join("crates/reposix-cli/src/main.rs")).expect("read main.rs");
    assert!(
        main_src.contains("EnvFilter::try_from_default_env"),
        "RUST_LOG: expected EnvFilter::try_from_default_env() call site in main.rs, not found"
    );
}

/// docs/reference/cli.md/exit_codes — drives the `reposix` binary to all
/// three codes cli.md's own "Exit codes" table (cli.md:345-351)
/// documents: 0 (success), 1 (expected/handled failure — anyhow `bail!`
/// propagation), 2 (malformed invocation — clap's own usage-error layer,
/// which runs BEFORE any subcommand handler and is a distinct mechanism
/// from the anyhow-propagation layer that produces code 1).
#[test]
fn exit_codes_match_documented_contract() {
    use assert_cmd::Command;

    let out = Command::cargo_bin("reposix")
        .unwrap()
        .arg("version")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "`reposix version` should exit 0: {out:?}"
    );

    // Expected/handled failure: `spaces` rejects non-Confluence backends
    // via `anyhow::bail!` (crates/reposix-cli/src/spaces.rs) — no
    // network attempted.
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .args(["spaces", "--backend", "sim"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "`reposix spaces --backend sim` should exit 1: {out:?}"
    );

    // Malformed invocation: `init` requires two positional args
    // (`<BACKEND::PROJECT> <PATH>`); omitting both is a clap usage
    // error, which clap itself resolves to exit 2 before our handler
    // code ever runs.
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .arg("init")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "malformed `reposix init` (missing required args) should exit 2: {out:?}"
    );
}

/// docs/reference/cli.md/spaces_confluence_only — `reposix spaces`
/// rejects every backend except Confluence, pre-egress (no network
/// attempted — `spaces::run`'s `ListBackend::Github` arm bails before
/// `read_confluence_env`/`ConfluenceBackend` are ever reached).
#[test]
fn spaces_rejects_non_confluence_backend() {
    use assert_cmd::Command;
    let out = Command::cargo_bin("reposix")
        .unwrap()
        .args(["spaces", "--backend", "github"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "spaces --backend github should exit non-zero: {out:?}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.to_lowercase().contains("confluence"),
        "spaces --backend github stderr should name confluence as the supported backend: {stderr}"
    );
}

/// docs/decisions/009-stability-commitment/exit-codes-locked — CLI arm.
/// Pins the `reposix` binary's exact locked exit-code set: {0, 1, 2}.
/// The `git-remote-reposix` helper's {0, 1, 2} arm lives in
/// `crates/reposix-remote/tests/exit_codes.rs::exit_codes_locked_reposix_and_helper`
/// (same fn name, sibling crate — the two arms of one claim).
#[test]
fn exit_codes_locked_reposix_and_helper() {
    use assert_cmd::Command;

    let table: &[(&[&str], i32)] = &[
        (&["version"], 0),
        (&["spaces", "--backend", "github"], 1),
        // Unrecognized subcommand — clap's usage-error layer, exit 2.
        (&["mount", "/tmp/nowhere"], 2),
    ];
    for (args, expected) in table {
        let out = Command::cargo_bin("reposix")
            .unwrap()
            .args(*args)
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(*expected),
            "reposix {args:?} should exit {expected}: {out:?}"
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
/// listed them, but the sim runner never received the values. (The runner
/// is now the in-process `run_sim` helper in `main.rs`; the flags still
/// flow through the same destructure this test guards.)
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
// dark-factory regression replaces it: `quality/gates/agent-ux/dark-factory.sh sim`
// + `crates/reposix-cli/tests/agent_flow.rs`.
