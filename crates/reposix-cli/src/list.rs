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
use reposix_confluence::{ConfluenceCreds, ConfluenceReadOnlyBackend};
use reposix_core::backend::sim::SimBackend;
use reposix_core::{Issue, IssueBackend};
use reposix_github::GithubReadOnlyBackend;

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
/// # Errors
/// Returns a wrapped error if the backend cannot be constructed (e.g. a bad
/// `REPOSIX_ALLOWED_ORIGINS` env var), if the HTTP call fails, or if JSON
/// serialization fails.
pub async fn run(
    project: String,
    origin: String,
    backend: ListBackend,
    format: ListFormat,
) -> Result<()> {
    let issues = match backend {
        ListBackend::Sim => {
            let b = SimBackend::new(origin).context("build SimBackend")?;
            b.list_issues(&project)
                .await
                .with_context(|| format!("sim list_issues project={project}"))?
        }
        ListBackend::Github => {
            let token = std::env::var("GITHUB_TOKEN").ok();
            let b = GithubReadOnlyBackend::new(token).context("build GithubReadOnlyBackend")?;
            b.list_issues(&project).await.with_context(|| {
                format!("github list_issues repo={project} (REPOSIX_ALLOWED_ORIGINS must include https://api.github.com)")
            })?
        }
        ListBackend::Confluence => {
            let (email, token, tenant) = read_confluence_env()
                .context("confluence backend requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, and REPOSIX_CONFLUENCE_TENANT env vars")?;
            let creds = ConfluenceCreds {
                email,
                api_token: token,
            };
            let b = ConfluenceReadOnlyBackend::new(creds, &tenant)
                .context("build ConfluenceReadOnlyBackend")?;
            b.list_issues(&project).await.with_context(|| {
                format!(
                    "confluence list_issues space_key={project} (REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net)"
                )
            })?
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

fn render_table(issues: &[Issue]) {
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
fn read_confluence_env() -> anyhow::Result<(String, String, String)> {
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
fn read_confluence_env_from(
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

#[cfg(test)]
mod tests {
    use super::{read_confluence_env_from, ListBackend};

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
}
