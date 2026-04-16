//! `reposix spaces` — list all readable Confluence spaces as a table.
//!
//! Dispatches over `--backend`; currently only `confluence` is supported
//! (the sim + github backends have no space concept). Requires
//! `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` +
//! `REPOSIX_ALLOWED_ORIGINS` containing the tenant origin.

use anyhow::{bail, Context, Result};
use reposix_confluence::{ConfSpaceSummary, ConfluenceBackend, ConfluenceCreds};

use crate::list::{read_confluence_env, ListBackend};

/// Execute `reposix spaces`.
///
/// Only `ListBackend::Confluence` is supported. Sim and GitHub return an
/// error (exit code 1) with a clear message explaining that the backend
/// has no space concept.
///
/// # Errors
/// - `ATLASSIAN_*` env vars missing → wrapped error from `read_confluence_env`.
/// - Transport/HTTP failure → wrapped `ConfluenceBackend::list_spaces` error.
/// - Backend is `Sim` or `Github` → `anyhow::bail!` with a clear message.
pub async fn run(backend: ListBackend) -> Result<()> {
    match backend {
        ListBackend::Sim => {
            bail!("spaces is only supported for --backend confluence (sim has no space concept)")
        }
        ListBackend::Github => {
            bail!("spaces is only supported for --backend confluence (github has no space concept; use `gh api` or GitHub's UI)")
        }
        ListBackend::Confluence => {
            let (email, token, tenant) = read_confluence_env()
                .context("spaces --backend confluence requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, and REPOSIX_CONFLUENCE_TENANT env vars")?;
            let creds = ConfluenceCreds {
                email,
                api_token: token,
            };
            let b = ConfluenceBackend::new(creds, &tenant).context("build ConfluenceBackend")?;
            let spaces = b.list_spaces().await.with_context(|| {
                format!(
                    "confluence list_spaces (REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net)"
                )
            })?;
            render_spaces_table(&spaces);
            Ok(())
        }
    }
}

/// Render spaces as a fixed-width table: `KEY<12> NAME<30> URL`.
///
/// Deliberately minimal — pipes cleanly into `column -t` or agent pipelines.
fn render_spaces_table(spaces: &[ConfSpaceSummary]) {
    // Literal header strings inlined to avoid clippy::print_literal.
    let key_col = "KEY";
    let name_col = "NAME";
    println!("{key_col:<12} {name_col:<30} URL");
    let dash12 = "------------";
    let dash30 = "------------------------------";
    let dash3 = "---";
    println!("{dash12} {dash30} {dash3}");
    for s in spaces {
        let key = &s.key;
        let name = &s.name;
        let url = &s.webui_url;
        println!("{key:<12} {name:<30} {url}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reposix_confluence::ConfSpaceSummary;

    #[test]
    fn render_table_prints_header_and_rows() {
        // Smoke test — exercises the format strings. Real output capture
        // would need a harness; this just ensures no panic.
        let spaces = vec![
            ConfSpaceSummary {
                key: "REPOSIX".to_owned(),
                name: "Reposix Project".to_owned(),
                webui_url: "https://example.atlassian.net/wiki/spaces/REPOSIX".to_owned(),
            },
            ConfSpaceSummary {
                key: "TEAM".to_owned(),
                name: "Team Space".to_owned(),
                webui_url: "https://example.atlassian.net/wiki/spaces/TEAM".to_owned(),
            },
        ];
        render_spaces_table(&spaces);
    }

    #[tokio::test]
    async fn sim_backend_returns_clear_error() {
        let err = run(ListBackend::Sim).await.expect_err("sim must fail");
        let msg = format!("{err}");
        assert!(
            msg.contains("confluence"),
            "error must point user to confluence: {msg}"
        );
    }

    #[tokio::test]
    async fn github_backend_returns_clear_error() {
        let err = run(ListBackend::Github)
            .await
            .expect_err("github must fail");
        let msg = format!("{err}");
        assert!(msg.contains("github"), "error must mention github: {msg}");
    }
}
