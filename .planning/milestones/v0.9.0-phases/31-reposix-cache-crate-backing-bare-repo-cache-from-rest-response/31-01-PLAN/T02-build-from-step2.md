← [back to index](./index.md)

# Task 2: Implement `Cache::build_from` — tree construction with lazy blobs (Steps 5–8 + acceptance)

Continued from [Steps 1–4](./T02-build-from-step1.md).

    Step 5 — Update `crates/reposix-cache/src/lib.rs` to wire the modules:
    ```rust
    // (keep the existing doc comment + attributes from Task 1)

    pub mod builder;
    pub mod cache;
    pub mod error;
    pub mod path;

    pub use cache::Cache;
    pub use error::{Error, Result};
    pub use path::{resolve_cache_path, CACHE_DIR_ENV};
    ```

    Step 6 — Create `crates/reposix-cache/tests/tree_contains_all_issues.rs`:
    ```rust
    //! ARCH-01: Cache::build_from produces a tree with one entry per seeded issue.

    use std::sync::Arc;

    use reposix_cache::Cache;
    use reposix_core::issue::{Issue, IssueId, IssueStatus};
    use reposix_core::BackendConnector;
    use reposix_sim::SimBackend;
    use tempfile::tempdir;

    #[tokio::test]
    async fn tree_contains_all_seeded_issues() {
        let tmp = tempdir().unwrap();
        // Point the cache at tmp via env var — deterministic path inside test sandbox.
        let prev = std::env::var(reposix_cache::CACHE_DIR_ENV).ok();
        std::env::set_var(reposix_cache::CACHE_DIR_ENV, tmp.path());

        // Seed sim with 10 issues (use the sim's documented seeder; if it
        // does not exist, construct SimBackend directly and insert via its
        // public trait methods — see crates/reposix-sim/src/lib.rs for the
        // exact seed helper or a mock harness).
        let sim = Arc::new(SimBackend::new_seeded("proj-1", 10));
        let cache = Cache::open(sim.clone(), "sim", "proj-1").unwrap();
        let _commit_oid = cache.build_from().await.unwrap();

        // Walk the tree at refs/heads/main and count blob entries.
        let head = cache
            .repo_path();
        let repo = gix::open(head).expect("open bare");
        let commit = repo
            .find_reference("refs/heads/main").unwrap()
            .peel_to_commit().unwrap();
        let tree = commit.tree().unwrap();
        let mut count = 0_usize;
        let mut saw_expected_path = false;
        for entry in tree.iter() {
            let entry = entry.unwrap();
            // The tree has one directory `issues/` — descend.
            if entry.filename() == "issues" {
                let sub = entry.object().unwrap().try_into_tree().unwrap();
                for child in sub.iter() {
                    let child = child.unwrap();
                    count += 1;
                    let name = child.filename().to_string();
                    if name == "1.md" { saw_expected_path = true; }
                }
            }
        }
        assert_eq!(count, 10, "expected 10 blob entries in the tree");
        assert!(saw_expected_path, "expected issues/1.md to exist in tree");

        match prev {
            Some(v) => std::env::set_var(reposix_cache::CACHE_DIR_ENV, v),
            None => std::env::remove_var(reposix_cache::CACHE_DIR_ENV),
        }
    }
    ```

    If `SimBackend::new_seeded` does not exist, inspect `crates/reposix-sim/src/lib.rs` (the executor has it in `<read_first>`) and use whichever public constructor + seeder IS present — the simulator already has tests that seed N issues, lift that pattern.

    Step 7 — Create `crates/reposix-cache/tests/blobs_are_lazy.rs`:
    ```rust
    //! ARCH-01: after build_from, no blob objects exist in .git/objects.
    //! The tree references blob OIDs, but the objects themselves are only
    //! persisted via Cache::read_blob (Plan 02).

    use std::sync::Arc;

    use reposix_cache::Cache;
    use reposix_sim::SimBackend;
    use tempfile::tempdir;

    #[tokio::test]
    async fn no_blob_objects_after_build_from() {
        let tmp = tempdir().unwrap();
        let prev = std::env::var(reposix_cache::CACHE_DIR_ENV).ok();
        std::env::set_var(reposix_cache::CACHE_DIR_ENV, tmp.path());

        let sim = Arc::new(SimBackend::new_seeded("proj-1", 5));
        let cache = Cache::open(sim.clone(), "sim", "proj-1").unwrap();
        let _ = cache.build_from().await.unwrap();

        // Walk .git/objects/ and assert every object we find has Kind::Tree
        // or Kind::Commit, never Kind::Blob.
        let repo = gix::open(cache.repo_path()).unwrap();
        let mut blob_count = 0_usize;
        let mut tree_count = 0_usize;
        let mut commit_count = 0_usize;
        // gix exposes an iterator over the loose-object store; adjust the
        // call to the actual 0.82 name if different (`objects_db.loose`...).
        for oid_res in repo.objects.iter().unwrap() {
            let oid = oid_res.unwrap();
            let header = repo.objects.header(oid).unwrap();
            match header.kind {
                gix::object::Kind::Blob => blob_count += 1,
                gix::object::Kind::Tree => tree_count += 1,
                gix::object::Kind::Commit => commit_count += 1,
                gix::object::Kind::Tag => {}
            }
        }
        assert_eq!(blob_count, 0, "expected zero blobs after lazy build_from; found {}", blob_count);
        assert!(tree_count >= 1, "expected at least one tree object");
        assert_eq!(commit_count, 1, "expected exactly one commit");

        match prev {
            Some(v) => std::env::set_var(reposix_cache::CACHE_DIR_ENV, v),
            None => std::env::remove_var(reposix_cache::CACHE_DIR_ENV),
        }
    }
    ```

    Step 8 — Run the full crate verification:
    ```bash
    cargo check -p reposix-cache
    cargo clippy -p reposix-cache --all-targets -- -D warnings
    cargo test -p reposix-cache
    ```

    All new tests (`tree_contains_all_issues`, `blobs_are_lazy`, `gix_api_smoke`, the `path::tests::env_var_wins` unit test) must pass.
  </action>
  <acceptance_criteria>
    - `test -f crates/reposix-cache/src/error.rs && test -f crates/reposix-cache/src/path.rs && test -f crates/reposix-cache/src/cache.rs && test -f crates/reposix-cache/src/builder.rs` returns 0.
    - `grep -q 'pub struct Cache' crates/reposix-cache/src/cache.rs` returns 0.
    - `grep -q 'async fn build_from' crates/reposix-cache/src/builder.rs` returns 0.
    - `grep -q 'pub fn resolve_cache_path' crates/reposix-cache/src/path.rs` returns 0.
    - `grep -q 'CACHE_DIR_ENV.*REPOSIX_CACHE_DIR' crates/reposix-cache/src/path.rs` returns 0.
    - `grep -q 'pub enum Error' crates/reposix-cache/src/error.rs` returns 0.
    - `grep -q 'refs/heads/main' crates/reposix-cache/src/builder.rs` returns 0.
    - `grep -qE 'sync\(.*\{.*\}.*:.*\{.*\}.*\).*issues at' crates/reposix-cache/src/builder.rs` returns 0 (commit message format present).
    - `cargo test -p reposix-cache --test tree_contains_all_issues` exits 0.
    - `cargo test -p reposix-cache --test blobs_are_lazy` exits 0.
    - `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.
    - Whole-workspace regression: `cargo check --workspace` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p reposix-cache &amp;&amp; cargo clippy -p reposix-cache --all-targets -- -D warnings &amp;&amp; cargo check --workspace</automated>
  </verify>
  <done>`Cache::build_from` produces a valid bare repo with a tree enumerating every seeded issue on `refs/heads/main` and zero blob objects on disk. Two integration tests prove both invariants. Existing workspace still builds and lints clean.</done>
</task>
