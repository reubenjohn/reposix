// Compile-fail fixture for FIX 4 (plan-checker): proves
// `reposix_core::Untainted::new` is `pub(crate)` — calling it from outside
// the crate MUST NOT compile. Without this fixture, a future edit promoting
// `pub(crate) fn new` to `pub fn new` would silently bypass `sanitize()`.

use chrono::Utc;
use reposix_core::{Issue, IssueId, IssueStatus, Untainted};

fn main() {
    let some_issue = Issue {
        id: IssueId(1),
        title: String::new(),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 0,
        body: String::new(),
    };
    let _u = Untainted::new(some_issue);
}
