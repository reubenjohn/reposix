// Compile-fail fixture for H-01 (phase 1 review): proves that
// `reposix_core::http::HttpClient::inner` is private — callers from outside
// the crate cannot reach the raw `reqwest::Client` to bypass the allowlist
// gate. Without this fixture, a future edit promoting `inner` to `pub` (or
// adding an `as_ref()` / `inner_client()` / `Deref` accessor) would silently
// re-open the SG-01 hole.

use reposix_core::http::{client, ClientOpts, HttpClient};

fn main() {
    let hc: HttpClient = client(ClientOpts::default()).expect("client builds");
    // Direct field access MUST be rejected: `inner` is private.
    let _raw = hc.inner;
}
