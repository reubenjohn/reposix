//! URL-scheme backend dispatch for the git remote helper.
//!
//! Closes the v0.9.0 Phase 32 carry-forward tech debt where the helper
//! hardcoded `SimBackend` regardless of the `remote.origin.url`. The
//! helper now parses the URL, identifies the target backend (sim,
//! github, confluence, jira), and instantiates the corresponding
//! [`BackendConnector`].
//!
//! ## URL forms accepted
//!
//! `reposix init` (see `crates/reposix-cli/src/init.rs`) emits one of:
//!
//! - **Sim** — `reposix::http://127.0.0.1:<port>/projects/<slug>`
//!   (or any loopback origin — `127.0.0.1`, `localhost`, `[::1]`).
//! - **GitHub** — `reposix::https://api.github.com/projects/<owner>/<repo>`
//!   (the project segment carries `<owner>/<repo>` literally).
//! - **Confluence** — `reposix::https://<tenant>.atlassian.net/confluence/projects/<space>`
//!   (the `/confluence/` path-segment marker disambiguates from JIRA;
//!   added in Phase 36-followup).
//! - **JIRA** — `reposix::https://<tenant>.atlassian.net/jira/projects/<key>`
//!   (analogous `/jira/` marker).
//!
//! ## Cache-slug naming
//!
//! `Cache::open(backend, backend_slug, project)` joins to a filesystem
//! path `<root>/reposix/<backend_slug>-<project>.git`. The project
//! string must be filesystem-safe; GitHub's `owner/repo` form is
//! sanitized via [`sanitize_project_for_cache`] (replace `/` with `-`)
//! before reaching `Cache::open`. The original slash-bearing project
//! name is still passed to [`BackendConnector`] methods so the GitHub
//! adapter assembles `repos/{owner}/{repo}/...` URLs correctly.

#![allow(clippy::module_name_repetitions)]

use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use reposix_cache::path::resolve_cache_path;
use reposix_core::backend::{sim::SimBackend, BackendConnector};
use reposix_core::split_reposix_url;
use rusqlite::Connection;

/// Which concrete backend the helper should instantiate for a given URL.
///
/// Closed enum — a new backend addition is an API change and lands with
/// a workspace version bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// Loopback simulator (default for tests / autonomous mode).
    Sim,
    /// GitHub Issues REST v3 (read-only at v0.9.0).
    GitHub,
    /// Atlassian Confluence Cloud REST v2.
    Confluence,
    /// Atlassian JIRA Cloud REST v3.
    Jira,
}

impl BackendKind {
    /// Stable slug used as the cache-path prefix and as the audit-row
    /// `backend` column. Matches the `<backend_name>-<project>.git`
    /// convention in `reposix-cache`.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Self::Sim => "sim",
            Self::GitHub => "github",
            Self::Confluence => "confluence",
            Self::Jira => "jira",
        }
    }
}

/// Parsed remote URL: which backend to dispatch, the origin to talk to,
/// and the backend-specific project identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRemote {
    /// Which backend to instantiate.
    pub kind: BackendKind,
    /// Scheme + host + optional port, e.g. `https://api.github.com` or
    /// `http://127.0.0.1:7878`. No trailing slash.
    pub origin: String,
    /// Backend-specific project identifier — `demo` for sim, `owner/repo`
    /// for GitHub, `TokenWorld` for Confluence, `TEST` for JIRA.
    pub project: String,
}

/// Parse a `reposix::<...>` remote URL into the dispatch tuple.
///
/// Accepts the form emitted by `reposix init`:
/// `reposix::<scheme>://<host>[:port][/<backend-marker>]/projects/<project>`.
///
/// The leading `reposix::` is optional — git strips it before invoking
/// the helper, but `assert_cmd` test harnesses pass it verbatim.
///
/// # Errors
///
/// - The URL has no `/projects/` segment.
/// - The origin is empty or unrecognizable as one of the supported
///   backends (loopback, `api.github.com`, `*.atlassian.net`).
/// - The project segment is empty or path-traversal-unsafe (`.`, `..`).
/// - The Atlassian URL is missing the `/confluence/` or `/jira/`
///   path-segment marker that disambiguates the two adapters.
pub fn parse_remote_url(url: &str) -> Result<ParsedRemote> {
    // Delegate the prefix-strip + `/projects/` split to `reposix-core`'s
    // canonical splitter. We layer the Atlassian path-marker handling
    // and `BackendKind` resolution on top — the splitter intentionally
    // does not enforce a project-slug character set so this dispatcher
    // can accept GitHub's `owner/repo` form.
    let (pre, project) = split_reposix_url(url)
        .map_err(|e| anyhow!("remote url `{url}` rejected by reposix-core splitter: {e}"))?;
    if project == "." || project == ".." {
        return Err(anyhow!(
            "remote url `{url}` has empty or path-traversal project segment"
        ));
    }

    // Pull off any trailing path-segment marker (the `/confluence` or
    // `/jira` disambiguator for Atlassian URLs). Whatever remains is
    // the origin.
    let (origin, marker) = match pre.rsplit_once('/') {
        Some((before_marker, last)) if matches!(last, "confluence" | "jira") => {
            (before_marker.trim_end_matches('/'), Some(last))
        }
        _ => (pre, None),
    };

    let kind = classify_origin(origin, marker)?;
    Ok(ParsedRemote {
        kind,
        origin: origin.to_owned(),
        project: project.to_owned(),
    })
}

/// Map a `(origin, marker)` pair to a [`BackendKind`].
///
/// - Loopback origins (`http://127.0.0.1`, `localhost`, `[::1]`) → Sim.
/// - `https://api.github.com` → GitHub.
/// - `https://*.atlassian.net` + marker `confluence` → Confluence.
/// - `https://*.atlassian.net` + marker `jira` → Jira.
/// - Anything else → error.
fn classify_origin(origin: &str, marker: Option<&str>) -> Result<BackendKind> {
    let lower = origin.to_ascii_lowercase();

    // Loopback variants (any port). `http://127.0.0.1`, `https://localhost`,
    // `http://[::1]:7878`, etc.
    let is_loopback = lower.starts_with("http://127.0.0.1")
        || lower.starts_with("https://127.0.0.1")
        || lower.starts_with("http://localhost")
        || lower.starts_with("https://localhost")
        || lower.starts_with("http://[::1]")
        || lower.starts_with("https://[::1]");
    if is_loopback {
        return Ok(BackendKind::Sim);
    }

    if lower == "https://api.github.com" || lower == "http://api.github.com" {
        return Ok(BackendKind::GitHub);
    }

    if lower.starts_with("https://") && lower.ends_with(".atlassian.net") {
        return match marker {
            Some("confluence") => Ok(BackendKind::Confluence),
            Some("jira") => Ok(BackendKind::Jira),
            Some(other) => Err(anyhow!(
                "atlassian origin `{origin}` requires `/confluence/` or `/jira/` path marker; got `/{other}/`"
            )),
            None => Err(anyhow!(
                "atlassian origin `{origin}` requires a `/confluence/projects/...` or `/jira/projects/...` URL form to disambiguate"
            )),
        };
    }

    Err(anyhow!(
        "remote origin `{origin}` is not a recognised reposix backend; expected loopback (sim), https://api.github.com (github), or https://<tenant>.atlassian.net with a /confluence or /jira marker"
    ))
}

/// Re-export of the canonical cache-slug sanitizer, which now lives in
/// [`reposix_core::sanitize_project_for_cache`] (single source of truth,
/// S-260707-gh404). Kept here for backward-compatible call sites.
///
/// NOTE: production callers no longer sanitize the project BEFORE
/// [`reposix_cache::Cache::open`] — the backend-facing slug must stay RAW
/// (`owner/repo`) or GitHub 404s; sanitization happens ONCE, inside
/// `reposix_cache::path::resolve_cache_path`, for the on-disk path only.
pub use reposix_core::sanitize_project_for_cache;

/// Build the concrete [`BackendConnector`] for a parsed URL, reading
/// credentials from environment variables per
/// `docs/reference/testing-targets.md`.
///
/// Writes a best-effort audit row (`helper_backend_instantiated`) to
/// the cache DB if a `Cache` handle is available — the caller is
/// responsible for that side effect post-instantiation; this function
/// returns the backend handle only.
///
/// # Errors
///
/// - For Sim: never — Sim has no required credentials. (`REPOSIX_SIM_URL`
///   is consulted only as a default-origin override and is optional.)
/// - For GitHub: returns an error listing `GITHUB_TOKEN` if the env var
///   is unset.
/// - For Confluence: returns an error listing
///   `ATLASSIAN_API_KEY`/`ATLASSIAN_EMAIL`/`REPOSIX_CONFLUENCE_TENANT`
///   if any are unset.
/// - For JIRA: returns an error listing
///   `JIRA_EMAIL`/`JIRA_API_TOKEN`/`REPOSIX_JIRA_INSTANCE` if any are
///   unset.
///
/// All credential errors include a pointer to
/// `docs/reference/testing-targets.md` so the agent has a single place
/// to look for the expected env-var matrix.
pub fn instantiate(parsed: &ParsedRemote) -> Result<Arc<dyn BackendConnector>> {
    match parsed.kind {
        BackendKind::Sim => instantiate_sim(&parsed.origin),
        BackendKind::GitHub => instantiate_github(&parsed.origin),
        BackendKind::Confluence => Ok(Arc::new(build_confluence(parsed)?)),
        BackendKind::Jira => Ok(Arc::new(build_jira(parsed)?)),
    }
}

/// Open the `audit_events` (backend-mutation) audit DB for a write-capable
/// connector and wrap it for [`with_audit`](reposix_confluence::ConfluenceBackend::with_audit).
///
/// OP-3 (audit is non-optional): every network-touching mutation the
/// helper drives against a real Confluence/JIRA backend MUST write an
/// `audit_events` row. The `sim` backend records its own audit rows
/// server-side (the simulator's middleware), and `github` is read-only
/// (writes are `NotSupported`), so only Confluence + JIRA wire a
/// client-side audit connection here.
///
/// ## DB location — a SIBLING of the bare-repo cache dir
///
/// The audit DB lives at `<root>/reposix/<backend>-<project>.audit.db`,
/// a sibling *file* of the bare-repo cache dir
/// (`<root>/reposix/<backend>-<project>.git/`). It deliberately does NOT
/// live inside the `.git` dir: `reposix_cache::Cache::open` calls
/// `gix::init_bare`, which refuses a non-empty directory, so a `cache.db`
/// created here before the cache initialises would break the first
/// fetch/push against a fresh project. The sibling file sidesteps that
/// ordering hazard entirely. A complete forensic query joins this DB's
/// `audit_events` with the bare repo's `cache.db::audit_events_cache`
/// (the dual-table design — see `reposix_core::audit` module docs).
///
/// ## Failure semantics — hard error (OP-3)
///
/// A failure to open the audit DB is a hard error, propagated to the
/// helper's top-level exit. Rationale: the audit DB sits in the same
/// `<root>/reposix/` directory as the cache, so if it cannot be opened
/// (read-only FS, permission error) the cache cannot be opened either and
/// the session is already non-functional — failing loud at construction
/// is strictly more informative than a silent no-audit write path. This
/// matches CLI/simulator semantics, where [`open_audit_db`] errors
/// propagate rather than being swallowed.
///
/// [`open_audit_db`]: reposix_core::audit::open_audit_db
fn open_connector_audit(kind: BackendKind, project: &str) -> Result<Arc<Mutex<Connection>>> {
    // `resolve_cache_path` sanitizes the slug internally (single site,
    // S-260707-gh404) — pass the RAW `project` and let it derive the flat
    // `<backend>-<sanitized>.git` dir; the audit DB is its `.audit.db` sibling.
    let bare = resolve_cache_path(kind.slug(), project)
        .map_err(|e| anyhow!("resolve cache path for audit DB: {e}"))?;
    // Sibling file: `<...>-<project>.git` -> `<...>-<project>.audit.db`.
    let audit_path = bare.with_extension("audit.db");
    if let Some(parent) = audit_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create audit DB dir {}", parent.display()))?;
    }
    let conn = reposix_core::audit::open_audit_db(&audit_path)
        .map_err(|e| anyhow!("open audit DB at {}: {e}", audit_path.display()))?;
    Ok(Arc::new(Mutex::new(conn)))
}

/// Build a Confluence backend with the `audit_events` connection wired
/// (OP-3). Split from [`instantiate`] so the `connector_audit_wired_*`
/// tests can assert on the concrete `has_audit()` state.
fn build_confluence(parsed: &ParsedRemote) -> Result<reposix_confluence::ConfluenceBackend> {
    let origin = &parsed.origin;
    let required = [
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT",
    ];
    let missing = collect_missing(&required);
    if !missing.is_empty() {
        return Err(missing_env_error(BackendKind::Confluence, &missing));
    }
    // SAFETY: we just verified all three are present and non-empty.
    let api_token = std::env::var("ATLASSIAN_API_KEY").expect("checked");
    let email = std::env::var("ATLASSIAN_EMAIL").expect("checked");
    let creds = reposix_confluence::ConfluenceCreds { email, api_token };
    let audit = open_connector_audit(BackendKind::Confluence, &parsed.project)?;
    let backend =
        reposix_confluence::ConfluenceBackend::new_with_base_url(creds, origin.to_owned())
            .map_err(|e| anyhow!("instantiate confluence backend at `{origin}`: {e}"))?
            .with_audit(audit);
    Ok(backend)
}

/// Build a JIRA backend with the `audit_events` connection wired (OP-3).
/// Split from [`instantiate`] for the same reason as [`build_confluence`].
fn build_jira(parsed: &ParsedRemote) -> Result<reposix_jira::JiraBackend> {
    let origin = &parsed.origin;
    let required = ["JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE"];
    let missing = collect_missing(&required);
    if !missing.is_empty() {
        return Err(missing_env_error(BackendKind::Jira, &missing));
    }
    let email = std::env::var("JIRA_EMAIL").expect("checked");
    let api_token = std::env::var("JIRA_API_TOKEN").expect("checked");
    let creds = reposix_jira::JiraCreds { email, api_token };
    let audit = open_connector_audit(BackendKind::Jira, &parsed.project)?;
    let backend = reposix_jira::JiraBackend::new_with_base_url(creds, origin.to_owned())
        .map_err(|e| anyhow!("instantiate jira backend at `{origin}`: {e}"))?
        .with_audit(audit);
    Ok(backend)
}

fn instantiate_sim(origin: &str) -> Result<Arc<dyn BackendConnector>> {
    let backend = SimBackend::with_agent_suffix(origin.to_owned(), Some("remote"))
        .map_err(|e| anyhow!("instantiate sim backend at `{origin}`: {e}"))?;
    Ok(Arc::new(backend))
}

fn instantiate_github(origin: &str) -> Result<Arc<dyn BackendConnector>> {
    let token = required_env("GITHUB_TOKEN", &["GITHUB_TOKEN"])?;
    let backend =
        reposix_github::GithubReadOnlyBackend::new_with_base_url(Some(token), origin.to_owned())
            .map_err(|e| anyhow!("instantiate github backend at `{origin}`: {e}"))?;
    Ok(Arc::new(backend))
}

/// Resolve a single required env var, returning a doc-linked error on absence.
fn required_env(name: &str, all_required: &[&str]) -> Result<String> {
    match std::env::var(name) {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => {
            let kind = match name {
                "GITHUB_TOKEN" => BackendKind::GitHub,
                _ => unreachable!("required_env called for unmapped var {name}"),
            };
            Err(missing_env_error(kind, all_required))
        }
    }
}

/// Build the list of env-var names that are unset or empty.
fn collect_missing(required: &[&'static str]) -> Vec<&'static str> {
    required
        .iter()
        .filter(|name| std::env::var(name).map_or(true, |v| v.is_empty()))
        .copied()
        .collect()
}

/// Render a uniform "missing creds" error message that lists each
/// absent env var on its own line and points at the canonical
/// testing-targets doc.
fn missing_env_error(kind: BackendKind, missing: &[&str]) -> anyhow::Error {
    let lines: Vec<String> = missing.iter().map(|n| format!("  - {n}")).collect();
    anyhow!(
        "git-remote-reposix: cannot instantiate {kind:?} backend — required env var(s) unset:\n{}\n\nSee docs/reference/testing-targets.md for the env-var matrix per backend.",
        lines.join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(url: &str) -> ParsedRemote {
        parse_remote_url(url).unwrap_or_else(|e| panic!("expected ok parse for `{url}`: {e}"))
    }

    #[test]
    fn parse_remote_url_sim() {
        let p = parse("reposix::http://127.0.0.1:7878/projects/demo");
        assert_eq!(p.kind, BackendKind::Sim);
        assert_eq!(p.origin, "http://127.0.0.1:7878");
        assert_eq!(p.project, "demo");
    }

    #[test]
    fn parse_remote_url_sim_localhost() {
        let p = parse("reposix::http://localhost:9999/projects/scratch");
        assert_eq!(p.kind, BackendKind::Sim);
    }

    #[test]
    fn parse_remote_url_github() {
        let p = parse("reposix::https://api.github.com/projects/reubenjohn/reposix");
        assert_eq!(p.kind, BackendKind::GitHub);
        assert_eq!(p.origin, "https://api.github.com");
        assert_eq!(p.project, "reubenjohn/reposix");
    }

    #[test]
    fn parse_remote_url_confluence() {
        let p = parse("reposix::https://reuben-john.atlassian.net/confluence/projects/TokenWorld");
        assert_eq!(p.kind, BackendKind::Confluence);
        assert_eq!(p.origin, "https://reuben-john.atlassian.net");
        assert_eq!(p.project, "TokenWorld");
    }

    #[test]
    fn parse_remote_url_jira() {
        let p = parse("reposix::https://reuben-john.atlassian.net/jira/projects/TEST");
        assert_eq!(p.kind, BackendKind::Jira);
        assert_eq!(p.origin, "https://reuben-john.atlassian.net");
        assert_eq!(p.project, "TEST");
    }

    #[test]
    fn parse_remote_url_double_reposix_prefix() {
        // Defence in depth: if a future git version double-strips, the
        // helper still parses cleanly.
        let p = parse("reposix::reposix::http://127.0.0.1:7878/projects/demo");
        assert_eq!(p.kind, BackendKind::Sim);
        assert_eq!(p.project, "demo");
    }

    #[test]
    fn parse_remote_url_no_prefix() {
        let p = parse("http://127.0.0.1:7878/projects/demo");
        assert_eq!(p.kind, BackendKind::Sim);
    }

    #[test]
    fn parse_remote_url_rejects_missing_projects_segment() {
        let err = parse_remote_url("reposix::http://127.0.0.1/no-projects-here").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("expected `/projects/"), "got: {msg}");
    }

    #[test]
    fn parse_remote_url_rejects_empty_project() {
        let err = parse_remote_url("reposix::http://127.0.0.1/projects/").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("empty project segment"), "got: {msg}");
    }

    #[test]
    fn parse_remote_url_rejects_traversal_project() {
        let err = parse_remote_url("reposix::http://127.0.0.1/projects/..").unwrap_err();
        assert!(err.to_string().contains("path-traversal"), "got: {err}");
    }

    #[test]
    fn parse_remote_url_rejects_unknown_origin() {
        let err = parse_remote_url("reposix::https://evil.example.com/projects/x").unwrap_err();
        assert!(
            err.to_string().contains("not a recognised reposix backend"),
            "got: {err}"
        );
    }

    #[test]
    fn parse_remote_url_rejects_atlassian_without_marker() {
        let err =
            parse_remote_url("reposix::https://reuben-john.atlassian.net/projects/TokenWorld")
                .unwrap_err();
        assert!(
            err.to_string()
                .contains("requires a `/confluence/projects/"),
            "got: {err}"
        );
    }

    #[test]
    fn sanitize_project_for_cache_replaces_slashes() {
        assert_eq!(
            sanitize_project_for_cache("reubenjohn/reposix"),
            "reubenjohn-reposix"
        );
        assert_eq!(sanitize_project_for_cache("a:b\\c/d"), "a-b-c-d");
        assert_eq!(sanitize_project_for_cache("plain"), "plain");
    }

    #[test]
    fn backend_kind_slug_matches_audit_convention() {
        assert_eq!(BackendKind::Sim.slug(), "sim");
        assert_eq!(BackendKind::GitHub.slug(), "github");
        assert_eq!(BackendKind::Confluence.slug(), "confluence");
        assert_eq!(BackendKind::Jira.slug(), "jira");
    }

    /// Sim instantiation requires no env vars and never errors.
    #[test]
    fn instantiate_sim_no_creds_required_succeeds() {
        let parsed = ParsedRemote {
            kind: BackendKind::Sim,
            origin: "http://127.0.0.1:7878".to_owned(),
            project: "demo".to_owned(),
        };
        let backend = instantiate(&parsed).expect("sim instantiate");
        assert_eq!(backend.name(), "simulator");
    }

    /// Process-wide lock serialising every env-mutating test in this
    /// module. Rust runs tests as parallel threads sharing one process
    /// environment; without this, a test that *sets* `ATLASSIAN_*` and a
    /// test that *clears* them race. Poison is ignored — a panicking test
    /// still leaves the env consistent because each helper restores.
    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Test helper: snapshot+clear a list of env vars for the duration
    /// of the closure, then restore. Required because Rust runs tests in
    /// the same process and other tests may set creds.
    fn with_cleared_env<F: FnOnce()>(names: &[&str], f: F) {
        let _g = env_guard();
        let saved: Vec<(String, Option<String>)> = names
            .iter()
            .map(|n| ((*n).to_owned(), std::env::var(n).ok()))
            .collect();
        for n in names {
            std::env::remove_var(n);
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        for (n, v) in saved {
            match v {
                Some(s) => std::env::set_var(n, s),
                None => std::env::remove_var(n),
            }
        }
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    /// Helper that runs `instantiate` and unwraps the `Err`. We can't
    /// use `Result::expect_err` because `Arc<dyn BackendConnector>`
    /// has no `Debug` impl.
    fn expect_instantiate_err(parsed: &ParsedRemote) -> anyhow::Error {
        match instantiate(parsed) {
            Ok(_) => panic!("instantiate({parsed:?}) unexpectedly succeeded"),
            Err(e) => e,
        }
    }

    #[test]
    fn instantiate_github_missing_token_errors_with_helpful_message() {
        with_cleared_env(&["GITHUB_TOKEN"], || {
            let parsed = ParsedRemote {
                kind: BackendKind::GitHub,
                origin: "https://api.github.com".to_owned(),
                project: "owner/repo".to_owned(),
            };
            let err = expect_instantiate_err(&parsed);
            let msg = format!("{err:#}");
            assert!(
                msg.contains("GITHUB_TOKEN"),
                "msg should name env var: {msg}"
            );
            assert!(
                msg.contains("docs/reference/testing-targets.md"),
                "msg should link to doc: {msg}"
            );
        });
    }

    #[test]
    fn instantiate_confluence_missing_creds_lists_all_three() {
        with_cleared_env(
            &[
                "ATLASSIAN_API_KEY",
                "ATLASSIAN_EMAIL",
                "REPOSIX_CONFLUENCE_TENANT",
            ],
            || {
                let parsed = ParsedRemote {
                    kind: BackendKind::Confluence,
                    origin: "https://reuben-john.atlassian.net".to_owned(),
                    project: "TokenWorld".to_owned(),
                };
                let err = expect_instantiate_err(&parsed);
                let msg = format!("{err:#}");
                assert!(msg.contains("ATLASSIAN_API_KEY"), "msg: {msg}");
                assert!(msg.contains("ATLASSIAN_EMAIL"), "msg: {msg}");
                assert!(msg.contains("REPOSIX_CONFLUENCE_TENANT"), "msg: {msg}");
            },
        );
    }

    #[test]
    fn instantiate_jira_missing_creds_lists_all_three() {
        with_cleared_env(
            &["JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE"],
            || {
                let parsed = ParsedRemote {
                    kind: BackendKind::Jira,
                    origin: "https://reuben-john.atlassian.net".to_owned(),
                    project: "TEST".to_owned(),
                };
                let err = expect_instantiate_err(&parsed);
                let msg = format!("{err:#}");
                assert!(msg.contains("JIRA_EMAIL"), "msg: {msg}");
                assert!(msg.contains("JIRA_API_TOKEN"), "msg: {msg}");
                assert!(msg.contains("REPOSIX_JIRA_INSTANCE"), "msg: {msg}");
            },
        );
    }

    /// Run `f` with `vars` set (and `REPOSIX_CACHE_DIR` pointed at a fresh
    /// temp dir), restoring every touched var afterwards. Serialised via
    /// [`env_guard`]. Returns the temp dir so the caller can inspect the
    /// audit DB it produced.
    fn with_env_and_cache<F: FnOnce()>(vars: &[(&str, &str)], f: F) -> tempfile::TempDir {
        let _g = env_guard();
        let cache = tempfile::tempdir().expect("cache tempdir");
        let mut names: Vec<&str> = vars.iter().map(|(k, _)| *k).collect();
        names.push(reposix_cache::path::CACHE_DIR_ENV);
        names.push(reposix_core::http::ALLOWLIST_ENV_VAR);
        let saved: Vec<(String, Option<String>)> = names
            .iter()
            .map(|n| ((*n).to_owned(), std::env::var(n).ok()))
            .collect();
        for (k, v) in vars {
            std::env::set_var(k, v);
        }
        std::env::set_var(reposix_cache::path::CACHE_DIR_ENV, cache.path());
        // A valid (loopback) allowlist so `client()` construction succeeds.
        std::env::set_var(reposix_core::http::ALLOWLIST_ENV_VAR, "http://127.0.0.1:*");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        for (n, v) in saved {
            match v {
                Some(s) => std::env::set_var(&n, s),
                None => std::env::remove_var(&n),
            }
        }
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
        cache
    }

    /// Assert the audit DB file the dispatch just created holds the
    /// backend-mutation `audit_events` table (proves the schema was
    /// loaded, not merely that a file was touched).
    fn assert_audit_events_table(cache_root: &std::path::Path, backend: &str, project: &str) {
        let bare = cache_root
            .join("reposix")
            .join(format!("{backend}-{project}.git"));
        let audit_db = bare.with_extension("audit.db");
        assert!(
            audit_db.exists(),
            "audit DB not created at {}",
            audit_db.display()
        );
        let conn = rusqlite::Connection::open(&audit_db).expect("open audit db");
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_events'",
                [],
                |r| r.get(0),
            )
            .expect("query sqlite_master");
        assert_eq!(n, 1, "audit_events table missing in {}", audit_db.display());
    }

    /// OP-3: a Confluence backend dispatched by the helper MUST have its
    /// `audit_events` connection wired. `connector_audit_wired` greps here.
    #[test]
    fn connector_audit_wired_confluence() {
        let parsed = ParsedRemote {
            kind: BackendKind::Confluence,
            origin: "https://reuben-john.atlassian.net".to_owned(),
            project: "TokenWorld".to_owned(),
        };
        let cache = with_env_and_cache(
            &[
                ("ATLASSIAN_API_KEY", "dummy-token"),
                ("ATLASSIAN_EMAIL", "dummy@example.com"),
                ("REPOSIX_CONFLUENCE_TENANT", "reuben-john"),
            ],
            || {
                let backend = build_confluence(&parsed).expect("build confluence");
                assert!(
                    backend.has_audit(),
                    "confluence backend dispatched without audit wired (OP-3 violation)"
                );
            },
        );
        assert_audit_events_table(cache.path(), "confluence", "TokenWorld");
    }

    /// OP-3: a JIRA backend dispatched by the helper MUST have its
    /// `audit_events` connection wired. `connector_audit_wired` greps here.
    #[test]
    fn connector_audit_wired_jira() {
        let parsed = ParsedRemote {
            kind: BackendKind::Jira,
            origin: "https://reuben-john.atlassian.net".to_owned(),
            project: "TEST".to_owned(),
        };
        let cache = with_env_and_cache(
            &[
                ("JIRA_EMAIL", "dummy@example.com"),
                ("JIRA_API_TOKEN", "dummy-token"),
                ("REPOSIX_JIRA_INSTANCE", "reuben-john"),
            ],
            || {
                let backend = build_jira(&parsed).expect("build jira");
                assert!(
                    backend.has_audit(),
                    "jira backend dispatched without audit wired (OP-3 violation)"
                );
            },
        );
        assert_audit_events_table(cache.path(), "jira", "TEST");
    }
}
