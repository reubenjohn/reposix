//! Bus URL parser — recognizes `reposix::<sot-spec>?mirror=<mirror-url>`
//! per `decisions.md` Q3.3 RATIFIED query-param form.
//!
//! Single-backend `reposix::<sot-spec>` URLs continue to work via the
//! existing `backend_dispatch::parse_remote_url` code path (the bus
//! parser is a SIBLING, not a replacement — it strips the optional
//! `?<query>` segment and delegates the base form to the existing
//! parser).
//!
//! ## Rejected forms
//!
//! - `reposix::<sot>+<mirror>` — the `+`-delimited form is dropped
//!   per Q3.3. The parser rejects it with a hint citing `?mirror=`
//!   as the canonical form.
//! - `reposix::<sot>?<key>=<value>` where `<key>` != `mirror` — only
//!   `mirror=` is recognized per Q-C / D-03 of the plan body.
//!   Rejecting unknown keys is forward-compat-via-explicit-opt-in:
//!   future params (`priority=`, `retry=`, `mirror_name=`) opt in
//!   without legacy-key compatibility debt.
//! - `reposix::<sot>?` (empty query string) — error: missing `mirror=`.
//!
//! ## Mirror URL with embedded `?`
//!
//! If the mirror URL itself contains `?` (e.g.,
//! `https://gh.example/foo?token=secret`), the user MUST percent-encode
//! the value. The first unescaped `?` in the bus URL is the bus
//! query-string boundary. CLAUDE.md § Architecture documents this
//! requirement.
//!
//! ## Trust boundary
//!
//! `mirror_url` is user-controlled (from argv[2]). The parser
//! produces a `Route` enum; the `mirror_url` field is consumed by
//! `bus_handler::precheck_mirror_drift` which mitigates argument
//! injection (T-82-01) via `--` separator + reject-`-` prefix.

use anyhow::{anyhow, Result};

use crate::backend_dispatch::{parse_remote_url, ParsedRemote};

/// Bus-vs-single-backend dispatch route. Returned by [`parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Route {
    /// Single-backend URL — `reposix::<sot-spec>` (no `?mirror=`).
    /// Carries the existing `ParsedRemote` shape verbatim; the helper
    /// continues to call `instantiate(&parsed)` + `handle_export` per
    /// the v0.9.0 contract.
    Single(ParsedRemote),
    /// Bus URL — `reposix::<sot-spec>?mirror=<mirror-url>` per Q3.3.
    /// `sot` is the `SoT` side (instantiated as a `BackendConnector` via
    /// the same `instantiate` path single-backend uses); `mirror_url`
    /// is the plain-git mirror remote URL (consumed as a shell-out
    /// argument by `bus_handler::precheck_mirror_drift`).
    Bus {
        sot: ParsedRemote,
        mirror_url: String,
    },
}

/// Parse a `reposix::...` remote URL into a [`Route`].
///
/// # Errors
/// - The URL contains `+` (the dropped `+`-delimited form).
/// - The URL has a query string but no `mirror=` parameter.
/// - The URL has a query parameter other than `mirror=` (Q-C reject).
/// - The base form (after query-strip) is rejected by
///   [`backend_dispatch::parse_remote_url`].
pub(crate) fn parse(url: &str) -> Result<Route> {
    // Strip the `reposix::` prefix if present. git typically strips it
    // before invoking the helper, but assert_cmd test harnesses pass
    // it verbatim.
    let stripped = url.strip_prefix("reposix::").unwrap_or(url);

    // Split on `?` FIRST. The base form is everything up to the first
    // unescaped `?`; the query string is everything after. The
    // `+`-delimited form is detected by `+` in the BASE form (a `+`
    // inside `?mirror=<...>` is a legitimate URL-escape per
    // form-urlencoded).
    let (base, query) = match stripped.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (stripped, None),
    };
    if base.contains('+') {
        return Err(anyhow!(
            "the `+`-delimited bus URL form is dropped (Q3.3) — \
             use `reposix::<sot-spec>?mirror=<mirror-url>` instead"
        ));
    }

    // Delegate the base form (no query string) to the existing parser.
    // `backend_dispatch::parse_remote_url` accepts both `reposix::...`
    // and bare forms — pass the prefix-stripped base.
    let parsed =
        parse_remote_url(base).map_err(|e| anyhow!("bus URL base form rejected: {e:#}"))?;

    let Some(query) = query else {
        return Ok(Route::Single(parsed));
    };

    // Empty query string after `?` — explicit error (typo or misuse).
    if query.is_empty() {
        return Err(anyhow!(
            "bus URL query string is empty; expected \
             `reposix::<sot-spec>?mirror=<mirror-url>`"
        ));
    }

    // Parse query manually. RESEARCH.md A3 (MEDIUM): the `url` crate's
    // strict scheme requirement may not handle `reposix::` cleanly,
    // so manual splitting is safer (~10 lines, byte-exact).
    let pairs: Vec<(&str, &str)> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| match p.split_once('=') {
            Some((k, v)) => (k, v),
            None => (p, ""), // `?mirror` (no `=`) → empty value
        })
        .collect();

    // Look up `mirror=`. Reject unknown keys per Q-C / D-03.
    let mut mirror_url: Option<String> = None;
    for (k, v) in &pairs {
        match *k {
            "mirror" => {
                mirror_url = Some((*v).to_owned());
            }
            other => {
                return Err(anyhow!(
                    "unknown query parameter `{other}` in bus URL; \
                     only `mirror=` is supported"
                ));
            }
        }
    }

    let Some(mirror_url) = mirror_url else {
        return Err(anyhow!(
            "bus URL query string present but `mirror=` parameter missing; \
             expected `reposix::<sot-spec>?mirror=<mirror-url>`"
        ));
    };

    if mirror_url.is_empty() {
        return Err(anyhow!(
            "bus URL `mirror=` parameter is empty; \
             expected `reposix::<sot-spec>?mirror=<mirror-url>`"
        ));
    }

    Ok(Route::Bus {
        sot: parsed,
        mirror_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DVCS-BUS-URL-01 — `?mirror=` parses to Route::Bus with the
    /// expected SoT + mirror split.
    #[test]
    fn parses_query_param_form_round_trip() {
        let url = "reposix::http://127.0.0.1:7878/projects/demo?mirror=file:///tmp/m.git";
        let route = parse(url).expect("bus URL should parse");
        match route {
            Route::Bus { sot, mirror_url } => {
                assert_eq!(sot.project, "demo");
                assert_eq!(mirror_url, "file:///tmp/m.git");
            }
            other => panic!("expected Route::Bus; got {other:?}"),
        }
    }

    /// DVCS-BUS-URL-01 — bare `reposix::<sot>` (no `?`) returns
    /// Route::Single (single-backend regression check).
    #[test]
    fn route_single_for_bare_reposix_url() {
        let url = "reposix::http://127.0.0.1:7878/projects/demo";
        let route = parse(url).expect("bare URL should parse");
        match route {
            Route::Single(sot) => assert_eq!(sot.project, "demo"),
            other => panic!("expected Route::Single; got {other:?}"),
        }
    }

    /// Q3.3 + DVCS-BUS-URL-01 — `+`-delimited form rejected with
    /// hint citing `?mirror=`.
    #[test]
    fn rejects_plus_delimited_bus_url() {
        // The `+` is before any `?`, so the base form contains `+`
        // and the parser rejects.
        let url = "reposix::http://127.0.0.1:7878/projects/demo+file:///tmp/m.git";
        let err = parse(url).expect_err("`+` form should reject");
        let msg = format!("{err:#}");
        assert!(
            msg.contains('+'),
            "expected reject message to mention `+`: {msg}"
        );
        assert!(
            msg.contains("?mirror="),
            "expected reject message to suggest `?mirror=`: {msg}"
        );
    }

    /// Q-C / D-03 — unknown query parameter rejected with verbatim
    /// "unknown query parameter `<key>`" hint.
    #[test]
    fn rejects_unknown_query_param() {
        let url = "reposix::http://127.0.0.1:7878/projects/demo?priority=high";
        let err = parse(url).expect_err("unknown query key should reject");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("unknown query parameter") && msg.contains("priority"),
            "expected reject message to name the unknown key: {msg}"
        );
        assert!(
            msg.contains("only `mirror=`"),
            "expected reject message to cite `mirror=` as the only supported key: {msg}"
        );
    }
}
