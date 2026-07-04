//! Mirror-egress allowlist gate for bus pushes (QL-006).
//!
//! ## Why this exists
//!
//! A bus URL is `reposix::<sot>?mirror=<mirror-url>`. The `<mirror-url>`
//! is user-controlled (it lands in `.git/config` via `reposix attach` /
//! `reposix init`, or is hand-written). On a bus push the helper shells
//! out `git ls-remote <mirror-url>` (PRECHECK A) and `git push <remote>
//! main` (STEP 6) — **both send issue content to whatever host the mirror
//! URL names.** That is a second egress channel alongside the REST writes
//! to the `SoT`, and the threat model (CLAUDE.md § Threat model) requires
//! *every* outbound channel that can carry tainted issue content to be
//! gated by `REPOSIX_ALLOWED_ORIGINS`. Before this module, the mirror
//! push bypassed the allowlist entirely: tainted `SoT` content could be
//! pushed to an arbitrary attacker-named mirror.
//!
//! ## The check: host-level, not scheme+port
//!
//! The `reposix_core::http` HTTP client gates on scheme + host + port
//! because it governs one concrete transport. The mirror push is a
//! *different* transport — commonly `ssh` (`git@github.com:org/repo.git`)
//! or `https`, sometimes a bare local path — and the HTTP allowlist
//! grammar cannot even express `ssh`. So the mirror gate matches on
//! **host** alone, via [`reposix_core::http::allowlisted_hosts`]. The
//! destination host is the exfiltration boundary; the wire protocol used
//! to reach it is not. An operator who set
//! `REPOSIX_ALLOWED_ORIGINS='https://github.com'` has already declared
//! `github.com` a sanctioned place for issue content to flow to, whether
//! the mirror reaches it over `https` or `ssh`.
//!
//! **Consequence for operators:** the allowlist grammar stays http/https
//! (an `ssh://` entry would be rejected by the parser). To authorise an
//! `ssh` mirror, add an `https://<host>` entry — the gate matches on host,
//! so the scheme of the allowlist entry is irrelevant. The teaching error
//! spells this out.
//!
//! ## Local mirrors are exempt (documented decision)
//!
//! `file://…` URLs and local filesystem paths (`/abs/path`, `./rel`,
//! `../rel`, `~/path`) are treated as [`MirrorOrigin::Local`] and skip the
//! allowlist: pushing to a path on the same machine performs **no network
//! egress**, so there is nothing for the allowlist to protect against.
//! This is also what every existing bus test uses (`file://` bare repos),
//! so the gate is invisible to the local-mirror happy path.

use anyhow::Result;

/// How a mirror URL maps onto the egress-check origin space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MirrorOrigin {
    /// A local filesystem mirror (`file://` or a bare path). No network
    /// egress leaves the machine → the allowlist does not apply.
    Local,
    /// A network mirror. `host` is the destination host used for the
    /// allowlist match; `display` is the human-readable origin string
    /// (`scheme://host[:port]`) shown in diagnostics.
    Network { host: String, display: String },
}

/// Classify a mirror URL into a [`MirrorOrigin`].
///
/// Recognised forms:
/// - `file://…`, `/abs`, `./rel`, `../rel`, `~/path` → [`MirrorOrigin::Local`].
/// - `http://` / `https://` / `git://` / `ssh://` → `Network { scheme://host[:port] }`.
/// - scp-like `[user@]host:path` (e.g. `git@github.com:org/repo.git`) →
///   `Network { ssh://host }`.
///
/// A single-letter drive prefix (`C:\…`) is treated as a Windows local
/// path, not an scp target.
pub(crate) fn classify_mirror_url(url: &str) -> MirrorOrigin {
    // Local filesystem paths — no egress.
    if url.starts_with('/')
        || url.starts_with("./")
        || url.starts_with("../")
        || url.starts_with("~/")
        || url == "."
        || url == ".."
    {
        return MirrorOrigin::Local;
    }

    if url.contains("://") {
        // Has an explicit `scheme://` — parse scheme + authority.
        let (scheme, after) = url.split_once("://").expect("contains checked above");
        let scheme = scheme.to_ascii_lowercase();
        if scheme == "file" {
            return MirrorOrigin::Local;
        }
        // authority is everything up to the first `/`, `?`, or `#`.
        let authority = after
            .split(['/', '?', '#'])
            .next()
            .unwrap_or("")
            .to_string();
        // Strip optional `user[:pass]@` userinfo.
        let hostport = authority.rsplit('@').next().unwrap_or(&authority);
        let (host, port) = split_host_port(hostport);
        let host = host.to_ascii_lowercase();
        let display = match port {
            Some(p) => format!("{scheme}://{host}:{p}"),
            None => format!("{scheme}://{host}"),
        };
        return MirrorOrigin::Network { host, display };
    }

    // No `://`. Could be scp-like `[user@]host:path`, or a bare relative
    // local path with no colon.
    if let Some((before_colon, _after)) = url.split_once(':') {
        // Windows drive letter (`C:\…`) → local path, not scp.
        let is_drive = before_colon.len() == 1
            && before_colon
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic());
        if !is_drive {
            let hostpart = before_colon.rsplit('@').next().unwrap_or(before_colon);
            let host = hostpart.to_ascii_lowercase();
            let display = format!("ssh://{host}");
            return MirrorOrigin::Network { host, display };
        }
    }

    // Bare path with no scheme and no host:path colon → local.
    MirrorOrigin::Local
}

/// Split a `host[:port]` authority, preserving bracketed IPv6 literals
/// (`[::1]:22`). Returns `(host, port_opt)` where host keeps its brackets.
fn split_host_port(hostport: &str) -> (&str, Option<u16>) {
    if let Some(rest) = hostport.strip_prefix('[') {
        // IPv6 literal: `[addr]` or `[addr]:port`.
        if let Some((addr, tail)) = rest.split_once(']') {
            let port = tail.strip_prefix(':').and_then(|p| p.parse::<u16>().ok());
            // Re-borrow the bracketed host from the original slice.
            let end = 1 + addr.len() + 1; // '[' + addr + ']'
            return (&hostport[..end], port);
        }
        return (hostport, None);
    }
    match hostport.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().ok()),
        None => (hostport, None),
    }
}

/// A rejected mirror-egress decision. Carries the origin that failed and
/// renders a recovery-teaching message.
#[derive(Debug, Clone)]
pub(crate) struct MirrorEgressDenied {
    /// Human-readable origin (`scheme://host[:port]`) that was rejected.
    pub(crate) origin: String,
    /// Just the host, for the `https://<host>` fix suggestion.
    host: String,
}

impl MirrorEgressDenied {
    /// A multi-line, recovery-teaching diagnostic naming the env var, the
    /// rejected origin, and the exact `export` line to authorise it.
    pub(crate) fn teaching_message(&self) -> String {
        let current = std::env::var(reposix_core::http::ALLOWLIST_ENV_VAR).unwrap_or_default();
        let fix = if current.is_empty() {
            format!("https://{}", self.host)
        } else {
            format!("{current},https://{}", self.host)
        };
        format!(
            "mirror push blocked: origin `{origin}` is not authorised by \
             {env}.\nThe mirror push sends issue content over the network, so the \
             mirror host must be on the egress allowlist (the allowlist matches on \
             HOST, so an `https://<host>` entry authorises an `ssh` mirror on the same \
             host).\nTo authorise it:\n  export {env}='{fix}'",
            origin = self.origin,
            env = reposix_core::http::ALLOWLIST_ENV_VAR,
        )
    }
}

/// Gate a mirror URL against `REPOSIX_ALLOWED_ORIGINS` (host-level).
///
/// Local mirrors (`file://`, bare paths) are always allowed — no egress.
/// Network mirrors are allowed iff their host appears among
/// [`reposix_core::http::allowlisted_hosts`].
///
/// # Errors
/// Returns [`anyhow::Error`] only if `REPOSIX_ALLOWED_ORIGINS` is set but
/// un-parseable (propagated from `allowlisted_hosts`). A *denied* origin
/// is `Ok(Err(MirrorEgressDenied))`, not an error — the caller renders the
/// teaching message and rejects the push cleanly.
pub(crate) fn check_mirror_allowed(
    url: &str,
) -> Result<std::result::Result<(), MirrorEgressDenied>> {
    match classify_mirror_url(url) {
        MirrorOrigin::Local => Ok(Ok(())),
        MirrorOrigin::Network { host, display } => {
            let hosts = reposix_core::http::allowlisted_hosts()?;
            if hosts.iter().any(|h| h == &host) {
                Ok(Ok(()))
            } else {
                Ok(Err(MirrorEgressDenied {
                    origin: display,
                    host,
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net(url: &str) -> (String, String) {
        match classify_mirror_url(url) {
            MirrorOrigin::Network { host, display } => (host, display),
            MirrorOrigin::Local => panic!("expected Network for {url}"),
        }
    }

    #[test]
    fn file_url_is_local() {
        assert_eq!(
            classify_mirror_url("file:///tmp/m.git"),
            MirrorOrigin::Local
        );
    }

    #[test]
    fn absolute_and_relative_paths_are_local() {
        assert_eq!(classify_mirror_url("/tmp/m.git"), MirrorOrigin::Local);
        assert_eq!(classify_mirror_url("./m.git"), MirrorOrigin::Local);
        assert_eq!(classify_mirror_url("../m.git"), MirrorOrigin::Local);
        assert_eq!(classify_mirror_url("~/m.git"), MirrorOrigin::Local);
    }

    #[test]
    fn windows_drive_path_is_local() {
        assert_eq!(classify_mirror_url(r"C:\repos\m.git"), MirrorOrigin::Local);
    }

    #[test]
    fn https_maps_to_scheme_host_port() {
        let (host, display) = net("https://github.com/org/repo.git");
        assert_eq!(host, "github.com");
        assert_eq!(display, "https://github.com");
        let (host, display) = net("https://git.internal:8443/x.git");
        assert_eq!(host, "git.internal");
        assert_eq!(display, "https://git.internal:8443");
    }

    #[test]
    fn https_strips_userinfo() {
        let (host, display) = net("https://user@github.com/org/repo.git");
        assert_eq!(host, "github.com");
        assert_eq!(display, "https://github.com");
    }

    #[test]
    fn scp_form_maps_to_ssh_host() {
        let (host, display) = net("git@github.com:org/repo.git");
        assert_eq!(host, "github.com");
        assert_eq!(display, "ssh://github.com");
    }

    #[test]
    fn ssh_scheme_maps_to_ssh_host_port() {
        let (host, display) = net("ssh://git@github.com:2222/org/repo.git");
        assert_eq!(host, "github.com");
        assert_eq!(display, "ssh://github.com:2222");
    }

    #[test]
    fn ipv6_ssh_scheme_keeps_brackets() {
        let (host, display) = net("ssh://git@[::1]:2222/x.git");
        assert_eq!(host, "[::1]");
        assert_eq!(display, "ssh://[::1]:2222");
    }

    #[test]
    fn host_matching_is_case_insensitive() {
        let (host, _) = net("https://GitHub.COM/x.git");
        assert_eq!(host, "github.com");
    }

    // ── Allowlist decision (env-var driven) ─────────────────────────────
    // These mutate REPOSIX_ALLOWED_ORIGINS; keep them in one test to avoid
    // cross-test env races (Rust runs tests in-process, parallel).

    #[test]
    fn allow_deny_decision_paths() {
        let saved = std::env::var(reposix_core::http::ALLOWLIST_ENV_VAR).ok();

        // Local mirror: allowed regardless of allowlist.
        std::env::set_var(reposix_core::http::ALLOWLIST_ENV_VAR, "https://github.com");
        assert!(check_mirror_allowed("file:///tmp/m.git").unwrap().is_ok());
        assert!(check_mirror_allowed("/tmp/m.git").unwrap().is_ok());

        // ssh mirror on an allowlisted host (via https:// entry) → allowed.
        assert!(check_mirror_allowed("git@github.com:org/repo.git")
            .unwrap()
            .is_ok());
        // https mirror on the allowlisted host → allowed.
        assert!(check_mirror_allowed("https://github.com/org/repo.git")
            .unwrap()
            .is_ok());

        // Non-allowlisted host → denied, with a teaching message.
        let denied = check_mirror_allowed("https://evil.example.com/x.git")
            .unwrap()
            .expect_err("evil host must be denied");
        assert_eq!(denied.origin, "https://evil.example.com");
        let msg = denied.teaching_message();
        assert!(msg.contains("REPOSIX_ALLOWED_ORIGINS"), "msg: {msg}");
        assert!(msg.contains("https://evil.example.com"), "msg: {msg}");
        assert!(
            msg.contains("export REPOSIX_ALLOWED_ORIGINS="),
            "teaching msg must show the export line: {msg}"
        );
        assert!(
            msg.contains("https://evil.example.com") && msg.contains(",https://evil.example.com"),
            "fix line must append the rejected host to the current allowlist: {msg}"
        );

        // ssh mirror on a non-allowlisted host → denied with ssh:// origin.
        let denied = check_mirror_allowed("git@evil.example.com:org/repo.git")
            .unwrap()
            .expect_err("evil ssh host must be denied");
        assert_eq!(denied.origin, "ssh://evil.example.com");

        match saved {
            Some(v) => std::env::set_var(reposix_core::http::ALLOWLIST_ENV_VAR, v),
            None => std::env::remove_var(reposix_core::http::ALLOWLIST_ENV_VAR),
        }
    }
}
