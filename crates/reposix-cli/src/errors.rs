//! Shared teaching-error constructors for the `reposix` CLI (Phase 120 / P120).
//!
//! Each fn wraps [`reposix_core::errmsg::teach`] in an `anyhow::Error` at the
//! binary boundary so every subcommand that hits the SAME failure shape emits
//! the SAME Rust-compiler-grade 3-part message (teach the fix / name the
//! alternative / give a copy-paste recovery) instead of a hand-rolled string.
//!
//! Three shapes recur across the CLI surface and are consolidated here (P120
//! leverage #1/#2/#3):
//!
//! - [`spec_parse_error`] — the `<backend>::<project>` spec-parse failure shared
//!   by `init` / `attach` / `sync` / `refresh` (root: `init::translate_spec_to_url`).
//! - [`missing_env_var_error`] — a Confluence/JIRA tenant/instance env var is
//!   unset; emits `export <VAR>=<value>` + a retry + the credential-free `sim::`
//!   alternative.
//! - [`cache_build_error`] — the `.context("build cache from backend")` /
//!   reconcile-wrapper epidemic: surfaces the connector's OWN message AND names
//!   the likely root cause (backend down / creds unset) with a runnable recovery.
//! - [`missing_cache_db_error`] — the "no synced cache / no `cache.db` yet"
//!   failure shared by `tokens` / `cost` / `gc` / `history` (P120 leverage #4).
//!
//! A later wave adds `malformed_bus_url_error` (W4, helper side) alongside these.

use std::path::Path;

use anyhow::anyhow;
use reposix_core::errmsg::teach;

/// The `<backend>::<project>` spec-parse failure (missing `::`, empty project, or
/// unknown backend). Shared by `init` / `attach` / `sync` / `refresh` via
/// `init::translate_spec_to_url`, so all four inherit the same teaching.
///
/// `cause` is the specific parse fault (e.g. "expected `<backend>::<project>`
/// form (missing `::` separator)"), interpolated into the headline.
#[must_use]
pub fn spec_parse_error(spec: &str, cause: &str) -> anyhow::Error {
    anyhow!(
        "{}",
        teach(
            &format!("invalid backend spec `{spec}`: {cause}."),
            "a spec is `<backend>::<project>` — one of `sim::<slug>`, `github::<owner>/<repo>`, \
             `confluence::<space>`, `jira::<key>`.",
            "start with the simulator, which needs no credentials: `sim::demo`.",
            &["reposix init sim::demo /tmp/demo"],
        )
    )
}

/// A real-backend env var (`REPOSIX_CONFLUENCE_TENANT` / `REPOSIX_JIRA_INSTANCE`)
/// is unset. Emits the `export <VAR>=<example>` recovery, a retry note, and the
/// credential-free `sim::` alternative (P120 leverage #2, CLI side).
///
/// `example_value` is a sample value for the `export` line (e.g. `mycompany`).
#[must_use]
pub fn missing_env_var_error(var: &str, backend: &str, example_value: &str) -> anyhow::Error {
    let headline = format!(
        "the `{backend}::…` backend needs the {var} environment variable, but it is unset."
    );
    let fix = format!(
        "{var} is your Atlassian Cloud subdomain — the `<x>` in `https://<x>.atlassian.net`."
    );
    let export = format!("export {var}={example_value}");
    anyhow!(
        "{}",
        teach(
            &headline,
            &fix,
            "no Atlassian tenant handy? the simulator needs no credentials — use `sim::demo` instead.",
            &[
                export.as_str(),
                "# then re-run the same reposix command (init / attach / sync / refresh)",
            ],
        )
    )
}

/// The `.context("build cache from backend")` / reconcile-wrapper failure shared
/// across `attach` / `list` / `refresh` / `sync`. SURFACES the connector's own
/// error (`{source}`) AND names the likely root cause + a runnable recovery
/// (P120 leverage #3 — do NOT swallow the source).
///
/// `source` is the underlying connector/cache error; its full Display chain is
/// appended so the specific cause is never hidden behind the teaching layer.
#[must_use]
pub fn cache_build_error(
    backend: &str,
    project: &str,
    source: impl std::fmt::Display,
) -> anyhow::Error {
    let headline = format!(
        "could not sync `{backend}::{project}` against the backend.\n(underlying: {source:#})"
    );
    let doctor =
        format!("reposix doctor   # check reachability + credentials for the `{backend}` backend");
    anyhow!(
        "{}",
        teach(
            &headline,
            "reposix builds a local git cache from the backend's REST API; that step could not \
             reach or read it — usually the backend is down or its credentials are unset.",
            "for a no-network smoke test, use the simulator: start it with `reposix sim`, then \
             target `sim::<slug>`.",
            &[
                "reposix sim   # start the simulator, if you meant sim::…",
                doctor.as_str(),
            ],
        )
    )
}

/// The "no synced cache / no `cache.db` yet" failure shared by `tokens` /
/// `cost` / `gc` / `history` (P120 leverage #4). The working tree is a valid
/// reposix tree, but its cache has never been synced — there is no
/// `<cache>/cache.db` ledger (nor, for `gc`, any cache directory) to read.
/// Routing all four subcommands through this ONE helper means they emit the
/// SAME populate-the-cache guidance instead of four near-identical hand-rolled
/// strings.
///
/// `cache_path` is the resolved-but-absent cache directory, echoed so the user
/// sees exactly which cache is missing.
#[must_use]
pub fn missing_cache_db_error(cache_path: &Path) -> anyhow::Error {
    let headline = format!(
        "no synced reposix cache at {} yet — there is nothing to read.",
        cache_path.display()
    );
    anyhow!(
        "{}",
        teach(
            &headline,
            "reposix builds this cache (and its token/audit ledger) from the backend on the first \
             fetch; run one from inside the working tree, then re-run the command.",
            "already synced in another checkout? re-run the command from that working tree instead.",
            &[
                "git fetch         # from the working tree — materializes the cache + audit ledger",
                "reposix refresh   # or rebuild the whole tree + cache from the backend",
            ],
        )
    )
}
