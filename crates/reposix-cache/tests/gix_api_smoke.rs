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

    // write_blob: persist a blob (used in read_blob; builder uses a
    // hash-only path via gix::objs::compute_hash).
    let oid: gix::ObjectId = repo.write_blob(b"hello").expect("write_blob").detach();

    // compute_hash (no-write OID): gix 0.82 exposes
    // `gix::objs::compute_hash(hash_kind, kind, data) -> Result<ObjectId, _>`.
    let hash_only = gix::objs::compute_hash(repo.object_hash(), gix::object::Kind::Blob, b"hello")
        .expect("compute_hash");
    assert_eq!(
        hash_only, oid,
        "compute_hash must agree with write_blob OID"
    );

    // edit_tree: returns an Editor. Signature check only; we then write.
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

    // Confirm the ref exists after commit.
    let head = repo.find_reference("refs/heads/main").expect("find ref");
    assert_eq!(
        head.target().try_id().map(gix::ObjectId::from),
        Some(head.id().detach())
    );
}
