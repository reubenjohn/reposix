//! This file MUST fail to compile.
//!
//! ARCH-02 locks the Tainted-vs-Untainted discipline at the type level.
//! A caller that tries to pass `Tainted<Vec<u8>>` (what `Cache::read_blob`
//! returns) to a privileged sink expecting `Untainted<Vec<u8>>` (what
//! Phase 34's push path will require) MUST get a compile error, not a
//! runtime check.
//!
//! The companion `.stderr` file captures the compiler diagnostic; the
//! trybuild driver asserts compilation fails AND the diagnostic matches.

use reposix_cache::sink::sink_egress;
use reposix_core::Tainted;

fn main() {
    let tainted: Tainted<Vec<u8>> = Tainted::new(vec![1, 2, 3]);
    // This line MUST NOT compile: sink_egress wants Untainted<Vec<u8>>,
    // and there is no From<Tainted<T>> for Untainted<T>, no Deref, no
    // AsRef. The only legal path is `reposix_core::sanitize`, which is
    // only defined for `Issue`, not `Vec<u8>`.
    sink_egress(tainted);
}
