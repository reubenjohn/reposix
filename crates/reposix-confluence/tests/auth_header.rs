//! P73 CONNECTOR-GAP-01: byte-exact Basic-auth header assertion via
//! wiremock + `BackendConnector` trait seam. Stub committed in
//! Wave 1; implementation lands in Wave 2 (Task 2).

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn auth_header_basic_byte_exact() {
    unimplemented!("P73 Task 2 — wiremock byte-exact Basic auth header assertion");
}
