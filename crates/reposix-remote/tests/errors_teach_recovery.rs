//! Phase 120 (P120) — per-error integration assertions that the
//! `git-remote-reposix` helper emits 3-part teaching errors on stderr.
//!
//! SCAFFOLD (W0): one ANCHOR test pins the already-teaching malformed-bus-URL
//! path (the empty-query arm of `bus_url::parse` names the canonical `?mirror=`
//! form) so the catalog-first commit has a real green baseline. Implementation
//! waves W4–W5 APPEND per-error cases — all six `bus_url` reject arms, missing
//! real-backend creds, upload-pack subprocess exit, unexpected EOF mid-request —
//! as each is retrofitted through `reposix_core::errmsg::teach` /
//! `malformed_bus_url_error`, asserting the full Fix:/Recovery: shape.
//!
//! Leaf isolation: the helper parses argv[2] (the remote URL) BEFORE any git or
//! network context, so no repo/seed is needed — the test is hermetic by
//! construction (never touches the shared repo).

use assert_cmd::Command;

/// Drive the built `git-remote-reposix` helper with `args` (env-`GITHUB_TOKEN`
/// cleared so the credential path is deterministic across dev/CI), returning
/// `(success, stderr)`. The helper parses argv/URL BEFORE any git or network
/// context, so every path exercised here is hermetic (no sim, no repo).
fn run_helper(args: &[&str]) -> (bool, String) {
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("git-remote-reposix binary built")
        .args(args)
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("run `git-remote-reposix`");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Assert a rejected invocation emits the Rust-compiler-grade 3-part shape:
/// non-zero exit + `Fix:` + an indented `Recovery:` block.
fn assert_three_part_reject(stderr: &str, success: bool, ctx: &str) {
    assert!(
        !success,
        "{ctx}: MUST be rejected (non-zero exit); stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("Fix:"),
        "{ctx}: missing `Fix:`; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("Recovery:"),
        "{ctx}: missing `Recovery:`; stderr:\n{stderr}"
    );
}

/// ANCHOR (W0 baseline, retained): a malformed bus URL using the dropped
/// `+`-delimited form (`reposix::sim+mirror`) is rejected and names the
/// canonical `reposix::<sot-spec>?mirror=<mirror-url>` form. W4 additionally
/// pins the full 3-part shape below.
#[test]
fn malformed_bus_url_names_canonical_mirror_form() {
    let (success, stderr) = run_helper(&["origin", "reposix::sim+mirror"]);
    assert!(
        !success,
        "a malformed bus URL MUST be rejected (non-zero exit)"
    );
    assert!(
        stderr.contains("?mirror=<mirror-url>"),
        "the reject must name the canonical \
         `reposix::<sot-spec>?mirror=<mirror-url>` form; got:\n{stderr}"
    );
}

/// P120 W4: EACH of the six `bus_url::parse` reject arms routes through
/// `malformed_bus_url_error`, so every malformed-bus-URL path teaches the
/// canonical form + a copy-paste recovery. git invokes the helper, so the body
/// must be legible in git's stderr — assert `?mirror=<mirror-url>` + Fix: +
/// Recovery: for all six arms.
#[test]
fn every_malformed_bus_url_arm_teaches_three_part() {
    // (label, url) — one per reject arm in `bus_url::parse`.
    let cases = [
        ("+-delimited form (arm 88)", "reposix::sim+mirror"),
        (
            "base form rejected (arm 98)",
            "reposix::http://127.0.0.1:7878/nope?mirror=file:///tmp/m.git",
        ),
        (
            "empty query (arm 106)",
            "reposix::http://127.0.0.1:7878/projects/demo?",
        ),
        (
            "unknown query param (arm 132)",
            "reposix::http://127.0.0.1:7878/projects/demo?priority=high",
        ),
        (
            "mirror= missing (arm 141)",
            "reposix::http://127.0.0.1:7878/projects/demo?&",
        ),
        (
            "mirror= empty (arm 148)",
            "reposix::http://127.0.0.1:7878/projects/demo?mirror=",
        ),
    ];
    for (label, url) in cases {
        let (success, stderr) = run_helper(&["origin", url]);
        assert_three_part_reject(&stderr, success, label);
        assert!(
            stderr.contains("?mirror=<mirror-url>"),
            "{label}: must name the canonical `reposix::<sot-spec>?mirror=<mirror-url>` \
             form; got:\n{stderr}"
        );
    }
}

/// P120 W4 (SECURITY): a malformed bus URL carrying embedded credentials must
/// have its userinfo REDACTED before it reaches stderr — echoing `user:token@`
/// would be an exfiltration leg (the URL lands in `.git/config` and helper
/// diagnostics; CLAUDE.md § Threat model). The reject still teaches.
#[test]
fn malformed_credentialed_bus_url_does_not_leak_userinfo() {
    // Creds in the SoT origin; the origin fails classification (arm 176) and the
    // bus wrapper (arm 98) echoes the offending URL — both must be redacted.
    let url = "reposix::https://x-access-token:ghp_SUPERSECRET@evil.example.com/projects/x?mirror=file:///tmp/m.git";
    let (success, stderr) = run_helper(&["origin", url]);
    assert_three_part_reject(&stderr, success, "credentialed malformed bus URL");
    assert!(
        !stderr.contains("ghp_SUPERSECRET"),
        "SECRET LEAKED to stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("x-access-token"),
        "username leaked to stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("<redacted>@evil.example.com"),
        "expected redacted host to survive; got:\n{stderr}"
    );
    // Still teaches the canonical form despite the redaction.
    assert!(
        stderr.contains("?mirror=<mirror-url>"),
        "must still teach the canonical form; got:\n{stderr}"
    );
}

/// P120 W4: `git-remote-reposix` invoked with too few args teaches that it is a
/// git remote helper (normally invoked BY git) + the `reposix init` path.
#[test]
fn too_few_args_teaches_remote_helper_usage() {
    let (success, stderr) = run_helper(&["origin"]);
    assert_three_part_reject(&stderr, success, "too few args");
    assert!(
        stderr.contains("remote helper") || stderr.contains("REMOTE HELPER"),
        "should teach that it is a git remote helper; got:\n{stderr}"
    );
    assert!(
        stderr.contains("reposix init"),
        "should point at `reposix init` as the recovery; got:\n{stderr}"
    );
}

/// P120 W4 (leverage #2, helper side): a `github::` push with `GITHUB_TOKEN`
/// unset teaches `export GITHUB_TOKEN=…` + the credential-free `sim::`
/// alternative + the env-var matrix doc. Hermetic: the missing-token check
/// fires BEFORE any HTTP client / network I/O.
#[test]
fn missing_github_token_teaches_export_and_sim_alternative() {
    let (success, stderr) = run_helper(&[
        "origin",
        "reposix::https://api.github.com/projects/owner/repo",
    ]);
    assert_three_part_reject(&stderr, success, "missing GITHUB_TOKEN");
    assert!(
        stderr.contains("export GITHUB_TOKEN="),
        "should give a copy-paste export recovery; got:\n{stderr}"
    );
    assert!(
        stderr.contains("sim::"),
        "should name the credential-free sim alternative; got:\n{stderr}"
    );
    assert!(
        stderr.contains("docs/reference/testing-targets.md"),
        "should point at the env-var matrix doc; got:\n{stderr}"
    );
}
