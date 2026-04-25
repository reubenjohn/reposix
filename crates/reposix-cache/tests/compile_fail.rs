//! ARCH-02: type-level locks on the Tainted-vs-Untainted discipline.
//!
//! Each fixture under `tests/compile-fail/` MUST fail to compile, and
//! its diagnostic MUST match the sibling `.stderr` file. trybuild
//! asserts both conditions.
//!
//! If the diagnostics drift (e.g. after a rustc upgrade), regenerate
//! the `.stderr` files with `TRYBUILD=overwrite cargo test -p
//! reposix-cache --test compile_fail` AND REVIEW THE DIFF — a silent
//! `TRYBUILD=overwrite` can mask an actual discipline regression.

#[test]
fn tainted_blob_cannot_flow_to_egress_sink() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/tainted_blob_into_egress.rs");
}

#[test]
fn untainted_new_is_not_pub_in_downstream_crates() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/untainted_new_is_pub_crate.rs");
}
