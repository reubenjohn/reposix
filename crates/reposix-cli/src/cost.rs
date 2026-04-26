//! `reposix cost` — per-op cost ledger over the audit log.
//!
//! Aggregates `op='token_cost'` rows from `cache.db::audit_events_cache`
//! (one per helper RPC turn — `fetch` or `push`), grouped by op (the
//! `kind` field embedded in the JSON `reason`). Output is a pipe-friendly
//! Markdown table:
//!
//! ```text
//! | op    | bytes_in | bytes_out | est_input_tokens | est_output_tokens |
//! | ----- | -------- | --------- | ---------------- | ----------------- |
//! | fetch |   12,345 |    67,890 |            3,527 |            19,397 |
//! | push  |      512 |       128 |              146 |                36 |
//! | TOTAL |   12,857 |    68,018 |            3,673 |            19,433 |
//! ```
//!
//! Token estimate is `bytes / chars_per_token` (default 3.5; configurable via
//! `--chars-per-token`). The wider companion `reposix tokens` ledger surfaces
//! a back-of-envelope MCP comparison; `cost` is the raw per-op view, suitable
//! for piping into a spreadsheet or `awk`.
//!
//! Design intent: `.planning/research/v0.11.0-vision-and-innovations.md` §3c.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

use crate::tokens::parse_reason;
use crate::worktree_helpers::cache_path_from_worktree;

/// Default characters-per-token heuristic. Conservative for English text;
/// over-estimates for binary protocol-v2 frames. Configurable via
/// `--chars-per-token`.
const DEFAULT_CHARS_PER_TOKEN: f64 = 3.5;

/// Per-op aggregate over a window of `token_cost` rows.
#[derive(Debug, Default, Clone)]
pub struct OpAggregate {
    /// Number of rows.
    pub rows: u64,
    /// Sum of bytes received from agent.
    pub bytes_in: u64,
    /// Sum of bytes returned to agent.
    pub bytes_out: u64,
}

/// Parse a `--since` value into a UTC timestamp.
///
/// Accepts:
/// - Duration shortcuts: `7d`, `30d`, `1m` (≈30 days), `1y` (≈365 days),
///   `12h`, `30min`.
/// - Full RFC-3339 timestamps (e.g. `2026-04-25T01:00:00Z`).
///
/// # Errors
/// Returns an error if the input matches neither shape.
pub fn parse_since(since: &str) -> Result<DateTime<Utc>> {
    let now = Utc::now();
    // Duration shortcut first — cheap to detect.
    if let Some(dur) = parse_duration_shortcut(since) {
        return Ok(now - dur);
    }
    // Fall through to RFC-3339.
    DateTime::parse_from_rfc3339(since)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow!("invalid --since value `{since}`: not a duration shortcut (e.g. `7d`, `1m`, `12h`) or RFC-3339 timestamp ({e})"))
}

fn parse_duration_shortcut(s: &str) -> Option<Duration> {
    // Try longest-suffix-first so `min` matches before `m`.
    fn try_suffix(s: &str, suffix: &str) -> Option<i64> {
        let n = s.strip_suffix(suffix)?.parse::<i64>().ok()?;
        if n >= 0 {
            Some(n)
        } else {
            None
        }
    }
    if let Some(n) = try_suffix(s, "min") {
        return Some(Duration::minutes(n));
    }
    if let Some(n) = try_suffix(s, "d") {
        return Some(Duration::days(n));
    }
    if let Some(n) = try_suffix(s, "h") {
        return Some(Duration::hours(n));
    }
    if let Some(n) = try_suffix(s, "w") {
        return Some(Duration::days(n.saturating_mul(7)));
    }
    if let Some(n) = try_suffix(s, "m") {
        return Some(Duration::days(n.saturating_mul(30)));
    }
    if let Some(n) = try_suffix(s, "y") {
        return Some(Duration::days(n.saturating_mul(365)));
    }
    None
}

/// Aggregate `token_cost` rows since `since` (UTC), grouped by op kind.
///
/// Returns a `BTreeMap` keyed by op (deterministic ordering for tests).
///
/// # Errors
/// - `cache.db` cannot be opened or the query fails.
pub fn aggregate_at(
    cache_path: &Path,
    since: Option<DateTime<Utc>>,
) -> Result<std::collections::BTreeMap<String, OpAggregate>> {
    let db = cache_path.join("cache.db");
    if !db.exists() {
        bail!("no cache.db at {}", db.display());
    }
    let conn =
        Connection::open(&db).with_context(|| format!("open cache.db at {}", db.display()))?;
    aggregate_from_conn(&conn, since)
}

/// Same as [`aggregate_at`] but takes an open connection. Useful for tests.
///
/// # Errors
/// Returns the underlying `rusqlite::Error` wrapped via anyhow on query failure.
pub fn aggregate_from_conn(
    conn: &Connection,
    since: Option<DateTime<Utc>>,
) -> Result<std::collections::BTreeMap<String, OpAggregate>> {
    let mut by_op: std::collections::BTreeMap<String, OpAggregate> =
        std::collections::BTreeMap::new();

    // We deliberately avoid binding `since` as a SQL parameter: ts is stored
    // as RFC-3339 text and string comparison gives chronological order
    // because the prefix is YYYY-MM-DD. But for safety we filter in Rust
    // after parsing — keeps the SQL trivially auditable.
    let mut stmt = conn
        .prepare("SELECT ts, reason FROM audit_events_cache WHERE op = 'token_cost'")
        .context("prepare select token_cost")?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .context("query token_cost rows")?;
    for row in rows.flatten() {
        let (ts, reason) = row;
        // Filter by --since (skip rows older than the cutoff).
        if let Some(cutoff) = since {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(&ts) {
                if parsed.with_timezone(&Utc) < cutoff {
                    continue;
                }
            }
        }
        let Some(parsed) = parse_reason(&reason) else {
            continue;
        };
        let entry = by_op.entry(parsed.kind).or_default();
        entry.rows += 1;
        entry.bytes_in = entry.bytes_in.saturating_add(parsed.chars_in);
        entry.bytes_out = entry.bytes_out.saturating_add(parsed.chars_out);
    }

    Ok(by_op)
}

/// Render a markdown-table cost report.
///
/// `chars_per_token` is the divisor for the token estimate columns
/// (default 3.5).
#[must_use]
pub fn render_markdown(
    by_op: &std::collections::BTreeMap<String, OpAggregate>,
    chars_per_token: f64,
) -> String {
    let mut out = String::new();
    out.push_str("| op       | bytes_in | bytes_out | est_input_tokens | est_output_tokens |\n");
    out.push_str("| -------- | -------- | --------- | ---------------- | ----------------- |\n");

    let mut total = OpAggregate::default();
    for (op, agg) in by_op {
        let in_tok = est_tokens(agg.bytes_in, chars_per_token);
        let out_tok = est_tokens(agg.bytes_out, chars_per_token);
        let _ = writeln!(
            out,
            "| {op:<8} | {bin:>8} | {bout:>9} | {in_tok:>16} | {out_tok:>17} |",
            bin = with_commas(agg.bytes_in),
            bout = with_commas(agg.bytes_out),
            in_tok = with_commas(in_tok),
            out_tok = with_commas(out_tok),
        );
        total.rows = total.rows.saturating_add(agg.rows);
        total.bytes_in = total.bytes_in.saturating_add(agg.bytes_in);
        total.bytes_out = total.bytes_out.saturating_add(agg.bytes_out);
    }
    let in_tok = est_tokens(total.bytes_in, chars_per_token);
    let out_tok = est_tokens(total.bytes_out, chars_per_token);
    let _ = writeln!(
        out,
        "| {label:<8} | {bin:>8} | {bout:>9} | {in_tok:>16} | {out_tok:>17} |",
        label = "TOTAL",
        bin = with_commas(total.bytes_in),
        bout = with_commas(total.bytes_out),
        in_tok = with_commas(in_tok),
        out_tok = with_commas(out_tok),
    );
    out
}

fn est_tokens(bytes: u64, chars_per_token: f64) -> u64 {
    if chars_per_token <= 0.0 {
        return 0;
    }
    // Cast precision: token counts on the order of 10^6 are well below 2^53.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    {
        ((bytes as f64) / chars_per_token).round() as u64
    }
}

fn with_commas(mut n: u64) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut parts: Vec<String> = Vec::new();
    while n > 0 {
        parts.push(format!("{:03}", n % 1000));
        n /= 1000;
    }
    let last = parts.last_mut().unwrap();
    *last = last.trim_start_matches('0').to_string();
    if last.is_empty() {
        *last = "0".to_string();
    }
    parts.reverse();
    parts.join(",")
}

/// `reposix cost` entry point.
///
/// `path` defaults to cwd (resolved via the working tree's `remote.origin.url`).
/// `since` accepts duration shortcuts (`7d`, `1m`, `12h`) or full RFC-3339.
/// `chars_per_token` overrides the default 3.5 heuristic.
///
/// # Errors
/// - The path has no parseable `reposix::` remote URL.
/// - `--since` is malformed.
/// - `cache.db` cannot be opened or queried.
pub fn run(
    path: Option<PathBuf>,
    since: Option<String>,
    chars_per_token: Option<f64>,
) -> Result<()> {
    let work = match path {
        Some(p) => p,
        None => std::env::current_dir().context("resolve current directory")?,
    };
    let cache_path = cache_path_from_worktree(&work)?;
    if !cache_path.exists() {
        bail!(
            "no cache at {} (run `git fetch` first)",
            cache_path.display()
        );
    }
    let cutoff = since.as_deref().map(parse_since).transpose()?;
    let cpt = chars_per_token.unwrap_or(DEFAULT_CHARS_PER_TOKEN);
    let by_op = aggregate_at(&cache_path, cutoff)?;
    let label = match (cutoff, since.as_deref()) {
        (Some(c), Some(s)) => format!("since {s} ({})", c.to_rfc3339()),
        (None, _) => "all-time".to_string(),
        _ => "since (unknown)".to_string(),
    };
    println!(
        "reposix cost — {} ({label}, chars/token={cpt})",
        cache_path.display()
    );
    println!();
    if by_op.is_empty() {
        println!(
            "  No token_cost rows in window. Run `git fetch` or `git push` to populate the audit log."
        );
        return Ok(());
    }
    print!("{}", render_markdown(&by_op, cpt));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_accepts_duration_shortcuts() {
        // Use a generous lower bound to avoid clock skew between
        // `Utc::now()` here and the call inside `parse_since`. We're
        // testing parse correctness, not microsecond accuracy.
        let now = Utc::now();
        let seven = parse_since("7d").unwrap();
        assert!(seven < now);
        assert!(
            (now - seven).num_hours() >= 7 * 24 - 1,
            "7d should be ~7 days before now, got {h}h",
            h = (now - seven).num_hours()
        );
        let one_month = parse_since("1m").unwrap();
        assert!((now - one_month).num_days() >= 29);
        let twelve_h = parse_since("12h").unwrap();
        assert!((now - twelve_h).num_minutes() >= 12 * 60 - 1);
    }

    #[test]
    fn parse_since_accepts_rfc3339() {
        let parsed = parse_since("2026-04-25T01:00:00Z").unwrap();
        assert_eq!(parsed.to_rfc3339(), "2026-04-25T01:00:00+00:00");
    }

    #[test]
    fn parse_since_rejects_garbage() {
        let err = parse_since("nope").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("invalid --since"), "got: {s}");
    }

    #[test]
    fn render_markdown_produces_table() {
        let mut by_op = std::collections::BTreeMap::new();
        by_op.insert(
            "fetch".to_string(),
            OpAggregate {
                rows: 3,
                bytes_in: 100,
                bytes_out: 200,
            },
        );
        by_op.insert(
            "push".to_string(),
            OpAggregate {
                rows: 1,
                bytes_in: 10,
                bytes_out: 5,
            },
        );
        let table = render_markdown(&by_op, 3.5);
        assert!(table.contains("| op"), "header missing: {table}");
        assert!(table.contains("| fetch"));
        assert!(table.contains("| push"));
        assert!(table.contains("| TOTAL"));
    }

    #[test]
    fn render_markdown_zero_chars_per_token_does_not_panic() {
        let mut by_op = std::collections::BTreeMap::new();
        by_op.insert(
            "fetch".to_string(),
            OpAggregate {
                rows: 1,
                bytes_in: 100,
                bytes_out: 100,
            },
        );
        let table = render_markdown(&by_op, 0.0);
        assert!(table.contains("| TOTAL"));
    }
}
