//! `reposix spaces` — list all readable Confluence spaces as a table.
//!
//! Dispatches over `--backend`; currently only `confluence` is supported
//! (the sim + github backends have no space concept). Requires
//! `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` +
//! `REPOSIX_ALLOWED_ORIGINS` containing the tenant origin.

use anyhow::{anyhow, Context, Result};
use reposix_confluence::{ConfSpaceSummary, ConfluenceBackend, ConfluenceCreds};
use reposix_core::errmsg::teach;

use crate::list::{read_confluence_env, ListBackend};

/// Execute `reposix spaces`.
///
/// Only `ListBackend::Confluence` is supported. Sim, GitHub, and JIRA return
/// ONE deduped teaching error (was three near-identical `bail!`s) that names
/// the backend the user actually requested, explains why there is nothing to
/// list, and points at the right alternative command.
///
/// # Errors
/// - `ATLASSIAN_*` env vars missing → wrapped error from `read_confluence_env`.
/// - Transport/HTTP failure → wrapped `ConfluenceBackend::list_spaces` error.
/// - Backend is not `Confluence` → the deduped [`spaces_requires_confluence_error`].
pub async fn run(backend: ListBackend) -> Result<()> {
    // Guard first, then run the (unindented) confluence path. Collapsing the
    // three former per-backend `bail!`s into ONE error that echoes the ACTUAL
    // requested backend removes a copy-paste smell and keeps the teaching in a
    // single place (OD-3: dedupe near-identical error strings).
    if backend != ListBackend::Confluence {
        let requested = match backend {
            ListBackend::Sim => "sim",
            ListBackend::Github => "github",
            ListBackend::Jira => "jira",
            // Guarded by the `!=` above; unreachable in practice.
            ListBackend::Confluence => unreachable!("confluence handled below the guard"),
        };
        return Err(spaces_requires_confluence_error(requested));
    }

    let (email, token, tenant) = read_confluence_env().context(
        "spaces --backend confluence requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, and \
         REPOSIX_CONFLUENCE_TENANT env vars",
    )?;
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

/// The single, deduped teaching error for `reposix spaces` against a
/// non-Confluence backend. Names the backend the caller actually requested
/// (`requested`), teaches why spaces is Confluence-only, and points at the
/// per-backend `reposix list` alternative + a copy-paste recovery.
fn spaces_requires_confluence_error(requested: &str) -> anyhow::Error {
    anyhow!(
        "{}",
        teach(
            &format!(
                "`reposix spaces` supports only the Confluence backend; you requested \
                 `{requested}`."
            ),
            "spaces lists Confluence spaces (wiki space directories); the sim, GitHub, and \
             JIRA backends have no space concept, so there is nothing to enumerate.",
            &format!(
                "to browse `{requested}` issues instead, list them by project: \
                 `reposix list --backend {requested} --project <KEY>`."
            ),
            &["reposix spaces --backend confluence   # list your Confluence spaces",],
        )
    )
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
