//! Harness for compile-fail fixtures under `tests/compile-fail/`.
//!
//! - `tainted_cannot_be_used_where_untainted_required` (ROADMAP phase-1 SC #1):
//!   `Tainted<T>` does NOT coerce to `Untainted<T>`; there's no `From` impl,
//!   no `Deref`, and no blanket coercion.
//! - `untainted_new_is_pub_crate_only` (plan-checker FIX 4):
//!   `Untainted::new` is private to the crate; outside-crate call sites MUST
//!   fail to compile. The lock prevents a future edit promoting `pub(crate)
//!   fn new` to `pub fn new` from silently bypassing `sanitize()`.

#[test]
fn tainted_cannot_be_used_where_untainted_required() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/tainted_into_untainted.rs");
}

#[test]
fn untainted_new_is_pub_crate_only() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/untainted_new_is_not_pub.rs");
}
