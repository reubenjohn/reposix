← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T02 — `bus_url.rs` parser module + 4 unit tests

<read_first>
- `crates/reposix-remote/src/backend_dispatch.rs` lines 1-200 (entire
  `parse_remote_url` body — confirm it accepts both `reposix::...`
  prefixed AND bare forms; the bus_url parser passes the
  prefix-stripped + query-stripped form back).
- `crates/reposix-core/src/remote.rs` (entire file ~50 lines) —
  `split_reposix_url` + `strip_reposix_prefix` (confirm name; if
  not exposed, manual `url.strip_prefix("reposix::").unwrap_or(url)`).
- `crates/reposix-remote/src/main.rs` lines 24-32 (existing module
  declarations — `mod bus_url;` joins alphabetically between
  `mod backend_dispatch;` line 24 and the next mod).
- `crates/reposix-remote/src/main.rs` line 113 (current `parse_dispatch_url`
  call site — replaced in T04 with `bus_url::parse(...)`).
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md` § Pattern 1
  (parser pseudocode).
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md` § Pitfall 2
  (query-string strip BEFORE delegating to parse_remote_url) +
  Pitfall 7 (mirror URL with embedded ?).
- `crates/reposix-remote/Cargo.toml` `[dependencies]` — confirm
  `anyhow` is present at the binary root.
</read_first>

<action>
Two concerns: new module → unit tests → cargo check + commit.

### 2a. New module — `crates/reposix-remote/src/bus_url.rs`

Author the new file. Estimated 100-130 lines including module-doc,
the `Route` enum, the `parse` function, and the inline tests.

```rust
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
    /// `sot` is the SoT side (instantiated as a `BackendConnector` via
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

    // Reject the `+`-delimited bus form (Q3.3).
    //
    // Heuristic: a `+` BEFORE the query-string boundary is the
    // dropped form. A `+` inside `?mirror=<...>` could be a legitimate
    // URL-escape (`+` = space in form-urlencoded). We split on `?`
    // FIRST and only check `+` in the base form.
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
    // backend_dispatch::parse_remote_url accepts both `reposix::...`
    // and bare forms — pass the prefix-stripped base.
    let parsed = parse_remote_url(base)
        .map_err(|e| anyhow!("bus URL base form rejected: {e:#}"))?;

    let Some(query) = query else {
        return Ok(Route::Single(parsed));
    };

    // Empty query string after `?` — explicit error (typo or
    // misuse).
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

    Ok(Route::Bus { sot: parsed, mirror_url })
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
        assert!(msg.contains("+"), "expected reject message to mention `+`: {msg}");
        assert!(msg.contains("?mirror="), "expected reject message to suggest `?mirror=`: {msg}");
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
        assert!(msg.contains("only `mirror=`"), "expected reject message to cite `mirror=` as the only supported key: {msg}");
    }
}
```

### 2b. `mod bus_url;` declaration — `crates/reposix-remote/src/main.rs`

Add `mod bus_url;` to the top-of-file mod declarations (alphabetical
placement — between `mod backend_dispatch;` at line 24 and the next
mod):

```rust
mod backend_dispatch;
mod bus_url;          // NEW
mod diff;
mod fast_import;
mod pktline;
mod precheck;
mod protocol;
mod stateless_connect;
```

(The exact alphabetical neighbors must be re-confirmed during T02
read_first via `head -40 crates/reposix-remote/src/main.rs`. Do
NOT touch the `bus_handler` module yet — that lands in T04.)

Build serially (per-crate per CLAUDE.md "Build memory budget"):

```bash
cargo check -p reposix-remote
cargo clippy -p reposix-remote -- -D warnings
cargo nextest run -p reposix-remote bus_url
```

If `cargo clippy` fires `clippy::pedantic` lints, fix inline; do
NOT add `#[allow(...)]` without rationale. The `parse` function must
have a `# Errors` doc.

### 2c. Stage and commit

```bash
git add crates/reposix-remote/src/bus_url.rs \
        crates/reposix-remote/src/main.rs
git commit -m "$(cat <<'EOF'
feat(remote): bus URL parser — bus_url::parse + Route::Single|Bus enum (DVCS-BUS-URL-01)

- crates/reposix-remote/src/bus_url.rs (new) — pub(crate) enum Route { Single(ParsedRemote), Bus { sot: ParsedRemote, mirror_url: String } } + pub(crate) fn parse(url: &str) -> Result<Route>
- Strips optional `?<query>` BEFORE delegating to backend_dispatch::parse_remote_url (RESEARCH.md Pitfall 2 — existing splitter rejects `?` in project segment)
- Rejects `+`-delimited bus form per Q3.3 with hint citing `?mirror=`
- Rejects unknown query keys per Q-C / D-03 with verbatim "unknown query parameter `<key>`" hint
- Manual query parser (`split('&')` + `split_once('=')`) — RESEARCH.md A3 fallback to avoid `url` crate's strict scheme requirement against `reposix::` form
- 4 unit tests inline: parses_query_param_form_round_trip + route_single_for_bare_reposix_url + rejects_plus_delimited_bus_url + rejects_unknown_query_param
- crates/reposix-remote/src/main.rs — mod bus_url; declaration (alphabetical)

Phase 82 / Plan 01 / Task 02 / DVCS-BUS-URL-01.
EOF
)"
```
</action>

<verify>
  <automated>cargo check -p reposix-remote && cargo clippy -p reposix-remote -- -D warnings && cargo nextest run -p reposix-remote bus_url && grep -q "pub(crate) enum Route" crates/reposix-remote/src/bus_url.rs && grep -q "mod bus_url" crates/reposix-remote/src/main.rs</automated>
</verify>

<done>
- `crates/reposix-remote/src/bus_url.rs` exists with `pub(crate) enum
  Route { Single(ParsedRemote), Bus { sot, mirror_url } }` + `pub(crate)
  fn parse(url) -> Result<Route>`.
- 4 unit tests pass (`cargo nextest run -p reposix-remote bus_url`).
- Module-doc names Q3.3 RATIFIED, the `+`-rejection rule, and Q-C
  unknown-key rejection.
- `parse` strips optional `?<query>` BEFORE delegating to
  `backend_dispatch::parse_remote_url` (Pitfall 2 mitigation).
- `crates/reposix-remote/src/main.rs` declares `mod bus_url;`.
- `cargo check -p reposix-remote` exits 0.
- `cargo clippy -p reposix-remote -- -D warnings` exits 0.
- `# Errors` doc on `parse`.
- Cargo serialized: T02 cargo invocations run only after T01's commit
  has landed; per-crate fallback used.
</done>

---

