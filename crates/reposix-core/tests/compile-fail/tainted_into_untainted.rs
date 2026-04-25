// Compile-fail fixture for SG-05 / ROADMAP phase-1 SC #1:
// passing a `Tainted<Record>` where `Untainted<Record>` is required MUST NOT
// compile. There is no `From<Tainted<T>> for Untainted<T>`, no `Deref`, no
// coercion — the only legal path is `sanitize(tainted, server_meta)`.

use chrono::Utc;
use reposix_core::{Record, RecordId, RecordStatus, Tainted, Untainted};

fn takes_untainted(_: Untainted<Record>) {}

fn main() {
    let tainted = Tainted::new(Record {
        id: RecordId(1),
        title: String::new(),
        status: RecordStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 0,
        body: String::new(),
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    });
    takes_untainted(tainted);
}
