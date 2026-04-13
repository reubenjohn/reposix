//! The one legal HTTP client factory + per-request allowlist gate.
//!
//! Security contract (see `.planning/PROJECT.md` SG-01 and SG-07):
//!
//! - Every outbound HTTP call in this workspace MUST go through
//!   [`client`] to build a [`reqwest::Client`] and [`request`] to send.
//! - Direct construction of `reqwest::Client` / `reqwest::ClientBuilder`
//!   is banned by the workspace-root `clippy.toml` `disallowed-methods` lint.
//!   The single legal construction site lives in [`client`] and carries a
//!   `#[allow(clippy::disallowed_methods)]` with justifying comment.
//! - The allowlist is read from `REPOSIX_ALLOWED_ORIGINS` (comma-separated
//!   `scheme://host[:port]` patterns; port may be `*`). Default when unset
//!   or empty is `http://127.0.0.1:*,http://localhost:*`.
//! - Redirects are not followed; callers who wish to follow a `Location`
//!   header MUST re-feed the target through [`request`], at which point the
//!   allowlist gate runs again.

#![allow(clippy::module_name_repetitions)]

use std::time::Duration;

use reqwest::Url;

use crate::error::{Error, Result};

/// Default allowlist when `REPOSIX_ALLOWED_ORIGINS` is unset or empty.
pub const DEFAULT_ALLOWLIST_RAW: &str = "http://127.0.0.1:*,http://localhost:*";

/// Environment variable that, when set, overrides the default allowlist.
pub const ALLOWLIST_ENV_VAR: &str = "REPOSIX_ALLOWED_ORIGINS";

/// Options for constructing an HTTP client. [`Default`] yields the 5-second
/// total-timeout client that ~95% of callers want (SG-07).
#[derive(Debug, Clone)]
pub struct ClientOpts {
    /// Total request timeout. Default `Duration::from_secs(5)`.
    pub total_timeout: Duration,
    /// User-Agent header. Default `Some("reposix/0.1.0")`.
    pub user_agent: Option<String>,
}

impl Default for ClientOpts {
    fn default() -> Self {
        Self {
            total_timeout: Duration::from_secs(5),
            user_agent: Some(concat!("reposix/", env!("CARGO_PKG_VERSION")).to_owned()),
        }
    }
}

/// Parsed allowlist entry. `port == None` means "any port" (`*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OriginGlob {
    scheme: String,
    host: String,
    port: Option<u16>,
}

impl OriginGlob {
    /// Returns true iff `url`'s scheme, host, and port all match this glob.
    pub(crate) fn matches(&self, url: &Url) -> bool {
        if url.scheme() != self.scheme {
            return false;
        }
        let Some(url_host) = url.host_str() else {
            return false;
        };
        if url_host != self.host {
            return false;
        }
        match self.port {
            None => true,
            Some(expected) => url.port_or_known_default() == Some(expected),
        }
    }
}

/// Parse a comma-separated allowlist spec.
///
/// Grammar (v0.1, hand-rolled — see 01-CONTEXT.md discretion): each entry is
/// `scheme://host[:port]` where `scheme` is `http` or `https`, `host` is any
/// non-empty run of chars up to `:` or end-of-entry, and `port` is either a
/// u16 decimal or `*` (any port). Leading/trailing whitespace per entry is
/// trimmed. An empty or all-whitespace input yields the default allowlist.
///
/// # Errors
/// Returns [`Error::Other`] if any entry fails to parse.
pub(crate) fn parse_allowlist(raw: &str) -> Result<Vec<OriginGlob>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return parse_allowlist_inner(DEFAULT_ALLOWLIST_RAW);
    }
    parse_allowlist_inner(trimmed)
}

fn parse_allowlist_inner(raw: &str) -> Result<Vec<OriginGlob>> {
    let mut out = Vec::new();
    for (idx, entry) in raw.split(',').enumerate() {
        let entry = entry.trim();
        if entry.is_empty() {
            return Err(Error::Other(format!(
                "REPOSIX_ALLOWED_ORIGINS: entry {idx} is empty"
            )));
        }
        out.push(parse_one(entry)?);
    }
    Ok(out)
}

fn parse_one(entry: &str) -> Result<OriginGlob> {
    let (scheme, rest) = entry.split_once("://").ok_or_else(|| {
        Error::Other(format!(
            "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} missing scheme://"
        ))
    })?;
    if scheme != "http" && scheme != "https" {
        return Err(Error::Other(format!(
            "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} scheme must be http or https"
        )));
    }
    let (host, port) = if let Some((host, port_raw)) = rest.rsplit_once(':') {
        if host.is_empty() {
            return Err(Error::Other(format!(
                "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} has empty host"
            )));
        }
        let port = if port_raw == "*" {
            None
        } else {
            let parsed = port_raw.parse::<u16>().map_err(|_| {
                Error::Other(format!(
                    "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} has invalid port {port_raw:?}"
                ))
            })?;
            Some(parsed)
        };
        (host.to_owned(), port)
    } else {
        if rest.is_empty() {
            return Err(Error::Other(format!(
                "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} has empty host"
            )));
        }
        (rest.to_owned(), None)
    };
    Ok(OriginGlob {
        scheme: scheme.to_owned(),
        host,
        port,
    })
}

/// Load the allowlist from `REPOSIX_ALLOWED_ORIGINS`, falling back to the
/// loopback-only default when unset/empty.
///
/// # Errors
/// Returns [`Error::Other`] if the env var is set but un-parseable.
pub(crate) fn load_allowlist_from_env() -> Result<Vec<OriginGlob>> {
    match std::env::var(ALLOWLIST_ENV_VAR) {
        Ok(v) => parse_allowlist(&v),
        Err(_) => parse_allowlist(""),
    }
}

/// Build the one-and-only legal HTTP client for this workspace.
///
/// The returned client has redirects disabled and a 5-second total timeout
/// (or whatever `opts.total_timeout` is). Callers MUST route every send
/// through [`request`] so the per-request allowlist recheck runs — the
/// factory alone is not a sufficient gate because callers can override the
/// target URL at send time.
///
/// # Errors
/// Returns [`Error::Other`] if `REPOSIX_ALLOWED_ORIGINS` is set but
/// un-parseable, or [`Error::Http`] if `reqwest` itself refuses to build the
/// client (e.g. a TLS-backend initialisation failure).
pub fn client(opts: ClientOpts) -> Result<reqwest::Client> {
    // Surface allowlist-parse errors at construction time so misconfigured
    // operators fail loudly rather than silently.
    let _ = load_allowlist_from_env()?;

    // SG-01: this `#[allow]` marks the single legal construction site in the
    // workspace. Any other construction of `reqwest::Client` / `ClientBuilder`
    // is rejected by the workspace-root `clippy.toml` `disallowed-methods`
    // lint. Do not remove this comment without updating that lint.
    #[allow(clippy::disallowed_methods)]
    let mut builder = reqwest::ClientBuilder::new();

    builder = builder
        .redirect(reqwest::redirect::Policy::none())
        .timeout(opts.total_timeout);
    if let Some(ua) = opts.user_agent {
        builder = builder.user_agent(ua);
    }
    let client = builder.build()?;
    Ok(client)
}

/// Send a request through `client`, re-checking `url` against the allowlist
/// before any I/O.
///
/// This is the hook callers MUST use after observing a 3xx: re-feed the
/// `Location` URL through [`request`] so the allowlist recheck rejects
/// redirect targets that escape the allowlist (SG-01 defence in depth).
///
/// # Errors
/// Returns [`Error::InvalidOrigin`] if `url` fails to parse or its origin
/// does not match any allowlist entry. Returns [`Error::Other`] if
/// `REPOSIX_ALLOWED_ORIGINS` is set but un-parseable. Returns
/// [`Error::Http`] for transport-level failures from `reqwest`.
pub async fn request(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
) -> Result<reqwest::Response> {
    let parsed = Url::parse(url).map_err(|_| Error::InvalidOrigin(url.to_owned()))?;
    let allowlist = load_allowlist_from_env()?;
    if !allowlist.iter().any(|g| g.matches(&parsed)) {
        return Err(Error::InvalidOrigin(url.to_owned()));
    }
    let resp = client.request(method, parsed).send().await?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_opts_default_is_5s_timeout() {
        let opts = ClientOpts::default();
        assert_eq!(opts.total_timeout, Duration::from_secs(5));
        assert!(opts.user_agent.as_deref().unwrap().starts_with("reposix/"));
    }

    #[test]
    fn parse_allowlist_default_has_two_entries() {
        let entries = parse_allowlist("http://127.0.0.1:*,http://localhost:*").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn parse_allowlist_empty_input_returns_default() {
        let entries = parse_allowlist("").unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|g| g.host == "127.0.0.1"));
        assert!(entries.iter().any(|g| g.host == "localhost"));
    }

    #[test]
    fn parse_allowlist_whitespace_only_returns_default() {
        let entries = parse_allowlist("   \t  ").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn parse_allowlist_bad_input_errors() {
        let err = parse_allowlist("not a url").unwrap_err();
        assert!(matches!(err, Error::Other(_)), "got {err:?}");
    }

    #[test]
    fn parse_allowlist_bad_scheme_errors() {
        assert!(matches!(
            parse_allowlist("ftp://127.0.0.1:*"),
            Err(Error::Other(_))
        ));
    }

    #[test]
    fn parse_allowlist_empty_host_errors() {
        assert!(matches!(
            parse_allowlist("http://:80"),
            Err(Error::Other(_))
        ));
    }

    #[test]
    fn parse_allowlist_bad_port_errors() {
        assert!(matches!(
            parse_allowlist("http://127.0.0.1:notaport"),
            Err(Error::Other(_))
        ));
    }

    #[test]
    fn origin_glob_matches_loopback_any_port() {
        let glob = &parse_allowlist("http://127.0.0.1:*").unwrap()[0];
        let url = Url::parse("http://127.0.0.1:7878").unwrap();
        assert!(glob.matches(&url));
    }

    #[test]
    fn origin_glob_rejects_https_when_http_configured() {
        let glob = &parse_allowlist("http://127.0.0.1:*").unwrap()[0];
        let url = Url::parse("https://127.0.0.1:7878").unwrap();
        assert!(!glob.matches(&url));
    }

    #[test]
    fn origin_glob_rejects_non_loopback_host() {
        let glob = &parse_allowlist("http://127.0.0.1:*").unwrap()[0];
        let url = Url::parse("http://evil.example:80").unwrap();
        assert!(!glob.matches(&url));
    }

    #[test]
    fn origin_glob_matches_exact_port() {
        let glob = &parse_allowlist("http://127.0.0.1:80").unwrap()[0];
        let url = Url::parse("http://127.0.0.1:80").unwrap();
        assert!(glob.matches(&url));
    }

    #[test]
    fn origin_glob_rejects_wrong_exact_port() {
        let glob = &parse_allowlist("http://127.0.0.1:80").unwrap()[0];
        let url = Url::parse("http://127.0.0.1:81").unwrap();
        assert!(!glob.matches(&url));
    }
}
