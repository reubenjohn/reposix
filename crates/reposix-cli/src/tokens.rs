//! `reposix tokens` — token-economy ledger over the audit log.
//!
//! Reads `op='token_cost'` rows from the cache's `audit_events_cache`
//! table, sums them, prints the totals plus an honest comparison against
//! a back-of-envelope MCP-equivalent estimate.
//!
//! Design intent: `.planning/research/v0.11.0-vision-and-innovations.md` §3c.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::Connection;

use crate::worktree_helpers::cache_path_from_worktree as resolve_cache_dir;

/// Aggregate stats over the token-cost rows.
#[derive(Debug, Default, Clone)]
pub struct TokenSummary {
    /// First `ts` seen (RFC-3339 string).
    pub since: Option<String>,
    /// Total `token_cost` rows (i.e. RPC turns).
    pub sessions: u64,
    /// Sum of `chars_in` across all rows.
    pub chars_in: u64,
    /// Sum of `chars_out` across all rows.
    pub chars_out: u64,
    /// Sub-totals split by `kind` (`fetch` / `push` / etc).
    pub by_kind: std::collections::BTreeMap<String, (u64, u64, u64)>, // kind -> (sessions, in, out)
}

impl TokenSummary {
    /// Token estimate from char count (chars / 4).
    #[must_use]
    pub fn estimated_tokens_in(&self) -> u64 {
        self.chars_in / 4
    }
    #[must_use]
    pub fn estimated_tokens_out(&self) -> u64 {
        self.chars_out / 4
    }
    #[must_use]
    pub fn total_tokens(&self) -> u64 {
        self.estimated_tokens_in() + self.estimated_tokens_out()
    }
}

/// `reposix tokens` entry point.
///
/// `path` defaults to cwd. Uses the `remote.origin.url` to resolve the
/// cache directory (same scheme as `reposix history` / `reposix gc`).
///
/// # Errors
/// - The path has no parseable `reposix::` remote URL.
/// - The cache directory cannot be resolved.
/// - The cache.db cannot be opened.
pub fn run(path: Option<PathBuf>) -> Result<()> {
    let work = match path {
        Some(p) => p,
        None => std::env::current_dir().context("resolve current directory")?,
    };
    let cache_path = cache_path_from_worktree(&work)?;
    let summary = aggregate_at(&cache_path)?;
    print_summary(&cache_path, &summary);
    Ok(())
}

/// Resolve the cache path from a working tree, additionally requiring that
/// `cache.db` exists (no token-cost rows otherwise).
fn cache_path_from_worktree(work: &Path) -> Result<PathBuf> {
    let cache_path = resolve_cache_dir(work)?;
    if !cache_path.exists() {
        bail!(
            "no cache at {} (run `git fetch` to populate token_cost audit rows)",
            cache_path.display()
        );
    }
    Ok(cache_path)
}

/// Aggregate `token_cost` rows from `<cache>/cache.db`. Public for tests.
///
/// # Errors
/// - The cache.db cannot be opened.
/// - The query fails.
pub fn aggregate_at(cache_path: &Path) -> Result<TokenSummary> {
    let db = cache_path.join("cache.db");
    if !db.exists() {
        bail!("no cache.db at {}", db.display());
    }
    let conn =
        Connection::open(&db).with_context(|| format!("open cache.db at {}", db.display()))?;
    aggregate_from_conn(&conn)
}

/// Same as [`aggregate_at`] but takes an open connection.
///
/// # Errors
/// Returns the underlying `rusqlite::Error` wrapped via anyhow on query
/// failure.
pub fn aggregate_from_conn(conn: &Connection) -> Result<TokenSummary> {
    let mut summary = TokenSummary::default();

    // Earliest ts.
    let since: Option<String> = conn
        .query_row(
            "SELECT MIN(ts) FROM audit_events_cache WHERE op = 'token_cost'",
            [],
            |r| r.get::<_, Option<String>>(0),
        )
        .unwrap_or(None);
    summary.since = since;

    let mut stmt = conn
        .prepare("SELECT reason FROM audit_events_cache WHERE op = 'token_cost'")
        .context("prepare select token_cost")?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .context("query token_cost rows")?;

    for row in rows.flatten() {
        if let Some(parsed) = parse_reason(&row) {
            summary.sessions += 1;
            summary.chars_in = summary.chars_in.saturating_add(parsed.chars_in);
            summary.chars_out = summary.chars_out.saturating_add(parsed.chars_out);
            let entry = summary.by_kind.entry(parsed.kind.clone()).or_default();
            entry.0 += 1;
            entry.1 = entry.1.saturating_add(parsed.chars_in);
            entry.2 = entry.2.saturating_add(parsed.chars_out);
        }
    }

    Ok(summary)
}

/// Parse the JSON-in-reason payload `{"in":N,"out":M,"kind":"fetch|push"}`.
/// Permissive: returns `None` if the payload is malformed, doesn't touch
/// `serde_json` (this crate already pulls it via dependencies but keeping
/// the parser dependency-free keeps the compile time low for tests).
#[must_use]
pub fn parse_reason(reason: &str) -> Option<TokenRowPub> {
    // Format produced by `reposix_cache::audit::log_token_cost`:
    //   {"in":1234,"out":5678,"kind":"fetch"}
    // Hand-roll a minimal parser: extract three substrings.
    let chars_in = extract_number(reason, "\"in\":")?;
    let chars_out = extract_number(reason, "\"out\":")?;
    let kind = extract_quoted(reason, "\"kind\":\"")?;
    Some(TokenRowPub {
        chars_in,
        chars_out,
        kind,
    })
}

/// Public mirror of the internal `TokenRow`. Returned by [`parse_reason`].
#[derive(Debug, Clone)]
pub struct TokenRowPub {
    /// Bytes the agent sent.
    pub chars_in: u64,
    /// Bytes the helper sent back.
    pub chars_out: u64,
    /// `"fetch"` or `"push"` (other kinds are accepted but tagged as-is).
    pub kind: String,
}

fn extract_number(s: &str, key: &str) -> Option<u64> {
    let rest = s.split_once(key)?.1;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    rest[..end].parse::<u64>().ok()
}

fn extract_quoted(s: &str, key: &str) -> Option<String> {
    let rest = s.split_once(key)?.1;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn print_summary(cache_path: &Path, summary: &TokenSummary) {
    let label = summary.since.as_deref().unwrap_or("(no token_cost rows)");
    println!(
        "Token economy for {} (since {}):",
        cache_path.display(),
        label
    );
    println!();
    if summary.sessions == 0 {
        println!("  No sessions recorded yet — run a `git fetch` or `git push`");
        println!("  against this cache to populate token_cost audit rows.");
        return;
    }

    let avg_in = summary.estimated_tokens_in() / summary.sessions.max(1);
    let avg_out = summary.estimated_tokens_out() / summary.sessions.max(1);

    println!("  Sessions:   {:>10}", summary.sessions);
    println!(
        "  Tokens-in:  {:>10}  (avg per session: {avg_in})",
        with_commas(summary.estimated_tokens_in())
    );
    println!(
        "  Tokens-out: {:>10}  (avg per session: {avg_out})",
        with_commas(summary.estimated_tokens_out())
    );
    println!("  Total:      {:>10}", with_commas(summary.total_tokens()));

    if !summary.by_kind.is_empty() {
        println!();
        println!("  Per-kind breakdown:");
        for (kind, (sessions, ci, co)) in &summary.by_kind {
            let est_in = ci / 4;
            let est_out = co / 4;
            println!("    {kind:<8} sessions={sessions:<5}  in={est_in:<10}  out={est_out:<10}");
        }
    }

    // MCP-equivalent estimate. Conservative: 100k schema discovery + 5k per call.
    let mcp_schema = 100_000_u64;
    let mcp_per_call = 5_000_u64;
    let mcp_total = mcp_schema + mcp_per_call.saturating_mul(summary.sessions);
    println!();
    println!("MCP-equivalent estimate (100k schema discovery + 5k per tool call):");
    println!("  Schema discovery: {}", with_commas(mcp_schema));
    println!(
        "  Per-call avg:     5,000 x {} sessions = {}",
        summary.sessions,
        with_commas(mcp_per_call.saturating_mul(summary.sessions))
    );
    println!("  Total:            {}", with_commas(mcp_total));

    // Honest comparison.
    println!();
    let reposix_total = summary.total_tokens();
    if reposix_total == 0 {
        println!(
            "Net effect: reposix used 0 tokens vs MCP {} estimated.",
            with_commas(mcp_total)
        );
    } else if mcp_total >= reposix_total {
        // Cast precision: ratio is a display-only float; both totals are
        // bounded well below 2^53 in practice (token counts on the order
        // of 10^6 - 10^9). Allow rather than reach for `f128`.
        #[allow(clippy::cast_precision_loss)]
        let ratio = (mcp_total as f64) / (reposix_total as f64);
        println!("Net effect: reposix used {ratio:.1}x FEWER tokens than the MCP estimate.");
    } else {
        #[allow(clippy::cast_precision_loss)]
        let ratio = (reposix_total as f64) / (mcp_total as f64);
        println!("Net effect: reposix used {ratio:.1}x MORE tokens than the MCP estimate.");
        println!(
            "  (Note: reposix scales with payload bytes; MCP scales with schema + invocations.)"
        );
    }
    println!(
        "  (Both estimates are heuristic. reposix uses chars/4 over WIRE bytes (incl. protocol-v2"
    );
    println!(
        "  framing); the MCP baseline is a conservative back-of-envelope. Actual savings vary by"
    );
    println!("  workload — blob-heavy reads favour reposix; metadata-only calls favour MCP.)");
}

/// Format `n` with thousand-separator commas (e.g. 1234567 -> "1,234,567").
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reason_round_trip() {
        let parsed = parse_reason(r#"{"in":1234,"out":5678,"kind":"fetch"}"#).unwrap();
        assert_eq!(parsed.chars_in, 1234);
        assert_eq!(parsed.chars_out, 5678);
        assert_eq!(parsed.kind, "fetch");
    }

    #[test]
    fn parse_reason_handles_push() {
        let parsed = parse_reason(r#"{"in":99,"out":18,"kind":"push"}"#).unwrap();
        assert_eq!(parsed.kind, "push");
    }

    #[test]
    fn parse_reason_rejects_garbage() {
        assert!(parse_reason("nope").is_none());
        assert!(parse_reason(r#"{"in":,"out":1,"kind":"x"}"#).is_none());
    }

    #[test]
    fn with_commas_works() {
        assert_eq!(with_commas(0), "0");
        assert_eq!(with_commas(5), "5");
        assert_eq!(with_commas(1_000), "1,000");
        assert_eq!(with_commas(1_234_567), "1,234,567");
    }
}
