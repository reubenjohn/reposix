//! Tainted / Untainted newtype discipline (SG-03 + SG-05).
//!
//! The `CaMeL` (Capability-modelling language) split: bytes that came from the
//! network are `Tainted<T>`; values that have been through `sanitize` and
//! carry server-authoritative metadata are `Untainted<T>`. Privileged sinks
//! (simulator write path, remote push) MUST accept only `Untainted<T>`.
//!
//! This module deliberately DOES NOT provide:
//!
//! - `impl<T> Deref for Tainted<T>` — autoderef would leak the inner value
//!   into untainted-only APIs.
//! - `impl<T> From<Tainted<T>> for Untainted<T>` — the only legal promotion
//!   path is [`sanitize`], which replaces server-authoritative fields.
//! - `serde::Serialize` / `serde::Deserialize` derives — callers must
//!   `into_inner()` first; serialization of a wrapper is a footgun.
//!
//! `Untainted::new` is `pub(crate)`. Downstream crates cannot construct an
//! `Untainted<T>` except via `sanitize`. The compile-fail fixture
//! `untainted_new_is_not_pub.rs` is the mechanical lock (FIX 4).

use chrono::{DateTime, Utc};

use crate::issue::{Record, RecordId};

/// Wrapper for values that originated from untrusted (network, agent) input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tainted<T>(T);

impl<T> Tainted<T> {
    /// Wrap a value as tainted. Always safe — you can never "untaint" by accident.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Consume the wrapper and return the inner value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Borrow the inner value.
    ///
    /// Named `inner_ref` (not `as_ref`) so we don't silently implement the
    /// [`std::convert::AsRef`] trait — that trait is available on many types
    /// and using it here would defeat the "no autoderef" discipline.
    #[must_use]
    pub fn inner_ref(&self) -> &T {
        &self.0
    }
}

/// Wrapper for values that have been sanitized and carry server-authoritative
/// metadata. Privileged sinks accept only this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Untainted<T>(T);

impl<T> Untainted<T> {
    /// Construct an `Untainted<T>`. Deliberately `pub(crate)` — the only
    /// legal construction path from user code is [`sanitize`].
    pub(crate) fn new(value: T) -> Self {
        Self(value)
    }

    /// Consume the wrapper and return the inner value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Borrow the inner value. Named `inner_ref` (see [`Tainted::inner_ref`]).
    #[must_use]
    pub fn inner_ref(&self) -> &T {
        &self.0
    }
}

/// Server-authoritative metadata for an [`Record`]. `sanitize` overwrites
/// these fields on the tainted value before wrapping it as `Untainted`.
#[derive(Debug, Clone)]
pub struct ServerMetadata {
    /// Server-assigned id.
    pub id: RecordId,
    /// Server-assigned creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Server-assigned last-update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Server-assigned optimistic-concurrency version.
    pub version: u64,
}

/// Strip server-controlled fields from a tainted `Record` and replace them
/// with the authoritative `ServerMetadata`. Agent-controlled fields
/// (`title`, `status`, `assignee`, `labels`, `body`) are preserved byte-for-byte.
///
/// This is the only legal path from `Tainted<Record>` to `Untainted<Record>`.
#[must_use]
// `ServerMetadata` is all-Copy fields, so clippy cannot see it as "consumed"
// even though we destructure it. We deliberately take ownership so future
// non-Copy fields (e.g. an audit-trail ID) cannot accidentally be passed by
// shared reference and shared across trust boundaries.
#[allow(clippy::needless_pass_by_value)]
pub fn sanitize(tainted: Tainted<Record>, server: ServerMetadata) -> Untainted<Record> {
    let issue = tainted.into_inner();
    let Record {
        id: _,
        title,
        status,
        assignee,
        labels,
        created_at: _,
        updated_at: _,
        version: _,
        body,
        parent_id,
        extensions,
    } = issue;
    let ServerMetadata {
        id,
        created_at,
        updated_at,
        version,
    } = server;
    Untainted::new(Record {
        id,
        title,
        status,
        assignee,
        labels,
        created_at,
        updated_at,
        version,
        body,
        parent_id,
        extensions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issue::IssueStatus;
    use chrono::TimeZone;

    #[test]
    fn tainted_inner_ref_works() {
        let t = Tainted::new(42_u64);
        assert_eq!(t.inner_ref(), &42_u64);
    }

    #[test]
    fn tainted_into_inner_works() {
        let t = Tainted::new(42_u64);
        assert_eq!(t.into_inner(), 42_u64);
    }

    #[test]
    fn tainted_derives_work() {
        let a = Tainted::new(7_u64);
        let b = a.clone();
        assert_eq!(a, b);
        // Debug should not panic.
        let _ = format!("{a:?}");
    }

    #[test]
    fn untainted_inner_ref_and_into_inner_work() {
        // Constructed via `pub(crate) new` from inside the crate — OK here.
        let u = Untainted::new(9_u32);
        assert_eq!(u.inner_ref(), &9_u32);
        assert_eq!(u.into_inner(), 9_u32);
    }

    fn tainted_issue_version_999999() -> Tainted<Record> {
        let t = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        Tainted::new(Record {
            id: RecordId(999_999),
            title: "agent-authored title".into(),
            status: IssueStatus::Open,
            assignee: Some("agent-alpha".into()),
            labels: vec!["bug".into()],
            created_at: t,
            updated_at: t,
            version: 999_999,
            body: "agent-authored body".into(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        })
    }

    fn server_meta() -> ServerMetadata {
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        ServerMetadata {
            id: RecordId(42),
            created_at: t,
            updated_at: t,
            version: 5,
        }
    }

    #[test]
    fn server_controlled_frontmatter_fields_are_stripped() {
        let tainted = tainted_issue_version_999999();
        let meta = server_meta();
        let untainted = sanitize(tainted, meta.clone());
        let inner = untainted.into_inner();
        // Server-controlled fields replaced from metadata.
        assert_eq!(inner.id, meta.id);
        assert_eq!(inner.version, meta.version);
        assert_eq!(inner.created_at, meta.created_at);
        assert_eq!(inner.updated_at, meta.updated_at);
        // Agent-controlled fields preserved byte-for-byte.
        assert_eq!(inner.title, "agent-authored title");
        assert_eq!(inner.status as u8, IssueStatus::Open as u8);
        assert_eq!(inner.assignee.as_deref(), Some("agent-alpha"));
        assert_eq!(inner.labels, vec!["bug".to_string()]);
        assert_eq!(inner.body, "agent-authored body");
    }

    #[test]
    fn sanitize_is_pure_and_cloneable() {
        let meta = server_meta();
        let u1 = sanitize(tainted_issue_version_999999(), meta.clone()).into_inner();
        let u2 = sanitize(tainted_issue_version_999999(), meta).into_inner();
        // Structural equality via field-by-field comparison — Issue is not
        // PartialEq (some embedded types might not be), so compare the fields
        // we care about.
        assert_eq!(u1.id, u2.id);
        assert_eq!(u1.version, u2.version);
        assert_eq!(u1.title, u2.title);
        assert_eq!(u1.body, u2.body);
    }
}
