//! Deterministic seed loader: reads a JSON file describing a project plus
//! issues, and `INSERT OR IGNORE`s them into the `issues` table.
//!
//! Determinism matters: the demo walkthrough (Phase 4) asserts exact file
//! contents. Every seeded row has a fixed `created_at` / `updated_at` of
//! `2026-04-13T00:00:00Z` and `version = 1`.
//!
//! Seed data intentionally includes adversarial fixtures: one body contains
//! a `<script>` tag, another contains a literal `version: 999` line. These
//! exercise downstream code's robustness to untrusted content.

use std::fs;
use std::path::Path;

use chrono::{SecondsFormat, TimeZone, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

/// The on-disk shape of `fixtures/seed.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct SeedFile {
    /// Project metadata (only the slug is used at seed time; `name` /
    /// `description` live here for future project endpoints).
    pub project: SeedProject,
    /// Issues to insert under `project.slug`.
    pub issues: Vec<SeedIssue>,
}

/// Project-level seed metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct SeedProject {
    /// URL-safe slug (becomes the `:slug` path parameter).
    pub slug: String,
    /// Human-readable project name.
    pub name: String,
    /// Free-form description.
    #[serde(default)]
    pub description: String,
}

/// Issue-level seed entry. Mirrors the writable subset of `Record`; server
/// fields (`created_at`, `updated_at`, `version`) are set by `load_seed`.
#[derive(Debug, Serialize, Deserialize)]
pub struct SeedIssue {
    /// Issue id (must be unique per project).
    pub id: u64,
    /// Title (single-line).
    pub title: String,
    /// One of the five `RecordStatus` values, snake-case: `open`,
    /// `in_progress`, `in_review`, `done`, `wont_fix`.
    pub status: String,
    /// Optional assignee.
    #[serde(default)]
    pub assignee: Option<String>,
    /// Labels.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Free-form Markdown body.
    #[serde(default)]
    pub body: String,
}

/// Insert every issue from `path` into the `issues` table under the
/// project slug found in the JSON. `INSERT OR IGNORE` makes the call
/// idempotent across reruns with an existing DB.
///
/// # Errors
/// Returns [`ApiError::Internal`] if the file cannot be read or the JSON
/// cannot be parsed; [`ApiError::Db`] if any INSERT fails.
pub fn load_seed(conn: &Connection, path: &Path) -> Result<usize, ApiError> {
    let raw = fs::read_to_string(path)
        .map_err(|e| ApiError::Internal(format!("read seed {}: {e}", path.display())))?;
    let parsed: SeedFile = serde_json::from_str(&raw)?;
    apply_seed(conn, &parsed)
}

/// Apply an already-parsed [`SeedFile`] to `conn`.
///
/// Issue ids are bound into `SQLite` as `i64` (the only integer type rusqlite
/// exposes). Seed ids come from developer-authored fixtures in a fixed small
/// range (currently 1..=6), so the `u64 -> i64` cast never wraps — the cast
/// is lint-exempt rather than runtime-checked.
///
/// # Errors
/// Returns [`ApiError::Db`] if any INSERT fails or label JSON serialization
/// fails.
///
/// # Panics
/// Panics only if `2026-04-13T00:00:00Z` somehow becomes ambiguous (it
/// cannot; the date is outside any DST transition).
pub fn apply_seed(conn: &Connection, seed: &SeedFile) -> Result<usize, ApiError> {
    // Fixed deterministic timestamps.
    let ts = Utc
        .with_ymd_and_hms(2026, 4, 13, 0, 0, 0)
        .single()
        .expect("2026-04-13T00:00:00Z is unambiguous")
        .to_rfc3339_opts(SecondsFormat::Secs, true);

    let mut inserted = 0usize;
    for issue in &seed.issues {
        let labels_json = serde_json::to_string(&issue.labels)?;
        #[allow(clippy::cast_possible_wrap)] // seeded ids are small (< i64::MAX)
        let id_signed = issue.id as i64;
        let affected = conn.execute(
            "INSERT OR IGNORE INTO issues \
             (project, id, title, status, assignee, labels, created_at, updated_at, version, body) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
            params![
                seed.project.slug,
                id_signed,
                issue.title,
                issue.status,
                issue.assignee,
                labels_json,
                ts,
                ts,
                issue.body,
            ],
        )?;
        inserted += affected;
    }
    Ok(inserted)
}

#[cfg(test)]
mod tests {
    use super::{apply_seed, load_seed, SeedFile};
    use crate::db::open_db;
    use std::path::{Path, PathBuf};

    fn seed_fixture_path() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("fixtures/seed.json");
        p
    }

    #[test]
    fn load_seed_inserts_six_issues() {
        let conn = open_db(Path::new(":memory:"), true).expect("open");
        let inserted = load_seed(&conn, &seed_fixture_path()).expect("load");
        assert_eq!(inserted, 6, "seed should insert exactly 6 rows");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM issues", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 6);
    }

    #[test]
    fn at_least_one_body_contains_fake_version_999_line() {
        let conn = open_db(Path::new(":memory:"), true).expect("open");
        load_seed(&conn, &seed_fixture_path()).expect("load");
        let bodies: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT body FROM issues ORDER BY id")
                .expect("prep");
            stmt.query_map([], |r| r.get::<_, String>(0))
                .expect("query")
                .map(std::result::Result::unwrap)
                .collect()
        };
        let has_fake_version = bodies.iter().any(|b| b.contains("version: 999"));
        assert!(
            has_fake_version,
            "at least one seed body must contain the literal `version: 999` adversarial fixture"
        );
    }

    #[test]
    fn at_least_one_body_contains_script_tag() {
        let conn = open_db(Path::new(":memory:"), true).expect("open");
        load_seed(&conn, &seed_fixture_path()).expect("load");
        let bodies: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT body FROM issues ORDER BY id")
                .expect("prep");
            stmt.query_map([], |r| r.get::<_, String>(0))
                .expect("query")
                .map(std::result::Result::unwrap)
                .collect()
        };
        let has_script = bodies.iter().any(|b| b.contains("<script>"));
        assert!(
            has_script,
            "at least one seed body must contain a literal <script> adversarial fixture"
        );
    }

    #[test]
    fn load_seed_is_idempotent() {
        let conn = open_db(Path::new(":memory:"), true).expect("open");
        let first = load_seed(&conn, &seed_fixture_path()).expect("first");
        let second = load_seed(&conn, &seed_fixture_path()).expect("second");
        assert_eq!(first, 6);
        assert_eq!(second, 0, "INSERT OR IGNORE must no-op on rerun");
    }

    #[test]
    fn apply_seed_round_trips() {
        let seed: SeedFile =
            serde_json::from_str(include_str!("../fixtures/seed.json")).expect("json");
        let conn = open_db(Path::new(":memory:"), true).expect("open");
        let n = apply_seed(&conn, &seed).expect("apply");
        assert_eq!(n, seed.issues.len());
    }
}
