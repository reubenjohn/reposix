//! Privileged-sink signatures that `reposix-cache` publishes to lock in
//! the `Tainted<T>` vs `Untainted<T>` type discipline at compile time.
//!
//! Phase 31 ships just the stub [`sink_egress`] — Phase 34 will flesh
//! it out as part of the push path. Until then, the function is a
//! no-op whose only purpose is to anchor the compile-fail fixture in
//! `tests/compile-fail/tainted_blob_into_egress.rs`.
//!
//! # Why this lives in `reposix-cache` and not `reposix-core`
//!
//! `reposix-core` defines the `Tainted`/`Untainted` newtypes but does
//! not itself have a privileged-sink concept — sinks are a downstream
//! helper concern (Phase 32 stateless-connect, Phase 34 export). We
//! put the stub in `reposix-cache` because the cache is the first
//! crate downstream of `reposix-core` that needs to prove the
//! discipline.
//!
//! The function is `#[doc(hidden)]` so rustdoc does not advertise it
//! as a real public API.

/// Stub privileged sink. Compile-time anchor for the
/// `Tainted`-vs-`Untainted` discipline — this function accepts only
/// `Untainted<Vec<u8>>`, and the compile-fail fixture
/// `tests/compile-fail/tainted_blob_into_egress.rs` verifies that the
/// compiler rejects calls with `Tainted<Vec<u8>>`.
///
/// Real implementation lands in Phase 34 (push path). Until then this
/// is a no-op.
#[doc(hidden)]
pub fn sink_egress(_bytes: reposix_core::Untainted<Vec<u8>>) {
    // intentional no-op — real impl in Phase 34
}
