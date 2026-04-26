//! Metrics accumulator: per-operation HDR histograms + error counters.
//!
//! Shared across swarm client tasks behind a `parking_lot::Mutex` wrapped in
//! an `Arc`. Record latencies via [`MetricsAccumulator::record`]; record
//! errors via [`MetricsAccumulator::record_error`]. At run end, call
//! [`MetricsAccumulator::render_markdown`] to produce the summary table.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use hdrhistogram::Histogram;
use parking_lot::Mutex;

/// The kinds of operations the workload loop performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OpKind {
    /// `list_records` on a project.
    List,
    /// `get_record` by id.
    Get,
    /// `update_record` (PATCH).
    Patch,
}

impl OpKind {
    /// Short human-readable label used in the markdown summary.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Get => "get",
            Self::Patch => "patch",
        }
    }
}

/// Error classes the swarm tracks. Bucketed rather than enumerating every
/// transport code so the markdown summary stays one narrow column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ErrorKind {
    /// HTTP 404 (or `Error::Other("not found: ...")`).
    NotFound,
    /// HTTP 409 (optimistic-concurrency miss).
    Conflict,
    /// HTTP 429 (rate limited).
    RateLimited,
    /// Client-side or server-side timeout.
    Timeout,
    /// Any other transport / serialization / unexpected error.
    Other,
}

impl ErrorKind {
    /// Canonical label used in markdown render.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "NotFound",
            Self::Conflict => "Conflict",
            Self::RateLimited => "RateLimited",
            Self::Timeout => "Timeout",
            Self::Other => "Other",
        }
    }

    /// Classify a [`reposix_core::Error`] into a coarse bucket. Heuristic:
    /// inspect the message for the usual substrings; fallback to `Other`.
    #[must_use]
    pub fn classify(err: &reposix_core::Error) -> Self {
        let msg = err.to_string();
        if msg.contains("not found") || msg.contains("404") {
            return Self::NotFound;
        }
        if msg.contains("409") || msg.contains("version mismatch") || msg.contains("Conflict") {
            return Self::Conflict;
        }
        if msg.contains("429") || msg.contains("rate") {
            return Self::RateLimited;
        }
        if msg.contains("timed out") || msg.contains("timeout") {
            return Self::Timeout;
        }
        Self::Other
    }
}

/// In-memory accumulator shared across swarm tasks.
///
/// Internally holds an HDR histogram per [`OpKind`] (resolution: 1µs..60s,
/// 3 significant digits) and a counter per [`ErrorKind`]. Mutation is
/// serialized behind a `parking_lot::Mutex` — contention at the rates we
/// run is a non-issue (µs-scale lock hold).
pub struct MetricsAccumulator {
    inner: Mutex<Inner>,
}

struct Inner {
    histograms: BTreeMap<OpKind, Histogram<u64>>,
    errors: BTreeMap<ErrorKind, u64>,
    total_ops: u64,
    total_errors: u64,
}

impl MetricsAccumulator {
    /// Build a fresh accumulator with empty histograms for every [`OpKind`].
    ///
    /// # Panics
    /// Never. The histogram bounds (1µs..60s, 3 sig-digits) are statically
    /// valid for [`Histogram::new_with_bounds`], so the `expect` is a
    /// compile-time-class invariant.
    #[must_use]
    pub fn new() -> Self {
        let mut histograms = BTreeMap::new();
        for k in [OpKind::List, OpKind::Get, OpKind::Patch] {
            let h = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).expect("valid hdr bounds");
            histograms.insert(k, h);
        }
        Self {
            inner: Mutex::new(Inner {
                histograms,
                errors: BTreeMap::new(),
                total_ops: 0,
                total_errors: 0,
            }),
        }
    }

    /// Record a single operation's latency in microseconds. Saturates at
    /// histogram bounds rather than panicking.
    pub fn record(&self, op: OpKind, latency_us: u64) {
        let mut g = self.inner.lock();
        g.total_ops += 1;
        if let Some(h) = g.histograms.get_mut(&op) {
            h.saturating_record(latency_us.max(1));
        }
    }

    /// Increment the error counter for `kind`. Does not record a latency —
    /// callers still record latency if the failing op had one.
    pub fn record_error(&self, kind: ErrorKind) {
        let mut g = self.inner.lock();
        g.total_errors += 1;
        *g.errors.entry(kind).or_insert(0) += 1;
    }

    /// Snapshot totals for callers that just want the numbers.
    #[must_use]
    pub fn totals(&self) -> (u64, u64) {
        let g = self.inner.lock();
        (g.total_ops, g.total_errors)
    }

    /// Render the markdown summary table.
    #[must_use]
    pub fn render_markdown(&self, header: &SummaryHeader<'_>) -> String {
        let g = self.inner.lock();
        let total = g.total_ops;
        let errors = g.total_errors;
        let err_pct = if total == 0 {
            0.0_f64
        } else {
            // Precision loss from u64->f64 is acceptable: we only display
            // two decimals and total/errors are bounded by run duration.
            #[allow(clippy::cast_precision_loss)]
            let p = (errors as f64) * 100.0 / (total as f64);
            p
        };

        let mut out = String::new();
        out.push_str("## Swarm summary\n");
        let _ = writeln!(
            out,
            "Clients: {}    Duration: {}s    Mode: {}    Target: {}",
            header.clients, header.duration_sec, header.mode, header.target
        );
        let _ = writeln!(
            out,
            "Total ops: {total}    Error rate: {err_pct:.2}% ({errors}/{total})\n"
        );

        out.push_str("| Op     | Count     | p50     | p95     | p99     | Max     |\n");
        out.push_str("|--------|-----------|---------|---------|---------|---------|\n");
        for (kind, h) in &g.histograms {
            let count = h.len();
            if count == 0 {
                let _ = writeln!(
                    out,
                    "| {:<6} | {:>9} | {:>7} | {:>7} | {:>7} | {:>7} |",
                    kind.as_str(),
                    0,
                    "-",
                    "-",
                    "-",
                    "-"
                );
                continue;
            }
            let p50 = fmt_us(h.value_at_quantile(0.50));
            let p95 = fmt_us(h.value_at_quantile(0.95));
            let p99 = fmt_us(h.value_at_quantile(0.99));
            let max = fmt_us(h.max());
            let _ = writeln!(
                out,
                "| {:<6} | {:>9} | {:>7} | {:>7} | {:>7} | {:>7} |",
                kind.as_str(),
                count,
                p50,
                p95,
                p99,
                max
            );
        }

        if !g.errors.is_empty() {
            out.push_str("\n### Errors by class\n\n");
            out.push_str("| Class        | Count |\n");
            out.push_str("|--------------|-------|\n");
            for (kind, count) in &g.errors {
                let _ = writeln!(out, "| {:<12} | {count:>5} |", kind.as_str());
            }
        }
        out
    }
}

impl Default for MetricsAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Header fields the markdown summary includes.
pub struct SummaryHeader<'a> {
    /// Number of concurrent client tasks.
    pub clients: usize,
    /// Run duration (seconds).
    pub duration_sec: u64,
    /// Mode name (`sim-direct`, `confluence-direct`, or `contention`).
    pub mode: &'a str,
    /// Target identifier (simulator URL or Confluence base URL).
    pub target: &'a str,
}

/// Format a microsecond count as a concise string (`"0.8ms"`, `"22ms"`,
/// `"1.2s"`, ...).
fn fmt_us(us: u64) -> String {
    if us < 1_000 {
        format!("{us}µs")
    } else if us < 1_000_000 {
        #[allow(clippy::cast_precision_loss)]
        let ms = (us as f64) / 1_000.0;
        if ms < 10.0 {
            format!("{ms:.1}ms")
        } else {
            format!("{ms:.0}ms")
        }
    } else {
        #[allow(clippy::cast_precision_loss)]
        let s = (us as f64) / 1_000_000.0;
        format!("{s:.2}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_reports_percentiles() {
        let m = MetricsAccumulator::new();
        for lat in [100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000] {
            m.record(OpKind::Get, lat);
        }
        let (total, errs) = m.totals();
        assert_eq!(total, 10);
        assert_eq!(errs, 0);
        let md = m.render_markdown(&SummaryHeader {
            clients: 1,
            duration_sec: 1,
            mode: "test",
            target: "x",
        });
        assert!(md.contains("| get"), "{md}");
        assert!(md.contains("Total ops: 10"), "{md}");
    }

    #[test]
    fn classify_common_errors() {
        let e = reposix_core::Error::Other("not found: /x".into());
        assert_eq!(ErrorKind::classify(&e), ErrorKind::NotFound);
        let e = reposix_core::Error::Other("version mismatch: stale".into());
        assert_eq!(ErrorKind::classify(&e), ErrorKind::Conflict);
        let e = reposix_core::Error::Other("rate limited, try again".into());
        assert_eq!(ErrorKind::classify(&e), ErrorKind::RateLimited);
        let e = reposix_core::Error::Other("request timed out".into());
        assert_eq!(ErrorKind::classify(&e), ErrorKind::Timeout);
        let e = reposix_core::Error::Other("boom".into());
        assert_eq!(ErrorKind::classify(&e), ErrorKind::Other);
    }

    #[test]
    fn fmt_us_buckets() {
        assert_eq!(fmt_us(500), "500µs");
        assert!(fmt_us(2_300).ends_with("ms"));
        assert!(fmt_us(20_000).ends_with("ms"));
        assert!(fmt_us(2_500_000).ends_with('s'));
    }
}
