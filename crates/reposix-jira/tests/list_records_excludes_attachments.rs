//! P73 CONNECTOR-GAP-03: assert JIRA list_records strips
//! `fields.attachment` + `fields.comment.comments` at the rendering
//! boundary (per docs/decisions/005-jira-issue-mapping.md:79-87).
//! Stub committed in Wave 1; implementation lands in Wave 2 (Task 4).

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_records_excludes_attachments_and_comments() {
    unimplemented!("P73 Task 4 — wiremock + assert body/extensions exclusion");
}
