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

/// ANCHOR (W0 baseline): a malformed bus URL using the dropped `+`-delimited
/// form (`reposix::sim+mirror`) is rejected by `bus_url::parse`, and the error
/// names the canonical `reposix::<sot-spec>?mirror=<mirror-url>` form. This arm
/// fires on `base.contains('+')` — BEFORE the base-form parse and BEFORE any
/// network/instantiate — so driving the built binary is fully hermetic (no sim,
/// no valid backend URL required).
///
/// Impl wave W4 routes all six `bus_url` reject arms (this one included) through
/// `malformed_bus_url_error`; a follow-up assertion there pins the full
/// Fix:/Recovery: teaching shape. Today this anchor proves the canonical form is
/// already surfaced (no regression while the retrofit lands).
#[test]
fn malformed_bus_url_names_canonical_mirror_form() {
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("git-remote-reposix binary built")
        .args(["origin", "reposix::sim+mirror"])
        .output()
        .expect("run `git-remote-reposix`");

    assert!(
        !out.status.success(),
        "a malformed bus URL MUST be rejected (non-zero exit)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("?mirror=<mirror-url>"),
        "the reject must name the canonical \
         `reposix::<sot-spec>?mirror=<mirror-url>` form; got:\n{stderr}"
    );
}
