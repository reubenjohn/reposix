//! Regression: `Cache::open` must succeed when the process carries git's
//! injected repo-context env vars (`GIT_DIR` et al).
//!
//! `git push` spawns git-remote-reposix with `GIT_DIR` set — typically the
//! RELATIVE `.git` of the user's working tree. `Cache::open`'s
//! `transfer.hideRefs` shell-out (`git -C <bare-cache> config …`) inherited
//! that env, git resolved `.git` against the bare cache path, and died with
//! "fatal: not in a git directory". The bus push path swallowed the error
//! (`let _ = ensure_cache(state)`), silently disabling ALL push-side OP-3
//! bookkeeping: no `helper_push_*` audit rows, no `refs/mirrors/*`
//! advancement, no token_cost ledger. Found by the P91 milestone-close
//! vision litmus against real TokenWorld (transcript
//! 2026-07-04T21-36-37Z: PUT 200 landed but zero cache-side rows).
//!
//! This test is its own integration binary so the `GIT_DIR` set_var cannot
//! race sibling tests; `CacheDirGuard` serializes the cache-dir var as
//! usual.

mod common;

use common::CacheDirGuard;
use reposix_cache::Cache;
use tempfile::tempdir;
use wiremock::MockServer;

#[tokio::test]
async fn open_succeeds_with_gitdir_env_pointing_elsewhere() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());
    let server = MockServer::start().await;

    // Simulate the env git gives a remote helper: a RELATIVE GIT_DIR that
    // does NOT resolve to anything under the bare cache path.
    let prev_git_dir = std::env::var("GIT_DIR").ok();
    let prev_work_tree = std::env::var("GIT_WORK_TREE").ok();
    // SAFETY (house style, cf. path.rs tests): single-test binary; values
    // restored below.
    std::env::set_var("GIT_DIR", ".git");
    std::env::set_var("GIT_WORK_TREE", ".");

    let result = Cache::open(common::sim_backend(&server), "sim", "gitdir-env");

    // Restore BEFORE asserting so a failure can't leak env into any
    // hypothetical future test in this binary.
    match prev_git_dir {
        Some(v) => std::env::set_var("GIT_DIR", v),
        None => std::env::remove_var("GIT_DIR"),
    }
    match prev_work_tree {
        Some(v) => std::env::set_var("GIT_WORK_TREE", v),
        None => std::env::remove_var("GIT_WORK_TREE"),
    }

    let cache = result.expect(
        "Cache::open must scrub inherited GIT_* repo-context env before its \
         `git config` shell-out (helper-under-git-push regression)",
    );

    // The hideRefs write must have actually landed in the BARE CACHE repo's
    // config — proving the shell-out operated on the right repository.
    let config = std::fs::read_to_string(cache.repo_path().join("config")).unwrap();
    assert!(
        config.contains("refs/reposix/sync/"),
        "transfer.hideRefs must be recorded in the bare cache config; got:\n{config}"
    );
}
