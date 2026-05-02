← [back to index](./index.md)

# Task 1: Scaffold `reposix-cache` crate (Cargo.toml + lib.rs) and smoke-test gix 0.82 API surface

<task type="auto" tdd="true">
  <name>Task 1: Scaffold `reposix-cache` crate (Cargo.toml + lib.rs) and smoke-test gix 0.82 API surface</name>
  <files>
    Cargo.toml,
    crates/reposix-cache/Cargo.toml,
    crates/reposix-cache/src/lib.rs
  </files>
  <read_first>
    Cargo.toml,
    crates/reposix-core/Cargo.toml,
    crates/reposix-core/src/lib.rs,
    .planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md
  </read_first>
  <behavior>
    - Workspace member list contains `crates/reposix-cache`.
    - `cargo check -p reposix-cache` succeeds against an empty lib.rs with only the forbid/warn attributes.
    - `cargo clippy -p reposix-cache --all-targets -- -D warnings` is clean.
    - A minimal smoke test (`cargo test -p reposix-cache --test gix_api_smoke`) confirms gix 0.82 exposes `init_bare`, `Repository::edit_tree`, `Repository::commit`, and `Repository::hash_object` with the shapes the builder will use (see `<interfaces>` block). If any signature differs, the test records the real signature in a `// NOTE:` comment.
  </behavior>
  <action>
    Step 1 — Workspace `Cargo.toml` edits (additive only, do not remove any existing member or dep):

    1a. Add `"crates/reposix-cache",` to the `[workspace] members = [...]` array, placed directly before the closing `]`. Preserve sort-ish order; it sits between `reposix-confluence` and whatever follows alphabetically is fine — end-of-list is acceptable.

    1b. Add to `[workspace.dependencies]`, placed after the existing `rusqlite = { version = "0.32", features = ["bundled"] }` line (keep the adjacent `fuser` block for now — Phase 36 removes it):
    ```toml
    # gix 0.82 — pure-Rust git object writer for the reposix-cache crate.
    # Pinned with `=` because gix is pre-1.0 (issue GitoxideLabs/gitoxide#470)
    # and minor versions have broken method signatures in the past.
    # See .planning/phases/31-.../31-RESEARCH.md §Pitfall 4.
    gix = "=0.82.0"
    # XDG cache dir resolution for the reposix-cache default path.
    dirs = "6"
    ```

    Step 2 — Create `crates/reposix-cache/Cargo.toml` (new file):
    ```toml
    [package]
    name = "reposix-cache"
    version.workspace = true
    edition.workspace = true
    rust-version.workspace = true
    authors.workspace = true
    license.workspace = true
    repository.workspace = true
    description = "Backing bare-repo cache built from REST responses via BackendConnector — substrate for the git-native architecture."

    [dependencies]
    reposix-core = { path = "../reposix-core" }
    tokio = { workspace = true }
    async-trait = { workspace = true }
    chrono = { workspace = true }
    rusqlite = { workspace = true }
    thiserror = { workspace = true }
    tracing = { workspace = true }
    serde_yaml = { workspace = true }
    gix = { workspace = true }
    dirs = { workspace = true }

    [dev-dependencies]
    reposix-sim = { path = "../reposix-sim" }
    tempfile = "3"
    trybuild = "1"
    tokio = { workspace = true, features = ["full", "test-util", "macros", "rt-multi-thread"] }
    anyhow = { workspace = true }
    ```

    Step 3 — Create `crates/reposix-cache/src/lib.rs` (new file):
    ```rust
    //! `reposix-cache` — backing bare-repo cache built from REST responses.
    //!
    //! This crate is the substrate for the git-native architecture pivot
    //! (v0.9.0). It materializes `BackendConnector` responses into a real
    //! on-disk bare git repo:
    //!
    //! - **Tree sync = full.** Every `Cache::build_from` call lists all
    //!   issues and writes a tree object with one entry per issue. Tree
    //!   metadata is cheap.
    //! - **Blob materialization = lazy.** Blobs are NOT written during
    //!   `build_from`; only `Cache::read_blob(oid)` persists a blob to
    //!   `.git/objects`. This is the whole point — the cache is a partial-
    //!   clone promisor, and writing all blobs upfront would defeat the
    //!   lazy invariant.
    //!
    //! Audit log, tainted-byte discipline, and egress allowlist enforcement
    //! land in Plan 02 and Plan 03 of Phase 31.
    //!
    //! ## Environment variables
    //! - `REPOSIX_CACHE_DIR` — overrides the default cache directory
    //!   (`$XDG_CACHE_HOME/reposix/`).
    //! - `REPOSIX_ALLOWED_ORIGINS` — egress allowlist, honored transitively
    //!   via `reposix_core::http::client()` which backend adapters use.

    #![forbid(unsafe_code)]
    #![warn(clippy::pedantic)]
    #![allow(clippy::module_name_repetitions)]

    // Re-exports land in Task 2; lib.rs is empty-ish for Task 1.
    ```

    Step 4 — Verify gix 0.82 API surface. Create `crates/reposix-cache/tests/gix_api_smoke.rs` with a test that compiles the exact method calls the builder will make:
    ```rust
    //! Smoke test for the gix 0.82 API surface used by `reposix-cache`.
    //! If this test stops compiling after `cargo update`, the builder plan
    //! assumptions are invalid — pin must be tightened or builder updated.

    use tempfile::tempdir;

    #[test]
    fn gix_082_exposes_expected_surface() {
        let tmp = tempdir().expect("tempdir");
        let repo_path = tmp.path().join("smoke.git");

        // init_bare: crate-level fn, returns Repository.
        let repo: gix::Repository = gix::init_bare(&repo_path).expect("init_bare");

        // object_hash: Repository::object_hash(&self) — needed for empty tree lookup.
        let _kind: gix::hash::Kind = repo.object_hash();

        // hash_object: compute OID without writing (critical for lazy invariant).
        let oid: gix::ObjectId = repo
            .write_blob(b"hello")
            .expect("write_blob")
            .detach();
        // (write_blob is OK in a smoke test; the builder uses hash_object for the
        // lazy path. If hash_object exists with a different name in 0.82, update
        // this comment with the real method name before Task 2.)

        // edit_tree: returns an Editor. Signature check only; we do not write.
        let empty = gix::ObjectId::empty_tree(repo.object_hash());
        let mut editor = repo.edit_tree(empty).expect("edit_tree");
        editor
            .upsert("issues/1.md", gix::object::tree::EntryKind::Blob, oid)
            .expect("upsert");
        let tree_oid = editor.write().expect("editor write");

        // commit: Repository::commit(ref, msg, tree, parents).
        let _commit_oid = repo
            .commit(
                "refs/heads/main",
                "smoke",
                tree_oid,
                std::iter::empty::<gix::ObjectId>(),
            )
            .expect("commit");

        // Confirm the ref points somewhere after commit.
        let head = repo.find_reference("refs/heads/main").expect("find ref");
        assert!(head.id().kind() == repo.object_hash());
    }
    ```

    If any method name or signature above fails to compile against gix 0.82, replace the failing line with the working 0.82 equivalent AND add a `// NOTE: gix 0.82 uses <real-name> instead of <sketched-name>` comment. Do NOT rewrite the test to use a different API shape without updating this plan's `<interfaces>` block in a follow-up commit note.

    Step 5 — Run:
    ```bash
    cargo check -p reposix-cache
    cargo clippy -p reposix-cache --all-targets -- -D warnings
    cargo test -p reposix-cache --test gix_api_smoke
    ```
  </action>
  <acceptance_criteria>
    - `grep -q 'crates/reposix-cache' Cargo.toml` returns 0 (workspace member added).
    - `grep -q 'gix = "=0.82.0"' Cargo.toml` returns 0 (pinned exactly).
    - `grep -q 'dirs = "6"' Cargo.toml` returns 0.
    - `test -f crates/reposix-cache/Cargo.toml && test -f crates/reposix-cache/src/lib.rs && test -f crates/reposix-cache/tests/gix_api_smoke.rs` all return 0.
    - `grep -q '#!\[forbid(unsafe_code)\]' crates/reposix-cache/src/lib.rs` returns 0.
    - `grep -q '#!\[warn(clippy::pedantic)\]' crates/reposix-cache/src/lib.rs` returns 0.
    - `cargo check -p reposix-cache 2>&1 | grep -E '^(error|warning:)' | wc -l` returns 0.
    - `cargo clippy -p reposix-cache --all-targets -- -D warnings 2>&1 | grep -E '^error' | wc -l` returns 0.
    - `cargo test -p reposix-cache --test gix_api_smoke` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>cargo check -p reposix-cache &amp;&amp; cargo clippy -p reposix-cache --all-targets -- -D warnings &amp;&amp; cargo test -p reposix-cache --test gix_api_smoke</automated>
  </verify>
  <done>Workspace builds with the new crate; gix API smoke test passes, confirming 0.82 exposes the exact methods the builder will call. Any signature surprise is recorded in a NOTE comment.</done>
</task>
