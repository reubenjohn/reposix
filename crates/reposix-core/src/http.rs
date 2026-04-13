//! The one legal HTTP client factory + per-request allowlist gate.
//!
//! Security contract (see `.planning/PROJECT.md` SG-01 and SG-07):
//!
//! - Every outbound HTTP call in this workspace MUST go through
//!   [`client`] to build an [`HttpClient`] and then call one of
//!   [`HttpClient::request`] / [`HttpClient::get`] / [`HttpClient::post`] /
//!   [`HttpClient::patch`] / [`HttpClient::delete`]. The raw
//!   [`reqwest::Client`] is deliberately hidden inside the [`HttpClient`]
//!   newtype with a private `inner` field — callers physically cannot reach
//!   the unchecked send methods.
//! - Direct construction of `reqwest::Client` / `reqwest::ClientBuilder`
//!   is banned by the workspace-root `clippy.toml` `disallowed-methods` lint.
//!   The single legal construction site lives in [`client`] and carries a
//!   `#[allow(clippy::disallowed_methods)]` with justifying comment.
//! - The allowlist is read from `REPOSIX_ALLOWED_ORIGINS` (comma-separated
//!   `scheme://host[:port]` patterns; port may be `*`). Default when unset
//!   or empty is `http://127.0.0.1:*,http://localhost:*`.
//! - Redirects are not followed; callers who wish to follow a `Location`
//!   header MUST re-feed the target through [`HttpClient::request`], at
//!   which point the allowlist gate runs again.

#![allow(clippy::module_name_repetitions)]

use std::time::Duration;

use reqwest::{IntoUrl, Method};
use url::Url;

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
/// Grammar (v0.1 — see 01-CONTEXT.md): each entry is `scheme://host[:port]`
/// where `scheme` is `http` or `https`, `host` is any valid URL host
/// (including bracketed IPv6 literals), and `port` is either a u16 decimal
/// or `*` (any port). Parsing is delegated to the `url` crate so bracketed
/// IPv6 literals (`http://[::1]:7777`) work correctly. Leading/trailing
/// whitespace per entry is trimmed. An empty or all-whitespace input yields
/// the default allowlist.
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
    // Support the "port = *" wildcard by stripping it before URL parsing,
    // since the `url` crate refuses a literal `*` in the port slot.
    let (url_src, wildcard_port) = if let Some(stripped) = entry.strip_suffix(":*") {
        (stripped.to_owned(), true)
    } else {
        (entry.to_owned(), false)
    };

    // Append a trailing `/` so `url::Url::parse` has a complete origin-style
    // URL to chew on. Without it, bare `http://host` still parses, but
    // requiring a path here makes the error message consistent.
    let mut to_parse = url_src;
    if !to_parse.ends_with('/') {
        to_parse.push('/');
    }

    let parsed = Url::parse(&to_parse).map_err(|e| {
        Error::Other(format!(
            "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} failed to parse: {e}"
        ))
    })?;

    let scheme = parsed.scheme().to_owned();
    if scheme != "http" && scheme != "https" {
        return Err(Error::Other(format!(
            "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} scheme must be http or https"
        )));
    }

    let Some(host) = parsed.host_str() else {
        return Err(Error::Other(format!(
            "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} has empty host"
        )));
    };
    if host.is_empty() {
        return Err(Error::Other(format!(
            "REPOSIX_ALLOWED_ORIGINS: entry {entry:?} has empty host"
        )));
    }

    let port = if wildcard_port {
        // Explicit `:*` — caller requested "any port".
        None
    } else {
        // `url::Url::port()` strips a port that matches the scheme's default
        // (e.g. `http://host:80` => `None`). To preserve the user's explicit
        // intent ("http://127.0.0.1:80" means port 80, not wildcard) we fall
        // back to `port_or_known_default()` when `.port()` is `None`. If the
        // entry has no port suffix at all (e.g. `http://host`), neither is
        // present and `port_or_known_default()` yields the scheme default,
        // which is also the correct match semantic.
        parsed.port_or_known_default()
    };

    Ok(OriginGlob {
        scheme,
        host: host.to_owned(),
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

/// Sealed HTTP client wrapper.
///
/// The internal [`reqwest::Client`] is deliberately private: callers have no
/// way to obtain `&reqwest::Client` from an [`HttpClient`] (no `Deref`, no
/// `AsRef`, no `inner_client()`), so they physically cannot invoke
/// `client.get(url).send()` and bypass the allowlist. Every send goes
/// through [`HttpClient::request`] (or one of the method-specific helpers),
/// which re-parses `url` and re-checks it against `REPOSIX_ALLOWED_ORIGINS`.
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
}

impl HttpClient {
    /// Send a `method` request to `url`, re-checking `url` against the
    /// allowlist before any I/O.
    ///
    /// This is the hook callers MUST use after observing a 3xx: re-feed the
    /// `Location` URL through [`HttpClient::request`] so the allowlist
    /// recheck rejects redirect targets that escape the allowlist (SG-01
    /// defence in depth).
    ///
    /// # Errors
    /// Returns [`Error::InvalidOrigin`] if `url` fails to parse or its origin
    /// does not match any allowlist entry. Returns [`Error::Other`] if
    /// `REPOSIX_ALLOWED_ORIGINS` is set but un-parseable. Returns
    /// [`Error::Http`] for transport-level failures from `reqwest`.
    pub async fn request<U: IntoUrl>(&self, method: Method, url: U) -> Result<reqwest::Response> {
        // `IntoUrl::into_url` parses the input into a `url::Url`. We feed
        // that through the allowlist gate before handing it to reqwest.
        let parsed = url
            .into_url()
            .map_err(|e| Error::InvalidOrigin(format!("{e}")))?;
        let allowlist = load_allowlist_from_env()?;
        if !allowlist.iter().any(|g| g.matches(&parsed)) {
            return Err(Error::InvalidOrigin(parsed.to_string()));
        }
        let resp = self.inner.request(method, parsed).send().await?;
        Ok(resp)
    }

    /// Convenience wrapper for `GET url`.
    ///
    /// # Errors
    /// Same as [`HttpClient::request`].
    pub async fn get<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response> {
        self.request(Method::GET, url).await
    }

    /// Convenience wrapper for `POST url`.
    ///
    /// # Errors
    /// Same as [`HttpClient::request`].
    pub async fn post<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response> {
        self.request(Method::POST, url).await
    }

    /// Convenience wrapper for `PATCH url`.
    ///
    /// # Errors
    /// Same as [`HttpClient::request`].
    pub async fn patch<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response> {
        self.request(Method::PATCH, url).await
    }

    /// Convenience wrapper for `DELETE url`.
    ///
    /// # Errors
    /// Same as [`HttpClient::request`].
    pub async fn delete<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response> {
        self.request(Method::DELETE, url).await
    }
}

// Intentionally NOT implemented:
//
//   impl HttpClient { pub fn inner_client(&self) -> &reqwest::Client { ... } }
//   impl AsRef<reqwest::Client> for HttpClient { ... }
//   impl Deref for HttpClient { type Target = reqwest::Client; ... }
//
// These would defeat SG-01: a caller could pull the raw client out and call
// `raw.get(url).send()` to bypass the allowlist gate. The compile-fail
// fixture at `tests/compile-fail/http_client_inner_not_pub.rs` locks this.

/// Build the one-and-only legal HTTP client for this workspace.
///
/// The returned [`HttpClient`] has redirects disabled and a 5-second total
/// timeout (or whatever `opts.total_timeout` is). Callers MUST route every
/// send through [`HttpClient::request`] so the per-request allowlist
/// recheck runs — the factory alone is not a sufficient gate because
/// callers can override the target URL at send time.
///
/// # Errors
/// Returns [`Error::Other`] if `REPOSIX_ALLOWED_ORIGINS` is set but
/// un-parseable, or [`Error::Http`] if `reqwest` itself refuses to build the
/// client (e.g. a TLS-backend initialisation failure).
pub fn client(opts: ClientOpts) -> Result<HttpClient> {
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
    let inner = builder.build()?;
    Ok(HttpClient { inner })
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

    // L-02: IPv6 literal allowlist support.

    #[test]
    fn parse_allowlist_accepts_ipv6_with_explicit_port() {
        let entries = parse_allowlist("http://[::1]:7777").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host, "[::1]");
        assert_eq!(entries[0].port, Some(7777));
    }

    #[test]
    fn parse_allowlist_accepts_ipv6_with_wildcard_port() {
        let entries = parse_allowlist("http://[::1]:*").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host, "[::1]");
        assert_eq!(entries[0].port, None);
    }

    #[test]
    fn origin_glob_matches_ipv6_loopback_any_port() {
        let glob = &parse_allowlist("http://[::1]:*").unwrap()[0];
        let url = Url::parse("http://[::1]:7777/").unwrap();
        assert!(glob.matches(&url));
    }

    #[test]
    fn origin_glob_matches_ipv6_loopback_exact_port() {
        let glob = &parse_allowlist("http://[::1]:7777").unwrap()[0];
        let url = Url::parse("http://[::1]:7777/").unwrap();
        assert!(glob.matches(&url));
    }

    #[test]
    fn origin_glob_ipv6_rejects_wrong_port() {
        let glob = &parse_allowlist("http://[::1]:7777").unwrap()[0];
        let url = Url::parse("http://[::1]:7778/").unwrap();
        assert!(!glob.matches(&url));
    }

    #[test]
    fn parse_allowlist_localhost_wildcard_still_parses() {
        let entries = parse_allowlist("https://localhost:*").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scheme, "https");
        assert_eq!(entries[0].host, "localhost");
        assert_eq!(entries[0].port, None);
    }
}
