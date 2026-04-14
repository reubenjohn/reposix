// Compile-fail fixture for SG-05 / ROADMAP phase-1 SC #1:
// passing a `Tainted<Issue>` where `Untainted<Issue>` is required MUST NOT
// compile. There is no `From<Tainted<T>> for Untainted<T>`, no `Deref`, no
// coercion — the only legal path is `sanitize(tainted, server_meta)`.

use chrono::Utc;
use reposix_core::{Issue, IssueId, IssueStatus, Tainted, Untainted};

fn takes_untainted(_: Untainted<Issue>) {}

fn main() {
    let tainted = Tainted::new(Issue {
        id: IssueId(1),
        title: String::new(),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 0,
        body: String::new(),
        parent_id: None,
    });
    takes_untainted(tainted);
}
