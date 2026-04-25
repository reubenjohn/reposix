//! This file MUST fail to compile.
//!
//! ARCH-02 relies on `Untainted::new` being `pub(crate)` in reposix-core
//! so downstream crates cannot directly construct an untainted value and
//! bypass the `sanitize` ritual. This fixture attempts the construction
//! from outside `reposix-core` and expects a privacy error.

fn main() {
    // This line MUST NOT compile: Untainted::new is pub(crate) inside
    // reposix-core; calling it from reposix-cache is a privacy error.
    let _u: reposix_core::Untainted<Vec<u8>> =
        reposix_core::Untainted::new(vec![1, 2, 3]);
}
