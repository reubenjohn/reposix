//! `reposix doctor` — diagnostic CLI subcommand.
//!
//! Audits a `reposix init`'d working tree and reports issues with copy-pastable
//! fix commands. Read-only on the cache DB; never mutates audit/backend state.
//!
//! With `--fix`, applies a tiny allowlist of deterministic, non-destructive
//! fixes (currently: `git config extensions.partialClone origin`).
//!
//! Inspired by `flutter doctor` / `brew doctor` — the agent-era pitch is in
//! `.planning/research/v0.11.0-vision-and-innovations.md` §3a.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use reposix_cache::db::open_cache_db;
use reposix_cache::path::resolve_cache_path;
use reposix_core::parse_remote_url;
use rusqlite::Connection;

use crate::worktree_helpers::{backend_slug_from_origin, git_config_get};

/// Severity tier for a single check finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Check passed.
    Ok,
    /// Informational — print state but do not affect exit code or warning count.
    Info,
    /// Possible issue. Does not fail exit.
    Warn,
    /// Hard failure. Causes a non-zero exit.
    Error,
}

impl Severity {
    /// Single-glyph icon prefix, with optional ANSI colour.
    fn icon(self, colour: bool) -> &'static str {
        match (self, colour) {
            (Severity::Ok, true) => "\x1b[32mOK\x1b[0m   ",
            (Severity::Ok, false) => "OK   ",
            (Severity::Info, true) => "\x1b[34mINFO\x1b[0m ",
            (Severity::Info, false) => "INFO ",
            (Severity::Warn, true) => "\x1b[33mWARN\x1b[0m ",
            (Severity::Warn, false) => "WARN ",
            (Severity::Error, true) => "\x1b[31mERR \x1b[0m",
            (Severity::Error, false) => "ERR  ",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Ok => "OK",
            Severity::Info => "INFO",
            Severity::Warn => "WARN",
            Severity::Error => "ERROR",
        };
        f.write_str(s)
    }
}

/// One finding from a single check.
#[derive(Debug, Clone)]
pub struct DoctorFinding {
    /// Severity tier.
    pub severity: Severity,
    /// Short check name (machine-stable; stable across versions).
    pub check: &'static str,
    /// Human-readable description of what was found.
    pub message: String,
    /// Optional copy-pastable fix command. `None` when nothing to fix.
    pub fix: Option<String>,
    /// True iff `--fix` may apply this fix automatically.
    pub auto_fixable: bool,
    /// True iff `--fix` actually applied a fix for this finding in this run.
    pub fix_applied: bool,
}

impl DoctorFinding {
    fn ok(check: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Ok,
            check,
            message: message.into(),
            fix: None,
            auto_fixable: false,
            fix_applied: false,
        }
    }

    fn info(check: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            check,
            message: message.into(),
            fix: None,
            auto_fixable: false,
            fix_applied: false,
        }
    }

    fn warn(check: &'static str, message: impl Into<String>, fix: Option<String>) -> Self {
        Self {
            severity: Severity::Warn,
            check,
            message: message.into(),
            fix,
            auto_fixable: false,
            fix_applied: false,
        }
    }

    fn error(check: &'static str, message: impl Into<String>, fix: Option<String>) -> Self {
        Self {
            severity: Severity::Error,
            check,
            message: message.into(),
            fix,
            auto_fixable: false,
            fix_applied: false,
        }
    }

    /// Mark this finding as eligible for `--fix` auto-application.
    fn auto_fixable(mut self) -> Self {
        self.auto_fixable = true;
        self
    }
}

/// Full doctor report — list of findings + summary tallies.
#[derive(Debug, Clone)]
pub struct DoctorReport {
    /// Path that was audited.
    pub path: PathBuf,
    /// Findings, in check order.
    pub findings: Vec<DoctorFinding>,
}

impl DoctorReport {
    /// Number of findings at each severity.
    #[must_use]
    pub fn tally(&self) -> (usize, usize, usize, usize) {
        let mut ok = 0;
        let mut info = 0;
        let mut warn = 0;
        let mut error = 0;
        for f in &self.findings {
            match f.severity {
                Severity::Ok => ok += 1,
                Severity::Info => info += 1,
                Severity::Warn => warn += 1,
                Severity::Error => error += 1,
            }
        }
        (ok, info, warn, error)
    }

    /// Process exit code: 0 if no errors, 1 otherwise.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        i32::from(self.findings.iter().any(|f| f.severity == Severity::Error))
    }

    /// Print the report to stdout.
    ///
    /// `colour` controls ANSI colouring. The conventional caller passes
    /// `std::io::IsTerminal::is_terminal(&std::io::stdout())`.
    pub fn print(&self, colour: bool) {
        println!("reposix doctor — {}", self.path.display());
        for f in &self.findings {
            println!("{} {}: {}", f.severity.icon(colour), f.check, f.message);
            if let Some(fix) = &f.fix {
                if f.fix_applied {
                    println!("     fix (applied): {fix}");
                } else {
                    println!("     fix: {fix}");
                }
            }
        }
        let (ok, info, warn, error) = self.tally();
        let total = self.findings.len();
        println!("\n{total} checks · {error} errors · {warn} warnings · {info} info · {ok} ok");
    }
}

/// Context gathered once before running checks. Avoids re-running `git config`
/// invocations across checks that need the same value.
struct DoctorCtx {
    path: PathBuf,
    is_git_repo: bool,
    partial_clone_value: Option<String>,
    remote_origin_url: Option<String>,
    sparse_checkout_lines: Option<String>,
}

impl DoctorCtx {
    fn gather(path: &Path) -> Self {
        let is_git_repo =
            git_in(path, &["rev-parse", "--git-dir"]).is_ok_and(|o| o.status.success());

        let partial_clone_value = if is_git_repo {
            git_config_get(path, "extensions.partialClone")
        } else {
            None
        };

        let remote_origin_url = if is_git_repo {
            git_config_get(path, "remote.origin.url")
        } else {
            None
        };

        let sparse_checkout_lines = if is_git_repo {
            std::fs::read_to_string(path.join(".git").join("info").join("sparse-checkout")).ok()
        } else {
            None
        };

        Self {
            path: path.to_path_buf(),
            is_git_repo,
            partial_clone_value,
            remote_origin_url,
            sparse_checkout_lines,
        }
    }
}

/// Public entry point — run all checks.
///
/// `path` defaults to `cwd` when `None`. With `fix=true`, applies the small
/// allowlist of safe auto-fixes inline (today: `extensions.partialClone`).
///
/// # Errors
/// Returns an error only if cwd resolution itself fails. All check failures
/// are encoded as findings, not Rust errors.
pub fn run(path: Option<&Path>, fix: bool) -> Result<DoctorReport> {
    let abspath: PathBuf = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().context("resolve current directory")?,
    };

    let ctx = DoctorCtx::gather(&abspath);

    let mut findings = vec![
        check_git_repo(&ctx),
        check_partial_clone(&ctx),
        check_remote_url(&ctx),
        check_helper_on_path(),
        check_git_version(),
    ];

    // Cache-related checks need a (backend, project) tuple. If we couldn't
    // parse one from the remote URL, skip the cache checks with INFO so the
    // user still sees that we tried.
    let parsed = ctx
        .remote_origin_url
        .as_deref()
        .and_then(|u| parse_remote_url(u).ok());

    // Check 3 from POLISH-09 spec: backend connector registered for the
    // scheme parsed out of `remote.origin.url`. Runs whether or not the
    // cache path resolves.
    findings.push(check_backend_registered(parsed.as_ref()));

    if let Some(spec) = parsed.as_ref() {
        let backend = backend_slug_from_origin(&spec.origin);
        let project = spec.project.as_str();
        match resolve_cache_path(&backend, project) {
            Ok(cache_path) => {
                findings.push(check_cache_db_exists(&cache_path));
                if cache_path.join("cache.db").exists() {
                    let conn_res = open_cache_db(&cache_path);
                    findings.push(check_cache_db_readable(&conn_res));
                    if let Ok(conn) = &conn_res {
                        findings.push(check_cache_integrity(conn));
                        findings.push(check_audit_table(conn));
                        findings.push(check_audit_triggers(conn));
                        findings.push(check_outdated_cache(conn));
                    }
                }
                findings.push(check_cache_has_main_commit(&cache_path));
                findings.push(check_worktree_head_drift(&ctx, &cache_path));
            }
            Err(e) => {
                findings.push(DoctorFinding::warn(
                    "cache.path",
                    format!("could not resolve cache path: {e}"),
                    Some("set REPOSIX_CACHE_DIR or ensure $XDG_CACHE_HOME/$HOME is set".into()),
                ));
            }
        }
    } else {
        findings.push(DoctorFinding::info(
            "cache.skipped",
            "skipping cache checks — no parseable reposix remote URL".to_string(),
        ));
    }

    findings.push(check_allowed_origins(parsed.as_ref()));
    findings.push(check_blob_limit(parsed.as_ref()));
    findings.push(check_sparse_checkout(&ctx));
    findings.push(check_rust_toolchain());

    if fix {
        for finding in &mut findings {
            if !finding.auto_fixable {
                continue;
            }
            // The only auto-fix today: setting extensions.partialClone=origin.
            if finding.check == "git.extensions.partialClone"
                && ctx.is_git_repo
                && git_config_set(&ctx.path, "extensions.partialClone", "origin").is_ok()
            {
                finding.severity = Severity::Ok;
                finding.message = "extensions.partialClone=origin (set by --fix)".to_string();
                finding.fix_applied = true;
            }
        }
    }

    Ok(DoctorReport {
        path: abspath,
        findings,
    })
}

// --------------------------------------------------------------------------
// Individual checks
// --------------------------------------------------------------------------

fn check_git_repo(ctx: &DoctorCtx) -> DoctorFinding {
    if ctx.is_git_repo {
        DoctorFinding::ok("git.repo", "working tree is a git repo")
    } else {
        DoctorFinding::error(
            "git.repo",
            "not a git repo (no .git directory)",
            Some(format!("cd {} && git init", ctx.path.display())),
        )
    }
}

fn check_partial_clone(ctx: &DoctorCtx) -> DoctorFinding {
    if !ctx.is_git_repo {
        return DoctorFinding::warn(
            "git.extensions.partialClone",
            "skipped — not a git repo".to_string(),
            None,
        );
    }
    match ctx.partial_clone_value.as_deref() {
        Some("origin") => DoctorFinding::ok(
            "git.extensions.partialClone",
            "extensions.partialClone=origin",
        ),
        Some(other) => DoctorFinding::warn(
            "git.extensions.partialClone",
            format!("extensions.partialClone={other} (expected `origin`)"),
            Some("git config extensions.partialClone origin".into()),
        )
        .auto_fixable(),
        None => DoctorFinding::warn(
            "git.extensions.partialClone",
            "extensions.partialClone is unset",
            Some("git config extensions.partialClone origin".into()),
        )
        .auto_fixable(),
    }
}

fn check_remote_url(ctx: &DoctorCtx) -> DoctorFinding {
    if !ctx.is_git_repo {
        return DoctorFinding::error("git.remote.origin.url", "skipped — not a git repo", None);
    }
    match ctx.remote_origin_url.as_deref() {
        None => DoctorFinding::error(
            "git.remote.origin.url",
            "remote.origin.url is unset — `git fetch` has nowhere to go",
            Some("git remote add origin reposix::sim::demo".into()),
        ),
        Some(url) => {
            if url.starts_with("reposix::") {
                if parse_remote_url(url).is_ok() {
                    DoctorFinding::ok("git.remote.origin.url", format!("remote.origin.url={url}"))
                } else {
                    DoctorFinding::error(
                        "git.remote.origin.url",
                        format!("remote.origin.url={url} — `reposix::` prefix present but URL doesn't parse (no `/projects/<slug>`?)"),
                        Some(
                            "reposix init <backend>::<project> <path> (re-run init in a fresh dir)"
                                .into(),
                        ),
                    )
                }
            } else {
                DoctorFinding::error(
                    "git.remote.origin.url",
                    format!("remote.origin.url={url} — does not use `reposix::` scheme"),
                    Some(format!(
                        "git remote set-url origin reposix::<backend>::<project> (replace `{url}`)"
                    )),
                )
            }
        }
    }
}

fn check_helper_on_path() -> DoctorFinding {
    let exists = which("git-remote-reposix");
    if exists {
        DoctorFinding::ok("helper.binary", "git-remote-reposix is on PATH")
    } else {
        DoctorFinding::error(
            "helper.binary",
            "git-remote-reposix is NOT on PATH — git can't dispatch reposix:: URLs",
            Some(
                "cargo install --path crates/reposix-remote (or download a release binary)".into(),
            ),
        )
    }
}

fn check_git_version() -> DoctorFinding {
    let out = Command::new("git").arg("--version").output();
    let Ok(out) = out else {
        return DoctorFinding::error(
            "git.version",
            "could not invoke `git --version` (is git installed?)",
            Some("install git >= 2.34 from your package manager".into()),
        );
    };
    let s = String::from_utf8_lossy(&out.stdout);
    // Format: "git version 2.43.0"
    let version = s.split_whitespace().nth(2).unwrap_or("?");
    let parts: Vec<&str> = version.split('.').collect();
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    if (major, minor) >= (2, 34) {
        DoctorFinding::ok("git.version", format!("git version {version}"))
    } else if (major, minor) >= (2, 27) {
        DoctorFinding::warn(
            "git.version",
            format!("git {version} works but >=2.34 is recommended (better partial-clone support)"),
            Some("install git >= 2.34 from your package manager".into()),
        )
    } else {
        DoctorFinding::error(
            "git.version",
            format!("git {version} is too old — partial-clone + stateless-connect needs >=2.27 (recommend 2.34)"),
            Some("install git >= 2.34 from your package manager".into()),
        )
    }
}

fn check_cache_db_exists(cache_path: &Path) -> DoctorFinding {
    let db = cache_path.join("cache.db");
    if db.exists() {
        DoctorFinding::ok("cache.db", format!("cache DB present at {}", db.display()))
    } else {
        DoctorFinding::warn(
            "cache.db",
            format!(
                "no cache DB at {} (fresh init not yet fetched?)",
                db.display()
            ),
            Some("git fetch origin --filter=blob:none".into()),
        )
    }
}

fn check_cache_db_readable(conn: &reposix_cache::Result<Connection>) -> DoctorFinding {
    match conn {
        Ok(_) => DoctorFinding::ok("cache.db.readable", "cache DB opens cleanly"),
        Err(e) => DoctorFinding::error(
            "cache.db.readable",
            format!("cache DB will not open: {e}"),
            Some("rm -rf <cache-dir> && git fetch origin (will rebuild)".into()),
        ),
    }
}

fn check_audit_table(conn: &Connection) -> DoctorFinding {
    let table_present: rusqlite::Result<i64> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_events_cache'",
        [],
        |r| r.get(0),
    );
    match table_present {
        Ok(0) => DoctorFinding::error(
            "cache.audit.table",
            "audit_events_cache table is missing — schema mismatch",
            Some("rm -rf <cache-dir> && git fetch origin (will rebuild schema)".into()),
        ),
        Err(e) => DoctorFinding::error(
            "cache.audit.table",
            format!("could not query audit_events_cache: {e}"),
            Some("rm -rf <cache-dir> && git fetch origin (will rebuild schema)".into()),
        ),
        Ok(_) => {
            // Count rows for an INFO/WARN signal.
            let row_count: rusqlite::Result<i64> =
                conn.query_row("SELECT COUNT(*) FROM audit_events_cache", [], |r| r.get(0));
            match row_count {
                Ok(n) if n > 0 => DoctorFinding::ok(
                    "cache.audit.table",
                    format!("audit_events_cache present ({n} rows)"),
                ),
                Ok(_) => DoctorFinding::warn(
                    "cache.audit.table",
                    "audit_events_cache present but empty (no helper traffic yet)",
                    Some("git fetch origin (any helper invocation will populate)".into()),
                ),
                Err(e) => DoctorFinding::warn(
                    "cache.audit.table",
                    format!("could not count audit rows: {e}"),
                    None,
                ),
            }
        }
    }
}

fn check_audit_triggers(conn: &Connection) -> DoctorFinding {
    let mut triggers = Vec::new();
    let stmt_res = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='trigger' AND tbl_name='audit_events_cache' ORDER BY name",
    );
    if let Ok(mut stmt) = stmt_res {
        if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
            for row in rows.flatten() {
                triggers.push(row);
            }
        }
    }
    let want_update = triggers.iter().any(|t| t == "audit_cache_no_update");
    let want_delete = triggers.iter().any(|t| t == "audit_cache_no_delete");
    if want_update && want_delete {
        DoctorFinding::ok(
            "cache.audit.triggers",
            "audit_events_cache append-only triggers present",
        )
    } else {
        DoctorFinding::error(
            "cache.audit.triggers",
            format!(
                "audit append-only triggers missing (have: {triggers:?}); the security guardrail is off"
            ),
            Some("rm -rf <cache-dir> && git fetch origin (will rebuild schema + triggers)".into()),
        )
    }
}

fn check_outdated_cache(conn: &Connection) -> DoctorFinding {
    let last: rusqlite::Result<Option<String>> = conn
        .query_row(
            "SELECT value FROM meta WHERE key='last_fetched_at'",
            [],
            |r| r.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        });
    match last {
        Ok(Some(ts)) => {
            // Parse RFC-3339; if parsing fails treat as INFO (don't escalate).
            match chrono::DateTime::parse_from_rfc3339(&ts) {
                Ok(parsed) => {
                    let now = chrono::Utc::now();
                    let age = now.signed_duration_since(parsed.with_timezone(&chrono::Utc));
                    if age.num_hours() > 24 {
                        DoctorFinding::warn(
                            "cache.freshness",
                            format!(
                                "last_fetched_at={ts} ({}h ago) — cache is stale",
                                age.num_hours()
                            ),
                            Some("git fetch origin".into()),
                        )
                    } else {
                        DoctorFinding::ok(
                            "cache.freshness",
                            format!("last_fetched_at={ts} ({}h ago)", age.num_hours().max(0)),
                        )
                    }
                }
                Err(e) => DoctorFinding::info(
                    "cache.freshness",
                    format!("last_fetched_at={ts} (could not parse: {e})"),
                ),
            }
        }
        Ok(None) => DoctorFinding::info(
            "cache.freshness",
            "no last_fetched_at meta row yet (cache fresh-init)",
        ),
        Err(e) => DoctorFinding::warn(
            "cache.freshness",
            format!("could not read meta.last_fetched_at: {e}"),
            None,
        ),
    }
}

fn check_allowed_origins(parsed: Option<&reposix_core::RemoteSpec>) -> DoctorFinding {
    let val = std::env::var("REPOSIX_ALLOWED_ORIGINS").ok();
    match (val.as_deref(), parsed) {
        (None | Some(""), Some(spec)) => {
            // Default allowlist is loopback only. If origin isn't loopback this
            // is a real warning.
            let origin = &spec.origin;
            let is_loopback = origin.contains("127.0.0.1") || origin.contains("localhost");
            if is_loopback {
                DoctorFinding::info(
                    "env.REPOSIX_ALLOWED_ORIGINS",
                    "unset — using default loopback allowlist (sim is OK)",
                )
            } else {
                DoctorFinding::warn(
                    "env.REPOSIX_ALLOWED_ORIGINS",
                    format!(
                        "unset — default allowlist is loopback-only, but remote origin is {origin}"
                    ),
                    Some(format!("export REPOSIX_ALLOWED_ORIGINS='{origin}'")),
                )
            }
        }
        (Some(v), Some(spec)) => {
            // Set + we have a parsed origin: verify the allowlist actually
            // covers the parsed origin. The `reposix_core` allowlist matcher
            // does port globbing on loopback; we mirror only the common
            // exact-host check here (good-enough for a doctor finding).
            let origin = &spec.origin;
            let allowed: Vec<&str> = v
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            let host_covered = allowed.iter().any(|entry| {
                // Exact match wins.
                if origin == entry {
                    return true;
                }
                // Loopback + port glob: `http://127.0.0.1:*` covers any port.
                if let Some(prefix) = entry.strip_suffix(":*") {
                    return origin.starts_with(prefix);
                }
                // Origin starts with entry (handles trailing-slash quirks).
                origin.starts_with(entry)
            });
            if host_covered {
                DoctorFinding::ok(
                    "env.REPOSIX_ALLOWED_ORIGINS",
                    format!("REPOSIX_ALLOWED_ORIGINS={v} (covers {origin})"),
                )
            } else {
                DoctorFinding::warn(
                    "env.REPOSIX_ALLOWED_ORIGINS",
                    format!(
                        "REPOSIX_ALLOWED_ORIGINS={v} does not cover remote origin {origin} — fetch will be rejected by the egress allowlist"
                    ),
                    Some(format!("export REPOSIX_ALLOWED_ORIGINS='{origin}'")),
                )
            }
        }
        (Some(v), None) => DoctorFinding::info(
            "env.REPOSIX_ALLOWED_ORIGINS",
            format!("REPOSIX_ALLOWED_ORIGINS={v}"),
        ),
        (None, None) => DoctorFinding::info(
            "env.REPOSIX_ALLOWED_ORIGINS",
            "unset — using default loopback allowlist".to_string(),
        ),
    }
}

/// Backends statically linked into this build. Used by `check_backend_registered`.
const KNOWN_BACKENDS: &[&str] = &["sim", "github", "confluence", "jira"];

/// Check #3: the backend referenced by `remote.origin.url` is one of the
/// schemes this build registers (sim/github/confluence/jira). The doctor
/// is statically linked against all four today, but the finding remains
/// valid against future feature-flagged builds.
fn check_backend_registered(parsed: Option<&reposix_core::RemoteSpec>) -> DoctorFinding {
    let Some(spec) = parsed else {
        return DoctorFinding::info(
            "backend.registered",
            "skipped — no parseable reposix remote URL",
        );
    };
    let backend = backend_slug_from_origin(&spec.origin);
    if KNOWN_BACKENDS.contains(&backend.as_str()) {
        DoctorFinding::ok(
            "backend.registered",
            format!("backend `{backend}` is registered in this build"),
        )
    } else {
        DoctorFinding::error(
            "backend.registered",
            format!(
                "backend `{backend}` (from origin {origin}) is NOT registered in this build",
                origin = spec.origin
            ),
            Some(
                "rebuild with the missing backend feature, or run `reposix list --backend <X>` to confirm support".into(),
            ),
        )
    }
}

/// Check #5: `PRAGMA integrity_check` returns `ok`. Detects on-disk
/// corruption that `cache.db` opens cleanly through.
fn check_cache_integrity(conn: &Connection) -> DoctorFinding {
    let row: rusqlite::Result<String> = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0));
    match row {
        Ok(s) if s == "ok" => DoctorFinding::ok("cache.integrity", "PRAGMA integrity_check = ok"),
        Ok(other) => DoctorFinding::error(
            "cache.integrity",
            format!("PRAGMA integrity_check returned `{other}` — cache.db is corrupted"),
            Some("rm -rf <cache-dir> && git fetch origin (no in-place rebuild path today)".into()),
        ),
        Err(e) => DoctorFinding::warn(
            "cache.integrity",
            format!("could not run PRAGMA integrity_check: {e}"),
            None,
        ),
    }
}

/// Check #9: the cache's bare repo has at least one commit on
/// `refs/heads/main` (sanity that init+fetch wrote something).
fn check_cache_has_main_commit(cache_path: &Path) -> DoctorFinding {
    if !cache_path.exists() {
        return DoctorFinding::info("cache.refs.main", "skipped — cache dir does not exist yet");
    }
    let out = Command::new("git")
        .arg("-C")
        .arg(cache_path)
        .args(["rev-parse", "--verify", "refs/heads/main"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let oid = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let short = oid.chars().take(12).collect::<String>();
            DoctorFinding::ok(
                "cache.refs.main",
                format!("refs/heads/main present in cache ({short})"),
            )
        }
        Ok(_) => DoctorFinding::warn(
            "cache.refs.main",
            "refs/heads/main is missing in the cache — nothing has been fetched yet",
            Some("git fetch --filter=blob:none origin (from the working tree)".into()),
        ),
        Err(e) => DoctorFinding::warn(
            "cache.refs.main",
            format!("could not invoke `git rev-parse refs/heads/main` against cache: {e}"),
            None,
        ),
    }
}

/// Check #10: working-tree HEAD vs cache main tip — warn when they
/// drift by more than a small commit window. Best-effort: if either
/// rev-parse fails the finding is INFO.
fn check_worktree_head_drift(ctx: &DoctorCtx, cache_path: &Path) -> DoctorFinding {
    if !ctx.is_git_repo {
        return DoctorFinding::info("worktree.head.drift", "skipped — not a git repo");
    }
    let head = git_in(&ctx.path, &["rev-parse", "HEAD"]);
    let head_oid = match head {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => {
            return DoctorFinding::info(
                "worktree.head.drift",
                "skipped — working tree has no HEAD yet (no commits checked out)",
            );
        }
    };
    if !cache_path.exists() {
        return DoctorFinding::info(
            "worktree.head.drift",
            "skipped — cache dir does not exist yet",
        );
    }
    let main = Command::new("git")
        .arg("-C")
        .arg(cache_path)
        .args(["rev-parse", "--verify", "refs/heads/main"])
        .output();
    let main_oid = match main {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => {
            return DoctorFinding::info(
                "worktree.head.drift",
                "skipped — cache has no refs/heads/main yet",
            );
        }
    };
    if head_oid == main_oid {
        return DoctorFinding::ok(
            "worktree.head.drift",
            format!(
                "working tree HEAD matches cache main ({})",
                short_oid(&head_oid)
            ),
        );
    }
    // Try to count commits between them in either direction. Walk the
    // cache repo's object graph (it contains both sides if the working
    // tree was fetched from the same cache).
    let count_cmd = Command::new("git")
        .arg("-C")
        .arg(cache_path)
        .args(["rev-list", "--count", &format!("{main_oid}..{head_oid}")])
        .output();
    let ahead = count_cmd
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let behind_cmd = Command::new("git")
        .arg("-C")
        .arg(cache_path)
        .args(["rev-list", "--count", &format!("{head_oid}..{main_oid}")])
        .output();
    let behind = behind_cmd
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let detail = match (ahead.as_deref(), behind.as_deref()) {
        (Some(a), Some(b)) => format!("ahead {a} / behind {b}"),
        _ => "drift count unavailable (commits may not be in cache)".to_string(),
    };
    DoctorFinding::warn(
        "worktree.head.drift",
        format!(
            "working tree HEAD ({head}) differs from cache main ({main}) — {detail}",
            head = short_oid(&head_oid),
            main = short_oid(&main_oid)
        ),
        Some("git pull --rebase origin main".into()),
    )
}

fn short_oid(oid: &str) -> String {
    oid.chars().take(7).collect()
}

fn check_blob_limit(parsed: Option<&reposix_core::RemoteSpec>) -> DoctorFinding {
    let val = std::env::var("REPOSIX_BLOB_LIMIT").ok();
    match val.as_deref() {
        None | Some("") => {
            DoctorFinding::info("env.REPOSIX_BLOB_LIMIT", "unset (default 200)".to_string())
        }
        Some("0") => {
            // 0 = unlimited per architecture-pivot-summary.
            let is_sim = parsed
                .is_some_and(|s| s.origin.contains("127.0.0.1") || s.origin.contains("localhost"));
            if is_sim {
                DoctorFinding::info(
                    "env.REPOSIX_BLOB_LIMIT",
                    "REPOSIX_BLOB_LIMIT=0 (unlimited; sim backend OK)",
                )
            } else {
                DoctorFinding::warn(
                    "env.REPOSIX_BLOB_LIMIT",
                    "REPOSIX_BLOB_LIMIT=0 (unlimited) on a non-sim backend — risk of runaway REST traffic",
                    Some("unset REPOSIX_BLOB_LIMIT (default 200) or set a finite cap".into()),
                )
            }
        }
        Some(v) => DoctorFinding::info("env.REPOSIX_BLOB_LIMIT", format!("REPOSIX_BLOB_LIMIT={v}")),
    }
}

fn check_sparse_checkout(ctx: &DoctorCtx) -> DoctorFinding {
    if !ctx.is_git_repo {
        return DoctorFinding::info("git.sparse-checkout", "skipped — not a git repo");
    }
    match &ctx.sparse_checkout_lines {
        Some(s) if !s.trim().is_empty() => {
            let count = s.lines().filter(|l| !l.trim().is_empty()).count();
            DoctorFinding::info(
                "git.sparse-checkout",
                format!("{count} sparse-checkout pattern(s) configured"),
            )
        }
        _ => DoctorFinding::info(
            "git.sparse-checkout",
            "no sparse-checkout patterns (working tree materialises all files)".to_string(),
        ),
    }
}

fn check_rust_toolchain() -> DoctorFinding {
    let out = Command::new("rustc").arg("--version").output();
    match out {
        Ok(o) if o.status.success() => DoctorFinding::info(
            "rustc",
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
        ),
        _ => DoctorFinding::info(
            "rustc",
            "rustc not found on PATH (only relevant for contributors)".to_string(),
        ),
    }
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

/// Run `git -C <path> <args...>` and return the `Output`.
fn git_in(path: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git").arg("-C").arg(path).args(args).output()
}

/// Set a git config value (local repo scope).
fn git_config_set(path: &Path, key: &str, value: &str) -> std::io::Result<()> {
    let out = git_in(path, &["config", key, value])?;
    if !out.status.success() {
        return Err(std::io::Error::other(format!(
            "git config {key} {value} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

/// Cross-platform `which`.
fn which(bin: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {bin}"))
        .output()
        .is_ok_and(|o| o.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_display_strings() {
        assert_eq!(format!("{}", Severity::Ok), "OK");
        assert_eq!(format!("{}", Severity::Error), "ERROR");
    }

    #[test]
    fn finding_constructors() {
        let f = DoctorFinding::ok("foo", "bar");
        assert_eq!(f.severity, Severity::Ok);
        let f = DoctorFinding::warn("foo", "bar", Some("fix".into()));
        assert_eq!(f.severity, Severity::Warn);
        assert_eq!(f.fix.as_deref(), Some("fix"));
    }

    #[test]
    fn tally_counts() {
        let r = DoctorReport {
            path: PathBuf::from("/tmp"),
            findings: vec![
                DoctorFinding::ok("a", "ok"),
                DoctorFinding::warn("b", "warn", None),
                DoctorFinding::error("c", "err", None),
            ],
        };
        assert_eq!(r.tally(), (1, 0, 1, 1));
        assert_eq!(r.exit_code(), 1);
    }

    #[test]
    fn empty_dir_yields_git_repo_error() {
        let tmp = tempfile::tempdir().unwrap();
        let report = run(Some(tmp.path()), false).unwrap();
        let git_finding = report
            .findings
            .iter()
            .find(|f| f.check == "git.repo")
            .expect("git.repo finding present");
        assert_eq!(git_finding.severity, Severity::Error);
        assert_eq!(report.exit_code(), 1);
    }

    // Note: `backend_slug_from_origin` now lives in `crate::worktree_helpers`
    // and is unit-tested there.

    #[test]
    fn check_backend_registered_recognises_sim() {
        let spec =
            reposix_core::parse_remote_url("reposix::http://127.0.0.1:7878/projects/demo").unwrap();
        let finding = check_backend_registered(Some(&spec));
        assert_eq!(finding.severity, Severity::Ok);
    }

    #[test]
    fn check_backend_registered_skips_when_unparsed() {
        let finding = check_backend_registered(None);
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn check_allowed_origins_ok_when_loopback_glob_covers_sim() {
        let prev = std::env::var("REPOSIX_ALLOWED_ORIGINS").ok();
        // SAFETY: restored at end of test.
        std::env::set_var("REPOSIX_ALLOWED_ORIGINS", "http://127.0.0.1:*");
        let spec =
            reposix_core::parse_remote_url("reposix::http://127.0.0.1:7878/projects/demo").unwrap();
        let finding = check_allowed_origins(Some(&spec));
        assert_eq!(finding.severity, Severity::Ok, "got {finding:?}");
        match prev {
            Some(v) => std::env::set_var("REPOSIX_ALLOWED_ORIGINS", v),
            None => std::env::remove_var("REPOSIX_ALLOWED_ORIGINS"),
        }
    }

    #[test]
    fn check_allowed_origins_warns_when_host_not_covered() {
        let prev = std::env::var("REPOSIX_ALLOWED_ORIGINS").ok();
        std::env::set_var("REPOSIX_ALLOWED_ORIGINS", "https://api.example.com");
        let spec =
            reposix_core::parse_remote_url("reposix::https://api.github.com/projects/o/r").unwrap();
        let finding = check_allowed_origins(Some(&spec));
        assert_eq!(finding.severity, Severity::Warn, "got {finding:?}");
        assert!(finding
            .fix
            .as_deref()
            .is_some_and(|f| f.contains("api.github.com")));
        match prev {
            Some(v) => std::env::set_var("REPOSIX_ALLOWED_ORIGINS", v),
            None => std::env::remove_var("REPOSIX_ALLOWED_ORIGINS"),
        }
    }

    #[test]
    fn check_cache_has_main_commit_skips_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("does-not-exist");
        let finding = check_cache_has_main_commit(&path);
        assert_eq!(finding.severity, Severity::Info);
    }
}
