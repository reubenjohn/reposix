//! Pattern-C round-trip recovery proof (attach-lineage bug, item 4a §5.2).
//!
//! Drives the REAL `git-remote-reposix` helper through a REAL `git fetch`
//! (git 2.25 selects the `import` verb natively) against a wiremock SoT, in a
//! `/tmp`-isolated git leaf, then a REAL `git rebase`. Two halves, in direct
//! contrast — the exact analog of `fast_import::tests::
//! git_fast_import_roundtrip_with_parent_fast_forwards` but end-to-end through
//! the helper process + git plumbing instead of feeding `git fast-import`:
//!
//! - **Happy half (`attach_pattern_c_roundtrip_recovers`)** — with
//!   `refs/reposix/origin/main` seeded to the mirror merge-base `M` (what
//!   `reposix attach` MUST do per §3.1 once Part A lands), the fetch's
//!   synthesized snapshot chains onto `M` (`resolve_import_parent` finds the
//!   ref), git fetch fast-forwards the tracking ref, and the agent's
//!   `git rebase refs/reposix/origin/main` replays the un-pushed edit cleanly
//!   — NO `CONFLICT (add/add)`. This is GREEN today because the seed is
//!   applied by hand; it is the regression guard that stays green after Part A.
//!
//! - **Falsifier half (`attach_pattern_c_roundtrip_falsifier_omitted_seed_hits_wall`)**
//!   — OMIT the seed (models today's unfixed `reposix attach`, which never runs
//!   `git update-ref refs/reposix/origin/main`). `resolve_import_parent`
//!   returns `None` → the fetched snapshot is a PARENTLESS root
//!   (`refs/reposix/origin/main~1` does not resolve), sharing no ancestor with
//!   the agent's branch → `git rebase` hits the cross-root `add/add` wall. This
//!   proves the seed is the load-bearing anchor.
//!
//! IMPORTANT (repro-lane honesty): NEITHER half here runs the real
//! `reposix attach` binary — the seed is applied/omitted by hand. The
//! red-BEFORE-fix, green-AFTER-fix proof that today's `reposix attach` fails to
//! seed the ref lives in the CLI test
//! `crates/reposix-cli/tests/attach.rs::attach_seeds_tracking_ref_at_mirror_base`
//! (drives REAL `reposix attach`). This file proves the underlying git
//! mechanism the fix relies on; that file proves attach doesn't wire it up yet.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)] // each half reads top-to-bottom as one scenario
#![allow(clippy::doc_markdown)] // prose names refs/CONFLICT/add-add verbatim

use std::path::Path;
use std::process::Command;

use reposix_core::{Record, RecordId, RecordStatus};
use wiremock::MockServer;

mod common;
use common::seed_mock;

const PROJECT: &str = "demo";

/// Build a deterministic record with a fixed timestamp so its rendered blob —
/// and therefore the git tree/commit oids — is stable across runs.
fn record(id: u64, body: &str) -> Record {
    use chrono::TimeZone;
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    Record {
        id: RecordId(id),
        title: format!("issue {id} in {PROJECT}"),
        status: RecordStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version: 1,
        body: body.to_owned(),
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    }
}

/// Prepend the built `git-remote-reposix` helper's directory to `PATH` so a
/// spawned `git fetch` discovers it (git resolves `git-remote-<scheme>` from
/// PATH). `CARGO_BIN_EXE_git-remote-reposix` is injected by cargo for this
/// crate's integration tests.
fn path_with_helper() -> String {
    let helper = Path::new(env!("CARGO_BIN_EXE_git-remote-reposix"));
    let dir = helper.parent().expect("helper bin has a parent dir");
    let existing = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), existing)
}

/// Run `git -C <leaf> <args...>` with the isolated env every leaf op needs
/// (helper on PATH, isolated cache, no system/global git config bleed).
/// Returns the raw `Output` for the caller to assert on.
fn git_in(leaf: &Path, cache: &Path, path_env: &str, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(leaf)
        .args(args)
        .env("PATH", path_env)
        .env("REPOSIX_CACHE_DIR", cache)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        // Deterministic identity + no user hooks/rerere leaking in.
        .env("GIT_AUTHOR_NAME", "reposix-test")
        .env("GIT_AUTHOR_EMAIL", "test@reposix.invalid")
        .env("GIT_COMMITTER_NAME", "reposix-test")
        .env("GIT_COMMITTER_EMAIL", "test@reposix.invalid")
        .output()
        .unwrap_or_else(|e| panic!("spawn git {}: {e}", args.join(" ")))
}

/// `git rev-parse --verify --quiet <arg>` → `Some(oid)` when it resolves,
/// `None` when the ref/rev is absent (the parentless-root probe keys on this).
fn rev_parse(leaf: &Path, cache: &Path, path_env: &str, arg: &str) -> Option<String> {
    let out = git_in(
        leaf,
        cache,
        path_env,
        &["rev-parse", "--verify", "--quiet", arg],
    );
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!s.is_empty()).then_some(s)
}

/// Shared leaf setup: a `/tmp` git repo with `issues/1.md` at base `M` (rendered
/// byte-identically to what the helper emits, so a later fetch's snapshot tree
/// for issue 1 equals the base), the reposix remote configured with the
/// init-style `+refs/heads/*:refs/reposix/origin/*` refspec (the mechanism that
/// makes git fetch the sole writer of the tracking ref), and an agent edit to
/// `issues/1.md` committed as `M'` (Pattern-C "commit before attach"). Returns
/// `(leaf, cache, path_env, m, m_prime)`.
fn setup_leaf_with_edit(
    server: &MockServer,
) -> (tempfile::TempDir, tempfile::TempDir, String, String, String) {
    let leaf = tempfile::tempdir().expect("leaf tempdir");
    let cache = tempfile::tempdir().expect("cache tempdir");
    let path_env = path_with_helper();
    let lp = leaf.path();
    let cp = cache.path();

    assert!(
        git_in(lp, cp, &path_env, &["init", "-q"]).status.success(),
        "git init"
    );

    // Base M: issues/1.md rendered EXACTLY as the helper renders it, so the
    // fetched snapshot's issues/1.md is byte-identical to the base and the
    // agent's edit replays with no content conflict.
    let base_blob = reposix_core::frontmatter::render(&record(1, "base body 1\n"))
        .expect("render base issue 1");
    std::fs::create_dir_all(lp.join("issues")).expect("mkdir issues");
    std::fs::write(lp.join("issues/1.md"), &base_blob).expect("write base issues/1.md");
    assert!(
        git_in(lp, cp, &path_env, &["add", "-A"]).status.success(),
        "git add base"
    );
    assert!(
        git_in(lp, cp, &path_env, &["commit", "-q", "-m", "base M"])
            .status
            .success(),
        "git commit base"
    );
    let m = rev_parse(lp, cp, &path_env, "HEAD").expect("base commit M");

    // Configure the reposix remote (URL + init-style tracking refspec).
    //
    // A BUS URL (`?mirror=`) is deliberate: bus URLs omit the
    // `stateless-connect` capability (main.rs capabilities arm), so `git fetch`
    // falls through to the `import` verb on EVERY git version — not just git
    // 2.25, which selects `import` natively. Without the `?mirror=`, git 2.34+
    // would pick `stateless-connect` (the cache read path) instead, and this
    // round-trip's `resolve_import_parent` chaining would never be exercised.
    // The mirror is push-only and never contacted on fetch (the import lists
    // from the SoT), so an invalid mirror URL is fine here.
    let url = format!(
        "reposix::{}/projects/{PROJECT}?mirror=https://example.invalid/mirror.git",
        server.uri()
    );
    assert!(
        git_in(lp, cp, &path_env, &["config", "remote.reposix.url", &url])
            .status
            .success(),
        "config remote.reposix.url"
    );
    assert!(
        git_in(
            lp,
            cp,
            &path_env,
            &[
                "config",
                "remote.reposix.fetch",
                "+refs/heads/*:refs/reposix/origin/*",
            ],
        )
        .status
        .success(),
        "config remote.reposix.fetch"
    );

    // Agent edit → M' (un-pushed local edit atop the base).
    std::fs::write(
        lp.join("issues/1.md"),
        base_blob.replace("base body 1", "AGENT-EDITED body 1"),
    )
    .expect("write edited issues/1.md");
    assert!(
        git_in(lp, cp, &path_env, &["add", "-A"]).status.success(),
        "git add edit"
    );
    assert!(
        git_in(lp, cp, &path_env, &["commit", "-q", "-m", "agent edit M'"])
            .status
            .success(),
        "git commit edit"
    );
    let m_prime = rev_parse(lp, cp, &path_env, "HEAD").expect("edit commit M'");
    assert_ne!(m, m_prime, "M' must differ from base M");

    (leaf, cache, path_env, m, m_prime)
}

/// §5.2 happy half — WITH the seed, the fetch fast-forwards the tracking ref
/// (snapshot chains onto M) and the agent's rebase reconciles cleanly. GREEN
/// today (the seed is applied by hand); the regression guard for Part A.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "shells out to real git + the git-remote-reposix helper binary; requires `cargo build -p reposix-remote --bins` first (matches the attach/agent_flow integration-test convention)"]
// test-name-honesty: ok — drives REAL git fetch (import verb) through the helper
// + REAL git rebase; asserts fast-forward-with-parent and a clean (no add/add)
// Pattern-C reconciliation. Manually seeds the ref (does NOT run `reposix
// attach`) — the attach-driven red-today proof is the CLI sibling test.
async fn attach_pattern_c_roundtrip_recovers() {
    let server = MockServer::start().await;
    // SoT: issue 1 UNCHANGED at base (so its snapshot blob == base M's file),
    // plus a NEW issue 2 added out-of-band — a non-overlapping SoT move so the
    // agent's issue-1 edit replays with zero content conflict.
    seed_mock(
        &server,
        PROJECT,
        &[record(1, "base body 1\n"), record(2, "new sot record 2\n")],
    )
    .await;

    let (leaf, cache, path_env, m, m_prime) = setup_leaf_with_edit(&server);
    let lp = leaf.path();
    let cp = cache.path();

    // SEED refs/reposix/origin/main = M (the mirror merge-base; §3.1). This is
    // exactly what Part A adds to `reposix attach` — applied by hand here.
    assert!(
        git_in(
            lp,
            cp,
            &path_env,
            &["update-ref", "refs/reposix/origin/main", &m]
        )
        .status
        .success(),
        "seed refs/reposix/origin/main = M"
    );

    // REAL git fetch → helper `import` verb → chained snapshot.
    let fetch = git_in(lp, cp, &path_env, &["fetch", "reposix"]);
    assert!(
        fetch.status.success(),
        "git fetch reposix must succeed; stderr=\n{}",
        String::from_utf8_lossy(&fetch.stderr)
    );

    // Fast-forward: the tracking ref advanced past M and chains DIRECTLY onto M
    // (has a parent == M) — NOT a parentless root.
    let ff_tip =
        rev_parse(lp, cp, &path_env, "refs/reposix/origin/main").expect("tracking ref after fetch");
    assert_ne!(
        ff_tip, m,
        "the fetched snapshot must advance the tracking ref past M"
    );
    let parent = rev_parse(lp, cp, &path_env, "refs/reposix/origin/main~1");
    assert_eq!(
        parent.as_deref(),
        Some(m.as_str()),
        "the fetched snapshot MUST chain onto M (fast-forward), not be a parentless root"
    );

    // The documented Pattern-C recovery replays the agent's edit cleanly.
    let rebase = git_in(lp, cp, &path_env, &["rebase", "refs/reposix/origin/main"]);
    let rebase_out = format!(
        "{}\n{}",
        String::from_utf8_lossy(&rebase.stdout),
        String::from_utf8_lossy(&rebase.stderr)
    );
    assert!(
        rebase.status.success(),
        "git rebase onto the fetched snapshot must reconcile cleanly; output=\n{rebase_out}"
    );
    assert!(
        !rebase_out.contains("CONFLICT") && !rebase_out.contains("add/add"),
        "Pattern-C rebase must NOT hit the cross-root add/add wall; output=\n{rebase_out}"
    );

    // Agent's edit preserved; the SoT's new record 2 merged in.
    let issue1 = std::fs::read_to_string(lp.join("issues/1.md")).expect("issues/1.md after rebase");
    assert!(
        issue1.contains("AGENT-EDITED body 1"),
        "the agent's un-pushed edit must survive the rebase; got:\n{issue1}"
    );
    assert!(
        lp.join("issues/2.md").exists(),
        "the SoT's out-of-band new record must be present after reconciliation"
    );
    // Guard against the §3.1 silent-revert mode: the reconciled tip must NOT be
    // the raw M' (that would mean the edit was never replayed onto the snapshot).
    let head = rev_parse(lp, cp, &path_env, "HEAD").expect("HEAD after rebase");
    assert_ne!(
        head, m_prime,
        "HEAD must be M' REPLAYED onto the snapshot, not the un-reconciled M'"
    );

    drop(server);
}

/// §5.2 falsifier half — OMIT the seed (models today's unfixed `reposix
/// attach`). The fetched snapshot is a PARENTLESS root and the agent's rebase
/// hits the cross-root wall. Proves the seed is what makes recovery possible.
/// GREEN today AND after Part A (a raw seedless fetch is unchanged by the fix);
/// a permanent control, NOT the red-before-fix proof.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "shells out to real git + the git-remote-reposix helper binary; requires `cargo build -p reposix-remote --bins` first (matches the attach/agent_flow integration-test convention)"]
// test-name-honesty: ok — drives REAL git fetch + rebase with the seed OMITTED;
// asserts the fetched root is parentless (refs/reposix/origin/main~1 absent) and
// the rebase fails to reconcile. The falsifier control for the happy half.
async fn attach_pattern_c_roundtrip_falsifier_omitted_seed_hits_wall() {
    let server = MockServer::start().await;
    // SoT issue 1 has DIVERGED from the base (different content). This content
    // divergence is load-bearing: the cross-root add/add wall only bites when
    // the parentless root's `issues/1.md` CONTENT differs from the agent
    // branch's — git dedups byte-IDENTICAL adds even across unrelated histories
    // (a real nuance; see NOTICED in the item-4a report). A real backend
    // virtually always diverges from a hand-authored local file, so this is the
    // realistic case — and exactly what the CLI attach test hit (sim content vs
    // the local base file).
    seed_mock(&server, PROJECT, &[record(1, "SOT-DIVERGED body 1\n")]).await;

    let (leaf, cache, path_env, _m, _m_prime) = setup_leaf_with_edit(&server);
    let lp = leaf.path();
    let cp = cache.path();

    // NO seed. `resolve_import_parent` finds no refs/reposix/origin/main → None.

    let fetch = git_in(lp, cp, &path_env, &["fetch", "reposix"]);
    assert!(
        fetch.status.success(),
        "git fetch reposix should still succeed (it just seeds a parentless root); stderr=\n{}",
        String::from_utf8_lossy(&fetch.stderr)
    );

    // The fetched snapshot exists but is a PARENTLESS root — its `~1` ancestor
    // does not resolve. This is the exact bug shape §1 describes.
    let tip = rev_parse(lp, cp, &path_env, "refs/reposix/origin/main")
        .expect("seedless fetch still writes a tracking tip (a parentless root)");
    let parent = rev_parse(lp, cp, &path_env, "refs/reposix/origin/main~1");
    assert!(
        parent.is_none(),
        "BUG: without the seed the fetched snapshot ({tip}) MUST be a parentless root \
         (refs/reposix/origin/main~1 must not resolve); got parent={parent:?}"
    );

    // The documented Pattern-C recovery cannot complete: the agent's branch and
    // the parentless root share no ancestor → cross-root add/add wall.
    let rebase = git_in(lp, cp, &path_env, &["rebase", "refs/reposix/origin/main"]);
    let rebase_out = format!(
        "{}\n{}",
        String::from_utf8_lossy(&rebase.stdout),
        String::from_utf8_lossy(&rebase.stderr)
    );
    // Leave no rebase-in-progress behind (best effort; tempdir is discarded anyway).
    let _ = git_in(lp, cp, &path_env, &["rebase", "--abort"]);
    assert!(
        !rebase.status.success(),
        "BUG: the parentless-root rebase must NOT cleanly reconcile; output=\n{rebase_out}"
    );
    assert!(
        rebase_out.contains("CONFLICT")
            || rebase_out.contains("add/add")
            || rebase_out.contains("unrelated"),
        "BUG: the parentless-root rebase must report the cross-root wall \
         (CONFLICT / add/add / unrelated histories); output=\n{rebase_out}"
    );

    drop(server);
}
