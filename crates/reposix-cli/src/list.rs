//! `reposix list` — dump issues for a project as JSON or a pretty table.
//!
//! Dispatches over `--backend`:
//! - `sim` (default): in-process simulator at `--origin`.
//! - `github`: real GitHub Issues at `https://api.github.com` for the public
//!   repo named by `--project` (e.g. `octocat/Hello-World`).
//!   Requires `REPOSIX_ALLOWED_ORIGINS` to include `https://api.github.com`
//!   and optionally `GITHUB_TOKEN` for the 1000 req/hr ceiling.
//! - `confluence`: real Atlassian Confluence Cloud REST v2 at
//!   `https://<tenant>.atlassian.net`. `--project` is the space KEY (e.g.
//!   `REPOSIX`, not the numeric space id). Requires `ATLASSIAN_API_KEY`,
//!   `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` env vars, plus
//!   `REPOSIX_ALLOWED_ORIGINS` including the tenant origin.
//!
//! Output formats:
//! - `json` (default): `serde_json::to_string_pretty(&issues)` — machine-readable,
//!   diffable, the format `parity.sh` pipes through `jq`.
//! - `table`: fixed-width columns `ID | STATUS | TITLE` for human reading.

use anyhow::{Context, Result};
use clap::ValueEnum;
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::sim::SimBackend;
use reposix_core::{BackendConnector, Record};
use reposix_github::GithubReadOnlyBackend;
use reposix_jira::{JiraBackend, JiraCreds};

/// Backend choice for `reposix list --backend`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListBackend {
    /// In-process simulator at the configured `--origin`.
    Sim,
    /// Real GitHub Issues at `api.github.com`. `--project` is `owner/repo`.
    Github,
    /// Real Atlassian Confluence Cloud REST v2. `--project` is the
    /// space key (e.g. `REPOSIX`). Requires `ATLASSIAN_API_KEY`,
    /// `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` env vars plus
    /// `REPOSIX_ALLOWED_ORIGINS` that includes the tenant origin.
    Confluence,
    /// Real Atlassian JIRA Cloud REST v3. `--project` is the JIRA project
    /// key (e.g. `PROJ`). Requires `JIRA_EMAIL`, `JIRA_API_TOKEN`,
    /// `REPOSIX_JIRA_INSTANCE` env vars plus `REPOSIX_ALLOWED_ORIGINS`
    /// that includes `https://{instance}.atlassian.net`.
    Jira,
}

/// Output formats accepted by `reposix list --format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListFormat {
    /// Pretty-printed JSON (the default — diffable and scriptable).
    Json,
    /// Human-readable table with fixed-width columns.
    Table,
}

/// Execute `reposix list`. Builds the requested backend, fetches issues
/// for `project`, and writes formatted output to stdout.
///
/// When `no_truncate` is `true` and `backend` is [`ListBackend::Confluence`],
/// calls [`ConfluenceBackend::list_records_strict`] which returns an error
/// instead of silently capping at 500 pages. For other backends the flag is
/// accepted but has no effect (documented in help text).
///
/// # Errors
/// Returns a wrapped error if the backend cannot be constructed (e.g. a bad
/// `REPOSIX_ALLOWED_ORIGINS` env var), if the HTTP call fails, or if JSON
/// serialization fails.
pub async fn run(
    project: String,
    origin: String,
    backend: ListBackend,
    format: ListFormat,
    no_truncate: bool,
) -> Result<()> {
    let issues = match backend {
        ListBackend::Sim => {
            let b = SimBackend::new(origin).context("build SimBackend")?;
            b.list_records(&project)
                .await
                .with_context(|| format!("sim list_records project={project}"))?
        }
        ListBackend::Github => {
            let token = std::env::var("GITHUB_TOKEN").ok();
            let b = GithubReadOnlyBackend::new(token).context("build GithubReadOnlyBackend")?;
            b.list_records(&project).await.with_context(|| {
                format!("github list_records repo={project} (REPOSIX_ALLOWED_ORIGINS must include https://api.github.com)")
            })?
        }
        ListBackend::Confluence => {
            let (email, token, tenant) = read_confluence_env()
                .context("confluence backend requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, and REPOSIX_CONFLUENCE_TENANT env vars")?;
            let creds = ConfluenceCreds {
                email,
                api_token: token,
            };
            let b = ConfluenceBackend::new(creds, &tenant).context("build ConfluenceBackend")?;
            if no_truncate {
                b.list_records_strict(&project).await.with_context(|| {
                    format!(
                        "confluence list_records_strict space_key={project} (REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net)"
                    )
                })?
            } else {
                b.list_records(&project).await.with_context(|| {
                    format!(
                        "confluence list_records space_key={project} (REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net)"
                    )
                })?
            }
        }
        ListBackend::Jira => {
            let (email, token, instance) = read_jira_env()
                .context("jira backend requires JIRA_EMAIL, JIRA_API_TOKEN, and REPOSIX_JIRA_INSTANCE env vars")?;
            let creds = JiraCreds {
                email,
                api_token: token,
            };
            let b = JiraBackend::new(creds, &instance).context("build JiraBackend")?;
            if no_truncate {
                b.list_records_strict(&project).await.with_context(|| {
                    format!("jira list_records_strict project_key={project} (REPOSIX_ALLOWED_ORIGINS must include https://{instance}.atlassian.net)")
                })?
            } else {
                b.list_records(&project).await.with_context(|| {
                    format!("jira list_records project_key={project} (REPOSIX_ALLOWED_ORIGINS must include https://{instance}.atlassian.net)")
                })?
            }
        }
    };
    match format {
        ListFormat::Json => {
            let pretty = serde_json::to_string_pretty(&issues).context("serialize json")?;
            println!("{pretty}");
        }
        ListFormat::Table => {
            render_table(&issues);
        }
    }
    Ok(())
}

fn render_table(issues: &[Record]) {
    // Column widths: ID is never more than 10 digits for any realistic sim
    // seed; STATUS fits in 12 chars (`in_progress` is the longest). Title
    // takes whatever's left. Literal header text is inlined to avoid the
    // clippy::print_literal lint.
    let id_col = "ID";
    let status_col = "STATUS";
    println!("{id_col:<10} {status_col:<12} TITLE");
    let dash10 = "----------";
    let dash12 = "------------";
    let dash40 = "----------------------------------------";
    println!("{dash10} {dash12} {dash40}");
    for issue in issues {
        let id = issue.id.0;
        let status = issue.status.as_str();
        let title = &issue.title;
        println!("{id:<10} {status:<12} {title}");
    }
}

/// Read the three Atlassian env vars in one shot from the live process
/// environment.
///
/// Thin production adapter over the pure-fn [`read_confluence_env_from`]; the
/// latter is what the unit test exercises.
///
/// # Errors
///
/// Returns an error whose message lists ALL three env-var names and indicates
/// which were empty — so the user fixes them in one round-trip instead of N
/// error/edit/re-run cycles.
pub(crate) fn read_confluence_env() -> anyhow::Result<(String, String, String)> {
    read_confluence_env_from(|name| std::env::var(name).unwrap_or_default())
}

/// Pure-fn variant of [`read_confluence_env`] for testability.
///
/// `get` receives the env-var name and returns the value (or empty string if
/// unset). This shape avoids mutating real process env state in unit tests,
/// which is unsound under Rust's parallel-tests harness.
///
/// # Errors
///
/// Bails with a single message listing ALL three env-var names and the subset
/// currently empty.
pub(crate) fn read_confluence_env_from(
    get: impl Fn(&str) -> String,
) -> anyhow::Result<(String, String, String)> {
    let email = get("ATLASSIAN_EMAIL");
    let token = get("ATLASSIAN_API_KEY");
    let tenant = get("REPOSIX_CONFLUENCE_TENANT");
    let mut missing: Vec<&'static str> = Vec::new();
    if email.is_empty() {
        missing.push("ATLASSIAN_EMAIL");
    }
    if token.is_empty() {
        missing.push("ATLASSIAN_API_KEY");
    }
    if tenant.is_empty() {
        missing.push("REPOSIX_CONFLUENCE_TENANT");
    }
    if !missing.is_empty() {
        anyhow::bail!(
            "confluence backend requires these env vars; currently unset: {}. \
             Required: ATLASSIAN_EMAIL (your Atlassian account email), \
             ATLASSIAN_API_KEY (token from id.atlassian.com/manage-profile/security/api-tokens), \
             REPOSIX_CONFLUENCE_TENANT (your `<tenant>.atlassian.net` subdomain).",
            missing.join(", ")
        );
    }
    Ok((email, token, tenant))
}

/// Read the three JIRA env vars in one shot from the live process environment.
///
/// Thin production adapter over the pure-fn [`read_jira_env_from`].
///
/// # Errors
///
/// Returns an error listing ALL missing env-var names if any are unset.
pub(crate) fn read_jira_env() -> anyhow::Result<(String, String, String)> {
    read_jira_env_from(|name| std::env::var(name).unwrap_or_default())
}

/// Pure-fn variant of [`read_jira_env`] for testability.
///
/// `lookup` is called once per var name. Empty-string return is treated as missing.
/// All missing vars are collected into ONE error message — never partial.
/// Values are NEVER included in error messages.
///
/// # Errors
///
/// Returns a descriptive error listing all missing env var names if any are unset.
pub(crate) fn read_jira_env_from(
    lookup: impl Fn(&str) -> String,
) -> anyhow::Result<(String, String, String)> {
    let email = lookup("JIRA_EMAIL");
    let token = lookup("JIRA_API_TOKEN");
    let instance = lookup("REPOSIX_JIRA_INSTANCE");

    let mut missing: Vec<&'static str> = Vec::new();
    if email.is_empty() {
        missing.push("JIRA_EMAIL");
    }
    if token.is_empty() {
        missing.push("JIRA_API_TOKEN");
    }
    if instance.is_empty() {
        missing.push("REPOSIX_JIRA_INSTANCE");
    }

    if !missing.is_empty() {
        anyhow::bail!(
            "jira backend requires these env vars; currently unset: {}. \
             Required: JIRA_EMAIL (your Atlassian account email), \
             JIRA_API_TOKEN (token from id.atlassian.com/manage-profile/security/api-tokens), \
             REPOSIX_JIRA_INSTANCE (your `<tenant>.atlassian.net` subdomain, e.g. `mycompany`).",
            missing.join(", ")
        );
    }
    Ok((email, token, instance))
}

#[cfg(test)]
mod tests {
    use super::{read_confluence_env_from, read_jira_env_from, ListBackend};

    #[test]
    fn confluence_is_a_value_enum_variant() {
        // Compile-time check that the enum variant exists; `matches!` keeps
        // clippy happy vs a bare `let _ = ListBackend::Confluence;`.
        let b = ListBackend::Confluence;
        assert!(matches!(b, ListBackend::Confluence));
    }

    #[test]
    fn confluence_requires_all_three_env_vars() {
        // All three empty: error message must mention all three names and
        // must NOT leak any (absent, but check anyway) token/email value.
        let err = read_confluence_env_from(|_| String::new()).expect_err("all-empty must fail");
        let msg = format!("{err}");
        assert!(
            msg.contains("ATLASSIAN_EMAIL"),
            "error must mention ATLASSIAN_EMAIL: {msg}"
        );
        assert!(
            msg.contains("ATLASSIAN_API_KEY"),
            "error must mention ATLASSIAN_API_KEY: {msg}"
        );
        assert!(
            msg.contains("REPOSIX_CONFLUENCE_TENANT"),
            "error must mention REPOSIX_CONFLUENCE_TENANT: {msg}"
        );

        // Partial-set: only token is missing. Message still lists all three
        // NAMES (under "Required:") but the "currently unset:" subset is
        // just the token. T-11B-01: value must never appear in the error.
        let err = read_confluence_env_from(|name| match name {
            "ATLASSIAN_EMAIL" => "user@example.com".to_owned(),
            "REPOSIX_CONFLUENCE_TENANT" => "mytenant".to_owned(),
            _ => String::new(),
        })
        .expect_err("missing token must fail");
        let msg = format!("{err}");
        assert!(msg.contains("ATLASSIAN_API_KEY"));
        assert!(
            !msg.contains("user@example.com"),
            "error must not echo the email value: {msg}"
        );
        assert!(
            !msg.contains("mytenant"),
            "error must not echo the tenant value: {msg}"
        );
    }

    #[test]
    fn confluence_all_set_returns_values() {
        let (email, token, tenant) = read_confluence_env_from(|name| match name {
            "ATLASSIAN_EMAIL" => "a@b.c".to_owned(),
            "ATLASSIAN_API_KEY" => "TOKEN".to_owned(),
            "REPOSIX_CONFLUENCE_TENANT" => "tenant".to_owned(),
            _ => String::new(),
        })
        .expect("all set must succeed");
        assert_eq!(email, "a@b.c");
        assert_eq!(token, "TOKEN");
        assert_eq!(tenant, "tenant");
    }

    #[test]
    fn read_jira_env_from_all_empty_fails() {
        let err = read_jira_env_from(|_| String::new()).expect_err("all-empty must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("JIRA_EMAIL"),
            "must list JIRA_EMAIL, got: {msg}"
        );
        assert!(
            msg.contains("JIRA_API_TOKEN"),
            "must list JIRA_API_TOKEN, got: {msg}"
        );
        assert!(
            msg.contains("REPOSIX_JIRA_INSTANCE"),
            "must list REPOSIX_JIRA_INSTANCE, got: {msg}"
        );
    }

    #[test]
    fn read_jira_env_from_partial_missing_lists_all() {
        let err = read_jira_env_from(|name| match name {
            "JIRA_EMAIL" => "user@example.com".to_owned(),
            _ => String::new(),
        })
        .expect_err("partial must fail");
        let msg = err.to_string();
        assert!(msg.contains("JIRA_API_TOKEN"), "msg: {msg}");
        assert!(msg.contains("REPOSIX_JIRA_INSTANCE"), "msg: {msg}");
        assert!(
            !msg.contains("user@example.com"),
            "error must not echo email value: {msg}"
        );
    }

    #[test]
    fn read_jira_env_from_all_set_succeeds() {
        let (email, token, instance) = read_jira_env_from(|name| match name {
            "JIRA_EMAIL" => "user@example.com".to_owned(),
            "JIRA_API_TOKEN" => "secret".to_owned(),
            "REPOSIX_JIRA_INSTANCE" => "mycompany".to_owned(),
            _ => String::new(),
        })
        .expect("all set must succeed");
        assert_eq!(email, "user@example.com");
        assert_eq!(token, "secret");
        assert_eq!(instance, "mycompany");
    }
}
